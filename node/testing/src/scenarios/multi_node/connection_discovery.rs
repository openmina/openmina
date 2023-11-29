use std::time::Duration;

use node::{
    event_source::Event,
    p2p::{P2pConnectionEvent, P2pEvent, P2pPeerStatus, PeerId},
    State,
};
use tokio::time::Instant;

use crate::{
    cluster::ClusterNodeId, node::RustNodeTestingConfig, ocaml, scenario::ScenarioStep,
    scenarios::ClusterRunner,
};

/// Ensure that Rust node can pass information about peers when used as a seed node.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RustNodeAsSeed;

impl RustNodeAsSeed {
    pub async fn run<'a>(self, mut runner: ClusterRunner<'a>) {
        let temp_dir = temp_dir::TempDir::new().unwrap();
        let dir = temp_dir.path();

        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::berkeley_default());
        let rust_node_ma = {
            let rust_node = runner.node(rust_node_id).unwrap();
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

        let mut driver = Driver::new(runner);

        let ocaml_node0 =
            ocaml::Node::spawn(8302, 8303, 8301, &dir.join("ocaml0"), [&rust_node_ma]).unwrap();
        let ocaml_peer_id0 = ocaml_node0.peer_id().into();

        let ocaml_node1 =
            ocaml::Node::spawn(18302, 18303, 18301, &dir.join("ocaml1"), [&rust_node_ma]).unwrap();
        let ocaml_peer_id1 = ocaml_node1.peer_id().into();

        let mut peers = vec![ocaml_peer_id0, ocaml_peer_id1];
        let mut duration = Duration::from_secs(8 * 60);
        while !peers.is_empty() {
            // wait for ocaml node to connect
            let connected = driver
                .wait_for(
                    duration,
                    connection_finalized_event(|peer| peers.contains(peer)),
                )
                .await
                .unwrap()
                .expect("expected connected event");
            let (ocaml_peer, _) = as_connection_finalized_event(&connected.1).unwrap();
            peers.retain(|peer| peer != ocaml_peer);
            let ocaml_peer = ocaml_peer.clone();
            // execute it
            let state = driver.exec_step(connected).await.unwrap().unwrap();
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
            let node0_peers = ocaml_node0
                .grapql_query(PEERS_QUERY)
                .expect("peers graphql query");
            println!("{}", serde_json::to_string_pretty(&node0_peers).unwrap());
            node0_has_node1 = get_peers_iter(&node0_peers)
                .unwrap()
                .any(|peer| peer.unwrap().2 == &ocaml_node1.peer_id().to_string());

            let node1_peers = ocaml_node1
                .grapql_query(PEERS_QUERY)
                .expect("peers graphql query");
            println!("{}", serde_json::to_string_pretty(&node1_peers).unwrap());
            node1_has_node0 = get_peers_iter(&node1_peers)
                .unwrap()
                .any(|peer| peer.unwrap().2 == &ocaml_node0.peer_id().to_string());

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

        let state = driver.runner.node(rust_node_id).unwrap().state();
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

fn get_peers_iter(
    data: &serde_json::Value,
) -> Option<impl Iterator<Item = Option<(&str, i64, &str)>>> {
    let iter = data
        .as_object()?
        .get("data")?
        .get("getPeers")?
        .as_array()?
        .iter()
        .map(|elt| {
            let elt = elt.as_object()?;
            let host = elt.get("host")?.as_str()?;
            let port = elt.get("libp2pPort")?.as_i64()?;
            let peer_id = elt.get("peerId")?.as_str()?;
            Some((host, port, peer_id))
        });
    Some(iter)
}

const PEERS_QUERY: &str = r#"query {
  getPeers {
    host
    libp2pPort
    peerId
  }
}"#;

fn connection_finalized_event(
    pred: impl Fn(&PeerId) -> bool,
) -> impl Fn(ClusterNodeId, &Event, &State) -> bool {
    move |_, event, _| {
        matches!(
            event,
            Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(peer, res))) if pred(peer) && res.is_ok()
        )
    }
}

fn as_connection_finalized_event(event: &Event) -> Option<(&PeerId, &Result<(), String>)> {
    if let Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(peer, res))) = event {
        Some((peer, res))
    } else {
        None
    }
}

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
