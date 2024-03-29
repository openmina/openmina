use std::time::Duration;

use node::{
    event_source::Event,
    p2p::{
        connection::P2pConnectionState, P2pConnectionEvent, P2pDiscoveryEvent, P2pEvent,
        P2pPeerStatus,
    },
};
use tokio::time::Instant;

use crate::{
    node::{DaemonJson, OcamlNodeTestingConfig, OcamlStep, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{
        as_connection_finalized_event, connection_finalized_event, get_peers_iter, identify_event,
        match_addr_with_port_and_peer_id, ClusterRunner, Driver, PEERS_QUERY,
    },
};

/// Ensure that Rust node can pass information about peers when used as a seed node.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RustNodeAsSeed;

impl RustNodeAsSeed {
    pub async fn run<'a>(self, mut runner: ClusterRunner<'a>) {
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::berkeley_default());
        let rust_node_dial_addr = runner.node(rust_node_id).unwrap().dial_addr();

        let ocaml_node_config = OcamlNodeTestingConfig {
            initial_peers: vec![rust_node_dial_addr],
            daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
            block_producer: None,
        };

        let ocaml_node0 = runner.add_ocaml_node(ocaml_node_config.clone());
        let ocaml_peer_id0 = runner.ocaml_node(ocaml_node0).unwrap().peer_id();

        let ocaml_node1 = runner.add_ocaml_node(ocaml_node_config.clone());
        let ocaml_peer_id1 = runner.ocaml_node(ocaml_node1).unwrap().peer_id();

        let mut peers = vec![ocaml_peer_id0, ocaml_peer_id1];
        let mut duration = Duration::from_secs(8 * 60);
        let mut driver = Driver::new(runner);
        while !peers.is_empty() {
            // wait for ocaml node to connect
            let connected = driver
                .wait_for(
                    duration,
                    connection_finalized_event(|_, peer| peers.contains(peer)),
                )
                .await
                .unwrap()
                .expect("expected connected event");
            let (ocaml_peer, _) = as_connection_finalized_event(&connected.1).unwrap();
            peers.retain(|peer| peer != ocaml_peer);
            let ocaml_peer = ocaml_peer.clone();
            // execute it
            let state = driver.exec_even_step(connected).await.unwrap().unwrap();
            // check that now there is an outgoing connection to the ocaml peer
            assert!(matches!(
                &state.p2p.peers.get(&ocaml_peer).unwrap().status,
                P2pPeerStatus::Ready(ready) if ready.is_incoming
            ));
            duration = Duration::from_secs(1 * 60);
        }

        let timeout = Instant::now() + Duration::from_secs(60);
        let mut node0_has_node1 = false;
        let mut node1_has_node0 = false;
        while !node0_has_node1 && !node1_has_node0 && Instant::now() < timeout {
            let node0_peers = driver
                .inner()
                .ocaml_node(ocaml_node0)
                .unwrap()
                .grapql_query(PEERS_QUERY)
                .expect("peers graphql query");
            println!("{}", serde_json::to_string_pretty(&node0_peers).unwrap());
            node0_has_node1 = get_peers_iter(&node0_peers)
                .unwrap()
                .any(|peer| peer.unwrap().2 == &ocaml_peer_id1.to_string());

            let node1_peers = driver
                .inner()
                .ocaml_node(ocaml_node1)
                .unwrap()
                .grapql_query(PEERS_QUERY)
                .expect("peers graphql query");
            println!("{}", serde_json::to_string_pretty(&node1_peers).unwrap());
            node1_has_node0 = get_peers_iter(&node1_peers)
                .unwrap()
                .any(|peer| peer.unwrap().2 == &ocaml_peer_id0.to_string());

            tokio::time::sleep(Duration::from_secs(10)).await;
        }

        assert!(
            node0_has_node1,
            "ocaml node0 should have node1 as its peers"
        );
        assert!(
            node1_has_node0,
            "ocaml node1 should have node0 as its peers"
        );

        let state = driver.inner().node(rust_node_id).unwrap().state();
        assert!(
            state.p2p.kademlia.known_peers.contains_key(&ocaml_peer_id0),
            "kademlia in rust seed statemachine should know ocaml node0"
        );
        assert!(
            state.p2p.kademlia.known_peers.contains_key(&ocaml_peer_id1),
            "kademlia in rust seed statemachine should know ocaml node1"
        );
    }
}

/// Test Rust node peer discovery when OCaml node connects to it
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct OCamlToRust;

impl OCamlToRust {
    pub async fn run<'a>(self, mut runner: ClusterRunner<'a>) {
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::berkeley_default());
        let rust_node_dial_addr = runner.node(rust_node_id).unwrap().dial_addr();

        let ocaml_node_config = OcamlNodeTestingConfig {
            initial_peers: vec![rust_node_dial_addr],
            daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
            block_producer: None,
        };

        let ocaml_node = runner.add_ocaml_node(ocaml_node_config.clone());
        let ocaml_peer_id = runner.ocaml_node(ocaml_node).unwrap().peer_id();

        let mut driver = Driver::new(runner);

        // wait for ocaml node to connect
        let connected = driver
            .wait_for(
                Duration::from_secs(5 * 60),
                connection_finalized_event(|_, peer| peer == &ocaml_peer_id),
            )
            .await
            .unwrap()
            .expect("expected connected event");
        // execute it
        let state = driver.exec_even_step(connected).await.unwrap().unwrap();
        // check that now there is an outgoing connection to the ocaml peer
        assert!(matches!(
            &state.p2p.peers.get(&ocaml_peer_id).unwrap().status,
            P2pPeerStatus::Ready(ready) if ready.is_incoming
        ));

        // wait for identify message
        let identify = driver
            .wait_for(
                Duration::from_secs(5 * 60),
                identify_event(ocaml_peer_id.clone().into()),
            )
            .await
            .unwrap()
            .expect("expected connected event");
        // execute it
        let state = driver.exec_even_step(identify).await.unwrap().unwrap();
        // check that the peer address is added to kademlia
        assert!(
            state
                .p2p
                .kademlia
                .routes
                .get(&ocaml_peer_id.clone().into())
                .map_or(false, |l| !l.is_empty()),
            "kademlia should know ocaml node's addresses"
        );
    }
}

/// Tests Rust node peer discovery when it connects to OCaml node
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RustToOCaml;

impl RustToOCaml {
    pub async fn run<'a>(self, mut runner: ClusterRunner<'a>) {
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::berkeley_default());

        let ocaml_seed_config = OcamlNodeTestingConfig {
            initial_peers: Vec::new(),
            daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
            block_producer: None,
        };

        let seed_node = runner.add_ocaml_node(ocaml_seed_config);
        let seed_peer_id = runner.ocaml_node(seed_node).unwrap().peer_id();

        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: seed_node,
                step: OcamlStep::WaitReady {
                    timeout: Duration::from_secs(5 * 60),
                },
            })
            .await
            .unwrap();

        let mut driver = Driver::new(runner);

        driver
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: rust_node_id,
                listener: ListenerNode::Ocaml(seed_node),
            })
            .await
            .unwrap();

        // wait for conection finalize event
        let connected = driver
            .wait_for(
                Duration::from_secs(5),
                connection_finalized_event(|_, peer| peer == &seed_peer_id),
            )
            .await
            .unwrap()
            .expect("expected connected event");
        // execute it
        let state = driver.exec_even_step(connected).await.unwrap().unwrap();
        // check that now there is an outgoing connection to the ocaml peer
        assert!(matches!(
            &state.p2p.peers.get(&seed_peer_id.clone().into()).unwrap().status,
            P2pPeerStatus::Ready(ready) if !ready.is_incoming
        ));

        // wait for kademlia to add the ocaml peer
        let kad_add_rounte = driver.wait_for(Duration::from_secs(1), |_, event, _| {
            matches!(event, Event::P2p(P2pEvent::Discovery(P2pDiscoveryEvent::AddRoute(peer, addresses)))
                     if peer == &seed_peer_id && addresses.iter().any(match_addr_with_port_and_peer_id(8302, seed_peer_id.clone().into()))
            )
        }).await.unwrap().expect("expected add route event");
        let state = driver
            .exec_even_step(kad_add_rounte)
            .await
            .unwrap()
            .unwrap();
        assert!(
            state
                .p2p
                .kademlia
                .routes
                .get(&seed_peer_id.clone().into())
                .map_or(false, |l| !l.is_empty()),
            "kademlia should know ocaml node's addresses"
        );
    }
}

/// Tests Rust node peer discovery when OCaml node is connected to it via an OCaml seed node.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct OCamlToRustViaSeed;

impl OCamlToRustViaSeed {
    pub async fn run<'a>(self, mut runner: ClusterRunner<'a>) {
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::berkeley_default());

        let ocaml_seed_config = OcamlNodeTestingConfig {
            initial_peers: Vec::new(),
            daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
            block_producer: None,
        };

        let seed_node = runner.add_ocaml_node(ocaml_seed_config.clone());
        let (seed_peer_id, seed_addr) = runner
            .ocaml_node(seed_node)
            .map(|node| (node.peer_id(), node.dial_addr()))
            .unwrap();

        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: seed_node,
                step: OcamlStep::WaitReady {
                    timeout: Duration::from_secs(5 * 60),
                },
            })
            .await
            .unwrap();

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: rust_node_id,
                listener: ListenerNode::Ocaml(seed_node),
            })
            .await
            .unwrap();

        let mut driver = Driver::new(runner);

        let connected = driver
            .wait_for(
                Duration::from_secs(5),
                connection_finalized_event(|_, peer| peer == &seed_peer_id),
            )
            .await
            .unwrap()
            .expect("expected connected event");

        let state = driver.exec_even_step(connected).await.unwrap().unwrap();
        assert!(matches!(
            &state.p2p.peers.get(&seed_peer_id.clone().into()).unwrap().status,
            P2pPeerStatus::Ready(ready) if !ready.is_incoming
        ));

        let ocaml_node = driver.inner_mut().add_ocaml_node(OcamlNodeTestingConfig {
            initial_peers: vec![seed_addr],
            ..ocaml_seed_config
        });
        let ocaml_peer_id = driver.inner().ocaml_node(ocaml_node).unwrap().peer_id();

        driver
            .exec_step(ScenarioStep::ManualEvent {
                node_id: rust_node_id,
                event: Box::new(Event::P2p(node::p2p::P2pEvent::Connection(
                    P2pConnectionEvent::Closed(seed_peer_id.clone()),
                ))),
            })
            .await
            .unwrap();
        assert!(matches!(
            &driver
                .inner()
                .node(rust_node_id)
                .unwrap()
                .state()
                .p2p
                .peers
                .get(&seed_peer_id.clone().into())
                .unwrap()
                .status,
            P2pPeerStatus::Disconnected { .. }
        ));

        let connected = driver
            .wait_for(Duration::from_secs(5 * 60), |_, event, _| {
                matches!(
                    event,
                    Event::P2p(node::p2p::P2pEvent::Connection(
                        P2pConnectionEvent::Finalized(peer, res),
                    ))
                        if peer == &ocaml_peer_id && res.is_ok()
                )
            })
            .await
            .unwrap()
            .expect("expected connected event");

        let state = driver.exec_even_step(connected).await.unwrap().unwrap();
        assert!(matches!(
            &state.p2p.peers.get(&ocaml_peer_id).unwrap().status,
            P2pPeerStatus::Ready(ready) if ready.is_incoming
        ));
    }
}

/// Tests Rust node peer discovery when it connects to OCaml node via an OCaml seed node.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RustToOCamlViaSeed;

impl RustToOCamlViaSeed {
    pub async fn run<'a>(self, mut runner: ClusterRunner<'a>) {
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::berkeley_default());

        let ocaml_seed_config = OcamlNodeTestingConfig {
            initial_peers: Vec::new(),
            daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
            block_producer: None,
        };

        let seed_node = runner.add_ocaml_node(ocaml_seed_config.clone());
        let (seed_peer_id, seed_addr) = runner
            .ocaml_node(seed_node)
            .map(|node| (node.peer_id(), node.dial_addr()))
            .unwrap();

        tokio::time::sleep(Duration::from_secs(60)).await;

        let ocaml_node = runner.add_ocaml_node(OcamlNodeTestingConfig {
            initial_peers: vec![seed_addr],
            ..ocaml_seed_config
        });
        let ocaml_peer_id = runner.ocaml_node(ocaml_node).unwrap().peer_id();

        let wait_step = OcamlStep::WaitReady {
            timeout: Duration::from_secs(1 * 60),
        };
        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: seed_node,
                step: wait_step.clone(),
            })
            .await
            .unwrap();
        runner
            .exec_step(ScenarioStep::Ocaml {
                node_id: ocaml_node,
                step: wait_step,
            })
            .await
            .unwrap();

        let mut driver = Driver::new(runner);

        driver
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: rust_node_id,
                listener: ListenerNode::Ocaml(seed_node),
            })
            .await
            .unwrap();

        let connected = driver
            .wait_for(
                Duration::from_secs(5),
                connection_finalized_event(|_, peer| peer == &seed_peer_id),
            )
            .await
            .unwrap()
            .expect("expected connected event");

        let state = driver.exec_even_step(connected).await.unwrap().unwrap();
        assert!(matches!(
            &state.p2p.peers.get(&seed_peer_id.clone().into()).unwrap().status,
            P2pPeerStatus::Ready(ready) if !ready.is_incoming
        ));

        let timeout = std::time::Instant::now() + Duration::from_secs(3 * 60);
        let mut found = false;
        while !found && std::time::Instant::now() < timeout {
            let mut steps = Vec::new();
            for (node_id, state, events) in driver.inner_mut().pending_events(true) {
                for (_, event) in events {
                    match event {
                        Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(
                            peer,
                            Ok(()),
                        ))) if peer == &ocaml_peer_id => {
                            if let Some(peer_state) = &state.p2p.peers.get(peer) {
                                let status = &peer_state.status;
                                if let P2pPeerStatus::Connecting(P2pConnectionState::Incoming(..)) =
                                    status
                                {
                                    steps.push(ScenarioStep::ManualEvent {
                                        node_id,
                                        event: Box::new(Event::P2p(P2pEvent::Connection(
                                            P2pConnectionEvent::Closed(peer.clone()),
                                        ))),
                                    });
                                } else {
                                    steps.push(ScenarioStep::Event {
                                        node_id,
                                        event: event.to_string(),
                                    });
                                    found = true;
                                }
                            }
                        }
                        _ => {
                            steps.push(ScenarioStep::Event {
                                node_id,
                                event: event.to_string(),
                            });
                        }
                    }
                }
            }
            for step in steps {
                driver.exec_step(step).await.unwrap();
            }
            if !found {
                driver.idle(Duration::from_millis(10)).await.unwrap();
            }
        }

        let p2p = &driver.inner().node(rust_node_id).unwrap().state().p2p;

        assert!(
            p2p.kademlia.known_peers.contains_key(&seed_peer_id),
            "should know seed node"
        );
        assert!(
            p2p.kademlia.known_peers.contains_key(&ocaml_peer_id),
            "should know ocaml node"
        );

        assert!(matches!(
            &p2p.peers.get(&ocaml_peer_id).expect("ocaml node should be connected").status,
            P2pPeerStatus::Ready(ready) if !ready.is_incoming
        ));
    }
}
