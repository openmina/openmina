use std::{
    collections::BTreeSet,
    fmt::Debug,
    net::SocketAddr,
    time::{Duration, Instant},
};

use libp2p::Multiaddr;
use node::{
    event_source::Event,
    p2p::{
        connection::outgoing::P2pConnectionOutgoingInitOpts,
        webrtc::{Host, HttpSignalingInfo, SignalingMethod},
        P2pConnectionEvent, P2pEvent, P2pListenEvent, P2pListenerId, P2pPeerState, P2pPeerStatus,
        P2pState, PeerId,
    },
    State,
};

#[cfg(feature = "p2p-libp2p")]
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitLibp2pOpts;

#[cfg(not(feature = "p2p-libp2p"))]
use node::p2p::{
    MioEvent, P2pNetworkAuthState, P2pNetworkNoiseState, P2pNetworkNoiseStateInner,
    P2pNetworkSelectState,
};

use crate::{cluster::ClusterNodeId, node::RustNodeTestingConfig, scenario::ScenarioStep};

use super::ClusterRunner;

pub fn match_addr_with_port_and_peer_id(
    port: u16,
    peer_id: PeerId,
) -> impl Fn(&P2pConnectionOutgoingInitOpts) -> bool {
    move |conn_opt| match conn_opt {
        P2pConnectionOutgoingInitOpts::WebRTC {
            peer_id: pid,
            signaling:
                SignalingMethod::Http(HttpSignalingInfo {
                    host: Host::Ipv4(_ip4),
                    port: p,
                }),
        }
        | P2pConnectionOutgoingInitOpts::WebRTC {
            peer_id: pid,
            signaling:
                SignalingMethod::Https(HttpSignalingInfo {
                    host: Host::Ipv4(_ip4),
                    port: p,
                }),
        } => &peer_id == pid && port == *p,
        P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) => {
            libp2p_opts.peer_id == peer_id && libp2p_opts.port == port
        }
        _ => false,
    }
}

pub fn get_peers_iter(
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

pub const PEERS_QUERY: &str = r#"query {
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

#[cfg(feature = "p2p-libp2p")]
pub fn as_connection_finalized_event(event: &Event) -> Option<(&PeerId, &Result<(), String>)> {
    if let Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Finalized(peer, res))) = event {
        Some((peer, res))
    } else {
        None
    }
}

pub fn identify_event(peer_id: PeerId) -> impl Fn(ClusterNodeId, &Event, &State) -> bool {
    move |_, event, _| match event {
        #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
        Event::P2p(P2pEvent::Libp2pIdentify(peer, _)) if peer == &peer_id => true,
        _ => {
            let _ = peer_id;
            false
        }
    }
}

#[cfg(not(feature = "p2p-libp2p"))]
pub fn as_event_mio_interface_detected(event: &Event) -> Option<&std::net::IpAddr> {
    if let Event::P2p(P2pEvent::MioEvent(MioEvent::InterfaceDetected(ip_addr))) = event {
        Some(ip_addr)
    } else {
        None
    }
}

#[cfg(not(feature = "p2p-libp2p"))]
pub fn as_event_mio_data_send_receive(event: &Event) -> Option<SocketAddr> {
    match event {
        Event::P2p(P2pEvent::MioEvent(
            MioEvent::IncomingDataDidReceive(addr, _) | MioEvent::OutgoingDataDidSend(addr, _),
        )) => Some(*addr),
        _ => None,
    }
}

#[cfg(not(feature = "p2p-libp2p"))]
pub fn as_event_mio_outgoing_connection(
    event: &Event,
) -> Option<(SocketAddr, &Result<(), String>)> {
    match event {
        Event::P2p(P2pEvent::MioEvent(MioEvent::OutgoingConnectionDidConnect(addr, result))) => {
            Some((*addr, result))
        }
        _ => None,
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
            for (node_id, state, events) in self.runner.pending_events(true) {
                for (_, event) in events {
                    if f(node_id, event, state) {
                        found = Some((node_id, event.clone()));
                        break;
                    } else {
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
            'pending_events: for (node_id, state, events) in self.runner.pending_events(true) {
                for (_, event) in events {
                    found = f(node_id, event, state);
                    steps.push(ScenarioStep::Event {
                        node_id,
                        event: event.to_string(),
                    });
                    if found {
                        break 'pending_events;
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

    /// Executes all events as steps, until the predicate `f` reports true. The
    /// predicate is checked each time after executing an event step.
    pub async fn exec_steps_until(
        &mut self,
        duration: Duration,
        mut f: impl FnMut(ClusterNodeId, &Event, &State) -> bool,
    ) -> anyhow::Result<bool> {
        let timeout = std::time::Instant::now() + duration;
        while std::time::Instant::now() < timeout {
            while let Some((node_id, event)) = self.next_event() {
                let step = ScenarioStep::Event {
                    node_id,
                    event: event.to_string(),
                };
                self.runner.exec_step(step).await?;
                let state = self.runner.node(node_id).unwrap().state();
                if f(node_id, &event, state) {
                    return Ok(true);
                }
            }
            self.idle(Duration::from_millis(100)).await?;
        }
        Ok(false)
    }

    pub fn next_event(&mut self) -> Option<(ClusterNodeId, Event)> {
        self.runner
            .pending_events(true)
            .find_map(|(node_id, _, mut events)| {
                events.next().map(|(_, event)| (node_id, event.clone()))
            })
    }

    pub async fn trace_steps(&mut self) -> anyhow::Result<()> {
        loop {
            while let Some((node_id, event)) = self.next_event() {
                println!("{node_id} event: {event}");
                let step = ScenarioStep::Event {
                    node_id,
                    event: event.to_string(),
                };
                self.runner.exec_step(step).await?;
                let _state = self.runner.node(node_id).unwrap().state();
                // println!("{node_id} state: {state:#?}, state = state.p2p");
            }
            self.idle(Duration::from_millis(100)).await?;
        }
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
                by_nanos: duration.as_nanos().try_into()?,
            })
            .await?;
        let nodes = self
            .runner
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

    pub fn inner_mut(&mut self) -> &mut ClusterRunner<'cluster> {
        &mut self.runner
    }

    #[allow(dead_code)]
    pub fn into_inner(self) -> ClusterRunner<'cluster> {
        self.runner
    }
}

/// Runs the cluster until each of the `nodes` is listening on the localhost interface.
#[cfg(feature = "p2p-libp2p")]
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

/// Runs the cluster until each of the `nodes` is listening on the localhost interface.
#[cfg(not(feature = "p2p-libp2p"))]
pub async fn wait_for_nodes_listening_on_localhost<'cluster>(
    driver: &mut Driver<'cluster>,
    duration: Duration,
    nodes: impl IntoIterator<Item = ClusterNodeId>,
) -> anyhow::Result<bool> {
    let mut nodes = std::collections::BTreeSet::from_iter(nodes); // TODO: filter out nodes that already listening

    // predicate matching event "listening on localhost interface"
    let _ip4_localhost = libp2p::multiaddr::Protocol::Ip4("127.0.0.1".parse().unwrap());
    let pred = |node_id, event: &_, _state: &_| {
        if let Some(_addr) = as_event_mio_interface_detected(event) {
            nodes.remove(&node_id);
            nodes.is_empty()
        } else {
            false
        }
    };

    // wait for all peers to listen
    driver.exec_steps_until(duration, pred).await
}

pub trait PeerPredicate {
    fn matches(&mut self, node_id: ClusterNodeId, peer_id: &PeerId) -> bool;
}

impl<F> PeerPredicate for F
where
    F: FnMut(ClusterNodeId, &PeerId) -> bool,
{
    fn matches(&mut self, node_id: ClusterNodeId, peer_id: &PeerId) -> bool {
        self(node_id, peer_id)
    }
}

impl PeerPredicate for ClusterNodeId {
    fn matches(&mut self, node_id: ClusterNodeId, _peer_id: &PeerId) -> bool {
        *self == node_id
    }
}

impl PeerPredicate for (ClusterNodeId, &PeerId) {
    fn matches(&mut self, node_id: ClusterNodeId, peer_id: &PeerId) -> bool {
        self.0 == node_id && self.1 == peer_id
    }
}

impl PeerPredicate for (ClusterNodeId, &mut BTreeSet<PeerId>) {
    fn matches(&mut self, node_id: ClusterNodeId, peer_id: &PeerId) -> bool {
        self.0 == node_id && {
            self.1.remove(peer_id);
            self.1.is_empty()
        }
    }
}

/// Runst the cluster until the node is connected to the node that satisfies the predicate.
#[cfg(not(feature = "p2p-libp2p"))]
pub async fn wait_for_connection_established<'cluster, F: PeerPredicate>(
    driver: &mut Driver<'cluster>,
    duration: Duration,
    mut f: F,
) -> anyhow::Result<bool> {
    let pred = |node_id, event: &_, state: &State| {
        if let Some(addr) = as_event_mio_data_send_receive(event) {
            let Some(conn_state) = state.p2p.network.scheduler.connections.get(&addr) else {
                return false;
            };
            let Some(P2pNetworkAuthState::Noise(P2pNetworkNoiseState {
                inner:
                    Some(P2pNetworkNoiseStateInner::Done {
                        remote_peer_id: peer_id,
                        ..
                    }),
                ..
            })) = &conn_state.auth
            else {
                return false;
            };
            let P2pNetworkSelectState {
                negotiated: Some(Some(_)),
                ..
            } = conn_state.select_mux
            else {
                return false;
            };
            if f.matches(node_id, peer_id) {
                true
            } else {
                false
            }
        } else {
            false
        }
    };
    driver.exec_steps_until(duration, pred).await
}

/// Runst the cluster until the node is connected to the node that satisfies the predicate.
#[cfg(feature = "p2p-libp2p")]
pub async fn wait_for_connection_established<'cluster, F: PeerPredicate>(
    driver: &mut Driver<'cluster>,
    duration: Duration,
    mut f: F,
) -> anyhow::Result<bool> {
    let pred = |node_id, event: &_, _state: &State| {
        if let Some((peer_id, Ok(_))) = as_connection_finalized_event(event) {
            f.matches(node_id, peer_id)
        } else {
            false
        }
    };
    driver.exec_steps_until(duration, pred).await
}

// pub async fn wait_for_disconnected<P: PeerPredicate>(
//     driver: &mut Driver<'_>,
//     duration: Duration,
//     mut p: P,
// ) -> anyhow::Result<bool> {
//     driver.exec_steps_until(duration, |node_id, event, state| {
//         if let as_
//     })
// }

/// Creates `num` Rust nodes in the cluster
pub fn add_rust_nodes1<N, T>(driver: &mut Driver, num: N, config: RustNodeTestingConfig) -> T
where
    N: Into<u16>,
    T: FromIterator<(ClusterNodeId, PeerId)>,
{
    (0..num.into())
        .map(|_| driver.add_rust_node(config.clone()))
        .collect()
}

pub fn add_rust_nodes<N, NodeIds, PeerIds>(
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
        .map(|_| driver.add_rust_node(config.clone()))
        .unzip()
}

/// Creates `num` Rust nodes in the cluster
pub fn add_rust_nodes_with<N, NodeIds, Items, Item, F>(
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
        .map(|_| driver.add_rust_node_with(config.clone(), &mut f))
        .unzip()
}

/// Runs cluster until there is a `quiet_dur` period of no events, returning
/// `Ok(true)` in this case. If there is no such period for `timeout` period of
/// time, then returns `Ok(false)`
pub async fn run_until_no_events<'cluster>(
    driver: &mut Driver<'cluster>,
    quiet_dur: Duration,
    timeout: Duration,
) -> anyhow::Result<bool> {
    let timeout = Instant::now() + timeout;
    while driver.run_until(quiet_dur, |_, _, _| true).await? {
        if Instant::now() >= timeout {
            return Ok(false);
        }
    }
    Ok(true)
}

pub trait ConnectionPredicate {
    fn matches(
        &mut self,
        node_id: ClusterNodeId,
        peer_addr: SocketAddr,
        result: &Result<(), String>,
    ) -> bool;
}

impl<F> ConnectionPredicate for F
where
    F: FnMut(ClusterNodeId, SocketAddr, &Result<(), String>) -> bool,
{
    fn matches(
        &mut self,
        node_id: ClusterNodeId,
        peer_addr: SocketAddr,
        result: &Result<(), String>,
    ) -> bool {
        self(node_id, peer_addr, result)
    }
}

#[cfg(feature = "p2p-libp2p")]
pub async fn wait_for_connection_event<'cluster, F>(
    driver: &mut Driver<'cluster>,
    duration: Duration,
    mut f: F,
) -> anyhow::Result<bool>
where
    F: ConnectionPredicate,
{
    driver
        .run_until(duration, |node_id, event: &_, state: &_| {
            let Some((peer_id, result)) = as_connection_finalized_event(event) else {
                return false;
            };
            let Some(P2pPeerState {
                dial_opts:
                    Some(P2pConnectionOutgoingInitOpts::LibP2P(P2pConnectionOutgoingInitLibp2pOpts {
                        host,
                        port,
                        ..
                    })),
                ..
            }) = state.p2p.peers.get(peer_id)
            else {
                return false;
            };

            let addr = SocketAddr::new(
                match host {
                    Host::Ipv4(ip4) => (*ip4).into(),
                    Host::Ipv6(ip6) => (*ip6).into(),
                    Host::Domain(_) => unreachable!(),
                },
                *port,
            );
            f.matches(node_id, addr, result)
        })
        .await
}

#[cfg(not(feature = "p2p-libp2p"))]
pub async fn wait_for_connection_event<'cluster, F>(
    driver: &mut Driver<'cluster>,
    duration: Duration,
    mut f: F,
) -> anyhow::Result<bool>
where
    F: ConnectionPredicate,
{
    Ok(driver
        .exec_steps_until(duration, |node_id, event: &_, _: &_| {
            if let Some((addr, result)) = as_event_mio_outgoing_connection(event) {
                f.matches(node_id, addr, result)
            } else {
                false
            }
        })
        .await?)
}

pub fn get_peer_state<'a>(
    cluster: &'a ClusterRunner<'_>,
    node_id: ClusterNodeId,
    peer_id: &PeerId,
) -> Option<&'a P2pPeerState> {
    let store = cluster.node(node_id).expect("node does not exist");
    store.state().p2p.peers.get(peer_id)
}

pub fn peer_exists(cluster: &ClusterRunner<'_>, node_id: ClusterNodeId, peer_id: &PeerId) -> bool {
    get_peer_state(cluster, node_id, peer_id).is_some()
}

pub fn peer_is_ready(
    cluster: &ClusterRunner<'_>,
    node_id: ClusterNodeId,
    peer_id: &PeerId,
) -> bool {
    matches!(
        get_peer_state(cluster, node_id, peer_id),
        Some(P2pPeerState {
            status: P2pPeerStatus::Ready(_),
            ..
        })
    )
}

pub fn get_p2p_state<'a>(cluster: &'a ClusterRunner<'a>, node_id: ClusterNodeId) -> &P2pState {
    &cluster
        .node(node_id)
        .expect("node should exist")
        .state()
        .p2p
}
pub fn get_peers<'a>(
    cluster: &'a ClusterRunner<'a>,
    node_id: ClusterNodeId,
) -> impl Iterator<Item = (&'a PeerId, &'a P2pPeerState)> {
    cluster
        .node(node_id)
        .expect("node should exist")
        .state()
        .p2p
        .peers
        .iter()
}

pub async fn connect_rust_nodes(
    cluster: &mut ClusterRunner<'_>,
    dialer: ClusterNodeId,
    listener: ClusterNodeId,
) {
    cluster
        .exec_step(crate::scenario::ScenarioStep::ConnectNodes {
            dialer,
            listener: crate::scenario::ListenerNode::Rust(listener),
        })
        .await
        .expect("connect event should be dispatched");
}

pub async fn trace_steps(runner: &mut ClusterRunner<'_>) -> anyhow::Result<()> {
    loop {
        while let Some((node_id, event)) = next_event(runner) {
            println!("{node_id} event: {event}");
            let step = ScenarioStep::Event {
                node_id,
                event: event.to_string(),
            };
            runner.exec_step(step).await?;
        }
        idle(runner, Duration::from_millis(100)).await?;
    }
}

pub async fn trace_steps_state<T: Debug, F: Fn(&State) -> T>(
    runner: &mut ClusterRunner<'_>,
    f: F,
) -> anyhow::Result<()> {
    loop {
        while let Some((node_id, event)) = next_event(runner) {
            println!("{node_id} event: {event}");
            let step = ScenarioStep::Event {
                node_id,
                event: event.to_string(),
            };
            runner.exec_step(step).await?;
            let state = runner.node(node_id).unwrap().state();
            let t = f(state);
            println!("{node_id} state: {t:#?}");
        }
        idle(runner, Duration::from_millis(100)).await?;
    }
}

pub async fn idle(runner: &mut ClusterRunner<'_>, duration: Duration) -> anyhow::Result<()> {
    tokio::time::sleep(duration).await;
    runner
        .exec_step(ScenarioStep::AdvanceTime {
            by_nanos: duration.as_nanos().try_into()?,
        })
        .await?;
    let nodes = runner
        .nodes_iter()
        .map(|(node_id, _)| node_id)
        .collect::<Vec<_>>();
    for node_id in nodes {
        runner
            .exec_step(ScenarioStep::CheckTimeouts { node_id })
            .await?;
    }
    Ok(())
}
pub fn next_event(runner: &mut ClusterRunner<'_>) -> Option<(ClusterNodeId, Event)> {
    runner
        .pending_events(true)
        .find_map(|(node_id, _, mut events)| {
            events.next().map(|(_, event)| (node_id, event.clone()))
        })
}
