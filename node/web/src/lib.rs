#![cfg(target_family = "wasm")]

pub use openmina_node_common::*;

mod rayon;
pub use rayon::init_rayon;

mod node;
pub use node::{Node, NodeBuilder};

use std::sync::Arc;

use ::node::account::AccountSecretKey;
use ::node::core::thread;
use ::node::snark::{get_verifier_index, VerifierKind};
use anyhow::Context;
use ledger::proofs::gates::BlockProver;
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
pub async fn run(block_producer: Option<String>) -> RpcSender {
    let block_producer: Option<AccountSecretKey> = block_producer.map(|key| {
        key.parse()
            .expect("failed to parse passed block producer keys")
    });

    let (rpc_sender_tx, rpc_sender_rx) = ::node::core::channels::oneshot::channel();
    let _ = thread::spawn(move || {
        wasm_bindgen_futures::spawn_local(async move {
            let mut node = setup_node(block_producer).await;
            let _ = rpc_sender_tx.send(node.rpc());
            node.run_forever().await;
        });

        wasm_bindgen::throw_str("Cursed hack to keep workers alive. See https://github.com/rustwasm/wasm-bindgen/issues/2945");
    });

    rpc_sender_rx.await.unwrap()
}

async fn setup_node(
    block_producer: Option<AccountSecretKey>,
) -> openmina_node_common::Node<NodeService> {
    let block_verifier_index = get_verifier_index(VerifierKind::Blockchain);
    let work_verifier_index = get_verifier_index(VerifierKind::Transaction);

    let genesis_config = ::node::config::DEVNET_CONFIG.clone();
    let mut node_builder: NodeBuilder = NodeBuilder::new(None, genesis_config);
    node_builder
        .block_verifier_index(block_verifier_index.clone())
        .work_verifier_index(work_verifier_index.clone());

    if let Some(bp_key) = block_producer {
        let provers =
            BlockProver::make(Some(block_verifier_index), Some(work_verifier_index)).await;
        node_builder.block_producer(provers, bp_key);
    }

    node_builder
        .p2p_no_discovery()
        .p2p_custom_task_spawner(P2pTaskRemoteSpawner {})
        .unwrap();
    node_builder.gather_stats();
    node_builder.build().context("node build failed!").unwrap()
}
