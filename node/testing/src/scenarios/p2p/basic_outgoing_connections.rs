use std::{collections::BTreeSet, time::Duration};

use node::p2p::{P2pPeerState, P2pPeerStatus, PeerId};

use crate::{
    cluster::ClusterNodeId,
    node::RustNodeTestingConfig,
    scenarios::{
        add_rust_nodes, as_connection_finalized_event, connection_finalized_event,
        wait_for_nodes_listening_on_localhost, ClusterRunner, Driver,
    },
};

/// Node should be able to make an outgoing connection to a listening node.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MakeOutgoingConnection;

impl MakeOutgoingConnection {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);

        let (node1, _) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());
        let (node2, peer_id2) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());

        // wait for the peer to listen
        let satisfied = wait_for_nodes_listening_on_localhost(
            &mut driver,
            Duration::from_secs(3 * 60),
            [node2],
        )
        .await
        .unwrap();
        assert!(satisfied, "the peer should be listening");

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node1,
                listener: crate::scenario::ListenerNode::Rust(node2),
            })
            .await
            .expect("connect event should be dispatched");

        let connected = driver
            .wait_for(
                Duration::from_secs(10),
                connection_finalized_event(|node_id, peer| node_id == node1 && peer == &peer_id2),
            )
            .await
            .unwrap()
            .expect("connected event");
        let state = driver
            .exec_even_step(connected)
            .await
            .unwrap()
            .expect("connected event sholuld be executed");
        assert!(
            matches!(
                state.p2p.peers.get(&peer_id2),
                Some(P2pPeerState {
                    status: P2pPeerStatus::Ready(..),
                    ..
                })
            ),
            "peer should exist"
        );
    }
}

/// Node should be able to create multiple outgoing connections.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MakeMultipleOutgoingConnections;

impl MakeMultipleOutgoingConnections {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: u8 = 32;

        let mut driver = Driver::new(runner);

        let (node_ut, _) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());
        let (peers, mut peer_ids): (Vec<ClusterNodeId>, BTreeSet<PeerId>) =
            add_rust_nodes(&mut driver, MAX, RustNodeTestingConfig::berkeley_default());

        // wait for all peers to listen
        let satisfied = wait_for_nodes_listening_on_localhost(
            &mut driver,
            Duration::from_secs(3 * 60),
            peers.clone(),
        )
        .await
        .unwrap();
        assert!(satisfied, "all peers should be listening");

        // connect node under test to all peers
        driver
            .run(Duration::from_secs(15))
            .await
            .expect("cluster should be running");
        for peer in peers {
            driver
                .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                    dialer: node_ut,
                    listener: crate::scenario::ListenerNode::Rust(peer),
                })
                .await
                .expect("connect event should be dispatched");
        }

        // matches event "the node established connection with peer"
        let pred = |node_id, event: &_, _state: &_| {
            if node_id != node_ut {
                false
            } else if let Some((peer_id, res)) = as_connection_finalized_event(event) {
                assert!(res.is_ok(), "connection to {peer_id} should succeed");
                peer_ids.remove(&peer_id);
                peer_ids.is_empty()
            } else {
                false
            }
        };

        let satisfied = driver
            .run_until(Duration::from_secs(3 * 60), pred)
            .await
            .unwrap();
        assert!(satisfied, "did not connect to peers: {:?}", peer_ids);
    }
}
