use std::time::Duration;

use libp2p::Multiaddr;
use node::{
    event_source::Event,
    p2p::{
        connection::outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionEvent, P2pEvent,
        P2pListenEvent, P2pListenerId, PeerId,
    },
    State,
};

use crate::{cluster::ClusterNodeId, node::RustNodeTestingConfig, scenario::ScenarioStep};

use super::ClusterRunner;

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

pub fn connection_finalized_event(
    pred: impl Fn(ClusterNodeId, &PeerId) -> bool,
) -> impl Fn(ClusterNodeId, &Event, &State) -> bool {
    move |node_id, event, _| {
        matches!(
            event,
            Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(peer, res))) if pred(node_id, peer) && res.is_ok()
        )
    }
}

pub fn connection_finalized_with_res_event(
    pred: impl Fn(ClusterNodeId, &PeerId, &Result<(), String>) -> bool,
) -> impl Fn(ClusterNodeId, &Event, &State) -> bool {
    move |node_id, event, _| {
        matches!(
            event,
            Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(peer, res))) if pred(node_id, peer, res)
        )
    }
}

pub fn as_listen_new_addr_event(event: &Event) -> Option<(&Multiaddr, &P2pListenerId)> {
    if let Event::P2p(P2pEvent::Listen(P2pListenEvent::NewListenAddr { listener_id, addr })) = event
    {
        Some((addr, listener_id))
    } else {
        None
    }
}

pub fn as_connection_finalized_event(event: &Event) -> Option<(&PeerId, &Result<(), String>)> {
    if let Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(peer, res))) = event {
        Some((peer, res))
    } else {
        None
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

pub struct Driver<'cluster> {
    runner: ClusterRunner<'cluster>,
    emulated_time: bool,
}

impl<'cluster> Driver<'cluster> {
    pub fn new(runner: ClusterRunner<'cluster>) -> Self {
        Driver {
            runner,
            emulated_time: false,
        }
    }

    pub fn with_emulated_time(runner: ClusterRunner<'cluster>) -> Self {
        Driver {
            runner,
            emulated_time: true,
        }
    }

    async fn sleep(&self, duration: Duration) {
        if !self.emulated_time {
            tokio::time::sleep(duration).await;
        }
    }

    pub async fn wait_for(
        &mut self,
        duration: Duration,
        mut f: impl FnMut(ClusterNodeId, &Event, &State) -> bool,
    ) -> anyhow::Result<Option<(ClusterNodeId, Event)>> {
        let timeout = std::time::Instant::now() + duration;
        while std::time::Instant::now() < timeout {
            let mut steps = Vec::new();
            let mut found = None;
            for (node_id, state, events) in self.runner.pending_events() {
                for (_, event) in events {
                    if f(node_id, event, state) {
                        eprintln!("!!! {node_id}: {event:?}");
                        found = Some((node_id, event.clone()));
                        break;
                    } else {
                        eprintln!(">>> {node_id}: {event:?}");
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
            self.idle(Duration::from_millis(100)).await?;
        }
        Ok(None)
    }

    pub async fn run_until(
        &mut self,
        duration: Duration,
        mut f: impl FnMut(ClusterNodeId, &Event, &State) -> bool,
    ) -> anyhow::Result<bool> {
        let timeout = std::time::Instant::now() + duration;
        while std::time::Instant::now() < timeout {
            let mut steps = Vec::new();
            let mut found = false;
            'pending_events: for (node_id, state, events) in self.runner.pending_events() {
                for (_, event) in events {
                    found = f(node_id, event, state);
                    steps.push(ScenarioStep::Event {
                        node_id,
                        event: event.to_string(),
                    });
                    if found {
                        eprintln!("!!! {node_id}: {event:?}");
                        break 'pending_events;
                    } else {
                        eprintln!(">>> {node_id}: {event:?}");
                    }
                }
            }
            for step in steps {
                self.runner.exec_step(step).await?;
            }
            if found {
                return Ok(true);
            }
            self.idle(Duration::from_millis(100)).await?;
        }
        Ok(false)
    }

    pub async fn run(&mut self, duration: Duration) -> anyhow::Result<()> {
        let finish = std::time::Instant::now() + duration;
        while std::time::Instant::now() < finish {
            self.idle(Duration::from_millis(100)).await?;
        }
        Ok(())
    }

    pub async fn idle(&mut self, duration: Duration) -> anyhow::Result<()> {
        self.sleep(duration).await;
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

    pub async fn exec_step(&mut self, step: ScenarioStep) -> anyhow::Result<bool> {
        self.runner.exec_step(step).await
    }

    pub async fn exec_even_step(
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

    pub fn add_rust_node(
        &mut self,
        testing_config: RustNodeTestingConfig,
    ) -> (ClusterNodeId, PeerId) {
        let node_id = self.runner.add_rust_node(testing_config);
        let peer_id = self.runner.node(node_id).unwrap().peer_id();
        (node_id, peer_id)
    }

    pub fn add_rust_node_with<Item, F>(
        &mut self,
        testing_config: RustNodeTestingConfig,
        mut f: F,
    ) -> (ClusterNodeId, Item)
    where
        F: FnMut(&State) -> Item,
    {
        let node_id = self.runner.add_rust_node(testing_config);
        let state = self.runner.node(node_id).unwrap().state();
        let item = f(state);
        (node_id, item)
    }

    pub fn inner(&self) -> &ClusterRunner {
        &self.runner
    }

    #[allow(dead_code)]
    pub fn into_inner(self) -> ClusterRunner<'cluster> {
        self.runner
    }
}

/// Runs the cluster until each of the `nodes` is listening on the localhost interface.
pub async fn wait_for_nodes_listening_on_localhost<'cluster>(
    driver: &mut Driver<'cluster>,
    duration: Duration,
    nodes: impl IntoIterator<Item = ClusterNodeId>,
) -> anyhow::Result<bool> {
    let mut nodes = std::collections::BTreeSet::from_iter(nodes); // TODO: filter out nodes that already listening

    // predicate matching event "listening on localhost interface"
    let ip4_localhost = libp2p::multiaddr::Protocol::Ip4("127.0.0.1".parse().unwrap());
    let pred = |node_id, event: &_, _state: &_| {
        if let Some((addr, _)) = as_listen_new_addr_event(event) {
            if Some(&ip4_localhost) == addr.iter().next().as_ref() {
                nodes.remove(&node_id);
            }
            nodes.is_empty()
        } else {
            false
        }
    };

    // wait for all peers to listen
    driver.run_until(duration, pred).await
}

/// Creates `num` Rust nodes in the cluster
pub fn add_rust_nodes<'cluster, N, NodeIds, PeerIds>(
    driver: &mut Driver,
    num: N,
    config: RustNodeTestingConfig,
) -> (NodeIds, PeerIds)
where
    N: Into<u16>,
    NodeIds: Default + Extend<ClusterNodeId>,
    PeerIds: Default + Extend<PeerId>,
{
    (0..num.into())
        .into_iter()
        .map(|_| driver.add_rust_node(config.clone()))
        .unzip()
}

/// Creates `num` Rust nodes in the cluster
pub fn add_rust_nodes_with<'cluster, N, NodeIds, Items, Item, F>(
    driver: &mut Driver,
    num: N,
    config: RustNodeTestingConfig,
    mut f: F,
) -> (NodeIds, Items)
where
    N: Into<u16>,
    NodeIds: Default + Extend<ClusterNodeId>,
    Items: Default + Extend<Item>,
    F: FnMut(&State) -> Item,
{
    (0..num.into())
        .into_iter()
        .map(|_| driver.add_rust_node_with(config.clone(), &mut f))
        .unzip()
}
