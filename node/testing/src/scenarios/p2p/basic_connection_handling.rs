use std::time::Duration;

use node::{
    event_source::Event,
    p2p::{
        common::P2pGenericPeer, P2pEvent, P2pLibP2pEvent, P2pLibP2pPeerState,
        P2pPeerStatus, P2pState, PeerId,
    },
};

use crate::{
    node::RustNodeTestingConfig,
    scenarios::{
        add_rust_nodes, run_until_no_events, wait_for_nodes_listening_on_localhost, ClusterRunner,
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

        let (node1, peer_id1) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());
        let (node2, peer_id2) = driver.add_rust_node(RustNodeTestingConfig::berkeley_default());

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
            Duration::from_secs(30),
            Duration::from_secs(60),
        )
        .await
        .unwrap();
        assert!(
            quiet,
            "no quiet period with no events since nodes are connected"
        );

        let p2p_state1 = &driver.inner().node(node1).unwrap().state().p2p;
        let p2p_state2 = &driver.inner().node(node2).unwrap().state().p2p;

        let node1_peer = p2p_state1.get_libp2p_peer(&peer_id2);
        let node2_peer = p2p_state2.get_libp2p_peer(&peer_id1);

        assert!(
            matches!(
                node1_peer,
                Some(P2pLibP2pPeerState {
                    status: P2pPeerStatus::Ready(..),
                    ..
                })
            ),
            "node2 should be a ready peer of node1, but it is {node1_peer:?}"
        );
        assert!(
            matches!(
                node2_peer,
                Some(P2pLibP2pPeerState {
                    status: P2pPeerStatus::Ready(..),
                    ..
                })
            ),
            "node1 should be a ready peer of node2, but it is {node2_peer:?}"
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

        let (_, (_, seed_addr)) =
            driver.add_rust_node_with(RustNodeTestingConfig::berkeley_default(), |state| {
                let config = &state.p2p.config;
                let port = config.libp2p_port.unwrap();
                let peer_id = config.identity_pub_key.peer_id();
                let addr = format!(
                    "/ip4/127.0.0.1/tcp/{port}/p2p/{}",
                    peer_id.clone().to_libp2p_string()
                )
                .parse::<P2pGenericPeer>()
                .unwrap();
                (peer_id, addr)
            });

        let peers: Vec<_> = (0..MAX)
            .into_iter()
            .map(|_| {
                driver.add_rust_node(
                    RustNodeTestingConfig::berkeley_default()
                        .initial_peers(vec![seed_addr.clone()]),
                )
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

        let (node_ut, (node_ut_peer_id, seed_addr)) =
            driver.add_rust_node_with(RustNodeTestingConfig::berkeley_default(), |state| {
                let config = &state.p2p.config;
                let port = config.libp2p_port.unwrap();
                let peer_id = config.identity_pub_key.peer_id();
                let addr = format!(
                    "/ip4/127.0.0.1/tcp/{port}/p2p/{}",
                    peer_id.clone().to_libp2p_string()
                )
                .parse::<P2pGenericPeer>()
                .unwrap();
                (peer_id, addr)
            });

        let peers: Vec<_> = (0..MAX)
            .into_iter()
            .map(|_| {
                driver.add_rust_node(
                    RustNodeTestingConfig::berkeley_default()
                        .initial_peers(vec![seed_addr.clone()]),
                )
            })
            .collect();

        // // wait for all peers to listen
        // let satisfied = wait_for_nodes_listening_on_localhost(
        //     &mut driver,
        //     Duration::from_secs(3 * 60),
        //     peers.clone(),
        // )
        // .await
        // .unwrap();
        // assert!(satisfied, "all peers should be listening");

        // Run the cluster for a while
        driver
            .run_until(Duration::from_secs(2 * 60), |_, _, _| false)
            .await
            .unwrap();

        // Check that for each peer, if it is in the node's peer list, then the node is in the peer's peer list
        let node_ut_p2p_state = &driver.inner().node(node_ut).unwrap().state().p2p;
        for (peer, peer_id) in peers {
            let peer_p2p_state = &driver.inner().node(peer).unwrap().state().p2p;

            if peer_p2p_state
                .ready_peers_iter()
                .any(|(peer_id, _)| peer_id == &node_ut_peer_id)
            {
                assert!(
                    node_ut_p2p_state
                        .ready_peers_iter()
                        .any(|(pid, _)| pid == &peer_id),
                    "node {peer} should be in the node's peer list"
                );
            } else {
                assert!(
                    !node_ut_p2p_state
                        .ready_peers_iter()
                        .any(|(pid, _)| pid == &peer_id),
                    "node {peer} should not be in the node's peer list"
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

        let (peers, _): (Vec<_>, Vec<_>) = add_rust_nodes(
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

        for peer in &peers {
            driver
                .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                    dialer: *peer,
                    listener: crate::scenario::ListenerNode::Rust(node_ut),
                })
                .await
                .expect("connect event should be dispatched");
        }

        let mut connected = 0_i32;

        while let Some(exceeded) = driver
            .wait_for(Duration::from_secs(2 * 60), |node_id, event, _| {
                if node_id != node_ut {
                    return false;
                }
                let Event::P2p(P2pEvent::LibP2p(conn_event)) = event else {
                    return false;
                };
                match conn_event {
                    node::p2p::P2pLibP2pEvent::ConnectionEstablished { .. } => {
                        connected += 1;
                    }
                    node::p2p::P2pLibP2pEvent::ConnectionClosed { .. } => {
                        connected -= 1;
                    }
                    _ => {}
                }
                return connected > MAX.into();
            })
            .await
            .unwrap()
        {
            let state = driver
                .exec_even_step(exceeded)
                .await
                .unwrap()
                .expect("connect message should be dispatched");
            let count = state.p2p.ready_peers_iter().count();
            assert!(count <= MAX.into(), "max number of peers exceeded: {count}");
        }

        // check that the number of ready peers does not exceed the maximal allowed number
        let state = driver.inner().node(node_ut).unwrap().state();
        let count = state.p2p.ready_peers_iter().count();
        assert!(count <= MAX.into(), "max number of peers exceeded: {count}");

        // check that the number of nodes with the node as their peer does not exceed the maximal allowed number
        let peers_connected = peers
            .into_iter()
            .filter_map(|peer| driver.inner().node(peer))
            .filter_map(|peer| peer.state().p2p.get_libp2p_peer(&nut_peer_id))
            .filter(|state| matches!(state.status, P2pPeerStatus::Ready(..)))
            .count();
        assert!(
            peers_connected <= MAX.into(),
            "peers connections to the node exceed the max number of connections: {peers_connected}"
        );
    }
}

/// Two nodes should stay connected for a long period of time.
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
                    Event::P2p(P2pEvent::LibP2p(P2pLibP2pEvent::ConnectionClosed { .. }))
                )
            })
            .await
            .unwrap();

        assert!(!disconnected, "there shouldn't be a disconnection");
    }
}
