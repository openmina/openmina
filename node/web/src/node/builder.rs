use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use mina_p2p_messages::v2::{self, NonZeroCurvePoint};
use node::{
    account::AccountSecretKey,
    core::{consensus::ConsensusConstants, constants::constraint_constants},
    p2p::{
        channels::ChannelId, connection::outgoing::P2pConnectionOutgoingInitOpts,
        identity::SecretKey as P2pSecretKey, P2pLimits, P2pTimeouts,
    },
    snark::{get_srs, get_verifier_index, VerifierIndex, VerifierKind, VerifierSRS},
    transition_frontier::genesis::GenesisConfig,
    BlockProducerConfig, GlobalConfig, LedgerConfig, P2pConfig, SnarkConfig, SnarkerConfig,
    SnarkerStrategy, TransitionFrontierConfig,
};
use openmina_node_common::{p2p::TaskSpawner, NodeServiceCommonBuilder};
use rand::Rng;

use super::{Node, P2pTaskSpawner};

pub struct NodeBuilder {
    rng_seed: [u8; 32],
    custom_initial_time: Option<redux::Timestamp>,
    genesis_config: Arc<GenesisConfig>,
    p2p_sec_key: Option<P2pSecretKey>,
    p2p_is_seed: bool,
    p2p_no_discovery: bool,
    p2p_is_started: bool,
    initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    block_producer: Option<BlockProducerConfig>,
    snarker: Option<SnarkerConfig>,
    service: NodeServiceCommonBuilder,
    verifier_srs: Option<Arc<Mutex<VerifierSRS>>>,
    block_verifier_index: Option<Arc<VerifierIndex>>,
    work_verifier_index: Option<Arc<VerifierIndex>>,
}

impl NodeBuilder {
    pub fn new(custom_rng_seed: Option<[u8; 32]>, genesis_config: Arc<GenesisConfig>) -> Self {
        let rng_seed = custom_rng_seed.unwrap_or_else(|| rand::thread_rng().gen());
        Self {
            rng_seed,
            custom_initial_time: None,
            genesis_config,
            p2p_sec_key: None,
            p2p_is_seed: false,
            p2p_no_discovery: false,
            p2p_is_started: false,
            initial_peers: Vec::new(),
            block_producer: None,
            snarker: None,
            service: NodeServiceCommonBuilder::new(rng_seed),
            verifier_srs: None,
            block_verifier_index: None,
            work_verifier_index: None,
        }
    }

    /// Set custom initial time. Used for testing.
    pub fn custom_initial_time(&mut self, time: redux::Timestamp) -> &mut Self {
        self.custom_initial_time = Some(time);
        self
    }

    /// If not called, random one will be generated and used instead.
    pub fn p2p_sec_key(&mut self, key: P2pSecretKey) -> &mut Self {
        assert!(self.p2p_sec_key.is_none());
        self.p2p_sec_key = Some(key);
        self
    }

    /// Set up node as a seed node.
    pub fn p2p_seed_node(&mut self) -> &mut Self {
        self.p2p_is_seed = true;
        self
    }

    pub fn p2p_no_discovery(&mut self) -> &mut Self {
        self.p2p_no_discovery = true;
        self
    }

    /// Extend p2p initial peers from an iterable.
    pub fn initial_peers(
        &mut self,
        peers: impl IntoIterator<Item = P2pConnectionOutgoingInitOpts>,
    ) -> &mut Self {
        self.initial_peers.extend(peers);
        self
    }

    // /// Extend p2p initial peers by opening the url.
    // pub async fn initial_peers_from_url(
    //     &mut self,
    //     url: impl reqwest::IntoUrl,
    // ) -> anyhow::Result<&mut Self> {
    //     let url = url.into_url().context("failed to parse peers url")?;
    //     peers_from_reader(
    //         &mut self.initial_peers,
    //         reqwest::get(url.clone()).await
    //             .context(anyhow::anyhow!("reading peer list url {url}"))?,
    //     )
    //     .context(anyhow::anyhow!("reading peer list url {url}"))?;
    //     Ok(self)
    // }

    /// Override default p2p task spawner.
    pub fn p2p_custom_task_spawner(
        &mut self,
        spawner: impl TaskSpawner,
    ) -> anyhow::Result<&mut Self> {
        let sec_key = self.p2p_sec_key.get_or_insert_with(P2pSecretKey::rand);
        self.service.p2p_init(sec_key.clone(), spawner);
        self.p2p_is_started = true;
        Ok(self)
    }

    /// Set up block producer.
    pub fn block_producer(&mut self, key: AccountSecretKey) -> &mut Self {
        let config = BlockProducerConfig {
            pub_key: key.public_key().into(),
            custom_coinbase_receiver: None,
            proposed_protocol_version: None,
        };
        self.block_producer = Some(config);
        self.service.block_producer_init(key);
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
    pub fn verifier_srs(&mut self, srs: Arc<Mutex<VerifierSRS>>) -> &mut Self {
        self.verifier_srs = Some(srs);
        self
    }

    pub fn block_verifier_index(&mut self, index: Arc<VerifierIndex>) -> &mut Self {
        self.block_verifier_index = Some(index);
        self
    }

    pub fn work_verifier_index(&mut self, index: Arc<VerifierIndex>) -> &mut Self {
        self.work_verifier_index = Some(index);
        self
    }

    pub fn gather_stats(&mut self) -> &mut Self {
        self.service.gather_stats();
        self
    }

    pub fn build(self) -> anyhow::Result<Node> {
        let p2p_sec_key = self.p2p_sec_key.unwrap_or_else(P2pSecretKey::rand);
        let initial_peers = if self.initial_peers.is_empty() && !self.p2p_is_seed {
            default_peers()
        } else {
            self.initial_peers
        };

        let srs = self.verifier_srs.unwrap_or_else(get_srs);
        let block_verifier_index = self
            .block_verifier_index
            .unwrap_or_else(|| get_verifier_index(VerifierKind::Blockchain).into());
        let work_verifier_index = self
            .work_verifier_index
            .unwrap_or_else(|| get_verifier_index(VerifierKind::Transaction).into());

        let initial_time = self
            .custom_initial_time
            .unwrap_or_else(redux::Timestamp::global_now);

        // build config
        let node_config = node::Config {
            global: GlobalConfig {
                build: node::BuildEnv::get().into(),
                snarker: self.snarker,
            },
            p2p: P2pConfig {
                libp2p_port: None,
                listen_port: None,
                identity_pub_key: p2p_sec_key.public_key(),
                initial_peers,
                ask_initial_peers_interval: Duration::from_secs(3600),
                enabled_channels: ChannelId::iter_all().collect(),
                peer_discovery: !self.p2p_no_discovery,
                initial_time: initial_time
                    .checked_sub(redux::Timestamp::ZERO)
                    .unwrap_or_default(),
                timeouts: P2pTimeouts::default(),
                limits: P2pLimits::default().with_max_peers(Some(100)),
            },
            ledger: LedgerConfig {},
            snark: SnarkConfig {
                block_verifier_index,
                block_verifier_srs: srs.clone(),
                work_verifier_index,
                work_verifier_srs: srs,
            },
            transition_frontier: TransitionFrontierConfig::new(self.genesis_config),
            block_producer: self.block_producer,
            tx_pool: ledger::transaction_pool::Config {
                trust_system: (),
                pool_max_size: node::daemon_json::Daemon::DEFAULT.tx_pool_max_size(),
                slot_tx_end: node::daemon_json::Daemon::DEFAULT.slot_tx_end(),
            },
        };

        // build service
        let mut service = self.service;
        service.ledger_init();

        if !self.p2p_is_started {
            service.p2p_init(p2p_sec_key, P2pTaskSpawner {});
        }

        let protocol_constants = node_config
            .transition_frontier
            .genesis
            .protocol_constants()?;
        let consensus_consts =
            ConsensusConstants::create(constraint_constants(), &protocol_constants);

        let service = service.build()?;
        let state = node::State::new(node_config, &consensus_consts, initial_time);

        Ok(Node::new(self.rng_seed, state, service, None))
    }
}

fn default_peers() -> Vec<P2pConnectionOutgoingInitOpts> {
    ["/2cBFzmUmkYgMUrxdv5S2Udyv8eiuhokAFS4WnYfHiAJLWoQ3yL9/http/webrtc3.webnode.openmina.com/3000",
    "/2aevrztkwUrbPDP85ZZHkNYM6qha2GYT9sUwPhttKAKzqQdF4nV/http/localhost/3000"]
        .into_iter()
        .map(|s| s.parse().unwrap())
        .collect()
}

// fn peers_from_reader(
//     peers: &mut Vec<P2pConnectionOutgoingInitOpts>,
//     read: impl Read,
// ) -> anyhow::Result<()> {
//     let read = BufReader::new(read);
//     for line in read.lines() {
//         let line = line.context("reading line")?;
//         let l = line.trim();
//         if !l.is_empty() {
//             peers.push(l.parse().context(anyhow::anyhow!("parsing entry"))?);
//         }
//     }
//     Ok(())
// }
