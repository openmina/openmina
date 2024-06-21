use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Range,
    task::{ready, Context, Poll},
    time::{Duration, Instant},
};

use futures::StreamExt;
use libp2p::{multiaddr::multiaddr, swarm::DialError, Multiaddr};
use openmina_core::{ChainId, Substate, DEVNET_CHAIN_ID};
use p2p::{
    connection::outgoing::{
        P2pConnectionOutgoingAction, P2pConnectionOutgoingInitLibp2pOpts,
        P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingInitOptsParseError,
    },
    identity::SecretKey,
    p2p_effects, p2p_timeout_effects, P2pConfig, P2pState, PeerId,
};
use redux::SystemTime;
use tokio::sync::mpsc;

use crate::{
    event::{event_mapper_effect, RustNodeEvent},
    libp2p_node::{create_swarm, Libp2pEvent, Libp2pNode, Libp2pNodeConfig, Libp2pNodeId},
    redux::State,
    redux::{log_action, Action},
    rust_node::{RustNode, RustNodeConfig, RustNodeId},
    service::ClusterService,
    stream::{ClusterStreamExt, MapErrors, TakeDuring},
    test_node::TestNode,
};

#[derive(Debug, Default, Clone)]
pub enum PeerIdConfig {
    #[default]
    Derived,
    Bytes([u8; 32]),
}

#[derive(Debug, Clone, derive_more::From)]
pub enum Listener {
    Rust(RustNodeId),
    Libp2p(Libp2pNodeId),
    Multiaddr(Multiaddr),
    SocketPeerId(SocketAddr, PeerId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From)]
pub enum NodeId {
    Rust(RustNodeId),
    Libp2p(Libp2pNodeId),
}

pub struct Cluster {
    chain_id: ChainId,
    ports: Range<u16>,
    ip: IpAddr,

    rust_nodes: Vec<RustNode>,
    libp2p_nodes: Vec<Libp2pNode>,
    last_idle_instant: Instant,
    idle_interval: tokio::time::Interval,
    next_poll: NextPoll,
    timeouts: BTreeMap<usize, Duration>,

    is_error: fn(&ClusterEvent) -> bool,
    total_duration: Duration,
}

enum PortsConfig {
    Range(Range<u16>),
    Len(u16),
    ExactLen(u16),
}

impl PortsConfig {
    async fn ports(self) -> Result<Range<u16>> {
        match self {
            PortsConfig::Range(range) => Ok(range),
            PortsConfig::Len(len) => PORTS.take(len).await,
            PortsConfig::ExactLen(len) => PORTS.take_exact(len).await,
        }
    }
}

pub struct ClusterBuilder {
    chain_id: ChainId,
    ports: Option<PortsConfig>,
    ip: IpAddr,
    idle_duration: Duration,
    is_error: fn(&ClusterEvent) -> bool,
    total_duration: Duration,
}

impl Default for ClusterBuilder {
    fn default() -> Self {
        ClusterBuilder {
            chain_id: DEVNET_CHAIN_ID,
            ports: None,
            ip: Ipv4Addr::LOCALHOST.into(),
            idle_duration: Duration::from_millis(100),
            is_error: super::event::is_error,
            total_duration: Duration::from_secs(60),
        }
    }
}

impl ClusterBuilder {
    pub fn new() -> Self {
        ClusterBuilder::default()
    }

    pub fn chain_id(mut self, chain_id: ChainId) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn ports(mut self, ports: Range<u16>) -> Self {
        self.ports = Some(PortsConfig::Range(ports));
        self
    }

    pub fn ports_with_len(mut self, len: u16) -> Self {
        self.ports = Some(PortsConfig::Len(len));
        self
    }

    pub fn ports_with_exact_len(mut self, len: u16) -> Self {
        self.ports = Some(PortsConfig::ExactLen(len));
        self
    }

    pub fn ip(mut self, ip: IpAddr) -> Self {
        self.ip = ip;
        self
    }

    pub fn idle_duration(mut self, duration: Duration) -> Self {
        self.idle_duration = duration;
        self
    }

    pub fn is_error(mut self, f: fn(&ClusterEvent) -> bool) -> Self {
        self.is_error = f;
        self
    }

    pub fn total_duration(mut self, duration: Duration) -> Self {
        self.total_duration = duration;
        self
    }

    pub async fn start(self) -> Result<Cluster> {
        *crate::log::LOG;

        let chain_id = self.chain_id;
        let ports = self
            .ports
            .ok_or_else(|| Error::UninitializedField("ports"))?
            .ports()
            .await?;
        let ip = self.ip;
        let mut idle_interval = tokio::time::interval(self.idle_duration);
        let last_idle_instant = idle_interval.tick().await.into_std();
        let is_error = self.is_error;
        let total_duration = self.total_duration;
        openmina_core::info!(openmina_core::log::system_time(); "starting the cluster");
        let next_poll = Default::default();
        Ok(Cluster {
            chain_id,
            ports,
            ip,
            is_error,
            total_duration,
            rust_nodes: Default::default(),
            libp2p_nodes: Default::default(),
            last_idle_instant,
            idle_interval,
            next_poll,
            timeouts: Default::default(),
        })
    }
}

pub struct Ports {
    start: tokio::sync::Mutex<u16>,
    end: u16,
}

impl Ports {
    pub fn new(range: Range<u16>) -> Self {
        Ports {
            start: range.start.into(),
            end: range.end,
        }
    }

    fn round(u: u16) -> u16 {
        ((u + 99) / 100) * 100
    }

    pub async fn take(&self, len: u16) -> Result<Range<u16>> {
        let mut start = self.start.lock().await;
        let res = Self::round(*start)..Self::round(*start + len);
        if res.end > self.end {
            return Err(Error::NoMorePorts);
        }
        *start = res.end;
        Ok(res)
    }

    pub async fn take_exact(&self, len: u16) -> Result<Range<u16>> {
        let mut start = self.start.lock().await;
        let res = *start..(*start + len);
        if res.end > self.end {
            return Err(Error::NoMorePorts);
        }
        *start += len;
        Ok(res)
    }
}

impl Default for Ports {
    fn default() -> Self {
        Self {
            start: 10000.into(),
            end: 20000,
        }
    }
}

/// Declares a shared storage for ports.
///
/// ```
/// ports_store!(PORTS);
///
/// #[tokio::test]
/// fn test1() {
///     let cluster = ClusterBuilder::default()
///         .ports(PORTS.take(20).await.expect("enough ports"))
///         .start()
///         .await;
/// }
///
/// ```
#[macro_export]
macro_rules! ports_store {
    ($name:ident, $range:expr) => {
        $crate::lazy_static::lazy_static! {
            static ref PORTS: $crate::cluster::Ports = $crate::cluster::Ports::new($range);
        }
    };
    ($name:ident) => {
        $crate::lazy_static::lazy_static! {
            static ref PORTS: $crate::cluster::Ports = $crate::cluster::Ports::default();
        }
    };
}

ports_store!(PORTS);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No more ports")]
    NoMorePorts,
    #[error(transparent)]
    AddrParse(#[from] P2pConnectionOutgoingInitOptsParseError),
    #[error("uninitialized field `{0}`")]
    UninitializedField(&'static str),
    #[error("swarm creation error: {0}")]
    Libp2pSwarm(String),
    #[error(transparent)]
    Libp2pDial(#[from] DialError),
    #[error("Error occurred: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

const RUST_NODE_SIG_BYTE: u8 = 0xf0;
#[allow(dead_code)]
const LIBP2P_NODE_SIG_BYTE: u8 = 0xf1;

impl Cluster {
    pub fn next_port(&mut self) -> Result<u16> {
        self.ports.next().ok_or(Error::NoMorePorts)
    }

    fn init_opts(&self, listener: Listener) -> Result<P2pConnectionOutgoingInitOpts> {
        match listener {
            Listener::Rust(RustNodeId(i)) => {
                let node = &self.rust_nodes[i];
                let port = node.libp2p_port();
                let peer_id = node.peer_id();
                let host = self.ip.into();
                Ok(P2pConnectionOutgoingInitOpts::LibP2P(
                    P2pConnectionOutgoingInitLibp2pOpts {
                        peer_id,
                        host,
                        port,
                    },
                ))
            }
            Listener::Libp2p(Libp2pNodeId(i)) => {
                let node = &self.libp2p_nodes[i];
                let port = node.libp2p_port();
                let peer_id = node.peer_id();
                let host = self.ip.into();
                Ok(P2pConnectionOutgoingInitOpts::LibP2P(
                    P2pConnectionOutgoingInitLibp2pOpts {
                        peer_id,
                        host,
                        port,
                    },
                ))
            }
            Listener::Multiaddr(ref maddr) => {
                Ok(P2pConnectionOutgoingInitOpts::LibP2P(maddr.try_into()?))
            }
            Listener::SocketPeerId(socket_addr, peer_id) => Ok(
                P2pConnectionOutgoingInitOpts::LibP2P((peer_id, socket_addr).into()),
            ),
        }
    }

    fn secret_key(config: PeerIdConfig, index: usize, fill_byte: u8) -> SecretKey {
        let bytes = match config {
            PeerIdConfig::Derived => {
                let mut bytes = [fill_byte; 32];
                let bytes_len = bytes.len();
                let i_bytes = index.to_be_bytes();
                let i = bytes_len - i_bytes.len();
                bytes[i..bytes_len].copy_from_slice(&i_bytes);
                bytes
            }
            PeerIdConfig::Bytes(bytes) => bytes,
        };
        SecretKey::from_bytes(bytes)
    }

    fn rust_node_config(&mut self, config: RustNodeConfig) -> Result<(P2pConfig, SecretKey)> {
        let secret_key =
            Self::secret_key(config.peer_id, self.rust_nodes.len(), RUST_NODE_SIG_BYTE);
        let libp2p_port = self.next_port()?;
        let listen_port = self.next_port()?;
        let initial_peers = config
            .initial_peers
            .into_iter()
            .map(|p| self.init_opts(p))
            .collect::<Result<_>>()?;
        let config = P2pConfig {
            libp2p_port: Some(libp2p_port),
            listen_port,
            identity_pub_key: secret_key.public_key(),
            initial_peers,
            ask_initial_peers_interval: Duration::from_secs(5),
            enabled_channels: p2p::channels::ChannelId::for_libp2p().collect(),
            peer_discovery: config.discovery,
            timeouts: config.timeouts,
            limits: config.limits,
            initial_time: Duration::ZERO,
        };

        Ok((config, secret_key))
    }

    pub fn add_rust_node(&mut self, config: RustNodeConfig) -> Result<RustNodeId> {
        let node_idx = self.rust_nodes.len();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (config, secret_key) = self.rust_node_config(config)?;
        let (cmd_sender, _cmd_receiver) = mpsc::unbounded_channel();

        let service = ClusterService::new(
            node_idx,
            secret_key,
            event_sender,
            cmd_sender,
            self.last_idle_instant,
        );

        let store = crate::redux::Store::new(
            |state, action, dispatcher| {
                log_action(action.action(), action.meta(), state.0.my_id());
                if let Action::P2p(p2p_action) = action.action() {
                    P2pState::reducer(
                        Substate::new(state, dispatcher),
                        action.meta().clone().with_action(p2p_action),
                    )
                }
            },
            |store, action| {
                let (action, meta) = action.split();
                match action {
                    Action::P2p(a) => {
                        p2p_effects(store, meta.with_action(a.clone()));
                        event_mapper_effect(store, a);
                    }
                    Action::Idle(_) => p2p_timeout_effects(store, &meta),
                }
            },
            service,
            SystemTime::now(),
            State(P2pState::new(config, &self.chain_id)),
        );

        let node_id = RustNodeId(self.rust_nodes.len());
        self.rust_nodes.push(RustNode::new(store, event_receiver));
        Ok(node_id)
    }

    pub fn add_libp2p_node(&mut self, config: Libp2pNodeConfig) -> Result<Libp2pNodeId> {
        let node_id = Libp2pNodeId(self.libp2p_nodes.len());
        let secret_key = Self::secret_key(config.peer_id, node_id.0, LIBP2P_NODE_SIG_BYTE);
        let libp2p_port = self.next_port()?;

        let swarm = create_swarm(secret_key, libp2p_port, config.port_reuse, &self.chain_id)
            .map_err(|err| Error::Libp2pSwarm(err.to_string()))?;
        self.libp2p_nodes.push(Libp2pNode::new(swarm));

        Ok(node_id)
    }

    fn rust_dial_opts(&self, listener: Listener) -> Result<P2pConnectionOutgoingInitOpts> {
        match listener {
            Listener::Rust(id) => Ok(self.rust_node(id).rust_dial_opts(self.ip)),
            Listener::Libp2p(id) => Ok(self.libp2p_node(id).rust_dial_opts(self.ip)),
            Listener::Multiaddr(maddr) => Ok(maddr.try_into().map_err(Error::AddrParse)?),
            Listener::SocketPeerId(socket, peer_id) => Ok(P2pConnectionOutgoingInitOpts::LibP2P(
                (peer_id, socket).into(),
            )),
        }
    }

    fn libp2p_dial_opts(&self, listener: Listener) -> Result<Multiaddr> {
        match listener {
            Listener::Rust(id) => Ok(self.rust_node(id).libp2p_dial_opts(self.ip)),
            Listener::Libp2p(id) => Ok(self.libp2p_node(id).libp2p_dial_opts(self.ip)),
            Listener::Multiaddr(maddr) => Ok(maddr),
            Listener::SocketPeerId(socket, peer_id) => match socket {
                SocketAddr::V4(ipv4) => {
                    Ok(multiaddr!(Ip4(*ipv4.ip()), Tcp(ipv4.port()), P2p(peer_id)))
                }
                SocketAddr::V6(ipv6) => {
                    Ok(multiaddr!(Ip6(*ipv6.ip()), Tcp(ipv6.port()), P2p(peer_id)))
                }
            },
        }
    }

    pub fn connect<T, U>(&mut self, id: T, other: U) -> Result<()>
    where
        T: Into<NodeId>,
        U: Into<Listener>,
    {
        match id.into() {
            NodeId::Rust(id) => {
                let dial_opts = self.rust_dial_opts(other.into())?;
                self.rust_node_mut(id)
                    .dispatch_action(P2pConnectionOutgoingAction::Init {
                        opts: dial_opts,
                        rpc_id: None,
                    });
            }
            NodeId::Libp2p(id) => {
                let dial_opts = self.libp2p_dial_opts(other.into())?;
                self.libp2p_node_mut(id).swarm_mut().dial(dial_opts)?;
            }
        }
        Ok(())
    }

    pub fn rust_node(&self, id: RustNodeId) -> &RustNode {
        &self.rust_nodes[id.0]
    }

    pub fn rust_node_mut(&mut self, id: RustNodeId) -> &mut RustNode {
        &mut self.rust_nodes[id.0]
    }

    pub fn libp2p_node(&self, id: Libp2pNodeId) -> &Libp2pNode {
        &self.libp2p_nodes[id.0]
    }

    pub fn libp2p_node_mut(&mut self, id: Libp2pNodeId) -> &mut Libp2pNode {
        &mut self.libp2p_nodes[id.0]
    }

    pub fn timestamp(&self) -> Instant {
        self.last_idle_instant
    }

    pub fn peer_id<T>(&self, id: T) -> PeerId
    where
        T: Into<NodeId>,
    {
        match id.into() {
            NodeId::Rust(id) => self.rust_node(id).peer_id(),
            NodeId::Libp2p(id) => self.libp2p_node(id).peer_id(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error event: {:?}", self)]
pub enum ClusterEvent {
    Rust {
        id: RustNodeId,
        event: RustNodeEvent,
    },
    Libp2p {
        id: Libp2pNodeId,
        event: Libp2pEvent,
    },
    Idle {
        // TODO(akoptelov): individual events on timeout?
        instant: Instant,
    },
}

impl ClusterEvent {
    pub fn rust(&self) -> Option<(&RustNodeId, &RustNodeEvent)> {
        if let ClusterEvent::Rust { id, event } = self {
            Some((id, event))
        } else {
            None
        }
    }

    pub fn idle(&self) -> Option<&Instant> {
        if let ClusterEvent::Idle { instant } = self {
            Some(instant)
        } else {
            None
        }
    }
}

pub trait TimestampEvent {
    fn timestamp(&self) -> Option<Instant>;
}

impl TimestampEvent for ClusterEvent {
    fn timestamp(&self) -> Option<Instant> {
        if let ClusterEvent::Idle { instant } = self {
            Some(*instant)
        } else {
            None
        }
    }
}

impl<T> TimestampEvent for std::result::Result<ClusterEvent, T> {
    fn timestamp(&self) -> Option<Instant> {
        self.as_ref().ok().and_then(|event| event.timestamp())
    }
}

pub trait TimestampSource {
    fn timestamp(&self) -> Instant;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum NextPoll {
    #[default]
    Idle,
    Rust(RustNodeId),
    Libp2p(Libp2pNodeId),
}

impl Cluster {
    fn next_poll(&mut self) {
        self.next_poll = match self.next_poll {
            NextPoll::Rust(RustNodeId(id)) if id + 1 < self.rust_nodes.len() => {
                NextPoll::Rust(RustNodeId(id + 1))
            }
            NextPoll::Rust(_) if !self.libp2p_nodes.is_empty() => NextPoll::Libp2p(Libp2pNodeId(0)),
            NextPoll::Rust(_) => NextPoll::Idle,
            NextPoll::Libp2p(Libp2pNodeId(id)) if id + 1 < self.libp2p_nodes.len() => {
                NextPoll::Libp2p(Libp2pNodeId(id + 1))
            }
            NextPoll::Libp2p(_) => NextPoll::Idle,
            NextPoll::Idle if !self.rust_nodes.is_empty() => NextPoll::Rust(RustNodeId(0)),
            NextPoll::Idle if !self.libp2p_nodes.is_empty() => NextPoll::Libp2p(Libp2pNodeId(0)),
            NextPoll::Idle => NextPoll::Idle,
        }
    }

    fn poll_idle(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        Poll::Ready({
            let instant = ready!(self.idle_interval.poll_tick(cx)).into_std();
            let dur = instant - self.last_idle_instant;
            for i in 0..self.rust_nodes.len() {
                self.timeouts
                    .entry(i)
                    .and_modify(|d| *d += dur)
                    .or_insert(dur);
            }
            self.last_idle_instant = instant;
            instant
        })
    }

    fn poll_rust_node(
        &mut self,
        id: RustNodeId,
        cx: &mut Context<'_>,
    ) -> Poll<Option<RustNodeEvent>> {
        let rust_node = &mut self.rust_nodes[id.0];
        let res = if let Some(dur) = self.timeouts.remove(&id.0) {
            // trigger timeout action and return idle event
            Poll::Ready(Some(rust_node.idle(dur)))
        } else {
            // poll next available event from the node
            rust_node.poll_next_unpin(cx)
        };
        if crate::log::ERROR.swap(false, std::sync::atomic::Ordering::Relaxed) {
            if let Err(err) = self.dump_state() {
                eprintln!("error dumping state: {err}");
            }
            panic!("error detected");
        }
        res
    }

    fn poll_libp2p_node(
        &mut self,
        id: Libp2pNodeId,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Libp2pEvent>> {
        let libp2p_node = &mut self.libp2p_nodes[id.0];
        Poll::Ready(ready!(libp2p_node.swarm_mut().poll_next_unpin(cx)))
    }

    fn dump_state(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join("p2p-test-node.json");
        eprintln!("saving state of rust nodes to {:?}", path);
        let file = std::fs::File::create(path)?;
        let states = serde_json::Map::from_iter(
            self.rust_nodes
                .iter()
                .map(|node| {
                    Ok((
                        node.peer_id().to_string(),
                        serde_json::to_value(node.state())?,
                    ))
                })
                .collect::<std::result::Result<Vec<_>, serde_json::Error>>()?,
        );
        serde_json::to_writer(file, &states)?;
        Ok(())
    }
}

impl ::futures::stream::Stream for Cluster {
    type Item = ClusterEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let np = this.next_poll;
        loop {
            let poll = match this.next_poll {
                NextPoll::Rust(id) => this
                    .poll_rust_node(id, cx)
                    .map(|event| event.map(|event| ClusterEvent::Rust { id, event })),
                NextPoll::Libp2p(id) => this
                    .poll_libp2p_node(id, cx)
                    .map(|event| event.map(|event| ClusterEvent::Libp2p { id, event })),
                NextPoll::Idle => this
                    .poll_idle(cx)
                    .map(|instant| Some(ClusterEvent::Idle { instant })),
            };
            if poll.is_ready() {
                return poll;
            }
            this.next_poll();
            if this.next_poll == np {
                return Poll::Pending;
            }
        }
    }
}

impl TimestampSource for Cluster {
    fn timestamp(&self) -> Instant {
        self.last_idle_instant
    }
}

impl TimestampSource for &mut Cluster {
    fn timestamp(&self) -> Instant {
        self.last_idle_instant
    }
}

impl Cluster {
    pub fn stream(&mut self) -> TakeDuring<&mut Cluster> {
        let duration = self.total_duration;
        self.take_during(duration)
    }

    pub fn try_stream(&mut self) -> MapErrors<TakeDuring<&mut Cluster>, ClusterEvent> {
        let is_error = self.is_error;
        let duration = self.total_duration;
        self.take_during(duration).map_errors(is_error)
    }
}
