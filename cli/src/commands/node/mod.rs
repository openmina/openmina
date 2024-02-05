use std::ffi::OsString;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use libp2p_identity::Keypair;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};
use rand::prelude::*;

use tokio::select;

use node::account::AccountPublicKey;
use node::core::channels::mpsc;
use node::core::log::inner::Level;
use node::event_source::{
    EventSourceProcessEventsAction, EventSourceWaitForEventsAction, EventSourceWaitTimeoutAction,
};
use node::ledger::LedgerCtx;
use node::p2p::channels::ChannelId;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use node::p2p::identity::SecretKey;
use node::p2p::service_impl::webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p;
use node::p2p::P2pConfig;
use node::service::{Recorder, Service};
use node::snark::{get_srs, get_verifier_index, VerifierKind};
use node::stats::Stats;
use node::{
    BuildEnv, Config, GlobalConfig, LedgerConfig, SnarkConfig, SnarkerConfig, SnarkerStrategy,
    State, TransitionFrontierConfig,
};

use openmina_node_native::rpc::RpcService;
use openmina_node_native::{http_server, tracing, NodeService, P2pTaskSpawner, RpcSender};

/// Openmina node
#[derive(Debug, clap::Args)]
pub struct Node {
    #[arg(long, short = 'd', default_value = "~/.openmina")]
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

    #[arg(long, short = 'P', alias = "peer", num_args = 0.., default_values_t = default_peers(), env, value_delimiter = ' ')]
    pub peers: Vec<P2pConnectionOutgoingInitOpts>,

    /// Run Snark Worker.
    ///
    /// Pass snarker public key as an argument.
    #[arg(long, env)]
    pub run_snarker: Option<AccountPublicKey>,

    // /// Enable block producer with this key
    // #[arg(long, env)]
    // pub producer_key: Option<String>,
    /// Snark fee, in Mina
    #[arg(long, env, default_value_t = 1_000_000)]
    pub snarker_fee: u64,

    #[arg(long, env, default_value = "seq")]
    pub snarker_strategy: SnarkerStrategy,

    /// Mina snark worker path
    #[arg(long, env, default_value = "cli/bin/snark-worker")]
    pub snarker_exe_path: OsString,

    #[arg(long, default_value = "none")]
    pub record: String,

    #[arg(long, default_value = "none")]
    pub additional_ledgers_path: Option<PathBuf>,
}

fn default_peers() -> Vec<P2pConnectionOutgoingInitOpts> {
    [
        // "/2ajh5CpZCHdv7tmMrotVnLjQXuhcuCzqKosdDmvN3tNTScw2fsd/http/65.109.110.75/10000",

        // "/dns4/seed-1.berkeley.o1test.net/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        "/ip4/34.170.114.52/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        // "/dns4/seed-2.berkeley.o1test.net/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        // "/dns4/seed-3.berkeley.o1test.net/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",

        // "/dns4/webrtc2.webnode.openmina.com/tcp/443/p2p/12D3KooWFpqySZDHx7k5FMjdwmrU3TLhDbdADECCautBcEGtG4fr",
        // "/dns4/webrtc2.webnode.openmina.com/tcp/4431/p2p/12D3KooWJBeXosFxdBwe2mbKRjgRG69ERaUTpS9qo9NRkoE8kBpj",

        // "/ip4/78.27.236.28/tcp/8302/p2p/12D3KooWDLNXPq28An4s2QaPZX5ftem1AfaCWuxHHJq97opeWxLy",
    ]
    .into_iter()
    .map(|s| s.parse().unwrap())
    .collect()
}

impl Node {
    pub fn run(self) -> Result<(), crate::CommandError> {
        tracing::initialize(self.verbosity);

        if let Err(ref e) = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get().max(2) - 1)
            .thread_name(|i| format!("openmina_rayon_{i}"))
            .build_global()
        {
            openmina_core::log::error!(openmina_core::log::system_time();
                    kind = "FatalError",
                    summary = "failed to initialize threadpool",
                    error = format!("{:?}", e));
            panic!("FatalError: {:?}", e);
        }

        let mut rng = ThreadRng::default();

        let secret_key = self.p2p_secret_key.unwrap_or_else(|| {
            let bytes = rng.gen();
            SecretKey::from_bytes(bytes)
        });
        let pub_key = secret_key.public_key();

        // let block_producer: Option<BlockProducerConfig> =
        //     self.producer_key.clone().map(|producer_key| {
        //         let compressed_pub_key = keypair_from_bs58_string(&producer_key)
        //             .public
        //             .into_compressed();
        //         BlockProducerConfig {
        //             pub_key: NonZeroCurvePoint::from(NonZeroCurvePointUncompressedStableV1 {
        //                 x: compressed_pub_key.x.into(),
        //                 is_odd: compressed_pub_key.is_odd,
        //             }),
        //             custom_coinbase_receiver: None,
        //             proposed_protocol_version: None,
        //         }
        //     });

        let work_dir = shellexpand::full(&self.work_dir).unwrap().into_owned();
        let rng_seed = rng.next_u64();
        let srs: Arc<_> = get_srs();
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
                initial_peers: self.peers,
                max_peers: 100,
                ask_initial_peers_interval: Duration::from_secs(3600),
                enabled_channels: ChannelId::iter_all().collect(),
            },
            transition_frontier: TransitionFrontierConfig::default(),
            block_producer: None,
        };
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let keypair = Keypair::ed25519_from_bytes(secret_key.to_bytes())
            .expect("secret key bytes must be valid");

        let p2p_service_ctx = <NodeService as P2pServiceWebrtcWithLibp2p>::init(
            Some(self.libp2p_port),
            secret_key.clone(),
            "3c41383994b87449625df91769dff7b507825c064287d30fada9286f3f1cb15e".to_owned(),
            event_sender.clone(),
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

        let ledger = if let Some(path) = &self.additional_ledgers_path {
            LedgerCtx::new_with_additional_snarked_ledgers(path)
        } else {
            LedgerCtx::default()
        };

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .thread_stack_size(64 * 1024 * 1024)
            .build()
            .unwrap();

        runtime.block_on(async move {
            let service = NodeService {
                rng: StdRng::seed_from_u64(rng_seed),
                event_sender,
                event_receiver: event_receiver.into(),
                cmd_sender: p2p_service_ctx.webrtc.cmd_sender,
                ledger,
                peers: p2p_service_ctx.webrtc.peers,
                #[cfg(feature = "p2p-libp2p")]
                libp2p: p2p_service_ctx.libp2p,
                #[cfg(not(feature = "p2p-libp2p"))]
                mio: p2p_service_ctx.mio,
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
            let state = State::new(config);
            let mut node = ::node::Node::new(state, service, None);

            // record initial state.
            {
                let store = node.store_mut();
                store
                    .service
                    .recorder()
                    .initial_state(rng_seed, store.state.get());
            }

            node.store_mut().dispatch(EventSourceProcessEventsAction {});
            loop {
                node.store_mut().dispatch(EventSourceWaitForEventsAction {});

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
                            node.store_mut().dispatch(EventSourceProcessEventsAction {});
                        }
                    }
                    req = rpc_req_fut => {
                        node.store_mut().service.process_rpc_request(req);
                        // TODO(binier): remove loop once ledger communication is async.
                        while let Ok(req) = node.store_mut().service.rpc.req_receiver().try_recv() {
                            node.store_mut().service.process_rpc_request(req);
                        }
                    }
                    _ = timeout => {
                        node.store_mut().dispatch(EventSourceWaitTimeoutAction {});
                    }
                }
            }
        });

        Ok(())
    }
}
