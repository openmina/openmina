use openmina_core::{
    block::{ArcBlockWithHash, BlockWithHash},
    impl_substate_access,
    requests::RpcId,
    snark::{Snark, SnarkInfo, SnarkJobCommitment},
    ChainId, SubstateAccess,
};
use redux::{Callback, Timestamp};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use crate::{
    bootstrap::P2pNetworkKadBootstrapState,
    channels::{
        rpc::{P2pRpcId, P2pRpcRequest, P2pRpcResponse},
        streaming_rpc::{P2pStreamingRpcId, P2pStreamingRpcResponseFull},
        ChannelId, P2pChannelsState,
    },
    connection::{
        incoming::P2pConnectionIncomingState,
        outgoing::{
            P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState,
        },
        P2pConnectionResponse, P2pConnectionState,
    },
    is_time_passed,
    network::{
        identify::{P2pNetworkIdentify, P2pNetworkIdentifyState},
        P2pNetworkState,
    },
    Limit, P2pConfig, P2pLimits, P2pNetworkKadState, P2pNetworkPubsubState,
    P2pNetworkSchedulerState, P2pTimeouts, PeerId,
};
use mina_p2p_messages::v2::{MinaBaseUserCommandStableV2, MinaBlockBlockStableV2};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pState {
    pub chain_id: ChainId,
    pub config: P2pConfig,
    pub network: P2pNetworkState,
    pub peers: BTreeMap<PeerId, P2pPeerState>,
    pub callbacks: P2pCallbacks,
}

impl P2pState {
    pub fn new(config: P2pConfig, callbacks: P2pCallbacks, chain_id: &ChainId) -> Self {
        let addrs = if cfg!(feature = "p2p-libp2p") {
            config
                .libp2p_port
                .map(|port| multiaddr::multiaddr!(Ip4([127, 0, 0, 1]), Tcp((port))))
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };

        let my_id = config.identity_pub_key.peer_id();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let peer_id_str = my_id.to_libp2p_string();

            openmina_core::log::info!(
                openmina_core::log::system_time();
                kind = "P2pState new",
                summary = format!("Current node's id: {peer_id_str}"),
                peer_id_str = peer_id_str,
            );
        }

        let initial_peers = config
            .initial_peers
            .iter()
            .filter(|peer| peer.peer_id() != &my_id);

        let known_peers = if cfg!(feature = "p2p-libp2p") {
            initial_peers
                .clone()
                .filter_map(|peer| {
                    if let P2pConnectionOutgoingInitOpts::LibP2P(peer) = peer {
                        Some(peer.into())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let peers = initial_peers
            .map(|peer| {
                (
                    *peer.peer_id(),
                    P2pPeerState {
                        dial_opts: Some(peer.clone()),
                        is_libp2p: peer.is_libp2p(),
                        status: P2pPeerStatus::Disconnected {
                            time: Timestamp::ZERO,
                        },
                        identify: None,
                    },
                )
            })
            .collect();

        let network = P2pNetworkState::new(
            config.identity_pub_key.clone(),
            addrs,
            known_peers,
            chain_id,
            config.peer_discovery,
        );
        Self {
            chain_id: chain_id.clone(),
            config,
            network,
            peers,
            callbacks,
        }
    }

    pub fn my_id(&self) -> PeerId {
        self.config.identity_pub_key.peer_id()
    }

    pub fn peer_connection_rpc_id(&self, peer_id: &PeerId) -> Option<RpcId> {
        self.peers.get(peer_id)?.connection_rpc_id()
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer(&self, peer_id: &PeerId) -> Option<&P2pPeerStatusReady> {
        self.peers.get(peer_id)?.status.as_ready()
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut P2pPeerStatusReady> {
        self.peers.get_mut(peer_id)?.status.as_ready_mut()
    }

    pub fn any_ready_peers(&self) -> bool {
        self.peers
            .iter()
            .any(|(_, p)| p.status.as_ready().is_some())
    }

    pub fn disconnected_peers(&self) -> impl '_ + Iterator<Item = P2pConnectionOutgoingInitOpts> {
        self.peers.iter().filter_map(|(_, state)| {
            if let P2pPeerState {
                status: P2pPeerStatus::Disconnected { .. },
                dial_opts: Some(opts),
                ..
            } = state
            {
                Some(opts.clone())
            } else {
                None
            }
        })
    }

    pub fn ready_peers_iter(&self) -> impl Iterator<Item = (&PeerId, &P2pPeerStatusReady)> {
        self.peers
            .iter()
            .filter_map(|(id, p)| Some((id, p.status.as_ready()?)))
    }

    pub fn ready_rpc_peers_iter(&self) -> impl '_ + Iterator<Item = (PeerId, P2pRpcId)> {
        self.ready_peers_iter()
            .filter(|(_, p)| p.channels.rpc.can_send_request())
            .map(|(peer_id, p)| (*peer_id, p.channels.next_local_rpc_id()))
    }

    pub fn ready_peers(&self) -> Vec<PeerId> {
        self.peers
            .iter()
            .filter(|(_, p)| p.status.as_ready().is_some())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn connected_or_connecting_peers_count(&self) -> usize {
        self.peers
            .iter()
            .filter(|(_, p)| p.status.is_connected_or_connecting())
            .count()
    }

    pub fn is_peer_connecting(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .and_then(|p| p.status.as_connecting())
            .map_or(false, |p| !p.is_error())
    }

    pub fn is_peer_connected_or_connecting(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .map_or(false, |p| p.status.is_connected_or_connecting())
    }

    pub fn is_libp2p_peer(&self, peer_id: &PeerId) -> bool {
        self.peers.get(peer_id).map_or(false, |p| p.is_libp2p())
    }

    pub fn is_peer_rpc_timed_out(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
        now: redux::Timestamp,
    ) -> bool {
        self.get_ready_peer(peer_id).map_or(false, |p| {
            p.channels
                .rpc
                .is_timed_out(rpc_id, now, &self.config.timeouts)
        })
    }

    pub fn is_peer_streaming_rpc_timed_out(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pStreamingRpcId,
        now: redux::Timestamp,
    ) -> bool {
        self.get_ready_peer(peer_id).map_or(false, |p| {
            p.channels
                .streaming_rpc
                .is_timed_out(rpc_id, now, &self.config.timeouts)
        })
    }

    pub fn peer_rpc_timeouts(&self, now: redux::Timestamp) -> Vec<(PeerId, P2pRpcId, bool)> {
        let config = &self.config.timeouts;
        self.ready_peers_iter()
            .flat_map(|(peer_id, s)| {
                let pending_rpc = s.channels.rpc.pending_local_rpc_id();
                let pending_streaming_rpc = s.channels.streaming_rpc.pending_local_rpc_id();
                IntoIterator::into_iter([
                    pending_rpc
                        .filter(|id| s.channels.rpc.is_timed_out(*id, now, config))
                        .map(|id| (*peer_id, id, false)),
                    pending_streaming_rpc
                        .filter(|id| s.channels.streaming_rpc.is_timed_out(*id, now, config))
                        .map(|id| (*peer_id, id, true)),
                ])
                .flatten()
            })
            .collect()
    }

    pub fn peer_streaming_rpc_timeouts(
        &self,
        now: redux::Timestamp,
    ) -> Vec<(PeerId, P2pStreamingRpcId)> {
        self.ready_peers_iter()
            .filter_map(|(peer_id, s)| {
                let rpc_id = s.channels.streaming_rpc.pending_local_rpc_id()?;
                if !s
                    .channels
                    .streaming_rpc
                    .is_timed_out(rpc_id, now, &self.config.timeouts)
                {
                    return None;
                }

                Some((*peer_id, rpc_id))
            })
            .collect()
    }

    pub fn already_has_min_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= self.config.limits.min_peers()
    }

    pub fn already_has_max_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= self.config.limits.max_peers()
    }

    /// The peers capacity is exceeded.
    pub fn already_has_max_ready_peers(&self) -> bool {
        self.ready_peers_iter().count() >= self.config.limits.max_peers()
    }

    /// Minimal number of peers that the node should connect
    pub fn min_peers(&self) -> Limit<usize> {
        self.config.limits.min_peers()
    }

    /// Peer with libp2p connection identified by `conn_id`.
    #[cfg(feature = "p2p-libp2p")]
    pub fn peer_with_connection(
        &self,
        conn_id: crate::ConnectionAddr,
    ) -> Option<(PeerId, &P2pPeerState)> {
        let result = if let crate::ConnectionAddr {
            sock_addr,
            incoming: false,
        } = conn_id
        {
            self.peers
                .iter()
                .find(|(_, peer_state)| match &peer_state.dial_opts {
                    Some(P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts)) => {
                        libp2p_opts.matches_socket_addr(sock_addr)
                    }
                    _ => false,
                })
        } else {
            None
        };

        result
            .or_else(|| {
                self.network
                    .scheduler
                    .connections
                    .get(&conn_id)
                    .and_then(|state| {
                        state
                            .peer_id()
                            .and_then(|peer_id| self.peers.iter().find(|(id, _)| *id == peer_id))
                    })
            })
            .map(|(peer_id, peer_state)| (*peer_id, peer_state))
    }

    pub fn incoming_peer_connection_mut(
        &mut self,
        peer_id: &PeerId,
    ) -> Option<&mut P2pConnectionIncomingState> {
        let peer_state = self.peers.get_mut(peer_id)?;

        match &mut peer_state.status {
            P2pPeerStatus::Connecting(P2pConnectionState::Incoming(state)) => Some(state),
            _ => None,
        }
    }

    pub fn outgoing_peer_connection_mut(
        &mut self,
        peer_id: &PeerId,
    ) -> Option<&mut P2pConnectionOutgoingState> {
        let peer_state = self.peers.get_mut(peer_id)?;

        match &mut peer_state.status {
            P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(state)) => Some(state),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerState {
    pub is_libp2p: bool,
    pub dial_opts: Option<P2pConnectionOutgoingInitOpts>,
    pub status: P2pPeerStatus,
    pub identify: Option<P2pNetworkIdentify>,
}

impl P2pPeerState {
    pub fn is_libp2p(&self) -> bool {
        self.is_libp2p
    }

    pub fn connection_rpc_id(&self) -> Option<RpcId> {
        match &self.status {
            P2pPeerStatus::Connecting(v) => v.rpc_id(),
            _ => None,
        }
    }

    /// Returns true if the peer can be reconnected, that is:
    /// - it has available dial options
    /// - it is never been connected yet or enough time is passed since its connection failure or disconnection.
    pub fn can_reconnect(&self, now: Timestamp, timeouts: &P2pTimeouts) -> bool {
        self.dial_opts.is_some()
            && match &self.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::Error { time, .. },
                )) => is_time_passed(now, *time, timeouts.incoming_error_reconnect_timeout),
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Error { time, .. },
                )) => is_time_passed(now, *time, timeouts.outgoing_error_reconnect_timeout),
                P2pPeerStatus::Disconnected { time } | P2pPeerStatus::Disconnecting { time } => {
                    *time == Timestamp::ZERO
                        || is_time_passed(now, *time, timeouts.reconnect_timeout)
                }
                _ => false,
            }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "state")]
pub enum P2pPeerStatus {
    Connecting(P2pConnectionState),
    Disconnecting { time: redux::Timestamp },
    Disconnected { time: redux::Timestamp },

    Ready(P2pPeerStatusReady),
}

impl P2pPeerStatus {
    /// Checks if the peer is in `Connecting` state and we have finished
    /// connecting to the peer.
    pub fn is_connecting_success(&self) -> bool {
        match self {
            Self::Connecting(v) => v.is_success(),
            _ => false,
        }
    }

    pub fn is_connected_or_connecting(&self) -> bool {
        match self {
            Self::Connecting(s) => !s.is_error(),
            Self::Ready(_) => true,
            Self::Disconnecting { .. } => false,
            Self::Disconnected { .. } => false,
        }
    }

    pub fn is_disconnected_or_disconnecting(&self) -> bool {
        matches!(self, Self::Disconnecting { .. } | Self::Disconnected { .. })
    }

    pub fn as_connecting(&self) -> Option<&P2pConnectionState> {
        match self {
            Self::Connecting(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_ready(&self) -> Option<&P2pPeerStatusReady> {
        match self {
            Self::Ready(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_ready_mut(&mut self) -> Option<&mut P2pPeerStatusReady> {
        match self {
            Self::Ready(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, P2pPeerStatus::Connecting(s) if s.is_error())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerStatusReady {
    pub is_incoming: bool,
    pub connected_since: redux::Timestamp,
    pub channels: P2pChannelsState,
    pub best_tip: Option<ArcBlockWithHash>,
}

impl P2pPeerStatusReady {
    pub fn new(
        is_incoming: bool,
        time: redux::Timestamp,
        enabled_channels: &BTreeSet<ChannelId>,
    ) -> Self {
        Self {
            is_incoming,
            connected_since: time,
            channels: P2pChannelsState::new(enabled_channels),
            best_tip: None,
        }
    }
}

impl SubstateAccess<P2pState> for P2pState {
    fn substate(&self) -> openmina_core::SubstateResult<&P2pState> {
        Ok(self)
    }

    fn substate_mut(&mut self) -> openmina_core::SubstateResult<&mut P2pState> {
        Ok(self)
    }
}

type OptionalCallback<T> = Option<Callback<T>>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct P2pCallbacks {
    /// Callback for [`P2pChannelsTransactionAction::Libp2pReceived`]
    pub on_p2p_channels_transaction_libp2p_received:
        OptionalCallback<Box<MinaBaseUserCommandStableV2>>,
    /// Callback for [`P2pChannelsSnarkJobCommitmentAction::Received`]
    pub on_p2p_channels_snark_job_commitment_received:
        OptionalCallback<(PeerId, Box<SnarkJobCommitment>)>,

    /// Callback for [`P2pChannelsSnarkAction::Received`]
    pub on_p2p_channels_snark_received: OptionalCallback<(PeerId, Box<SnarkInfo>)>,
    /// Callback for [`P2pChannelsSnarkAction::Libp2pReceived`]
    pub on_p2p_channels_snark_libp2p_received: OptionalCallback<(PeerId, Box<Snark>)>,

    /// Callback for [`P2pChannelsBestTipAction::RequestReceived`]
    pub on_p2p_channels_best_tip_request_received: OptionalCallback<PeerId>,

    /// Callback for [`P2pDisconnectionAction::Finish`]
    pub on_p2p_disconnection_finish: OptionalCallback<PeerId>,

    /// TODO: these 2 should be set by `P2pConnectionOutgoingAction::Init`
    /// Callback for [`P2pConnectionOutgoingAction::Error`]
    pub on_p2p_connection_outgoing_error: OptionalCallback<(RpcId, P2pConnectionOutgoingError)>,
    /// Callback for [`P2pConnectionOutgoingAction::Success`]
    pub on_p2p_connection_outgoing_success: OptionalCallback<RpcId>,

    /// TODO: these 3 should be set by `P2pConnectionIncomingAction::Init`
    /// Callback for [`P2pConnectionIncomingAction::Error`]
    pub on_p2p_connection_incoming_error: OptionalCallback<(RpcId, String)>,
    /// Callback for [`P2pConnectionIncomingAction::Success`]
    pub on_p2p_connection_incoming_success: OptionalCallback<RpcId>,
    /// Callback for [`P2pConnectionIncomingAction::AnswerReady`]
    pub on_p2p_connection_incoming_answer_ready:
        OptionalCallback<(RpcId, PeerId, P2pConnectionResponse)>,

    /// Callback for [`P2pPeerAction::BestTipUpdate`]
    pub on_p2p_peer_best_tip_update: OptionalCallback<BlockWithHash<Arc<MinaBlockBlockStableV2>>>,

    /// Callback for [`P2pChannelsRpcAction::Ready`]
    pub on_p2p_channels_rpc_ready: OptionalCallback<PeerId>,
    /// Callback for [`P2pChannelsRpcAction::Timeout`]
    pub on_p2p_channels_rpc_timeout: OptionalCallback<(PeerId, P2pRpcId)>,
    /// Callback for [`P2pChannelsRpcAction::ResponseReceived`]
    pub on_p2p_channels_rpc_response_received:
        OptionalCallback<(PeerId, P2pRpcId, Option<Box<P2pRpcResponse>>)>,
    /// Callback for [`P2pChannelsRpcAction::RequestReceived`]
    pub on_p2p_channels_rpc_request_received:
        OptionalCallback<(PeerId, P2pRpcId, Box<P2pRpcRequest>)>,

    /// Callback for [`P2pChannelsStreamingRpcAction::Ready`]
    pub on_p2p_channels_streaming_rpc_ready: OptionalCallback<()>,
    /// Callback for [`P2pChannelsStreamingRpcAction::Timeout`]
    pub on_p2p_channels_streaming_rpc_timeout: OptionalCallback<(PeerId, P2pRpcId)>,
    /// Callback for [`P2pChannelsStreamingRpcAction::ResponseReceived`]
    pub on_p2p_channels_streaming_rpc_response_received:
        OptionalCallback<(PeerId, P2pRpcId, Option<P2pStreamingRpcResponseFull>)>,
}

impl_substate_access!(P2pState, P2pNetworkState, network);
impl_substate_access!(P2pState, P2pNetworkSchedulerState, network.scheduler);
impl_substate_access!(P2pState, P2pLimits, config.limits);

impl SubstateAccess<P2pNetworkKadState> for P2pState {
    fn substate(&self) -> openmina_core::SubstateResult<&P2pNetworkKadState> {
        self.network
            .scheduler
            .discovery_state()
            .ok_or_else(|| "kademlia state is unavailable".to_owned())
    }

    fn substate_mut(&mut self) -> openmina_core::SubstateResult<&mut P2pNetworkKadState> {
        self.network
            .scheduler
            .discovery_state
            .as_mut()
            .ok_or_else(|| "kademlia state is unavailable".to_owned())
    }
}

impl SubstateAccess<P2pNetworkKadBootstrapState> for P2pState {
    fn substate(&self) -> openmina_core::SubstateResult<&P2pNetworkKadBootstrapState> {
        let kad_state: &P2pNetworkKadState = self.substate()?;
        kad_state
            .bootstrap_state()
            .ok_or_else(|| "bootstrap state is unavailable".to_owned())
    }

    fn substate_mut(&mut self) -> openmina_core::SubstateResult<&mut P2pNetworkKadBootstrapState> {
        let kad_state: &mut P2pNetworkKadState = self.substate_mut()?;
        kad_state
            .bootstrap_state_mut()
            .ok_or_else(|| "bootstrap state is unavailable".to_owned())
    }
}

impl_substate_access!(
    P2pState,
    P2pNetworkIdentifyState,
    network.scheduler.identify_state
);
impl_substate_access!(
    P2pState,
    P2pNetworkPubsubState,
    network.scheduler.broadcast_state
);
impl_substate_access!(P2pState, P2pConfig, config);
