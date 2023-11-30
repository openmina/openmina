#![cfg(not(feature = "p2p-webrtc"))]

use std::time::Duration;

use node::{
    event_source::Event,
    p2p::{
        connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionState},
        P2pConnectionEvent, P2pDiscoveryEvent, P2pEvent, P2pPeerStatus, PeerId,
    },
    State,
};
use openmina_node_testing::{
    cluster::{Cluster, ClusterConfig, ClusterNodeId},
    node::RustNodeTestingConfig,
    ocaml,
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{multi_node::connection_discovery::RustNodeAsSeed, ClusterRunner},
};

mod common;

struct Driver<'cluster> {
    runner: ClusterRunner<'cluster>,
}

impl<'cluster> Driver<'cluster> {
    fn new(runner: ClusterRunner<'cluster>) -> Self {
        Driver { runner }
    }

    async fn wait_for(
        &mut self,
        duration: Duration,
        f: impl Fn(ClusterNodeId, &Event, &State) -> bool,
    ) -> anyhow::Result<Option<(ClusterNodeId, Event)>> {
        let timeout = std::time::Instant::now() + duration;
        while std::time::Instant::now() < timeout {
            let mut steps = Vec::new();
            let mut found = None;
            for (node_id, state, events) in self.runner.pending_events() {
                for (_, event) in events {
                    if f(node_id, event, state) {
                        eprintln!("!!! {event:?}");
                        found = Some((node_id, event.clone()));
                        break;
                    } else {
                        eprintln!(">>> {event:?}");
                        let event = event.to_string();
                        steps.push(ScenarioStep::Event { node_id, event });
                    }
                }
            }
            for step in steps {
                self.runner.exec_step(step).await?;
            }
            if found.is_some() {
                return Ok(found);
            }
            self.idle(Duration::from_millis(10)).await?;
        }
        Ok(None)
    }

    async fn idle(&mut self, duration: Duration) -> anyhow::Result<()> {
        tokio::time::sleep(duration).await;
        self.runner
            .exec_step(ScenarioStep::AdvanceTime {
                by_nanos: 10 * 1_000_000,
            })
            .await?;
        let nodes = self
            .runner
            .cluster()
            .nodes_iter()
            .map(|(node_id, _)| node_id)
            .collect::<Vec<_>>();
        for node_id in nodes {
            self.runner
                .exec_step(ScenarioStep::CheckTimeouts { node_id })
                .await?;
        }
        Ok(())
    }

    async fn exec_step(
        &mut self,
        (node_id, event): (ClusterNodeId, Event),
    ) -> anyhow::Result<Option<&State>> {
        let event = event.to_string();
        let result = if self
            .runner
            .exec_step(ScenarioStep::Event { node_id, event })
            .await?
        {
            Some(
                self.runner
                    .cluster()
                    .node(node_id)
                    .ok_or(anyhow::format_err!("no node {}", node_id.index()))?
                    .state(),
            )
        } else {
            None
        };
        Ok(result)
    }

    #[allow(dead_code)]
    fn into_inner(self) -> ClusterRunner<'cluster> {
        self.runner
    }
}

#[tokio::test]
async fn ocaml_to_rust() {
    scenario_doc!("Tests Rust node peer discovery when OCaml node connects to it");

    let temp_dir = temp_dir::TempDir::new().unwrap();
    let dir = temp_dir.path();

    openmina_node_testing::setup_without_rt();
    let config = ClusterConfig::default();

    let mut cluster = Cluster::new(config);

    let rust_node_id = cluster.add_rust_node(RustNodeTestingConfig::berkeley_default());
    let rust_node_ma = {
        let rust_node = cluster.node(rust_node_id).unwrap();
        let state = rust_node.state();
        let port = state.p2p.config.libp2p_port.unwrap();
        let peer_id = state
            .p2p
            .config
            .identity_pub_key
            .peer_id()
            .to_libp2p_string();
        format!("/ip4/127.0.0.1/tcp/{}/p2p/{}", port, peer_id)
    };
    let runner = openmina_node_testing::scenarios::ClusterRunner::new(&mut cluster, |_| {});

    let ocaml_node =
        ocaml::Node::spawn(8302, 8303, 8301, &dir.join("ocaml"), [&rust_node_ma]).unwrap();
    let ocaml_peer_id = &ocaml_node.peer_id;

    let mut driver = Driver::new(runner);

    // wait for ocaml node to connect
    let connected = driver
        .wait_for(
            Duration::from_secs(5 * 60),
            connection_finalized_event(ocaml_peer_id.clone().into()),
        )
        .await
        .unwrap()
        .expect("expected connected event");
    // execute it
    let state = driver.exec_step(connected).await.unwrap().unwrap();
    // check that now there is an outgoing connection to the ocaml peer
    assert!(matches!(
        &state.p2p.peers.get(&ocaml_peer_id.clone().into()).unwrap().status,
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
    let state = driver.exec_step(identify).await.unwrap().unwrap();
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

fn connection_finalized_event(peer_id: PeerId) -> impl Fn(ClusterNodeId, &Event, &State) -> bool {
    move |_, event, _| {
        matches!(
            event,
            Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(peer, res))) if peer == &peer_id && res.is_ok()
        )
    }
}

fn identify_event(peer_id: PeerId) -> impl Fn(ClusterNodeId, &Event, &State) -> bool {
    move |_, event, _| {
        matches!(
            event,
            Event::P2p(P2pEvent::Libp2pIdentify(peer, _)) if peer == &peer_id
        )
    }
}

#[tokio::test]
async fn rust_to_ocaml() {
    scenario_doc!("Tests Rust node peer discovery when it connects to OCaml node");

    let temp_dir = temp_dir::TempDir::new().unwrap();
    let dir = temp_dir.path();

    openmina_node_testing::setup_without_rt();
    let config = ClusterConfig::default();

    let mut cluster = Cluster::new(config);

    let rust_node_id = cluster.add_rust_node(RustNodeTestingConfig::berkeley_default());
    let runner = openmina_node_testing::scenarios::ClusterRunner::new(&mut cluster, |_| {});

    let seed_node = ocaml::Node::spawn::<_, &str>(8302, 8303, 8301, &dir.join("seed"), []).unwrap();
    let seed_peer_id = &seed_node.peer_id;
    let seed_ma = format!("/ip4/127.0.0.1/tcp/8302/p2p/{}", seed_peer_id);

    seed_node.wait_for_p2p(Duration::from_secs(5 * 60)).unwrap();

    let mut driver = Driver::new(runner);

    driver
        .runner
        .exec_step(ScenarioStep::ConnectNodes {
            dialer: rust_node_id,
            listener: ListenerNode::Custom(seed_ma.parse().unwrap()),
        })
        .await
        .unwrap();

    // wait for conection finalize event
    let connected = driver
        .wait_for(
            Duration::from_secs(5),
            connection_finalized_event(seed_peer_id.clone().into()),
        )
        .await
        .unwrap()
        .expect("expected connected event");
    // execute it
    let state = driver.exec_step(connected).await.unwrap().unwrap();
    // check that now there is an outgoing connection to the ocaml peer
    assert!(matches!(
        &state.p2p.peers.get(&seed_peer_id.clone().into()).unwrap().status,
        P2pPeerStatus::Ready(ready) if !ready.is_incoming
    ));

    // wait for kademlia to add the ocaml peer
    let kad_add_rounte = driver.wait_for(Duration::from_secs(1), |_, event, _| {
        matches!(event, Event::P2p(P2pEvent::Discovery(P2pDiscoveryEvent::AddRoute(peer, addresses)))
                 if peer == seed_peer_id && addresses.iter().any(match_addr_with_port_and_peer_id(8302, seed_peer_id.clone().into()))
        )
    }).await.unwrap().expect("expected add route event");
    let state = driver.exec_step(kad_add_rounte).await.unwrap().unwrap();
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

fn match_addr_with_port_and_peer_id(
    port: u16,
    peer_id: PeerId,
) -> impl Fn(&P2pConnectionOutgoingInitOpts) -> bool {
    move |conn_opt| match conn_opt {
        P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) => {
            &libp2p_opts.peer_id == &peer_id && libp2p_opts.port == port
        }
        _ => false,
    }
}

#[tokio::test]
async fn ocaml_to_rust_via_seed() {
    scenario_doc!("Tests Rust node peer discovery when OCaml node is connected to it via an OCaml seed node");

    let temp_dir = temp_dir::TempDir::new().unwrap();
    let dir = temp_dir.path();

    openmina_node_testing::setup_without_rt();
    let config = ClusterConfig::default();
    let mut cluster = Cluster::new(config);

    let rust_node_id = cluster.add_rust_node(RustNodeTestingConfig::berkeley_default());
    let runner = openmina_node_testing::scenarios::ClusterRunner::new(&mut cluster, |_| {});

    let seed_node = ocaml::Node::spawn::<_, &str>(8302, 8303, 8301, &dir.join("seed"), []).unwrap();
    let seed_peer_id = &seed_node.peer_id;
    let seed_ma = format!("/ip4/127.0.0.1/tcp/8302/p2p/{}", seed_peer_id);

    seed_node.wait_for_p2p(Duration::from_secs(5 * 60)).unwrap();

    let mut driver = Driver::new(runner);

    driver
        .runner
        .exec_step(ScenarioStep::ConnectNodes {
            dialer: rust_node_id,
            listener: ListenerNode::Custom(seed_ma.parse().unwrap()),
        })
        .await
        .unwrap();

    let connected = driver
        .wait_for(Duration::from_secs(5), |_, event, _| {
            matches!(
                event,
                Event::P2p(node::p2p::P2pEvent::Connection(
                    P2pConnectionEvent::Finalized(peer, res),
                ))
                if &libp2p::PeerId::from(peer.clone()) == seed_peer_id && res.is_ok()
            )
        })
        .await
        .unwrap()
        .expect("expected connected event");

    let state = driver.exec_step(connected).await.unwrap().unwrap();
    assert!(matches!(
        &state.p2p.peers.get(&seed_peer_id.clone().into()).unwrap().status,
        P2pPeerStatus::Ready(ready) if !ready.is_incoming
    ));

    let ocaml_node =
        ocaml::Node::spawn(18302, 18303, 18301, &dir.join("ocaml"), [&seed_ma]).unwrap();
    let ocaml_peer_id = &ocaml_node.peer_id;

    driver
        .runner
        .exec_step(ScenarioStep::ManualEvent {
            node_id: rust_node_id,
            event: Box::new(Event::P2p(node::p2p::P2pEvent::Connection(
                P2pConnectionEvent::Closed((*seed_peer_id).into()),
            ))),
        })
        .await
        .unwrap();
    assert!(matches!(
        &driver
            .runner
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
                if &libp2p::PeerId::from(peer.clone()) == ocaml_peer_id && res.is_ok()
            )
        })
        .await
        .unwrap()
        .expect("expected connected event");

    let state = driver.exec_step(connected).await.unwrap().unwrap();
    assert!(matches!(
        &state.p2p.peers.get(&ocaml_peer_id.clone().into()).unwrap().status,
        P2pPeerStatus::Ready(ready) if ready.is_incoming
    ));

    driver
        .wait_for(Duration::from_secs(60), |_, _, _| false)
        .await
        .unwrap();
}

#[tokio::test]
async fn rust_to_ocaml_via_seed() {
    scenario_doc!("Tests Rust node peer discovery when it connects to OCaml node via an OCaml seed node");
    let temp_dir = temp_dir::TempDir::new().unwrap();
    let dir = temp_dir.path();

    openmina_node_testing::setup_without_rt();
    let config = ClusterConfig::default();

    let mut cluster = Cluster::new(config);

    let rust_node_id = cluster.add_rust_node(RustNodeTestingConfig::berkeley_default());
    let runner = openmina_node_testing::scenarios::ClusterRunner::new(&mut cluster, |_| {});

    let seed_node = ocaml::Node::spawn::<_, &str>(8302, 8303, 8301, &dir.join("seed"), []).unwrap();
    let seed_peer_id = &seed_node.peer_id;
    let seed_ma = format!("/ip4/127.0.0.1/tcp/8302/p2p/{}", seed_peer_id);

    tokio::time::sleep(Duration::from_secs(60)).await;

    let ocaml_node =
        ocaml::Node::spawn(18302, 18303, 18301, &dir.join("ocaml"), [&seed_ma]).unwrap();
    let ocaml_peer_id = &ocaml_node.peer_id;

    seed_node.wait_for_p2p(Duration::from_secs(5 * 60)).unwrap();
    ocaml_node
        .wait_for_p2p(Duration::from_secs(1 * 60))
        .unwrap();

    let mut driver = Driver::new(runner);

    driver
        .runner
        .exec_step(ScenarioStep::ConnectNodes {
            dialer: rust_node_id,
            listener: ListenerNode::Custom(seed_ma.parse().unwrap()),
        })
        .await
        .unwrap();

    let connected = driver
        .wait_for(Duration::from_secs(5), |_, event, _| {
            matches!(
                event,
                Event::P2p(node::p2p::P2pEvent::Connection(
                    P2pConnectionEvent::Finalized(peer, res),
                ))
                if &libp2p::PeerId::from(peer.clone()) == seed_peer_id && res.is_ok()
            )
        })
        .await
        .unwrap()
        .expect("expected connected event");

    let state = driver.exec_step(connected).await.unwrap().unwrap();
    assert!(matches!(
        &state.p2p.peers.get(&seed_peer_id.clone().into()).unwrap().status,
        P2pPeerStatus::Ready(ready) if !ready.is_incoming
    ));

    let timeout = std::time::Instant::now() + Duration::from_secs(3 * 60);
    let mut found = false;
    while !found && std::time::Instant::now() < timeout {
        let mut steps = Vec::new();
        for (node_id, state, events) in driver.runner.pending_events() {
            for (_, event) in events {
                match event {
                    Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(
                        peer,
                        Ok(()),
                    ))) if peer == ocaml_peer_id => {
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
            driver.runner.exec_step(step).await.unwrap();
        }
        if !found {
            driver.idle(Duration::from_millis(10)).await.unwrap();
        }
    }

    let p2p = &driver.runner.node(rust_node_id).unwrap().state().p2p;

    assert!(
        p2p.kademlia
            .known_peers
            .contains_key(&seed_peer_id.clone().into()),
        "should know seed node"
    );
    assert!(
        p2p.kademlia
            .known_peers
            .contains_key(&ocaml_peer_id.clone().into()),
        "should know ocaml node"
    );

    assert!(matches!(
        &p2p.peers.get(&ocaml_peer_id.clone().into()).expect("ocaml node should be connected").status,
        P2pPeerStatus::Ready(ready) if !ready.is_incoming
    ));

    driver
        .wait_for(Duration::from_secs(60), |_, _, _| false)
        .await
        .unwrap();
}

scenario_test!(rust_as_seed, RustNodeAsSeed, RustNodeAsSeed);
