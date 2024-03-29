use std::{collections::BTreeSet, time::Duration};

use crate::{
    node::RustNodeTestingConfig,
    scenarios::{
        add_rust_nodes, as_connection_finalized_event, connection_finalized_event,
        connection_finalized_with_res_event, wait_for_nodes_listening_on_localhost, ClusterRunner,
        Driver,
    },
};

/// Node should accept incoming connections.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct AcceptIncomingConnection;

impl AcceptIncomingConnection {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);

        let (node_ut, _) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());
        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node_ut])
                .await
                .unwrap(),
            "node should be listening"
        );
        let (node2, peer_id2) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node2,
                listener: crate::scenario::ListenerNode::Rust(node_ut),
            })
            .await
            .expect("node should be connected");

        // wait for node under test receives connection event
        let connected = driver
            .wait_for(
                Duration::from_secs(10),
                connection_finalized_event(|node_id, _peer| node_id == node_ut),
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
            state.p2p.peers.get(&peer_id2).is_some(),
            "peer should exist"
        );
    }
}

/// Node should accept multiple incoming connections.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct AcceptMultipleIncomingConnections;

impl AcceptMultipleIncomingConnections {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: u8 = 32;

        let mut driver = Driver::new(runner);

        let (node_ut, _) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());
        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node_ut])
                .await
                .unwrap(),
            "node should be listening"
        );

        let (peers, mut peer_ids): (Vec<_>, BTreeSet<_>) =
            add_rust_nodes(&mut driver, MAX, RustNodeTestingConfig::berkeley_default());

        // connect peers to the node under test
        for peer in peers {
            driver
                .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                    dialer: peer,
                    listener: crate::scenario::ListenerNode::Rust(node_ut),
                })
                .await
                .expect("connect event should be dispatched");
        }

        // matches event "the node established connection with peer"
        let pred = |node_id, event: &_, _state: &_| {
            if node_id != node_ut {
                false
            } else if let Some((peer_id, res)) = as_connection_finalized_event(event) {
                assert!(res.is_ok(), "connection from {peer_id} should succeed");
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
        assert!(
            satisfied,
            "did not accept connection from peers: {:?}",
            peer_ids
        );
    }
}

/// Node should not accept connection from itself.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct DoesNotAcceptConnectionFromSelf;

impl DoesNotAcceptConnectionFromSelf {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);
        let (node_ut, node_ut_peer_id) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default());

        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(60), [node_ut])
                .await
                .unwrap(),
            "node should be listening"
        );

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node_ut,
                listener: crate::scenario::ListenerNode::Rust(node_ut),
            })
            .await
            .expect("connect event should be dispatched"); // should it?

        // wait for node under test receives connection event
        let connected = driver
            .wait_for(
                Duration::from_secs(10),
                connection_finalized_with_res_event(|node_id, peer, res| {
                    node_id == node_ut && peer == &node_ut_peer_id && res.is_err()
                }),
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
            state.p2p.get_ready_peer(&node_ut_peer_id).is_none(),
            "self-peer should not be ready"
        );
    }
}
