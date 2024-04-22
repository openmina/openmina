use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Range,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use derive_builder::UninitializedFieldError;
use futures::StreamExt;
use libp2p::Multiaddr;
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
    libp2p_node::{Libp2pEvent, Libp2pNode, Libp2pNodeId},
    redux::State,
    redux::{log_action, Action},
    rust_node::{RustNode, RustNodeConfig, RustNodeId},
    service::ClusterService,
    stream::{ClusterStreamExt, MapErrors, TakeDuring},
    test_node::TestNode,
};

#[derive(Debug, Default)]
pub enum PeerIdConfig {
    #[default]
    Derived,
    Bytes([u8; 32]),
}

#[derive(Debug, derive_more::From)]
pub enum Listener {
    Rust(RustNodeId),
    Libp2p(Libp2pNodeId),
    Multiaddr(Multiaddr),
    SocketPeerId(SocketAddr, PeerId),
}

pub struct Cluster {
    chain_id: String,
    ports: Range<u16>,
    ip: IpAddr,

    rust_nodes: Vec<RustNode>,
    libp2p_nodes: Vec<Libp2pNode>,
    last_idle_instant: Instant,
    idle_interval: tokio::time::Interval,
    next_poll: NextPoll,

    is_error: fn(&ClusterEvent) -> bool,
    total_duration: Duration,
}

pub struct ClusterBuilder {
    chain_id: String,
    ports: Option<Range<u16>>,
    ip: IpAddr,
    idle_duration: Duration,
    is_error: fn(&ClusterEvent) -> bool,
    total_duration: Duration,
}

impl Default for ClusterBuilder {
    fn default() -> Self {
        ClusterBuilder {
            chain_id: openmina_core::CHAIN_ID.to_string(),
            ports: None,
            ip: Ipv4Addr::LOCALHOST.into(),
            idle_duration: Duration::from_millis(100),
            is_error: |_| false,
            total_duration: Duration::from_secs(60),
        }
    }
}

impl ClusterBuilder {
    pub fn new() -> Self {
        ClusterBuilder::default()
    }

    pub fn chain_id(mut self, chain_id: String) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn ports(mut self, ports: Range<u16>) -> Self {
        self.ports = Some(ports);
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
            .ok_or_else(|| UninitializedFieldError::new("ports"))?;
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
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No more ports")]
    NoMorePorts,
    #[error(transparent)]
    AddrParse(#[from] P2pConnectionOutgoingInitOptsParseError),
    #[error(transparent)]
    UninitializedField(#[from] UninitializedFieldError),
    #[error("Error occurred: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

const RUST_NODE_SIG_BYTE: u8 = 0xf0;
#[allow(dead_code)]
const LIBP2P_NODE_SIG_BYTE: u8 = 0xf1;

impl Cluster {
    fn next_port(&mut self) -> Result<u16> {
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

    fn rust_node_config(&mut self, config: RustNodeConfig) -> Result<(P2pConfig, SecretKey)> {
        let bytes = match config.peer_id {
            PeerIdConfig::Derived => {
                let mut bytes = [RUST_NODE_SIG_BYTE; 32];
                let bytes_len = bytes.len();
                let i_bytes = self.rust_nodes.len().to_be_bytes();
                let i = bytes_len - i_bytes.len();
                bytes[i..bytes_len].copy_from_slice(&i_bytes);
                bytes
            }
            PeerIdConfig::Bytes(bytes) => bytes,
        };
        let secret_key = SecretKey::from_bytes(bytes);
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
            enabled_channels: Default::default(),
            max_peers: 100,
            chain_id: self.chain_id.clone(),
            peer_discovery: true,
            timeouts: config.timeouts,
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
            |state, action| {
                log_action(action.action(), action.meta(), state.0.my_id());
                if let Action::P2p(p2p_action) = action.action() {
                    state
                        .0
                        .reducer(action.meta().clone().with_action(p2p_action))
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
            State(P2pState::new(config)),
        );

        let node_id = RustNodeId(self.rust_nodes.len());
        self.rust_nodes.push(RustNode::new(store, event_receiver));
        Ok(node_id)
    }

    pub fn add_libp2p_node(&mut self) -> Result<Libp2pNodeId> {
        let node_id = Libp2pNodeId(self.libp2p_nodes.len());
        self.libp2p_nodes.push(Libp2pNode {});
        Ok(node_id)
    }

    pub fn connect<T>(&mut self, id: RustNodeId, other: T)
    where
        T: Into<Listener>,
    {
        match other.into() {
            Listener::Rust(node_id) => {
                let opts = self.rust_node(node_id).dial_opts(self.ip);
                self.rust_node_mut(id)
                    .dispatch_action(P2pConnectionOutgoingAction::Init { opts, rpc_id: None });
            }
            Listener::Libp2p(node_id) => {
                let opts = self.libp2p_node(node_id).dial_opts(self.ip);
                self.rust_node_mut(id)
                    .dispatch_action(P2pConnectionOutgoingAction::Init { opts, rpc_id: None });
            }
            Listener::Multiaddr(_) => todo!(),
            Listener::SocketPeerId(_, _) => todo!(),
        }
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
}

#[derive(Debug)]
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
        let poll = self
            .idle_interval
            .poll_tick(cx)
            .map(|instant| instant.into_std());
        if let Poll::Ready(inst) = poll {
            let dur = inst - self.last_idle_instant;
            for rust_node in &mut self.rust_nodes {
                rust_node.idle(dur);
            }
            self.last_idle_instant = inst;
        }
        poll
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
                NextPoll::Rust(id) => {
                    let rust_node = this.rust_node_mut(id);
                    rust_node.poll_next_unpin(cx).map(|event| {
                        event.map(|event| {
                            rust_node.dispatch_event(event.clone());
                            ClusterEvent::Rust {
                                id,
                                event: rust_node
                                    .rust_node_event()
                                    .unwrap_or(RustNodeEvent::P2p { event }),
                            }
                        })
                    })
                }
                NextPoll::Libp2p(_) => todo!(),
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
