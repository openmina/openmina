use std::path::PathBuf;

use anyhow::Context;
use libp2p_identity::Keypair;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, NonZeroCurvePoint, NonZeroCurvePointUncompressedStableV1,
    UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};
use node::{account::AccountSecretKey, transition_frontier::genesis::GenesisConfig};
use openmina_core::consensus::ConsensusConstants;
use openmina_core::constants::CONSTRAINT_CONSTANTS;
use rand::prelude::*;

use reqwest::Url;

use node::core::log::inner::Level;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use node::p2p::identity::SecretKey;
use node::service::Recorder;
use node::SnarkerStrategy;

use openmina_node_native::{tracing, NodeBuilder};

/// Openmina node
#[derive(Debug, clap::Args)]
pub struct Node {
    #[arg(long, short = 'd', default_value = "~/.openmina", env)]
    pub work_dir: String,

    /// Peer secret key
    #[arg(long, short = 's', env = "OPENMINA_P2P_SEC_KEY")]
    pub p2p_secret_key: Option<SecretKey>,

    /// Http port to listen on
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,

    /// LibP2P port to listen on
    #[arg(long, env, default_value = "8302")]
    pub libp2p_port: u16,

    /// Verbosity level
    #[arg(long, short, env, default_value = "info")]
    pub verbosity: Level,

    #[arg(long, short = 'P', alias = "peer")]
    pub peers: Vec<P2pConnectionOutgoingInitOpts>,

    /// File containing initial peers.
    ///
    /// Each line should contain peer's multiaddr.
    #[arg(long, env)]
    pub peer_list_file: Option<PathBuf>,

    /// File containing initial peers.
    ///
    /// Each line should contain peer's multiaddr.
    #[arg(long, env)]
    pub peer_list_url: Option<Url>,

    /// Run the node in seed mode. No default peers will be added.
    #[arg(long, env)]
    pub seed: bool,

    /// Run Snark Worker.
    ///
    /// Pass snarker private key as an argument.
    #[arg(long, env)]
    pub run_snarker: Option<AccountSecretKey>,

    /// Enable block producer with this key file
    ///
    /// MINA_PRIVKEY_PASS must be set to decrypt the keyfile
    #[arg(long, env)]
    pub producer_key: Option<PathBuf>,
    /// Snark fee, in Mina
    #[arg(long, env, default_value_t = 1_000_000)]
    pub snarker_fee: u64,

    #[arg(long, env, default_value = "seq")]
    pub snarker_strategy: SnarkerStrategy,

    #[arg(long, default_value = "none", env)]
    pub record: String,

    /// Do not use peers discovery.
    #[arg(long)]
    pub no_peers_discovery: bool,

    /// Config JSON file to load at startup.
    // TODO: make this argument required.
    #[arg(short = 'c', long, env)]
    pub config: Option<PathBuf>,
}

impl Node {
    pub fn run(self) -> anyhow::Result<()> {
        tracing::initialize(self.verbosity);

        rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get().max(2) - 1)
            .thread_name(|i| format!("openmina_rayon_{i}"))
            .build_global()
            .context("failed to initialize threadpool")?;

        let rng_seed = rand::thread_rng().next_u64();
        let p2p_secret_key = self.p2p_secret_key.unwrap_or_else(SecretKey::rand);

        let initial_peers = {
            let mut result = Vec::new();

            result.append(&mut self.peers);

            if let Some(path) = self.peer_list_file {
                Self::peers_from_reader(
                    &mut result,
                    File::open(&path)
                        .context(anyhow::anyhow!("opening peer list file {path:?}"))?,
                )
                .context(anyhow::anyhow!("reading peer list file {path:?}"))?;
            }

            if let Some(url) = self.peer_list_url {
                Self::peers_from_reader(
                    &mut result,
                    reqwest::blocking::get(url.clone())
                        .context(anyhow::anyhow!("reading peer list url {url}"))?,
                )
                .context(anyhow::anyhow!("reading peer list url {url}"))?;
            }

            if result.is_empty() && !self.seed {
                result.extend(default_peers());
            }

            result
        };

        let block_producer = self.producer_key.clone().map(|producer_key_path| {
            let keypair = AccountSecretKey::from_encrypted_file(producer_key_path)
                .expect("Failed to decrypt secret key file");
            let compressed_pub_key = keypair.public_key_compressed();
            (
                BlockProducerConfig {
                    pub_key: NonZeroCurvePoint::from(NonZeroCurvePointUncompressedStableV1 {
                        x: compressed_pub_key.x.into(),
                        is_odd: compressed_pub_key.is_odd,
                    }),
                    custom_coinbase_receiver: None,
                    proposed_protocol_version: None,
                },
                keypair,
            )
        });

        let work_dir = shellexpand::full(&self.work_dir).unwrap().into_owned();
        let srs: Arc<_> = get_srs();

        let (daemon_conf, genesis_conf) = match self.config {
            Some(config) => {
                let reader = File::open(config).context("config file {config:?}")?;
                let config: DaemonJson =
                    serde_json::from_reader(reader).context("config file {config:?}")?;
                (
                    config
                        .daemon
                        .clone()
                        .unwrap_or(daemon_json::Daemon::DEFAULT),
                    Arc::new(GenesisConfig::DaemonJson(config)),
                )
            }
            None => (
                daemon_json::Daemon::DEFAULT,
                node::config::DEVNET_CONFIG.clone(),
            ),
        };

        let protocol_constants = genesis_conf.protocol_constants()?;
        let consensus_consts =
            ConsensusConstants::create(&CONSTRAINT_CONSTANTS, &protocol_constants);

        let transition_frontier = TransitionFrontierConfig::new(genesis_conf);
        let config = Config {
            ledger: LedgerConfig {},
            snark: SnarkConfig {
                // TODO(binier): use cache
                block_verifier_index: get_verifier_index(VerifierKind::Blockchain).into(),
                block_verifier_srs: srs.clone(),
                work_verifier_index: get_verifier_index(VerifierKind::Transaction).into(),
                work_verifier_srs: srs,
            },
            global: GlobalConfig {
                build: BuildEnv::get().into(),
                snarker: self.run_snarker.map(|public_key| SnarkerConfig {
                    public_key,
                    fee: CurrencyFeeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                        self.snarker_fee.into(),
                    )),
                    strategy: self.snarker_strategy,
                    auto_commit: true,
                }),
            },
            p2p: P2pConfig {
                libp2p_port: Some(self.libp2p_port),
                listen_port: self.port,
                identity_pub_key: p2p_secret_key.public_key(),
                initial_peers,
                ask_initial_peers_interval: Duration::from_secs(3600),
                enabled_channels: ChannelId::iter_all().collect(),
                peer_discovery: !self.no_peers_discovery,
                initial_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("linear time"),
                timeouts: P2pTimeouts::default(),
                limits: P2pLimits::default().with_max_peers(Some(100)),
            },
            transition_frontier,
            block_producer: block_producer.clone().map(|(config, _)| config),
            tx_pool: ledger::transaction_pool::Config {
                trust_system: (),
                pool_max_size: daemon_conf.tx_pool_max_size(),
                slot_tx_end: daemon_conf.slot_tx_end(),
            },
        };
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        if let Some(sec_key) = self.p2p_secret_key {
            node_builder.p2p_sec_key(sec_key);
        }
        node_builder.p2p_libp2p_port(self.libp2p_port);

        self.seed.then(|| node_builder.p2p_seed_node());
        self.no_peers_discovery
            .then(|| node_builder.p2p_no_discovery());

        node_builder.initial_peers(self.peers);
        if let Some(path) = self.peer_list_file {
            node_builder.initial_peers_from_file(path)?;
        }
        if let Some(url) = self.peer_list_url {
            node_builder.initial_peers_from_url(url)?;
        }

        if let Some(producer_key_path) = self.producer_key {
            node_builder.block_producer_from_file(producer_key_path)?;
        }

        if let Some(sec_key) = self.run_snarker {
            node_builder.snarker(sec_key, self.snarker_fee, self.snarker_strategy);
        }

        let work_dir = shellexpand::full(&self.work_dir).unwrap().into_owned();

        node_builder
            .http_server(self.port)
            .gather_stats()
            .record(match self.record.trim() {
                "none" => Recorder::None,
                "state-with-input-actions" => Recorder::only_input_actions(work_dir),
                _ => panic!("unknown --record strategy"),
            });

        let mut node = node_builder.build().context("node build failed!")?;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .thread_stack_size(64 * 1024 * 1024)
            .build()
            .unwrap();

        runtime.block_on(node.run_forever());

            if let Some((_, keypair)) = block_producer {
                service.block_producer_start(keypair);
            }

            let state = State::new(config, &consensus_consts, redux::Timestamp::global_now());
            let mut node = ::node::Node::new(state, service, None);

            // record initial state.
            {
                let store = node.store_mut();
                store
                    .service
                    .recorder()
                    .initial_state(rng_seed, p2p_secret_key, store.state.get());
            }

            node.store_mut().dispatch(EventSourceAction::ProcessEvents);
            loop {
                node.store_mut().dispatch(EventSourceAction::WaitForEvents);

                let service = &mut node.store_mut().service;
                let wait_for_events = service.event_receiver.wait_for_events();
                let rpc_req_fut = async {
                    // TODO(binier): optimize maybe to not check it all the time.
                    match service.rpc.req_receiver().recv().await {
                        Some(v) => v,
                        None => std::future::pending().await,
                    }
                };
                let timeout = tokio::time::sleep(Duration::from_millis(100));

                select! {
                    _ = wait_for_events => {
                        while node.store_mut().service.event_receiver.has_next() {
                            node.store_mut().dispatch(EventSourceAction::ProcessEvents);
                        }
                    }
                    req = rpc_req_fut => {
                        node.store_mut().service.process_rpc_request(req);
                    }
                    _ = timeout => {
                        node.store_mut().dispatch(EventSourceAction::WaitTimeout);
                    }
                }
            }
        });

        Ok(())
    }

    fn peers_from_reader<R: Read>(
        peers: &mut Vec<P2pConnectionOutgoingInitOpts>,
        read: R,
    ) -> anyhow::Result<()> {
        let read = BufReader::new(read);
        for line in read.lines() {
            let line = line.context("reading line")?;
            let l = line.trim();
            if !l.is_empty() {
                peers.push(l.parse().context(anyhow::anyhow!("parsing entry"))?);
            }
        }
        Ok(())
    }
}
