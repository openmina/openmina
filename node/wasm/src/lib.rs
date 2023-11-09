use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures::channel::{mpsc, oneshot};
use futures::{select_biased, FutureExt, SinkExt, StreamExt};
use gloo_utils::format::JsValueSerdeExt;
use js_sys::{Promise, Uint8Array};
use libp2p::futures;
use libp2p::multiaddr::{Multiaddr, Protocol as MultiaddrProtocol};
use libp2p::wasm_ext::ffi::ManualConnector as JsManualConnector;
use mina_p2p_messages::bigint::BigInt;
use mina_p2p_messages::v2::{
    MinaBaseAccountIdDigestStableV1, MinaBaseUserCommandStableV2,
    NetworkPoolTransactionPoolDiffVersionedStableV2, NonZeroCurvePoint, StateHash, TokenIdKeyHash,
};
use mina_signer::PubKey;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

use lib::event_source::{
    Event, EventSourceProcessEventsAction, EventSourceWaitForEventsAction,
    EventSourceWaitTimeoutAction,
};
use lib::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use lib::p2p::pubsub::{GossipNetMessageV2, PubsubTopic};
use lib::p2p::PeerId;
use lib::rpc::{ActionStatsQuery, RpcRequest, WatchedAccountsGetError};
use lib::snark::{srs_from_bytes, verifier_index_from_bytes};
use lib::watched_accounts::WatchedAccountId;

mod service;
use service::libp2p::Libp2pService;
use service::rpc::{
    RpcActionStatsGetResponse, RpcP2pConnectionOutgoingResponse, RpcP2pPubsubPublishResponse,
    RpcService, RpcStateGetResponse, RpcWatchedAccountsAddResponse, RpcWatchedAccountsGetResponse,
};
pub use service::NodeWasmService;

pub mod logging;
use logging::LogLevel;
pub mod rayon;

mod transaction;
pub use transaction::new_signed_payment;

const BLOCK_VERIFIER_INDEX_HASH: &'static str =
    "9c9fc6dd2a2f73bf974a2055ac3173edb72bb2a9e8407210c7a64a9ef9737d3d";
const BLOCK_VERIFIER_SRS_HASH: &'static str =
    "d733ebb250be11f2de010b3dafae2034f1c85f1b126991f352cb2db0d3a0b9a2";

pub type Store = lib::Store<NodeWasmService>;
pub type Node = lib::Node<NodeWasmService>;

fn keypair_from_bs58check_secret_key(encoded_sec: &str) -> Result<mina_signer::Keypair, JsValue> {
    mina_signer::Keypair::from_hex(encoded_sec)
        .map_err(|err| format!("Invalid Private Key: {}", err).into())
}

#[wasm_bindgen]
pub struct ManualConnector {
    inner: JsManualConnector,
    rpc_sender: RpcSender,
}

#[wasm_bindgen]
impl ManualConnector {
    pub async fn dial(&mut self, peer_id: String) -> Result<Promise, JsValue> {
        let inner: JsManualConnector = self.inner.clone().into();
        let addr = format!("/p2p-webrtc-direct/p2p/{}", peer_id);
        let peer_id_str = peer_id;
        let peer_id = PeerId::from_str(&peer_id_str).map_err(|e| e.to_string())?;
        let maddr: Multiaddr = addr
            .parse()
            .map_err(|err: libp2p::multiaddr::Error| err.to_string())?;
        let rpc = self.rpc_sender.clone();
        spawn_local(async move {
            rpc.peer_connect(P2pConnectionOutgoingInitOpts {
                peer_id,
                addrs: vec![maddr],
            })
            .await;
        });
        Ok(inner.dial(peer_id_str))
    }

    pub fn listen(&self) -> JsValue {
        self.inner.listen()
    }
}

fn dial_opts(addr: &str, peer_id: &str) -> P2pConnectionOutgoingInitOpts {
    let maddr = format!("{}/http/p2p-webrtc-direct/p2p/{}", addr, peer_id);
    P2pConnectionOutgoingInitOpts {
        peer_id: peer_id.parse().unwrap(),
        addrs: vec![maddr.parse().unwrap()],
    }
}

/// Runs endless loop.
///
/// Doesn't exit.
pub async fn run(mut node: Node) {
    node.store_mut().dispatch(EventSourceProcessEventsAction {});
    loop {
        node.store_mut().dispatch(EventSourceWaitForEventsAction {});

        let service = &mut node.store_mut().service;
        let wait_for_events = service.event_source_receiver.wait_for_events();
        let wasm_rpc_req_fut = service.rpc.wasm_req_receiver().next().then(|res| async {
            // TODO(binier): optimize maybe to not check it all the time.
            match res {
                Some(v) => v,
                None => std::future::pending().await,
            }
        });
        let timeout = wasm_timer::Delay::new(Duration::from_millis(1000));

        select_biased! {
            _ = wait_for_events.fuse() => {
                while node.store_mut().service.event_source_receiver.has_next() {
                    node.store_mut().dispatch(EventSourceProcessEventsAction {});
                }
            }
            req = wasm_rpc_req_fut.fuse() => {
                node.store_mut().service.process_wasm_rpc_request(req);
            }
            _ = timeout.fuse() => {
                node.store_mut().dispatch(EventSourceWaitTimeoutAction {});
            }
        }
    }
}

#[wasm_bindgen]
pub struct WasmConfig {
    block_verifier_index: Option<JsFuture>,
    block_verifier_srs: Option<JsFuture>,
}

#[wasm_bindgen]
impl WasmConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            block_verifier_index: None,
            block_verifier_srs: None,
        }
    }

    pub fn set_block_verifier_index(&mut self, value: Option<Promise>) {
        self.block_verifier_index = value.map(|p| p.into());
    }

    pub fn set_block_verifier_srs(&mut self, value: Option<Promise>) {
        self.block_verifier_srs = value.map(|p| p.into());
    }
}

#[wasm_bindgen(js_name = start)]
pub async fn wasm_start(config: WasmConfig) -> Result<JsHandle, JsValue> {
    let logger_config = logging::InMemLoggerConfig {
        max_level: shared::log::inner::Level::DEBUG,
        max_len: 256,
    };
    let logs = logging::setup_global_logger(logger_config);
    {
        let logs = logs.clone();
        std::panic::set_hook(Box::new(move |info| {
            let logs = logs.get_range(None, usize::MAX);
            for log in logs.iter().rev() {
                let data = JsValue::from_serde(log).unwrap();
                match log.level() {
                    LogLevel::Trace => {
                        web_sys::console::trace_1(&data);
                    }
                    LogLevel::Debug => {
                        web_sys::console::debug_1(&data);
                    }
                    LogLevel::Info => {
                        web_sys::console::info_1(&data);
                    }
                    LogLevel::Warn => {
                        web_sys::console::warn_1(&data);
                    }
                    LogLevel::Error => {
                        web_sys::console::error_1(&data);
                    }
                }
                web_sys::console::error_2(&"PANIC!".into(), &format!("{:?}", info).into());
            }
        }));
    }

    let (tx, rx) = mpsc::channel(1024);

    if let Err(ref e) = rayon::init_rayon().await {
        shared::log::error!(shared::log::system_time();
            kind = "FatalError",
            summary = "failed to initialize threadpool",
            error = format!("{:?}", e));
        panic!("FatalError");
    }

    // TODO(binier): LocalStorage is too small. Use IndexDB instead.
    async fn cached_value<T, F>(storage: Option<&web_sys::Storage>, key: &'static str, calc: F) -> T
    where
        T: Send + 'static + serde::Serialize + for<'a> serde::Deserialize<'a>,
        F: Send + 'static + FnOnce() -> T,
    {
        let cached = storage
            .and_then(|s| s.get(key).ok()?)
            .and_then(|json| serde_json::from_str(&json).ok());

        match cached {
            Some(cached) => cached,
            None => {
                let (tx, rx) = oneshot::channel();
                ::rayon::spawn(move || {
                    tx.send(calc());
                });
                let index = rx.await.unwrap();
                // TODO(binier): for now, deserialized verifier_index
                // doesn't work for verification. Uncomment to cache
                // it once it's fixed.
                // if let Some(s) = storage {
                //     s.set(key, &serde_json::to_string(&index).unwrap());
                // }
                index
            }
        }
    }

    let (libp2p, manual_connector) = Libp2pService::run(tx.clone()).await;
    let mut service = NodeWasmService {
        rng: rand::rngs::OsRng::default(),
        stats: Some(Default::default()),
        event_source_sender: tx.clone(),
        event_source_receiver: rx.into(),
        libp2p,
        rpc: RpcService::new(),
    };
    let rpc_sender = service.wasm_rpc_req_sender().clone();

    spawn_local(async move {
        let storage = web_sys::window().and_then(|window| window.local_storage().ok().flatten());

        shared::log::info!(shared::log::system_time();
            kind = "SnarkBlockVerifyIndexGetInit",
            summary = "get block verifier index");

        let promise = config.block_verifier_index;
        let block_verifier_index = std::future::ready(())
            .then(move |_| async {
                let v = promise?.await.ok()?;
                let bytes = Uint8Array::new(&v).to_vec();

                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let hash = hex::encode(&hasher.finalize());

                if hash != BLOCK_VERIFIER_INDEX_HASH {
                    shared::log::info!(shared::log::system_time();
                        kind = "SnarkBlockVerifyIndexFetchError",
                        summary = "hash mismatch",
                        expected = BLOCK_VERIFIER_INDEX_HASH,
                        found = hash);
                    return None;
                }

                Some(verifier_index_from_bytes(&bytes))
            })
            .then(|opt| async {
                match opt {
                    Some(v) => v,
                    None => {
                        cached_value(storage.as_ref(), "block_verifier_index", || {
                            lib::snark::get_verifier_index(lib::snark::VerifierKind::Blockchain)
                        })
                        .await
                    }
                }
            })
            .await;

        shared::log::info!(shared::log::system_time();
            kind = "SnarkBlockVerifyIndexGetSuccess",
            summary = "get block verifier index successful",
            block_verifier_index = serde_json::to_string(&block_verifier_index).ok());

        shared::log::info!(shared::log::system_time();
            kind = "SnarkBlockVerifySRSGetInit",
            summary = "get block verifier srs");

        let promise = config.block_verifier_srs;
        let block_verifier_srs = std::future::ready(())
            .then(move |_| async {
                let v = promise?.await.ok()?;
                let bytes = Uint8Array::new(&v).to_vec();

                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let hash = hex::encode(&hasher.finalize());

                if hash != BLOCK_VERIFIER_SRS_HASH {
                    shared::log::info!(shared::log::system_time();
                        kind = "SnarkBlockVerifySRSFetchError",
                        summary = "hash mismatch",
                        expected = BLOCK_VERIFIER_SRS_HASH,
                        found = hash);
                    return None;
                }

                Some(srs_from_bytes(&bytes))
            })
            .then(|opt| async {
                match opt {
                    Some(v) => v,
                    None => {
                        cached_value(storage.as_ref(), "block_verifier_srs", lib::snark::get_srs)
                            .await
                    }
                }
            })
            .await;

        shared::log::info!(shared::log::system_time();
            kind = "SnarkBlockVerifySRSGetSuccess",
            summary = "get block verifier srs successful",
            verifier_srs = serde_json::to_string(&block_verifier_srs).ok());

        let state = lib::State::new(lib::Config {
            snark: lib::snark::SnarkConfig {
                block_verifier_index: Arc::new(block_verifier_index),
                block_verifier_srs: Arc::new(block_verifier_srs),
            },
            p2p: lib::p2p::P2pConfig {
                initial_peers: vec![
                    dial_opts(
                        "/dns4/webrtc2.webnode.openmina.com/tcp/443",
                        "12D3KooWFpqySZDHx7k5FMjdwmrU3TLhDbdADECCautBcEGtG4fr",
                    ),
                    dial_opts(
                        "/dns4/webrtc2.webnode.openmina.com/tcp/4431",
                        "12D3KooWJBeXosFxdBwe2mbKRjgRG69ERaUTpS9qo9NRkoE8kBpj",
                    ),
                    dial_opts(
                        "/dns4/webrtc2.webnode.openmina.com/tcp/4432",
                        "QmTyRcQ5oM4ZByekkKyh1EDVNy7Xvh32UdGKAMBqPTiUSR",
                    ),
                    dial_opts(
                        "/dns4/webrtc3.webnode.openmina.com/tcp/443",
                        "QmemRt4HfpNBsFY752tE5E3H6dV87JjtmgacN53BDGbaB6",
                    ),
                    dial_opts(
                        "/dns4/webrtc3.webnode.openmina.com/tcp/4431",
                        "QmZvoP2taWTUbuGp1xnu5AXizG9vr2MFbam7nSuGvpmfhM",
                    ),
                    dial_opts(
                        "/dns4/webrtc3.webnode.openmina.com/tcp/4432",
                        "QmVXA7fjnahFLC85jFHJYXapM61Yo1rvH7Live7szpUrzb",
                    ),
                    dial_opts(
                        "/dns4/webrtc4.webnode.openmina.com/tcp/443",
                        "QmQKwHVjkpqdLco5KpyMXjDwjxKHLdS78kmdDMWwvxYKFf",
                    ),
                    dial_opts(
                        "/dns4/webrtc4.webnode.openmina.com/tcp/4431",
                        "QmSXffHzFVSEoQCYBS1bPpCn4vgGEpQnCA9NLYuhamPBU3",
                    ),
                    dial_opts(
                        "/dns4/webrtc4.webnode.openmina.com/tcp/4432",
                        "QmdxcoKJTh2b8xLZcBk9zw2xHdEcNwzMr78jX99wUWcT8o",
                    ),
                ],
            },
        });
        let node = lib::Node::new(state, service);
        run(node).await;
    });

    Ok(JsHandle {
        sender: tx,
        rpc: RpcSender::new(rpc_sender),
        manual_connector,
        logs,
    })
}

pub struct WasmRpcRequest {
    pub req: RpcRequest,
    pub responder: Box<dyn std::any::Any>,
}

#[derive(Clone)]
pub struct RpcSender {
    tx: mpsc::Sender<WasmRpcRequest>,
}

impl RpcSender {
    pub fn new(tx: mpsc::Sender<WasmRpcRequest>) -> Self {
        Self { tx }
    }

    pub async fn oneshot_request<T>(&self, req: RpcRequest) -> Option<T>
    where
        T: 'static + Serialize,
    {
        let (tx, rx) = oneshot::channel::<T>();
        let responder = Box::new(tx);
        let mut sender = self.tx.clone();
        sender.send(WasmRpcRequest { req, responder }).await;

        rx.await.ok()
    }

    pub async fn peer_connect(
        &self,
        opts: P2pConnectionOutgoingInitOpts,
    ) -> Result<String, JsValue> {
        let peer_id = opts.peer_id;
        let req = RpcRequest::P2pConnectionOutgoing(opts);
        self.oneshot_request::<RpcP2pConnectionOutgoingResponse>(req)
            .await
            .ok_or_else(|| JsValue::from("state machine shut down"))??;

        Ok(peer_id.to_string())
    }
}

#[derive(Serialize)]
pub struct BestTipSummary {
    pub hash: StateHash,
    pub level: u32,
    pub timestamp: u64,
}

#[wasm_bindgen]
pub struct JsHandle {
    sender: mpsc::Sender<Event>,
    rpc: RpcSender,

    manual_connector: JsManualConnector,
    logs: logging::InMemLogs,
}

impl JsHandle {
    pub async fn pubsub_publish(&self, topic: PubsubTopic, msg: GossipNetMessageV2) -> JsValue {
        let req = RpcRequest::P2pPubsubPublish(topic, msg);
        let res = self
            .rpc
            .oneshot_request::<RpcP2pPubsubPublishResponse>(req)
            .await;
        JsValue::from_serde(&res).unwrap()
    }
}

#[wasm_bindgen]
impl JsHandle {
    pub async fn logs_range(&self, cursor: Option<usize>, limit: Option<usize>) -> JsValue {
        // TODO(binier): maybe somehow we could return Vec<logging::InMemLog>
        let logs = self.logs.get_range(cursor, limit.unwrap_or(128));
        JsValue::from_serde(&logs).unwrap()
    }

    pub fn manual_connector(&self) -> ManualConnector {
        ManualConnector {
            inner: self.manual_connector.clone().into(),
            rpc_sender: self.rpc.clone(),
        }
    }

    pub fn is_peer_id_valid(&self, id: &str) -> Result<(), String> {
        id.parse::<lib::p2p::PeerId>()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }

    pub fn is_account_pk_valid(&self, pk: String) -> Result<(), String> {
        pk.parse::<NonZeroCurvePoint>()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }

    pub async fn global_state_get(&self) -> JsValue {
        let req = RpcRequest::GetState;
        let res = self.rpc.oneshot_request::<RpcStateGetResponse>(req).await;
        JsValue::from_serde(&res).unwrap()
    }

    pub async fn action_stats_since_start_get(&self) -> JsValue {
        let req = RpcRequest::ActionStatsGet(ActionStatsQuery::SinceStart);
        let res = self
            .rpc
            .oneshot_request::<RpcActionStatsGetResponse>(req)
            .await;
        JsValue::from_serde(&res).unwrap()
    }

    pub async fn best_tip_summary(&self) -> JsValue {
        // TODO(binier): [PERF] inefficient as we clone the whole state.
        let req = RpcRequest::GetState;
        let res = self.rpc.oneshot_request::<RpcStateGetResponse>(req).await;
        let res = res.and_then(|s| {
            let b = s.consensus.best_tip()?;
            let genesis_timestamp = b
                .header
                .protocol_state
                .body
                .constants
                .genesis_state_timestamp
                .as_u64();
            let time_passed = 3
                * 60
                * 1000
                * b.header
                    .protocol_state
                    .body
                    .consensus_state
                    .global_slot_since_genesis
                    .as_u32() as u64;
            Some(BestTipSummary {
                hash: b.hash.clone(),
                level: b.height() as u32,
                timestamp: genesis_timestamp + time_passed,
            })
        });
        JsValue::from_serde(&res).unwrap()
    }

    pub async fn peer_connect(&self, addr: String) -> Result<String, JsValue> {
        let addr = Multiaddr::from_str(&addr).map_err(|err| err.to_string())?;
        let peer_id =
            peer_id_from_addr(&addr).ok_or_else(|| "Multiaddr missing PeerId".to_string())?;

        self.rpc
            .peer_connect(P2pConnectionOutgoingInitOpts {
                peer_id,
                addrs: vec![addr],
            })
            .await
    }

    #[wasm_bindgen(js_name = pubsub_publish)]
    pub async fn js_pubsub_publish(&self, topic: String, msg: JsValue) -> Result<(), JsValue> {
        let topic = PubsubTopic::from_str(&topic).map_err(|err| err.to_string())?;
        let msg = msg.into_serde().map_err(|err| err.to_string())?;
        let req = RpcRequest::P2pPubsubPublish(topic, msg);
        self.rpc
            .oneshot_request::<RpcP2pPubsubPublishResponse>(req)
            .await;
        Ok(())
    }

    pub fn generate_account_keys(&self) -> JsValue {
        let mut r = rand::rngs::OsRng::default();
        let keypair = mina_signer::Keypair::rand(&mut r);
        let priv_key = keypair.to_hex();
        let pub_key = keypair.get_address();
        JsValue::from_serde(&serde_json::json!({
            "priv_key": priv_key,
            "pub_key": pub_key,
        }))
        .unwrap()
    }

    pub async fn payment_sign_and_inject(&self, data: JsValue) -> Result<String, JsValue> {
        #[serde_as]
        #[derive(serde::Deserialize)]
        struct Payment {
            priv_key: String,
            to: String,
            #[serde_as(as = "DisplayFromStr")]
            amount: u64,
            #[serde_as(as = "DisplayFromStr")]
            fee: u64,
            #[serde_as(as = "DisplayFromStr")]
            nonce: u32,
            memo: Option<String>,
        }

        let data: Payment = data
            .into_serde()
            .map_err(|err| format!("Deserialize Error: {}", err))?;
        let keypair = keypair_from_bs58check_secret_key(&data.priv_key)?;
        let to =
            PubKey::from_address(&data.to).map_err(|err| format!("Bad `to` address: {}", err))?;

        let tx = new_signed_payment(
            &keypair,
            ledger::scan_state::currency::Fee::from_u64(data.fee),
            ledger::scan_state::currency::Nonce::from_u32(data.nonce),
            None,
            {
                if let Some(memo_str) = data.memo.filter(|s| !s.is_empty()) {
                    const MEMO_LEN: usize = 34;
                    let mut memo = ledger::scan_state::transaction_logic::Memo([0; MEMO_LEN]);
                    memo.0[0] = 0x01;
                    memo.0[1] = std::cmp::min(memo_str.len(), MEMO_LEN - 2) as u8;
                    let memo_str = format!("{:\0<32}", memo_str); // Pad user-supplied memo with zeros
                    memo.0[2..].copy_from_slice(
                        &memo_str.as_bytes()[..std::cmp::min(memo_str.len(), MEMO_LEN - 2)],
                    );
                    memo
                } else {
                    ledger::scan_state::transaction_logic::Memo::empty()
                }
            },
            to.into_compressed(),
            ledger::scan_state::currency::Amount::from_u64(data.amount),
        );

        let tx = MinaBaseUserCommandStableV2::from(&tx);
        let tx_hash = tx.hash().unwrap();
        let msg = {
            let nonce = 0.into();
            let message = NetworkPoolTransactionPoolDiffVersionedStableV2(vec![tx]);
            GossipNetMessageV2::TransactionPoolDiff { nonce, message }
        };

        shared::log::info!(
            shared::log::system_time();
            kind = "CreateTransaction",
            summary = tx_hash.to_string(),
            message = serde_json::to_string(&msg).ok()
        );
        self.pubsub_publish(PubsubTopic::CodaConsensusMessage, msg)
            .await;
        Ok(tx_hash.to_string())
    }

    #[wasm_bindgen]
    pub fn watched_accounts(&self) -> WatchedAccounts {
        WatchedAccounts {
            rpc: self.rpc.clone(),
        }
    }
}

fn default_token_id() -> TokenIdKeyHash {
    MinaBaseAccountIdDigestStableV1(BigInt::one()).into()
}

#[wasm_bindgen]
pub struct WatchedAccounts {
    rpc: RpcSender,
}

#[wasm_bindgen]
impl WatchedAccounts {
    pub async fn add(&self, pub_key: String, token_id: Option<String>) -> Result<bool, String> {
        let pub_key = NonZeroCurvePoint::from_str(&pub_key).map_err(|err| err.to_string())?;
        let token_id = token_id
            .map(|id| TokenIdKeyHash::from_str(&id).map_err(|err| err.to_string()))
            .unwrap_or_else(|| Ok(default_token_id()))?;
        let account_id = WatchedAccountId(pub_key, token_id);
        let req = RpcRequest::WatchedAccountsAdd(account_id);
        let resp = self
            .rpc
            .oneshot_request::<RpcWatchedAccountsAddResponse>(req)
            .await;
        resp.ok_or("rpc request dropped".into())
    }

    pub async fn get(&self, pub_key: String, token_id: Option<String>) -> Result<JsValue, String> {
        let pub_key = NonZeroCurvePoint::from_str(&pub_key).map_err(|err| err.to_string())?;
        let token_id = token_id
            .map(|id| TokenIdKeyHash::from_str(&id).map_err(|err| err.to_string()))
            .unwrap_or_else(|| Ok(default_token_id()))?;
        let account_id = WatchedAccountId(pub_key, token_id);
        loop {
            let req = RpcRequest::WatchedAccountsGet(account_id.clone());
            let resp = self
                .rpc
                .oneshot_request::<RpcWatchedAccountsGetResponse>(req)
                .await;
            let resp = resp.ok_or("rpc request dropped".to_string())?;
            match resp {
                Err(WatchedAccountsGetError::NotReady) => {
                    wasm_timer::Delay::new(Duration::from_millis(500)).await;
                }
                Ok(v) => return Ok(JsValue::from_serde(&v).map_err(|err| err.to_string())?),
                Err(err) => return Err(err.to_string()),
            }
        }
    }
}

fn peer_id_from_addr(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|p| match p {
        MultiaddrProtocol::P2p(id) => PeerId::from_multihash(id).ok(),
        _ => None,
    })
}
