use node::ActionKind;

use crate::{
    cluster::ClusterNodeId,
    node::RustNodeTestingConfig,
    scenario::ListenerNode,
    scenarios::{
        multi_node::connection_discovery::{
            check_kademlia_entries, wait_for_identify, wait_for_node_ready,
        },
        ClusterRunner, RunCfg,
    },
};

/// Kademlia bootstrap test.
/// 1. Create seed node and wait for it to be ready
/// 2. Create NUM nodes that will connect to seed node but not perform further discovery
/// 3. Check that seed nodes has all nodes in it's routing table
/// 4. Create new node with only seed node as peer
/// 5. Wait for new node to bootstrap
/// 6. Check that new node has all peers in it table
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct KademliaBootstrap;

impl KademliaBootstrap {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");
        const NUM: u8 = 16;

        let seed_node = runner.add_rust_node(RustNodeTestingConfig::devnet_default());
        wait_for_node_ready(&mut runner, seed_node).await;
        let mut nodes = vec![];

        let config = RustNodeTestingConfig {
            initial_peers: vec![ListenerNode::Rust(seed_node)],
            ..RustNodeTestingConfig::devnet_default()
        };
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "true");
        for _ in 0..NUM {
            let node_id = runner.add_rust_node(config.clone());
            let peer_id = runner.node(node_id).expect("Node not found").peer_id();

            nodes.push((node_id, peer_id));

            wait_for_bootstrap(&mut runner, node_id).await;
            wait_for_identify(&mut runner, seed_node, peer_id, "openmina").await;
        }

        let peer_ids = nodes.iter().map(|node| node.1);
        let seed_has_peers =
            check_kademlia_entries(&mut runner, seed_node, peer_ids.clone()).unwrap_or_default();
        assert!(
            seed_has_peers,
            "Seed doesn't have all peers in it's routing table"
        );

        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "true");
        let new_node = runner.add_rust_node(config);
        let new_node_peer_id = runner.node(new_node).expect("Not note found").peer_id();

        wait_for_bootstrap(&mut runner, new_node).await;
        wait_for_identify(&mut runner, seed_node, new_node_peer_id, "openmina").await;
        let new_has_peers =
            check_kademlia_entries(&mut runner, seed_node, peer_ids).unwrap_or_default();
        assert!(
            new_has_peers,
            "Node doesn't have all peers in it's routing table"
        );
    }
}

async fn wait_for_bootstrap(runner: &mut ClusterRunner<'_>, node_id: ClusterNodeId) {
    runner
        .run(RunCfg::default().action_handler(move |node, _, _, action| {
            node == node_id
                && matches!(
                    action.action().kind(),
                    ActionKind::P2pNetworkKademliaBootstrapFinished
                )
        }))
        .await
        .expect("Node failed to bootstrap")
}
