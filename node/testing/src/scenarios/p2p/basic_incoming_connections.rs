use std::{collections::BTreeSet, time::Duration};

use crate::{
    node::RustNodeTestingConfig,
    scenarios::{
        add_rust_nodes, peer_is_ready, wait_for_connection_established,
        wait_for_nodes_listening_on_localhost, ClusterRunner, Driver,
    },
};

/// Node should accept incoming connections.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct AcceptIncomingConnection;

impl AcceptIncomingConnection {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);

        let (node_ut, _) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());
        let (node2, peer_id2) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());
        assert!(
            wait_for_nodes_listening_on_localhost(
                &mut driver,
                Duration::from_secs(30),
                [node_ut, node2]
            )
            .await
            .unwrap(),
            "node should be listening"
        );

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node2,
                listener: crate::scenario::ListenerNode::Rust(node_ut),
            })
            .await
            .expect("node should be connected");

        // wait for node under test receives connection event
        let connected = wait_for_connection_established(
            &mut driver,
            Duration::from_secs(30),
            (node_ut, &peer_id2),
        )
        .await
        .unwrap();
        assert!(connected, "peer should be connected");

        assert!(
            peer_is_ready(driver.inner(), node_ut, &peer_id2),
            "peer should be ready, but it is \n{:#?}",
            driver
                .inner()
                .node(node_ut)
                .unwrap()
                .state()
                .p2p
                .get_peer(&peer_id2)
        );
    }
}

/// Node should accept multiple incoming connections.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct AcceptMultipleIncomingConnections;

impl AcceptMultipleIncomingConnections {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: u8 = 16;

        let mut driver = Driver::new(runner);

        let (node_ut, _) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());

        let (peers, mut peer_ids): (Vec<_>, BTreeSet<_>) =
            add_rust_nodes(&mut driver, MAX, RustNodeTestingConfig::berkeley_default());

        assert!(
            wait_for_nodes_listening_on_localhost(
                &mut driver,
                Duration::from_secs(30),
                peers.clone().into_iter().chain(std::iter::once(node_ut))
            )
            .await
            .unwrap(),
            "node should be listening"
        );

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

        // wait for node under test receives connection event
        let all_connected = wait_for_connection_established(
            &mut driver,
            Duration::from_secs(2 * 60),
            (node_ut, &mut peer_ids),
        )
        .await
        .unwrap();
        assert!(
            all_connected,
            "did not accept connection from peers: {:?}",
            peer_ids
        );
    }
}
