use std::time::Duration;

use node::{
    event_source::Event,
    p2p::{P2pConnectionEvent, P2pEvent, P2pPeerStatus, P2pState, P2pTimeouts, PeerId},
};

use crate::{
    node::RustNodeTestingConfig,
    scenarios::{
        add_rust_nodes1, connect_rust_nodes, get_peer_state, peer_is_ready, run_until_no_events,
        wait_for_connection_established, wait_for_nodes_listening_on_localhost, ClusterRunner,
        Driver,
    },
};

fn has_active_peer(p2p_state: &P2pState, peer_id: &PeerId) -> bool {
    p2p_state.ready_peers_iter().any(|(id, _)| id == peer_id)
}

/// Two nodes should properly handle a situation when they are connecting to each other simultaneously.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SimultaneousConnections;

impl SimultaneousConnections {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);

        let testing_config = RustNodeTestingConfig::berkeley_default().with_timeouts(P2pTimeouts {
            // test might be failing because of best tip RPC timeout...
            best_tip_with_proof: None,
            ..Default::default()
        });
        let (node1, peer_id1) = driver.add_rust_node(testing_config.clone());
        let (node2, peer_id2) = driver.add_rust_node(testing_config);

        assert!(
            wait_for_nodes_listening_on_localhost(
                &mut driver,
                Duration::from_secs(30),
                [node1, node2]
            )
            .await
            .unwrap(),
            "nodes should be listening"
        );

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node1,
                listener: crate::scenario::ListenerNode::Rust(node2),
            })
            .await
            .expect("connect event should be dispatched");
        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node2,
                listener: crate::scenario::ListenerNode::Rust(node1),
            })
            .await
            .expect("connect event should be dispatched");

        // Run the cluster while there are events
        let quiet = run_until_no_events(
            &mut driver,
            Duration::from_secs(10),
            Duration::from_secs(20),
        )
        .await
        .unwrap();
        assert!(
            quiet,
            "no quiet period with no events since nodes are connected"
        );

        assert!(
            peer_is_ready(driver.inner(), node1, &peer_id2),
            "node2 should be a ready peer of node1, but it is {:?}",
            get_peer_state(driver.inner(), node1, &peer_id2)
        );
        assert!(
            peer_is_ready(driver.inner(), node2, &peer_id1),
            "node2 should be a ready peer of node1, but it is {:?}",
            get_peer_state(driver.inner(), node2, &peer_id1)
        );
    }
}

/// Connections between all peers are symmetric, i.e. iff the node1 has the node2 among its active peers, then the node2 should have the node1 as its active peers.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct AllNodesConnectionsAreSymmetric;

impl AllNodesConnectionsAreSymmetric {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: u16 = 32;

        let mut driver = Driver::new(runner);

        let testing_config = RustNodeTestingConfig::berkeley_default().with_timeouts(P2pTimeouts {
            // test might be failing because of best tip RPC timeout...
            best_tip_with_proof: None,
            ..Default::default()
        });

        let (seed_id, _) = driver.add_rust_node(testing_config.clone());

        let peers: Vec<_> = (0..MAX)
            .into_iter()
            .map(|_| {
                driver.add_rust_node(testing_config.clone().initial_peers(vec![seed_id.into()]))
            })
            .collect();

        // Run the cluster while there are events
        let quiet = run_until_no_events(
            &mut driver,
            Duration::from_secs(30),
            Duration::from_secs(2 * 60),
        )
        .await
        .unwrap();
        assert!(
            quiet,
            "no quiet period with no events since nodes are connected"
        );

        // Check that for each peer, if it is in the node's peer list, then the node is in the peer's peer list
        for (peer1, peer_id1) in &peers {
            let peer1_p2p_state = &driver.inner().node(*peer1).unwrap().state().p2p;
            for (peer2, peer_id2) in &peers {
                if peer2 == peer1 {
                    continue;
                }
                let peer2_p2p_state = &driver.inner().node(*peer2).unwrap().state().p2p;

                if has_active_peer(peer2_p2p_state, peer_id1) {
                    assert!(
                        has_active_peer(peer1_p2p_state, peer_id2),
                        "node {peer2} should be an active peer of the node {peer1}, but it is {:?}",
                        peer1_p2p_state.peers.get(peer_id2)
                    );
                } else {
                    assert!(
                        !has_active_peer(peer1_p2p_state, peer_id2),
                        "node {peer2} should not be an active peer of the node {peer1}, but it is"
                    );
                }
            }
        }
    }
}

/// Connections with other peers are symmetric for seed node, i.e. iff a node is the seed's peer, then it has the node among its peers.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SeedConnectionsAreSymmetric;

impl SeedConnectionsAreSymmetric {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: u16 = 32;

        let mut driver = Driver::new(runner);

        let (node_ut, node_ut_peer_id) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default());

        let peers: Vec<_> = (0..MAX)
            .into_iter()
            .map(|_| {
                driver.add_rust_node(
                    RustNodeTestingConfig::berkeley_default().initial_peers(vec![node_ut.into()]),
                )
            })
            .collect();

        // Run the cluster for a while
        driver
            .run_until(Duration::from_secs(2 * 60), |_, _, _| false)
            .await
            .unwrap();

        // Check that for each peer, if it is in the node's peer list, then the node is in the peer's peer list
        for (peer, peer_id) in peers {
            if peer_is_ready(driver.inner(), peer, &node_ut_peer_id) {
                assert!(
                    peer_is_ready(driver.inner(), node_ut, &peer_id),
                    "node {peer} should be in the node's peer list"
                );
            } else {
                assert!(
                    !peer_is_ready(driver.inner(), node_ut, &peer_id),
                    "node {peer} should't be in the node's peer list"
                );
            }
        }
    }
}

/// A Rust node's incoming connections should be limited.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MaxNumberOfPeers;

impl MaxNumberOfPeers {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const TOTAL: u16 = 512;
        const MAX: u16 = 32;

        let mut driver = Driver::new(runner);

        let (node_ut, nut_peer_id) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(MAX.into()));

        let peers: Vec<_> = add_rust_nodes1(
            &mut driver,
            TOTAL,
            RustNodeTestingConfig::berkeley_default(),
        );

        // wait for all peers to listen
        let satisfied = wait_for_nodes_listening_on_localhost(
            &mut driver,
            Duration::from_secs(3 * 60),
            [node_ut],
        )
        .await
        .unwrap();
        assert!(satisfied, "all peers should be listening");

        println!("connecting nodes....");

        for (peer, peer_id) in &peers {
            connect_rust_nodes(driver.inner_mut(), *peer, node_ut).await;
            let connected = wait_for_connection_established(
                &mut driver,
                Duration::from_secs(10),
                (node_ut, peer_id),
            )
            .await
            .unwrap();
            assert!(connected, "node {peer} is not connected");
        }

        // let mut connected = 0_i32;

        // while let Some(exceeded) = driver
        //     .wait_for(Duration::from_secs(2 * 60), |node_id, event, _| {
        //         if node_id != node_ut {
        //             return false;
        //         }
        //         let Event::P2p(P2pEvent::Connection(conn_event)) = event else {
        //             return false;
        //         };
        //         match conn_event {
        //             node::p2p::P2pConnectionEvent::Finalized(_, Ok(())) => {
        //                 connected += 1;
        //             }
        //             node::p2p::P2pConnectionEvent::Closed(_) => {
        //                 connected -= 1;
        //             }
        //             _ => {}
        //         }
        //         return connected > MAX.into();
        //     })
        //     .await
        //     .unwrap()
        // {
        //     let state = driver
        //         .exec_even_step(exceeded)
        //         .await
        //         .unwrap()
        //         .expect("connect message should be dispatched");
        //     let count = state.p2p.ready_peers_iter().count();
        //     assert!(count <= MAX.into(), "max number of peers exceeded: {count}");
        // }

        driver.run(Duration::from_secs(1 * 60)).await.unwrap();

        // check that the number of ready peers does not exceed the maximal allowed number
        let state = driver.inner().node(node_ut).unwrap().state();
        let count = state.p2p.ready_peers_iter().count();
        assert!(count <= MAX.into(), "max number of peers exceeded: {count}");

        // check that the number of nodes with the node as their peer does not exceed the maximal allowed number
        let peers_connected = peers
            .into_iter()
            .filter_map(|(peer, _)| driver.inner().node(peer))
            .filter_map(|peer| peer.state().p2p.peers.get(&nut_peer_id))
            .filter(|state| matches!(state.status, P2pPeerStatus::Ready(..)))
            .count();
        assert!(
            peers_connected <= MAX.into(),
            "peers connections to the node exceed the max number of connections: {peers_connected}"
        );
    }
}

/// Two nodes with max peers = 1 can connect to each other.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MaxNumberOfPeersIs1;

impl MaxNumberOfPeersIs1 {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const CONNECTED_TIME_SEC: u64 = 10;
        let mut driver = Driver::new(runner);

        let (node1, _) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(1));
        let (node2, _) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(1));

        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node2])
                .await
                .unwrap(),
            "nodes should be listening"
        );

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node1,
                listener: crate::scenario::ListenerNode::Rust(node2),
            })
            .await
            .expect("connect event should be dispatched");

        // Run the cluster while there are events
        let disconnected = driver
            .run_until(Duration::from_secs(CONNECTED_TIME_SEC), |_, event, _| {
                matches!(
                    event,
                    Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Closed(_)))
                )
            })
            .await
            .unwrap();

        assert!(!disconnected, "there shouldn't be a disconnection");
    }
}

/// Two nodes should stay connected for a long period of time.
///
/// TODO: this is worth to make it slightly more sophisticated...
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct ConnectionStability;

impl ConnectionStability {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const CONNECTED_TIME_SEC: u64 = 1 * 60;
        let mut driver = Driver::new(runner);

        let (node1, _) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(1));
        let (node2, _) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(1));

        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node2])
                .await
                .unwrap(),
            "nodes should be listening"
        );

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node1,
                listener: crate::scenario::ListenerNode::Rust(node2),
            })
            .await
            .expect("connect event should be dispatched");

        // Run the cluster while there are events
        let disconnected = driver
            .run_until(Duration::from_secs(CONNECTED_TIME_SEC), |_, event, _| {
                matches!(
                    event,
                    Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Closed(_)))
                )
            })
            .await
            .unwrap();

        assert!(!disconnected, "there shouldn't be a disconnection");
    }
}
