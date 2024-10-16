use std::{collections::BTreeSet, time::Duration};

use node::{
    p2p::{P2pPeerAction, PeerId},
    Action, P2pAction,
};

use crate::{
    node::RustNodeTestingConfig,
    scenarios::{ClusterRunner, DynEffectsData, RunCfg},
};

/// Makes sure that when using WebRTC only nodes, peers can discover
/// each other and connect to each other via p2p signaling.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct P2pSignaling;

impl P2pSignaling {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        const NODES_N: usize = 4;

        let seed_config = RustNodeTestingConfig::devnet_default();
        let seed = runner.add_rust_node(seed_config.clone());

        let node_config = seed_config.initial_peers(vec![seed.into()]);
        let _node_1 = runner.add_rust_node(node_config.clone());
        let _node_2 = runner.add_rust_node(node_config.clone());
        let _node_3 = runner.add_rust_node(node_config.clone());

        let node_peers: [_; NODES_N] = std::array::from_fn(|_| BTreeSet::<PeerId>::new());
        let node_peers = DynEffectsData::new(node_peers);

        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(60))
                    .advance_time(1..=100)
                    .action_handler(move |node_id, _state, _, action| {
                        if action.action().kind().to_string().contains("Signaling") {
                            let me = _state.p2p.my_id();
                            let me_pk = me.to_public_key().unwrap();
                            dbg!((me, me_pk, action.action()));
                        }
                        match action.action() {
                            Action::P2p(P2pAction::Peer(P2pPeerAction::Ready {
                                peer_id, ..
                            })) => {
                                node_peers.inner()[node_id.index()].insert(*peer_id);
                                dbg!(node_peers.inner())
                                    .iter()
                                    .all(|v| v.len() == NODES_N - 1)
                            }
                            _ => false,
                        }
                    }),
            )
            .await
            .expect("peers didn't discover each other");
    }
}
