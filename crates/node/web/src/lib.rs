#![cfg(target_family = "wasm")]

pub use mina_node_common::*;

mod rayon;
pub use rayon::init_rayon;

mod node;
pub use node::{Node, NodeBuilder};

use anyhow::Context;
use gloo_utils::format::JsValueSerdeExt;
use ledger::proofs::provers::BlockProver;
use mina_node::{
    account::AccountSecretKey,
    core::{log, thread},
    snark::{BlockVerifier, TransactionVerifier},
    transition_frontier::genesis::GenesisConfig,
};
use mina_node_common::rpc::RpcSender;
use wasm_bindgen::prelude::*;

use crate::node::P2pTaskRemoteSpawner;

/// Automatically run after wasm is loaded.
#[wasm_bindgen(start)]
fn main() {
    thread::main_thread_init();
    wasm_bindgen_futures::spawn_local(async {
        console_error_panic_hook::set_once();
        tracing::initialize(tracing::Level::DEBUG);

        init_rayon().await.unwrap();
    });
}

#[wasm_bindgen]
pub fn build_env() -> JsValue {
    JsValue::from_serde(&::mina_node::BuildEnv::get()).unwrap_or_default()
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

/// Starts a Mina node in a WASM environment and returns an RPC interface.
///
/// This is the main entry point for running a Mina node from JavaScript/WASM.
/// It spawns the node in a separate thread, sets up all necessary components,
/// and returns an RPC sender that can be used to communicate with the running
/// node.
///
/// # Arguments
///
/// * `block_producer` - Block producer configuration as a JavaScript value.
///   Can be one of:
///   - `null`/`undefined`: No block production
///   - `string`: Plain text secret key
///   - `[encrypted_key, password]`: Array with encrypted key and password
/// * `seed_nodes_urls` - Optional list of URLs to fetch peer lists from.
///   Each URL should return newline-separated peer addresses. This is similar
///   to the `--peer-list-url` flag in the native node, but allows multiple URLs
///   to be supplied.
/// * `seed_nodes_addresses` - Optional list of peer addresses to connect to
///   directly, in [WebRTC multiaddr format](https://o1-labs.github.io/mina-rust/docs/developers/webrtc#address-formats).
///   This is directly comparable to the `--peers` flag in the native node.
/// * `genesis_config_url` - Optional URL to fetch genesis configuration from.
///   Genesis config must be in bin_prot format. If not provided, uses the default
///   devnet configuration.
///
/// # Returns
///
/// An `RpcSender` that can be used to send RPC commands to the running node.
///
/// # Panics
///
/// Panics if:
/// - Block producer key parsing fails
/// - Node setup or build fails
/// - Genesis configuration cannot be fetched
///
/// # Example
///
/// ```javascript
/// const rpc = await run(
///   null,  // No block production
///   ["https://bootnodes.minaprotocol.com/networks/devnet-webrtc.txt"],
///   ["/dns4/webrtc-peer-signaling.example.com/tcp/443/webrtc/https/p2p/PEER_ID"],
///   null, // Use the default devnet configuration
/// );
/// ```
#[wasm_bindgen]
pub async fn run(
    block_producer: JsValue,
    seed_nodes_urls: Option<Vec<String>>,
    seed_nodes_addresses: Option<Vec<String>>,
    genesis_config_url: Option<String>,
) -> RpcSender {
    let block_producer = parse_bp_key(block_producer);

    let (rpc_sender_tx, rpc_sender_rx) = ::mina_node::core::channels::oneshot::channel();
    let _ = thread::spawn(move || {
        wasm_bindgen_futures::spawn_local(async move {
            let mut node = setup_node(
                block_producer,
                seed_nodes_urls,
                seed_nodes_addresses,
                genesis_config_url,
            )
            .await;
            let _ = rpc_sender_tx.send(node.rpc());
            node.run_forever().await;
        });

        keep_worker_alive_cursed_hack();
    });

    rpc_sender_rx.await.unwrap()
}

async fn setup_node(
    block_producer: Option<AccountSecretKey>,
    seed_nodes_urls: Option<Vec<String>>,
    seed_nodes_addresses: Option<Vec<String>>,
    genesis_config_url: Option<String>,
) -> mina_node_common::Node<NodeService> {
    let block_verifier_index = BlockVerifier::make().await;
    let work_verifier_index = TransactionVerifier::make().await;

    let genesis_config = if let Some(genesis_config_url) = genesis_config_url {
        let bytes = ::mina_node::core::http::get_bytes(&genesis_config_url)
            .await
            .expect("failed to fetch genesis config");
        GenesisConfig::Prebuilt(bytes.into()).into()
    } else {
        ::mina_node::config::DEVNET_CONFIG.clone()
    };

    let mut node_builder: NodeBuilder = NodeBuilder::new(None, genesis_config);
    node_builder
        .block_verifier_index(block_verifier_index.clone())
        .work_verifier_index(work_verifier_index.clone());

    // TODO(binier): refactor
    let mut all_raw_peers = seed_nodes_addresses.unwrap_or_default();

    if let Some(seed_nodes_urls) = seed_nodes_urls {
        for seed_nodes_url in seed_nodes_urls {
            let peers = ::mina_node::core::http::get_bytes(&seed_nodes_url).await;
            match peers {
                Ok(s) => {
                    log::info!("Successfully fetched peers from {seed_nodes_url}");
                    all_raw_peers.extend(String::from_utf8_lossy(&s).split("\n").map(String::from));
                }
                Err(e) => {
                    log::error!("Failed to fetch peers from {seed_nodes_url}: {e}");
                }
            }
        }
    }

    node_builder.initial_peers(
        all_raw_peers
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .flat_map(|s| s.parse().ok())
            .inspect(|p| log::debug!("Using peer: {p:?}")),
    );

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
