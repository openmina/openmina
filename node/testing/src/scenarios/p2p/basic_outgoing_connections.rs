use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    time::Duration,
};

use node::p2p::{
    connection::outgoing::P2pConnectionOutgoingInitLibp2pOpts, identity::SecretKey, P2pPeerState,
    P2pPeerStatus, PeerId,
};

use crate::{
    cluster::ClusterNodeId,
    node::RustNodeTestingConfig,
    scenario::ListenerNode,
    scenarios::{
        add_rust_nodes, add_rust_nodes_with, peer_is_ready, wait_for_connection_established,
        wait_for_connection_event, wait_for_nodes_listening_on_localhost, ClusterRunner, Driver,
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
        let satisfied =
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node2])
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

        let connected = wait_for_connection_established(
            &mut driver,
            Duration::from_secs(30),
            (node1, &peer_id2),
        )
        .await
        .unwrap();
        assert!(connected, "peer should be connected");

        assert!(
            peer_is_ready(driver.inner(), node1, &peer_id2),
            "peer should be ready"
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

        let connected = wait_for_connection_established(
            &mut driver,
            Duration::from_secs(3 * 60),
            (node_ut, &mut peer_ids),
        )
        .await
        .unwrap();
        assert!(connected, "did not connect to peers: {:?}", peer_ids);
    }
}

/// Node shouldn't establish connection with a node with the same peer_id.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct DontConnectToNodeWithSameId;

impl DontConnectToNodeWithSameId {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);

        let bytes: [u8; 32] = rand::random();

        // start a node with the same peer_id on different port
        let (node, _) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().with_peer_id(bytes));
        // wait for it to be ready
        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node])
                .await
                .unwrap(),
            "node should be listening"
        );

        // start node under test with the other node as its initial peer
        let (node_ut, _) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().with_peer_id(bytes));

        driver
            .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
                dialer: node_ut,
                listener: crate::scenario::ListenerNode::Rust(node),
            })
            .await
            .expect("connect event should be dispatched");

        let connected =
            wait_for_connection_established(&mut driver, Duration::from_secs(3 * 60), node_ut)
                .await
                .unwrap();
        assert!(!connected, "the node sholdn't try to connect to itself");
    }
}

/// Node shouldn't connect to itself even if its address specified in initial peers.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct DontConnectToSelfInitialPeer;

impl DontConnectToSelfInitialPeer {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);

        let node_ut = ClusterNodeId::new_unchecked(0);
        let (node_ut, _) = driver.add_rust_node(
            RustNodeTestingConfig::berkeley_default().initial_peers(vec![node_ut.into()]),
        );
        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node_ut])
                .await
                .unwrap(),
            "node should be listening"
        );

        let connected =
            wait_for_connection_established(&mut driver, Duration::from_secs(3 * 60), node_ut)
                .await
                .unwrap();
        assert!(!connected, "the node sholdn't try to connect to itself");
    }
}

/// Node shouldn't connect to a node with the same peer id even if its address specified in initial peers.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct DontConnectToInitialPeerWithSameId;

impl DontConnectToInitialPeerWithSameId {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);

        let bytes: [u8; 32] = rand::random();

        // start a node with the same peer_id on different port
        let (node, _) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().with_peer_id(bytes));
        // wait for it to be ready
        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node])
                .await
                .unwrap(),
            "node should be listening"
        );

        // start node under test with the other node as its initial peer
        let (node_ut, _) = driver.add_rust_node(
            RustNodeTestingConfig::berkeley_default()
                .with_peer_id(bytes)
                .initial_peers(vec![node.into()]),
        );

        let connected =
            wait_for_connection_established(&mut driver, Duration::from_secs(3 * 60), node_ut)
                .await
                .unwrap();
        assert!(!connected, "the node sholdn't try to connect to itself");
    }
}

/// Node should be able to connect to all initial peers.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct ConnectToInitialPeers;

impl ConnectToInitialPeers {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: u8 = 32;

        let mut driver = Driver::new(runner);

        let (peers, peer_ids): (Vec<ClusterNodeId>, Vec<_>) = add_rust_nodes_with(
            &mut driver,
            MAX,
            RustNodeTestingConfig::berkeley_default(),
            |state| {
                let config = &state.p2p.config;
                let peer_id = config.identity_pub_key.peer_id();
                peer_id
            },
        );

        // wait for all peers to listen
        let satisfied = wait_for_nodes_listening_on_localhost(
            &mut driver,
            Duration::from_secs(3 * 60),
            peers.clone(),
        )
        .await
        .unwrap();
        assert!(satisfied, "all peers should be listening");

        let initial_peers = peers.iter().map(|id| (*id).into()).collect();
        let (node_ut, _) = driver
            .add_rust_node(RustNodeTestingConfig::berkeley_default().initial_peers(initial_peers));

        // matches event "the node established connection with peer"
        let mut peer_ids = peer_ids.into_iter().collect::<BTreeSet<_>>();
        let connected = wait_for_connection_established(
            &mut driver,
            Duration::from_secs(3 * 60),
            (node_ut, &mut peer_ids),
        )
        .await
        .unwrap();
        if !connected {
            println!(
                "{:#?}",
                driver.inner().node(node_ut).unwrap().state().p2p.peers
            );
        }
        assert!(connected, "did not connect to peers: {:?}", peer_ids);
    }
}

/// Node should be able to connect to all initial peers after they become ready.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct ConnectToInitialPeersBecomeReady;

impl ConnectToInitialPeersBecomeReady {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: usize = 32;

        let mut driver = Driver::new(runner);

        let initial_peers = (0..MAX)
            .map(|i| ClusterNodeId::new_unchecked(i).into())
            .collect::<Vec<_>>();
        let (node_ut, _) = driver
            .add_rust_node(RustNodeTestingConfig::berkeley_default().initial_peers(initial_peers));

        driver
            .wait_for(Duration::from_secs(10), |_, _, _| false)
            .await
            .unwrap();

        let (_peers, mut peer_ids): (Vec<ClusterNodeId>, BTreeSet<PeerId>) = (0..MAX)
            .map(|_| driver.add_rust_node(RustNodeTestingConfig::berkeley_default()))
            .unzip();

        let connected = wait_for_connection_established(
            &mut driver,
            Duration::from_secs(3 * 60),
            (node_ut, &mut peer_ids),
        )
        .await
        .unwrap();
        if !connected {
            println!(
                "{:#?}",
                driver.inner().node(node_ut).unwrap().state().p2p.peers
            );
        }
        assert!(connected, "did not connect to peers: {:?}", peer_ids);
    }
}

/// Node should repeat connecting to unavailable initial peer.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct ConnectToUnavailableInitialPeers;

impl ConnectToUnavailableInitialPeers {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const MAX: u16 = 2;
        const RETRIES: u8 = 3;

        let mut driver = Driver::new(runner);

        let (initial_peers, ports): (Vec<_>, Vec<_>) = (0..MAX)
            .into_iter()
            .map(|i| {
                let port = 11200 + i;
                let peer_id = SecretKey::rand().public_key().peer_id();
                let addr = ListenerNode::Custom(
                    P2pConnectionOutgoingInitLibp2pOpts {
                        peer_id,
                        host: [127, 0, 0, 1].into(),
                        port,
                    }
                    .into(),
                );
                (addr, port)
            })
            .unzip();

        let (node_ut, _) = driver
            .add_rust_node(RustNodeTestingConfig::berkeley_default().initial_peers(initial_peers));

        let mut peer_retries = BTreeMap::from_iter(ports.into_iter().map(|port| (port, 0_u8)));

        let satisfied = wait_for_connection_event(
            &mut driver,
            Duration::from_secs(3 * 60),
            |node_id: ClusterNodeId, peer_addr: SocketAddr, result: &Result<_, _>| {
                println!("{peer_addr}: {result:?}");
                if node_id != node_ut {
                    return false;
                }
                assert!(
                    result.is_err(),
                    "connection to {peer_addr} shouldn't succeed"
                );
                let retries = peer_retries.get_mut(&peer_addr.port()).unwrap();
                *retries += 1;
                if *retries >= RETRIES {
                    peer_retries.remove(&peer_addr.port());
                }
                peer_retries.is_empty()
            },
        )
        .await
        .unwrap();

        println!("{:#?}", driver.inner().node(node_ut).unwrap().state().p2p);

        assert!(
            satisfied,
            "did not reach retry limit for peers: {:?}",
            peer_retries
        );
    }
}
