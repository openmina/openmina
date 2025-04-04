use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    net::IpAddr,
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use ledger::proofs::provers::BlockProver;
use mina_p2p_messages::v2::{self, NonZeroCurvePoint};
use node::{
    account::AccountSecretKey,
    daemon_json::Daemon,
    p2p::{
        channels::ChannelId, connection::outgoing::P2pConnectionOutgoingInitOpts,
        identity::SecretKey as P2pSecretKey, P2pLimits, P2pMeshsubConfig, P2pTimeouts,
    },
    service::Recorder,
    snark::{get_srs, BlockVerifier, TransactionVerifier, VerifierSRS},
    transition_frontier::{archive::archive_config::ArchiveConfig, genesis::GenesisConfig},
    BlockProducerConfig, GlobalConfig, LedgerConfig, P2pConfig, SnarkConfig, SnarkerConfig,
    SnarkerStrategy, TransitionFrontierConfig,
};
use openmina_core::{consensus::ConsensusConstants, constants::constraint_constants};
use openmina_node_common::{archive::config::ArchiveStorageOptions, p2p::TaskSpawner};
use rand::Rng;

use crate::NodeServiceBuilder;

use super::Node;

pub struct NodeBuilder {
    rng_seed: [u8; 32],
    custom_initial_time: Option<redux::Timestamp>,
    genesis_config: Arc<GenesisConfig>,
    p2p: P2pConfig,
    p2p_sec_key: Option<P2pSecretKey>,
    p2p_is_seed: bool,
    p2p_is_started: bool,
    block_producer: Option<BlockProducerConfig>,
    archive: Option<ArchiveConfig>,
    snarker: Option<SnarkerConfig>,
    service: NodeServiceBuilder,
    verifier_srs: Option<Arc<VerifierSRS>>,
    block_verifier_index: Option<BlockVerifier>,
    work_verifier_index: Option<TransactionVerifier>,
    http_port: Option<u16>,
    daemon_conf: Daemon,
}

impl NodeBuilder {
    pub fn new(
        custom_rng_seed: Option<[u8; 32]>,
        daemon_conf: Daemon,
        genesis_config: Arc<GenesisConfig>,
    ) -> Self {
        let rng_seed = custom_rng_seed.unwrap_or_else(|| {
            let mut seed = [0; 32];
            getrandom::getrandom(&mut seed).unwrap_or_else(|_| {
                seed = rand::thread_rng().gen();
            });
            seed
        });
        Self {
            rng_seed,
            custom_initial_time: None,
            genesis_config,
            p2p: P2pConfig {
                libp2p_port: None,
                listen_port: None,
                // Must be replaced with builder api.
                identity_pub_key: P2pSecretKey::deterministic(0).public_key(),
                initial_peers: Vec::new(),
                external_addrs: Vec::new(),
                enabled_channels: ChannelId::iter_all().collect(),
                peer_discovery: true,
                meshsub: P2pMeshsubConfig {
                    initial_time: Duration::ZERO,
                    ..Default::default()
                },
                timeouts: P2pTimeouts::default(),
                limits: P2pLimits::default().with_max_peers(Some(100)),
            },
            p2p_sec_key: None,
            p2p_is_seed: false,
            p2p_is_started: false,
            block_producer: None,
            archive: None,
            snarker: None,
            service: NodeServiceBuilder::new(rng_seed),
            verifier_srs: None,
            block_verifier_index: None,
            work_verifier_index: None,
            http_port: None,
            daemon_conf,
        }
    }

    /// Set custom initial time. Used for testing.
    pub fn custom_initial_time(&mut self, time: redux::Timestamp) -> &mut Self {
        self.custom_initial_time = Some(time);
        self
    }

    /// If not called, random one will be generated and used instead.
    pub fn p2p_sec_key(&mut self, key: P2pSecretKey) -> &mut Self {
        self.p2p.identity_pub_key = key.public_key();
        self.p2p_sec_key = Some(key);
        self
    }

    pub fn p2p_libp2p_port(&mut self, port: u16) -> &mut Self {
        self.p2p.libp2p_port = Some(port);
        self
    }

    /// Set up node as a seed node.
    pub fn p2p_seed_node(&mut self) -> &mut Self {
        self.p2p_is_seed = true;
        self
    }

    pub fn p2p_no_discovery(&mut self) -> &mut Self {
        self.p2p.peer_discovery = false;
        self
    }

    /// Extend p2p initial peers from an iterable.
    pub fn initial_peers(
        &mut self,
        peers: impl IntoIterator<Item = P2pConnectionOutgoingInitOpts>,
    ) -> &mut Self {
        self.p2p.initial_peers.extend(peers);
        self
    }

    pub fn external_addrs(&mut self, v: impl Iterator<Item = IpAddr>) -> &mut Self {
        self.p2p.external_addrs.extend(v);
        self
    }

    /// Extend p2p initial peers from file.
    pub fn initial_peers_from_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<&mut Self> {
        peers_from_reader(
            &mut self.p2p.initial_peers,
            File::open(&path).context(anyhow::anyhow!(
                "opening peer list file {:?}",
                path.as_ref()
            ))?,
        )
        .context(anyhow::anyhow!(
            "reading peer list file {:?}",
            path.as_ref()
        ))?;

        Ok(self)
    }

    /// Extend p2p initial peers by opening the url.
    pub fn initial_peers_from_url(
        &mut self,
        url: impl reqwest::IntoUrl,
    ) -> anyhow::Result<&mut Self> {
        let url = url.into_url().context("failed to parse peers url")?;
        peers_from_reader(
            &mut self.p2p.initial_peers,
            reqwest::blocking::get(url.clone())
                .context(anyhow::anyhow!("reading peer list url {url}"))?,
        )
        .context(anyhow::anyhow!("reading peer list url {url}"))?;
        Ok(self)
    }

    pub fn p2p_max_peers(&mut self, limit: usize) -> &mut Self {
        self.p2p.limits = self.p2p.limits.with_max_peers(Some(limit));
        self
    }

    /// Override default p2p task spawner.
    pub fn p2p_custom_task_spawner(
        &mut self,
        spawner: impl TaskSpawner,
    ) -> anyhow::Result<&mut Self> {
        let sec_key: P2pSecretKey = self.p2p_sec_key.clone().ok_or_else(|| anyhow::anyhow!("before calling `with_p2p_custom_task_spawner` method, p2p secret key needs to be set with `with_p2p_sec_key`."))?;
        self.service
            .p2p_init_with_custom_task_spawner(sec_key, spawner);
        self.p2p_is_started = true;
        Ok(self)
    }

    /// Set up block producer.
    pub fn block_producer(
        &mut self,
        key: AccountSecretKey,
        provers: Option<BlockProver>,
    ) -> &mut Self {
        let config = BlockProducerConfig {
            pub_key: key.public_key().into(),
            custom_coinbase_receiver: None,
            proposed_protocol_version: None,
        };
        self.block_producer = Some(config);
        self.service.block_producer_init(key, provers);
        self
    }

    /// Set up block producer using keys from file.
    pub fn block_producer_from_file(
        &mut self,
        path: impl AsRef<Path>,
        password: &str,
        provers: Option<BlockProver>,
    ) -> anyhow::Result<&mut Self> {
        let key = AccountSecretKey::from_encrypted_file(path, password)
            .context("Failed to decrypt secret key file")?;
        Ok(self.block_producer(key, provers))
    }

    pub fn archive(&mut self, options: ArchiveStorageOptions, work_dir: String) -> &mut Self {
        self.archive = Some(ArchiveConfig::new(work_dir.clone()));
        self.service.archive_init(options, work_dir.clone());
        self
    }

    /// Receive block producer's coinbase reward to another account.
    pub fn custom_coinbase_receiver(
        &mut self,
        addr: NonZeroCurvePoint,
    ) -> anyhow::Result<&mut Self> {
        let bp = self.block_producer.as_mut().ok_or_else(|| {
            anyhow::anyhow!(
                "can't set custom_coinbase_receiver when block producer is not initialized."
            )
        })?;
        bp.custom_coinbase_receiver = Some(addr);
        Ok(self)
    }

    pub fn custom_block_producer_config(
        &mut self,
        config: BlockProducerConfig,
    ) -> anyhow::Result<&mut Self> {
        *self.block_producer.as_mut().ok_or_else(|| {
            anyhow::anyhow!("block producer not initialized! Call `block_producer` function first.")
        })? = config;
        Ok(self)
    }

    pub fn snarker(
        &mut self,
        sec_key: AccountSecretKey,
        fee: u64,
        strategy: SnarkerStrategy,
    ) -> &mut Self {
        let config = SnarkerConfig {
            public_key: sec_key.public_key(),
            fee: v2::CurrencyFeeStableV1(v2::UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                fee.into(),
            )),
            strategy,
            auto_commit: true,
        };
        self.snarker = Some(config);
        self
    }

    /// Set verifier srs. If not set, default will be used.
    pub fn verifier_srs(&mut self, srs: Arc<VerifierSRS>) -> &mut Self {
        self.verifier_srs = Some(srs);
        self
    }

    pub fn block_verifier_index(&mut self, index: BlockVerifier) -> &mut Self {
        self.block_verifier_index = Some(index);
        self
    }

    pub fn work_verifier_index(&mut self, index: TransactionVerifier) -> &mut Self {
        self.work_verifier_index = Some(index);
        self
    }

    pub fn gather_stats(&mut self) -> &mut Self {
        self.service.gather_stats();
        self
    }

    pub fn record(&mut self, recorder: Recorder) -> &mut Self {
        self.service.record(recorder);
        self
    }

    pub fn http_server(&mut self, port: u16) -> &mut Self {
        self.http_port = Some(port);
        self.service.http_server_init(port);
        self
    }

    pub fn build(mut self) -> anyhow::Result<Node> {
        let p2p_sec_key = self.p2p_sec_key.clone().unwrap_or_else(P2pSecretKey::rand);
        self.p2p_sec_key(p2p_sec_key.clone());
        if self.p2p.initial_peers.is_empty() && !self.p2p_is_seed {
            self.p2p.initial_peers = default_peers();
        }

        self.p2p.initial_peers = self
            .p2p
            .initial_peers
            .into_iter()
            .filter(|opts| *opts.peer_id() != p2p_sec_key.public_key().peer_id())
            .filter_map(|opts| match opts {
                P2pConnectionOutgoingInitOpts::LibP2P(mut opts) => {
                    opts.host = opts.host.resolve()?;
                    Some(P2pConnectionOutgoingInitOpts::LibP2P(opts))
                }
                x => Some(x),
            })
            .collect();

        let srs = self.verifier_srs.unwrap_or_else(get_srs);
        let block_verifier_index = self
            .block_verifier_index
            .unwrap_or_else(BlockVerifier::make);
        let work_verifier_index = self
            .work_verifier_index
            .unwrap_or_else(TransactionVerifier::make);

        let initial_time = self
            .custom_initial_time
            .unwrap_or_else(redux::Timestamp::global_now);
        self.p2p.meshsub.initial_time = initial_time
            .checked_sub(redux::Timestamp::ZERO)
            .unwrap_or_default();

        let protocol_constants = self.genesis_config.protocol_constants()?;
        let consensus_consts =
            ConsensusConstants::create(constraint_constants(), &protocol_constants);

        // build config
        let node_config = node::Config {
            global: GlobalConfig {
                build: node::BuildEnv::get().into(),
                snarker: self.snarker,
                consensus_constants: consensus_consts.clone(),
                testing_run: false,
                client_port: self.http_port,
            },
            p2p: self.p2p,
            ledger: LedgerConfig {},
            snark: SnarkConfig {
                block_verifier_index,
                block_verifier_srs: srs.clone(),
                work_verifier_index,
                work_verifier_srs: srs,
            },
            transition_frontier: TransitionFrontierConfig::new(self.genesis_config),
            block_producer: self.block_producer,
            archive: self.archive,
            tx_pool: ledger::transaction_pool::Config {
                trust_system: (),
                pool_max_size: self.daemon_conf.tx_pool_max_size(),
                slot_tx_end: self.daemon_conf.slot_tx_end(),
            },
        };

        // build service
        let mut service = self.service;
        service.ledger_init();

        if !self.p2p_is_started {
            service.p2p_init(p2p_sec_key);
        }

        let service = service.build()?;
        let state = node::State::new(node_config, &consensus_consts, initial_time);

        Ok(Node::new(self.rng_seed, state, service, None))
    }
}

fn default_peers() -> Vec<P2pConnectionOutgoingInitOpts> {
    openmina_core::NetworkConfig::global()
        .default_peers
        .iter()
        .map(|s| s.parse().unwrap())
        .collect()
}

fn peers_from_reader(
    peers: &mut Vec<P2pConnectionOutgoingInitOpts>,
    read: impl Read,
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
