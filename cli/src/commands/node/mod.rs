use std::ffi::OsString;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

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
use node::p2p::service_impl::webrtc::P2pServiceCtx;
use node::p2p::service_impl::webrtc_with_libp2p::{self, P2pServiceWebrtcWithLibp2p};
use node::p2p::{P2pConfig, P2pEvent};
use node::service::{Recorder, Service};
use node::snark::{get_srs, get_verifier_index, VerifierKind};
use node::stats::Stats;
use node::{
    BuildEnv, Config, GlobalConfig, LedgerConfig, SnarkConfig, SnarkerConfig, State,
    TransitionFrontierConfig,
};

use openmina_node_native::rpc::RpcService;
use openmina_node_native::{http_server, tracing, NodeService, P2pTaskSpawner, RpcSender};

/// Openmina node
#[derive(Debug, clap::Args)]
pub struct Node {
    #[arg(long, short = 'd', default_value = "~/.openmina")]
    pub work_dir: String,

    /// Chain ID
    #[arg(long, short = 'i', env)]
    pub chain_id: String,

    /// Peer secret key
    #[arg(long, short = 's', env = "OPENMINA_P2P_SEC_KEY")]
    pub p2p_secret_key: Option<SecretKey>,

    /// Port to listen to
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,

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

    /// Snark fee, in Mina
    #[arg(long, env, default_value_t = 1_000_000)]
    pub snarker_fee: u64,

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
        "/dns4/seed-1.berkeley.o1test.net/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        "/dns4/seed-2.berkeley.o1test.net/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        "/dns4/seed-3.berkeley.o1test.net/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",

        "/dns4/webrtc2.webnode.openmina.com/tcp/8306/p2p/12D3KooWFpqySZDHx7k5FMjdwmrU3TLhDbdADECCautBcEGtG4fr",
        "/dns4/webrtc2.webnode.openmina.com/tcp/8307/p2p/12D3KooWJBeXosFxdBwe2mbKRjgRG69ERaUTpS9qo9NRkoE8kBpj",

        "/ip4/65.21.123.88/tcp/8302/p2p/12D3KooWLKSM9oHWU7qwL7Ci75wunkjXpRmK6j5xq527zGw554AF",
        "/ip4/65.109.123.166/tcp/8302/p2p/12D3KooWGc9vwL9DUvoLdBFPSQGCT2QTULskzhmXcn8zg2j3jdFF",
        "/ip4/176.9.64.21/tcp/8302/p2p/12D3KooWG9owTshte2gR3joP4sgwAfdoV9bQeeB5y9R3QUprKLdJ",
        "/ip4/35.238.71.15/tcp/65454/p2p/12D3KooWHdUVpCZ9KcF5hNBrwf2uy7BaPDKrxyHJAaM5epJgQucX",
        "/ip4/35.224.199.118/tcp/25493/p2p/12D3KooWGbjV7ptpzLu4BuykKfEsF4ebLyR8gZAMUissMToKGVDQ",
        "/ip4/35.193.28.252/tcp/37470/p2p/12D3KooWFcCiQqrzBVLEkPdpkHDgWr6AkSMthT96agKYBBVuRhHg",
        "/ip4/142.132.154.120/tcp/58654/p2p/12D3KooWMPxTu24mCpi3TwmkU4fJk7a8TQ4agFZeTHQRi8KCc3nj",
        "/ip4/65.108.121.245/tcp/8302/p2p/12D3KooWGQ4g2eY44n5JLqymi8KC55GbnujAFeXNQrmNKSq4NYrv",
        "/ip4/65.109.123.173/tcp/8302/p2p/12D3KooWMd8K8FFd76cacUEE6sSzUPr7wj71TvMqGdFSgrpv923k",
        "/ip4/65.109.123.235/tcp/8302/p2p/12D3KooWBK3vz1inMubXCUeDF4Min6eG5418toceG8QvNPWRW1Gz",
        "/ip4/34.172.208.246/tcp/46203/p2p/12D3KooWNafCBobFGSdJyYonvSCB5KDzW3JZYnVBF6q22yhcXGjM",
        "/ip4/34.29.40.184/tcp/7528/p2p/12D3KooWJoVjUsnDosW3Ae78V4CSf5SSe9Wyetr5DxutmMMfwdp8",
        "/ip4/34.122.249.235/tcp/55894/p2p/12D3KooWMpGyhYHbzVeqYnxGHQQYmQNtYcoMLLZZmYRPvAJKxXXm",
        "/ip4/35.232.20.138/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        "/ip4/88.198.230.168/tcp/8302/p2p/12D3KooWGA7AS91AWNtGEBCBk64kgirtTiyaXDTyDtKPTjpefNL9",
        "/ip4/35.224.199.118/tcp/10360/p2p/12D3KooWDnC4XrJzas3heuz4LUehZjf2WJyfob2XEodrYL3soaf4",
        "/ip4/34.123.4.144/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        "/ip4/34.170.114.52/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        "/ip4/34.172.208.246/tcp/54351/p2p/12D3KooWEhCm8FVcqZSkXKNhuBPmsEfJGeqSmUxNQhpemZkENfik",
        "/ip4/34.29.161.11/tcp/10946/p2p/12D3KooWCntSrMqSiovXcVfMZ56aYbzpZoh4mi7gJJNiZBmzXrpa",
        "/ip4/35.238.71.15/tcp/23676/p2p/12D3KooWENsfMszNYBRfHZJUEAvXKThmZU3nijWVbLivq33AE2Vk",
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

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let _rt_guard = rt.enter();
        let mut rng = ThreadRng::default();

        let secret_key = self.p2p_secret_key.unwrap_or_else(|| {
            let bytes = rng.gen();
            SecretKey::from_bytes(bytes)
        });
        let pub_key = secret_key.public_key();

        let work_dir = shellexpand::full(&self.work_dir).unwrap().into_owned();
        let rng_seed = rng.next_u64();
        let srs: Arc<_> = get_srs().into();
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
                    auto_commit: true,
                    path: self.snarker_exe_path,
                }),
            },
            p2p: P2pConfig {
                identity_pub_key: pub_key,
                initial_peers: self.peers,
                max_peers: 100,
                enabled_channels: ChannelId::iter_all().collect(),
            },
            transition_frontier: TransitionFrontierConfig::default(),
        };
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let (p2p_event_sender, mut rx) = mpsc::unbounded_channel::<P2pEvent>();

        let webrtc_with_libp2p::P2pServiceCtx {
            libp2p,
            webrtc: P2pServiceCtx { cmd_sender, peers },
        } = <NodeService as P2pServiceWebrtcWithLibp2p>::init(
            secret_key,
            self.chain_id,
            p2p_event_sender.clone(),
            P2pTaskSpawner {},
        );

        let ev_sender = event_sender.clone();
        tokio::spawn(async move {
            while let Some(v) = rx.recv().await {
                if let Err(_) = ev_sender.send(v.into()) {
                    break;
                }
            }
        });

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
            .spawn(move || {
                let local_set = tokio::task::LocalSet::new();
                local_set.block_on(&runtime, http_server::run(http_port, rpc_sender))
            })
            .unwrap();

        // spawn state machine thread.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .thread_stack_size(64 * 1024 * 1024)
            .build()
            .unwrap();
        let (redux_exited_tx, redux_exited) = tokio::sync::oneshot::channel();
        let record = self.record;
        std::thread::Builder::new()
            .name("openmina_redux".to_owned())
            .spawn(move || {
                let ledger = if let Some(path) = &self.additional_ledgers_path {
                    LedgerCtx::new_with_additional_snarked_ledgers(path)
                } else {
                    LedgerCtx::default()
                };

                let local_set = tokio::task::LocalSet::new();
                local_set.block_on(&runtime, async move {
                    let service = NodeService {
                        rng: StdRng::seed_from_u64(rng_seed),
                        event_sender,
                        p2p_event_sender,
                        event_receiver: event_receiver.into(),
                        cmd_sender,
                        ledger,
                        peers,
                        libp2p,
                        rpc: rpc_service,
                        snark_worker_sender: None,
                        stats: Stats::new(),
                        recorder: match record.trim() {
                            "none" => Recorder::None,
                            "state-with-input-actions" => Recorder::only_input_actions(work_dir),
                            _ => panic!("unknown --record strategy"),
                        },
                        replayer: None,
                    };
                    let state = State::new(config);
                    let mut node = ::node::Node::new(state, service, None);

                    // record initial state.
                    {
                        let store = node.store_mut();
                        store.service.recorder().initial_state(rng_seed, store.state.get());
                    }

                    node
                        .store_mut()
                        .dispatch(EventSourceProcessEventsAction {});
                    loop {
                        node
                            .store_mut()
                            .dispatch(EventSourceWaitForEventsAction {});

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
                let _ = redux_exited_tx.send(());
            })
            .unwrap();

        rt.block_on(redux_exited)
            .expect("state machine task crashed!");
        Ok(())
    }
}
