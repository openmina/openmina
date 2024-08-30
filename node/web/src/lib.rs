#![cfg(target_family = "wasm")]

use openmina_node_common::rpc::RpcSender;
pub use openmina_node_common::*;

mod rayon;
pub use rayon::init_rayon;

mod node;
pub use node::{Node, NodeBuilder};

use ::node::core::thread;
use ::node::snark::{get_verifier_index, VerifierKind};
use anyhow::Context;
use wasm_bindgen::prelude::*;

use crate::node::P2pTaskRemoteSpawner;

/// Automatically run after wasm is loaded.
#[wasm_bindgen(start)]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        console_error_panic_hook::set_once();
        tracing::initialize(tracing::Level::INFO);

        init_rayon().await.unwrap();
    });
}

#[wasm_bindgen]
pub async fn run() -> RpcSender {
    // TODO(binier): what if above init in the `main` function isn't done
    // and this function gets called.
    let p2p_task_spawner = P2pTaskRemoteSpawner::create();

    let (rpc_sender_tx, rpc_sender_rx) = ::node::core::channels::oneshot::channel();
    let _ = thread::spawn(move || {
        let block_verifier_index = get_verifier_index(VerifierKind::Blockchain).into();
        let work_verifier_index = get_verifier_index(VerifierKind::Transaction).into();
        let genesis_config = ::node::config::DEVNET_CONFIG.clone();
        let mut node_builder: NodeBuilder = NodeBuilder::new(None, genesis_config);
        node_builder
            .block_verifier_index(block_verifier_index)
            .work_verifier_index(work_verifier_index);
        node_builder
            .p2p_no_discovery()
            .p2p_custom_task_spawner(p2p_task_spawner)
            .unwrap();
        node_builder.gather_stats();
        let mut node = node_builder.build().context("node build failed!").unwrap();
        let _ = rpc_sender_tx.send(node.rpc());

        wasm_bindgen_futures::spawn_local(async move {
            node.run_forever().await;
        });
        wasm_bindgen::throw_str("Cursed hack to keep workers alive. See https://github.com/rustwasm/wasm-bindgen/issues/2945");
    });

    rpc_sender_rx.await.unwrap()
}
