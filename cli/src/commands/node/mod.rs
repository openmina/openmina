use std::ffi::OsString;

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use libp2p_identity::Keypair;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, NonZeroCurvePoint, NonZeroCurvePointUncompressedStableV1,
    UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};
use node::transition_frontier::genesis::GenesisConfig;
use openmina_core::consensus::ConsensusConstants;
use openmina_core::constants::CONSTRAINT_CONSTANTS;
use rand::prelude::*;

use redux::SystemTime;
use reqwest::Url;
use tokio::select;

use node::account::{AccountPublicKey, AccountSecretKey};
use node::core::channels::mpsc;
use node::core::log::inner::Level;
use node::daemon_json::{self, DaemonJson};
use node::event_source::EventSourceAction;
use node::ledger::{LedgerCtx, LedgerManager};
use node::p2p::channels::ChannelId;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use node::p2p::identity::SecretKey;
use node::p2p::service_impl::webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p;
use node::p2p::{P2pConfig, P2pLimits, P2pTimeouts};
use node::service::{Recorder, Service};
use node::snark::{get_srs, get_verifier_index, VerifierKind};
use node::stats::Stats;
use node::{
    BlockProducerConfig, BuildEnv, Config, GlobalConfig, LedgerConfig, SnarkConfig, SnarkerConfig,
    SnarkerStrategy, State, TransitionFrontierConfig,
};

use openmina_node_native::rpc::RpcService;
use openmina_node_native::{http_server, tracing, NodeService, P2pTaskSpawner, RpcSender};

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

    /// Run Snark Worker.
    ///
    /// Pass snarker public key as an argument.
    #[arg(long, env)]
    pub run_snarker: Option<AccountPublicKey>,

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

    /// Mina snark worker path
    #[arg(long, env, default_value = "cli/bin/snark-worker")]
    pub snarker_exe_path: OsString,

    #[arg(long, default_value = "none", env)]
    pub record: String,

    #[arg(long, default_value = "none")]
    pub additional_ledgers_path: Option<PathBuf>,

    /// Do not use peers discovery.
    #[arg(long)]
    pub no_peers_discovery: bool,

    /// Config JSON file to load at startup.
    // TODO: make this argument required.
    #[arg(short = 'c', long, env)]
    pub config: Option<PathBuf>,
}

fn default_peers() -> Vec<P2pConnectionOutgoingInitOpts> {
    [
        // "/2ajh5CpZCHdv7tmMrotVnLjQXuhcuCzqKosdDmvN3tNTScw2fsd/http/65.109.110.75/10000",

        // "/dns4/seed-1.berkeley.o1test.net/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        // "/dns4/seed-2.berkeley.o1test.net/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        // "/dns4/seed-3.berkeley.o1test.net/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        "/ip4/34.170.114.52/tcp/10001/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        "/ip4/34.135.63.47/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        "/ip4/34.170.114.52/tcp/10001/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        //
        // "/dns4/webrtc2.webnode.openmina.com/tcp/443/p2p/12D3KooWFpqySZDHx7k5FMjdwmrU3TLhDbdADECCautBcEGtG4fr",
        // "/dns4/webrtc2.webnode.openmina.com/tcp/4431/p2p/12D3KooWJBeXosFxdBwe2mbKRjgRG69ERaUTpS9qo9NRkoE8kBpj",

        // "/ip4/78.27.236.28/tcp/8302/p2p/12D3KooWDLNXPq28An4s2QaPZX5ftem1AfaCWuxHHJq97opeWxLy",
    ]
    .into_iter()
    .map(|s| s.parse().unwrap())
    .collect()
}

impl Node {
    pub fn run(mut self) -> anyhow::Result<()> {
        tracing::initialize(self.verbosity);

        rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get().max(2) - 1)
            .thread_name(|i| format!("openmina_rayon_{i}"))
            .build_global()
            .context("failed to initialize threadpool")?;

        let mut rng = ThreadRng::default();

        let secret_key = self.p2p_secret_key.unwrap_or_else(|| {
            let bytes = rng.gen();
            SecretKey::from_bytes(bytes)
        });
        let pub_key = secret_key.public_key();

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

            if result.is_empty() {
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
        let rng_seed = rng.next_u64();
        let srs: Arc<_> = get_srs();

        let (daemon_conf, genesis_conf) = match self.config {
            Some(config) => (
                config.daemon.clone().unwrap_or(daemon_json::Daemon::DEFAULT),
                {
                    let reader = File::open(config).context("config file {config:?}")?;
                    let c = serde_json::from_reader(reader).context("config file {config:?}")?;
                    Arc::new(GenesisConfig::DaemonJson(c))
                },
            ),
            None => (
                daemon_json::Daemon::DEFAULT,
                node::config::BERKELEY_CONFIG.clone(),
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
                    path: self.snarker_exe_path,
                }),
            },
            p2p: P2pConfig {
                libp2p_port: Some(self.libp2p_port),
                listen_port: self.port,
                identity_pub_key: pub_key,
                initial_peers,
                ask_initial_peers_interval: Duration::from_secs(3600),
                enabled_channels: ChannelId::for_libp2p().collect(),
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

        let keypair = Keypair::ed25519_from_bytes(secret_key.to_bytes())
            .expect("secret key bytes must be valid");

        openmina_core::info!(
            openmina_core::log::system_time();
            peer_id = keypair.public().to_peer_id().to_base58(),
        );

        let p2p_service_ctx = <NodeService as P2pServiceWebrtcWithLibp2p>::init(
            secret_key.clone(),
            P2pTaskSpawner {},
        );

        let mut rpc_service = RpcService::new();

        let http_port = self.port;
        let rpc_sender = RpcSender::new(rpc_service.req_sender().clone());

        // spawn http-server
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        std::thread::Builder::new()
            .name("openmina_http_server".to_owned())
            .spawn(move || runtime.block_on(http_server::run(http_port, rpc_sender)))
            .unwrap();

        let record = self.record;

        let mut ledger = if let Some(path) = &self.additional_ledgers_path {
            LedgerCtx::new_with_additional_snarked_ledgers(path)
        } else {
            LedgerCtx::default()
        };

        // TODO(tizoc): Only used for the current workaround to make staged ledger
        // reconstruction async, can be removed when the ledger services are made async
        ledger.set_event_sender(event_sender.clone());

        let ledger_manager = LedgerManager::spawn(ledger);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .thread_stack_size(64 * 1024 * 1024)
            .build()
            .unwrap();

        runtime.block_on(async move {
            let mut service = NodeService {
                rng: StdRng::seed_from_u64(rng_seed),
                event_sender,
                event_receiver: event_receiver.into(),
                cmd_sender: p2p_service_ctx.webrtc.cmd_sender,
                ledger_manager,
                peers: p2p_service_ctx.webrtc.peers,
                #[cfg(feature = "p2p-libp2p")]
                mio: p2p_service_ctx.mio,
                network: Default::default(),
                block_producer: None,
                keypair,
                rpc: rpc_service,
                snark_worker_sender: None,
                stats: Stats::new(),
                recorder: match record.trim() {
                    "none" => Recorder::None,
                    "state-with-input-actions" => Recorder::only_input_actions(work_dir),
                    _ => panic!("unknown --record strategy"),
                },
                replayer: None,
                invariants_state: Default::default(),
            };

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
                    .initial_state(rng_seed, store.state.get());
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
