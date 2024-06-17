use std::time::Duration;

use crate::{
    node::RustNodeTestingConfig,
    scenario::ListenerNode,
    scenarios::{ClusterRunner, Driver},
};

/// Receive a block via meshsub
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct P2pReceiveBlock;

impl P2pReceiveBlock {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let config = RustNodeTestingConfig::devnet_default()
            // make sure it will not ask initial peers
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(1)
            .initial_peers(vec![ListenerNode::Custom("/ip4/34.135.63.47/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag".parse().unwrap())]);
        let retransmitter_openmina_node = runner.add_rust_node(config);
        let retransmitter_peer_id = runner
            .node(retransmitter_openmina_node)
            .unwrap()
            .state()
            .p2p
            .my_id();

        let config = RustNodeTestingConfig::devnet_default()
            // make sure it will not ask initial peers
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(1)
            .initial_peers(vec![retransmitter_openmina_node.into()]);
        let receiver_openmina_node = runner.add_rust_node(config);

        let mut driver = Driver::new(runner);
        driver
            .wait_for(Duration::from_secs(20 * 60), |node, _, state| {
                let Some(p2p) = state.p2p.ready() else {
                    return false;
                };
                node == receiver_openmina_node
                    && p2p
                        .network
                        .scheduler
                        .broadcast_state
                        .incoming_block
                        .as_ref()
                        .map_or(false, |(peer_id, _)| peer_id.eq(&retransmitter_peer_id))
            })
            .await
            .unwrap();

        eprintln!("passed");
    }
}
