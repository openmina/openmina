#![cfg(target_family = "wasm")]

use ::node::transition_frontier::genesis::GenesisConfig;
pub use openmina_node_common::*;

mod rayon;
pub use rayon::init_rayon;

mod node;
pub use node::{Node, NodeBuilder};

use ::node::account::AccountSecretKey;
use ::node::core::thread;
use ::node::snark::{BlockVerifier, TransactionVerifier};
use anyhow::Context;
use gloo_utils::format::JsValueSerdeExt;
use ledger::proofs::provers::BlockProver;
use openmina_node_common::rpc::RpcSender;
use wasm_bindgen::prelude::*;

use crate::node::P2pTaskRemoteSpawner;

/// Automatically run after wasm is loaded.
#[wasm_bindgen(start)]
fn main() {
    thread::main_thread_init();
    wasm_bindgen_futures::spawn_local(async {
        console_error_panic_hook::set_once();
        tracing::initialize(tracing::Level::INFO);

        init_rayon().await.unwrap();
    });
}

#[wasm_bindgen]
pub fn build_env() -> JsValue {
    JsValue::from_serde(&::node::BuildEnv::get()).unwrap_or_default()
}

fn parse_bp_key(key: JsValue) -> Option<AccountSecretKey> {
    if key.is_falsy() {
        return None;
    }

    if key.is_string() {
        return Some(
            key.as_string()
                .unwrap()
                .parse()
                .expect("failed to parse passed block producer keys"),
        );
    }

    let (encrypted, password) = if key.is_array() {
        let arr: js_sys::Array = key.into();
        let password = arr
            .at(1)
            .as_string()
            .expect("invalid block_producer password");
        let encrypted = arr
            .at(0)
            .into_serde()
            .expect("block_producer encrypted key decode failed");
        (encrypted, password)
    } else {
        panic!("unsupported block_producer keys type: {key:?}");
    };

    Some(
        AccountSecretKey::from_encrypted(&encrypted, &password)
            .expect("block_producer secret key decrypt failed"),
    )
}

#[wasm_bindgen]
pub async fn run(
    block_producer: JsValue,
    seed_nodes_url: Option<String>,
    genesis_config_url: Option<String>,
) -> RpcSender {
    let block_producer = parse_bp_key(block_producer);

    let (rpc_sender_tx, rpc_sender_rx) = ::node::core::channels::oneshot::channel();
    let _ = thread::spawn(move || {
        wasm_bindgen_futures::spawn_local(async move {
            let mut node = setup_node(block_producer, seed_nodes_url, genesis_config_url).await;
            let _ = rpc_sender_tx.send(node.rpc());
            node.run_forever().await;
        });

        keep_worker_alive_cursed_hack();
    });

    rpc_sender_rx.await.unwrap()
}

async fn setup_node(
    block_producer: Option<AccountSecretKey>,
    seed_nodes_url: Option<String>,
    genesis_config_url: Option<String>,
) -> openmina_node_common::Node<NodeService> {
    let block_verifier_index = BlockVerifier::make().await;
    let work_verifier_index = TransactionVerifier::make().await;

    let genesis_config = if let Some(genesis_config_url) = genesis_config_url {
        let bytes = ::node::core::http::get_bytes(&genesis_config_url)
            .await
            .expect("failed to fetch genesis config");
        GenesisConfig::Prebuilt(bytes.into()).into()
    } else {
        ::node::config::DEVNET_CONFIG.clone()
    };

    let mut node_builder: NodeBuilder = NodeBuilder::new(None, genesis_config);
    node_builder
        .block_verifier_index(block_verifier_index.clone())
        .work_verifier_index(work_verifier_index.clone());

    // TODO(binier): refactor
    if let Some(seed_nodes_url) = seed_nodes_url {
        let peers = ::node::core::http::get_bytes(&seed_nodes_url)
            .await
            .expect("failed to fetch seed nodes");
        node_builder.initial_peers(
            String::from_utf8_lossy(&peers)
                .split("\n")
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().parse().expect("failed to parse seed node addr")),
        );
    }

    if let Some(bp_key) = block_producer {
        thread::spawn(move || {
            BlockProver::make(Some(block_verifier_index), Some(work_verifier_index));
        });
        node_builder.block_producer(bp_key, None);
    }

    node_builder
        .p2p_custom_task_spawner(P2pTaskRemoteSpawner {})
        .unwrap();
    node_builder.gather_stats();
    node_builder.build().context("node build failed!").unwrap()
}

fn keep_worker_alive_cursed_hack() {
    wasm_bindgen::throw_str("Cursed hack to keep workers alive. See https://github.com/rustwasm/wasm-bindgen/issues/2945");
}
