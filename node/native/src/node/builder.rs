use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Context;
use mina_p2p_messages::v2::{self, NonZeroCurvePoint};
use node::{
    account::AccountSecretKey,
    p2p::{
        channels::ChannelId, connection::outgoing::P2pConnectionOutgoingInitOpts,
        identity::SecretKey as P2pSecretKey, P2pLimits, P2pTimeouts,
    },
    service::Recorder,
    snark::{get_srs, get_verifier_index, VerifierIndex, VerifierKind, VerifierSRS},
    transition_frontier::genesis::GenesisConfig,
    BlockProducerConfig, GlobalConfig, LedgerConfig, P2pConfig, SnarkConfig, SnarkerConfig,
    SnarkerStrategy, TransitionFrontierConfig,
};
use openmina_node_common::p2p::TaskSpawner;
use rand::Rng;

use crate::NodeServiceBuilder;

use super::Node;

pub struct NodeBuilder {
    rng_seed: [u8; 32],
    custom_initial_time: Option<redux::Timestamp>,
    genesis_config: Arc<GenesisConfig>,
    p2p_sec_key: Option<P2pSecretKey>,
    p2p_libp2p_port: Option<u16>,
    p2p_is_seed: bool,
    p2p_no_discovery: bool,
    p2p_is_started: bool,
    initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    block_producer: Option<BlockProducerConfig>,
    snarker: Option<SnarkerConfig>,
    service: NodeServiceBuilder,
    verifier_srs: Option<Arc<Mutex<VerifierSRS>>>,
    block_verifier_index: Option<Arc<VerifierIndex>>,
    work_verifier_index: Option<Arc<VerifierIndex>>,
    http_port: Option<u16>,
}

impl NodeBuilder {
    pub fn new(custom_rng_seed: Option<[u8; 32]>, genesis_config: Arc<GenesisConfig>) -> Self {
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
            p2p_sec_key: None,
            p2p_libp2p_port: None,
            p2p_is_seed: false,
            p2p_no_discovery: false,
            p2p_is_started: false,
            initial_peers: Vec::new(),
            block_producer: None,
            snarker: None,
            service: NodeServiceBuilder::new(rng_seed),
            verifier_srs: None,
            block_verifier_index: None,
            work_verifier_index: None,
            http_port: None,
        }
    }

    /// Set custom initial time. Used for testing.
    pub fn custom_initial_time(&mut self, time: redux::Timestamp) -> &mut Self {
        self.custom_initial_time = Some(time);
        self
    }

    /// If not called, random one will be generated and used instead.
    pub fn p2p_sec_key(&mut self, key: P2pSecretKey) -> &mut Self {
        self.p2p_sec_key = Some(key);
        self
    }

    pub fn p2p_libp2p_port(&mut self, port: u16) -> &mut Self {
        self.p2p_libp2p_port = Some(port);
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

    /// Extend p2p initial peers from file.
    pub fn initial_peers_from_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<&mut Self> {
        peers_from_reader(
            &mut self.initial_peers,
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
            &mut self.initial_peers,
            reqwest::blocking::get(url.clone())
                .context(anyhow::anyhow!("reading peer list url {url}"))?,
        )
        .context(anyhow::anyhow!("reading peer list url {url}"))?;
        Ok(self)
    }

    /// Override default p2p task spawner.
    pub fn p2p_custom_task_spawner(
        &mut self,
        spawner: impl TaskSpawner,
    ) -> anyhow::Result<&mut Self> {
        let sec_key = self.p2p_sec_key.clone().ok_or_else(|| anyhow::anyhow!("before calling `with_p2p_custom_task_spawner` method, p2p secret key needs to be set with `with_p2p_sec_key`."))?;
        self.service
            .p2p_init_with_custom_task_spawner(sec_key, spawner);
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

    /// Set up block producer using keys from file.
    pub fn block_producer_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<&mut Self> {
        let key = AccountSecretKey::from_encrypted_file(path)
            .context("Failed to decrypt secret key file")?;
        Ok(self.block_producer(key))
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

    pub fn record(&mut self, recorder: Recorder) -> &mut Self {
        self.service.record(recorder);
        self
    }

    pub fn http_server(&mut self, port: u16) -> &mut Self {
        self.http_port = Some(port);
        self.service.http_server_init(port);
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
                libp2p_port: self.p2p_libp2p_port,
                listen_port: self.http_port.unwrap_or(3000),
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
        };

        // build service
        let mut service = self.service;
        service.ledger_init();

        if !self.p2p_is_started {
            service.p2p_init(p2p_sec_key);
        }

        let service = service.build()?;
        let state = node::State::new(node_config, initial_time);

        Ok(Node::new(self.rng_seed, state, service, None))
    }
}

fn default_peers() -> Vec<P2pConnectionOutgoingInitOpts> {
    [
        // "/2ajh5CpZCHdv7tmMrotVnLjQXuhcuCzqKosdDmvN3tNTScw2fsd/http/65.109.110.75/10000",

        // Devnet
        // "/dns4/seed-1.devnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        // "/dns4/seed-2.devnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        // "/dns4/seed-3.devnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        "/ip4/34.48.73.58/tcp/10003/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        "/ip4/35.245.82.250/tcp/10003/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        "/ip4/34.118.163.79/tcp/10003/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        //
        // "/dns4/webrtc2.webnode.openmina.com/tcp/443/p2p/12D3KooWFpqySZDHx7k5FMjdwmrU3TLhDbdADECCautBcEGtG4fr",
        // "/dns4/webrtc2.webnode.openmina.com/tcp/4431/p2p/12D3KooWJBeXosFxdBwe2mbKRjgRG69ERaUTpS9qo9NRkoE8kBpj",

        // "/ip4/78.27.236.28/tcp/8302/p2p/12D3KooWDLNXPq28An4s2QaPZX5ftem1AfaCWuxHHJq97opeWxLy",
    ]
    .into_iter()
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