use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use node::p2p::{
    connection::outgoing::{P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts},
    identity::SecretKey,
    P2pPeerStatus, P2pTimeouts, PeerId,
};

use crate::{
    cluster::ClusterNodeId,
    node::RustNodeTestingConfig,
    scenario::ListenerNode,
    scenarios::{
        add_rust_nodes, add_rust_nodes_with, peer_is_ready, wait_for_connection_error,
        wait_for_connection_established, wait_for_nodes_listening_on_localhost, ClusterRunner,
        Driver,
    },
};

fn custom_listener(peer_id: PeerId, port: u16) -> ListenerNode {
    P2pConnectionOutgoingInitOpts::LibP2P(P2pConnectionOutgoingInitLibp2pOpts {
        peer_id,
        host: node::p2p::webrtc::Host::Ipv4([127, 0, 0, 1].into()),
        port,
    })
    .into()
}

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

        let config =
            RustNodeTestingConfig::berkeley_default().with_timeouts(P2pTimeouts::without_rpc());

        let (node_ut, _) = driver.add_rust_node(config.clone());
        let (peers, mut peer_ids): (Vec<ClusterNodeId>, BTreeSet<PeerId>) =
            add_rust_nodes(&mut driver, MAX, config.clone());

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

        let bytes = [0xfe; 32];
        let port = 13001;
        let peer_id = SecretKey::from_bytes(bytes).public_key().peer_id();
        let self_opts =
            P2pConnectionOutgoingInitOpts::LibP2P(P2pConnectionOutgoingInitLibp2pOpts {
                peer_id: peer_id.clone(),
                host: node::p2p::webrtc::Host::Ipv4([127, 0, 0, 1].into()),
                port,
            });
        let (node_ut, _) = driver.add_rust_node(
            RustNodeTestingConfig::berkeley_default()
                .with_peer_id(bytes)
                .with_libp2p_port(port)
                .initial_peers(vec![self_opts.into()]),
        );
        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node_ut])
                .await
                .unwrap(),
            "node should be listening"
        );

        let connected =
            wait_for_connection_established(&mut driver, Duration::from_secs(10), node_ut)
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
            |state| state.p2p.my_id(),
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
                driver
                    .inner()
                    .node(node_ut)
                    .unwrap()
                    .state()
                    .p2p
                    .unwrap()
                    .peers
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
        const MAX: u8 = 32;

        let mut driver = Driver::new(runner);

        let (initial_peers, peer_id_bytes_port): (Vec<_>, Vec<_>) = (0..MAX)
            .map(|i| {
                let bytes = [i + 1; 32];
                let port = 12000 + i as u16;
                let init_opts =
                    custom_listener(SecretKey::from_bytes(bytes).public_key().peer_id(), port);
                (init_opts, (bytes, port))
            })
            .unzip();
        let (node_ut, _) = driver
            .add_rust_node(RustNodeTestingConfig::berkeley_default().initial_peers(initial_peers));

        driver
            .wait_for(Duration::from_secs(10), |_, _, _| false)
            .await
            .unwrap();

        let (_peers, mut peer_ids): (Vec<ClusterNodeId>, BTreeSet<PeerId>) = peer_id_bytes_port
            .into_iter()
            .map(|(peer_id_bytes, port)| {
                driver.add_rust_node(
                    RustNodeTestingConfig::berkeley_default()
                        .with_libp2p_port(port)
                        .with_peer_id(peer_id_bytes),
                )
            })
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
                driver
                    .inner()
                    .node(node_ut)
                    .unwrap()
                    .state()
                    .p2p
                    .unwrap()
                    .peers
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

        let (initial_peers, peer_ids): (Vec<_>, Vec<_>) = (0..MAX)
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
                (addr, peer_id)
            })
            .unzip();

        let (node_ut, _) = driver.add_rust_node(
            RustNodeTestingConfig::berkeley_default()
                .initial_peers(initial_peers)
                .with_timeouts(P2pTimeouts {
                    outgoing_error_reconnect_timeout: Some(Duration::from_secs(3)),
                    ..Default::default()
                }),
        );

        let mut peer_retries = BTreeMap::from_iter(peer_ids.into_iter().map(|port| (port, 0_u8)));

        let satisfied = wait_for_connection_error(
            &mut driver,
            Duration::from_secs(3 * 60),
            |node_id: ClusterNodeId, peer_id: &PeerId, peer_status: &P2pPeerStatus| {
                if node_id != node_ut {
                    return false;
                }
                assert!(
                    peer_status.is_error(),
                    "connection to {peer_id} shouldn't succeed"
                );
                let retries = peer_retries.get_mut(peer_id).unwrap();
                *retries += 1;
                if *retries >= RETRIES {
                    peer_retries.remove(peer_id);
                }
                peer_retries.is_empty()
            },
        )
        .await
        .unwrap();

        assert!(
            satisfied,
            "did not reach retry limit for peers: {:?}",
            peer_retries
        );
    }
}
