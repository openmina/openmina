use openmina_core::SubstateAccess;

use crate::connection::incoming::{IncomingSignalingMethod, P2pConnectionIncomingAction};
use crate::connection::outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts};
use crate::connection::{p2p_connection_reducer, P2pConnectionAction, P2pConnectionState};
use crate::disconnection::P2pDisconnectionAction;
use crate::discovery::p2p_discovery_reducer;
use crate::peer::p2p_peer_reducer;
use crate::webrtc::{HttpSignalingInfo, SignalingMethod};
use crate::{P2pAction, P2pActionWithMetaRef, P2pPeerState, P2pPeerStatus, P2pState};

impl P2pState {
    pub fn reducer<State, Action>(
        mut state: openmina_core::Substate<Action, State, Self>,
        action: P2pActionWithMetaRef<'_>,
    ) where
        State: SubstateAccess<Self>,
        Action: From<P2pAction>,
    {
        let (action, meta) = action.split();
        match action {
            P2pAction::Initialization(_) => {
                // noop
            }
            P2pAction::Connection(action) => {
                let my_id = state.my_id();
                let Some(peer_id) = action.peer_id() else {
                    return;
                };
                let peer = match action {
                    P2pConnectionAction::Outgoing(P2pConnectionOutgoingAction::Init {
                        opts,
                        ..
                    }) => state.peers.entry(*peer_id).or_insert_with(|| P2pPeerState {
                        is_libp2p: opts.is_libp2p(),
                        dial_opts: Some(opts.clone()),
                        status: P2pPeerStatus::Connecting(P2pConnectionState::outgoing_init(opts)),
                        identify: None,
                    }),
                    P2pConnectionAction::Incoming(P2pConnectionIncomingAction::Init {
                        opts,
                        ..
                    }) => state.peers.entry(*peer_id).or_insert_with(|| P2pPeerState {
                        is_libp2p: false,
                        dial_opts: {
                            let signaling = match opts.signaling {
                                IncomingSignalingMethod::Http => {
                                    SignalingMethod::Http(HttpSignalingInfo {
                                        host: opts.offer.host.clone(),
                                        port: opts.offer.listen_port,
                                    })
                                }
                            };
                            Some(P2pConnectionOutgoingInitOpts::WebRTC {
                                peer_id: *peer_id,
                                signaling,
                            })
                        },
                        status: P2pPeerStatus::Connecting(P2pConnectionState::incoming_init(opts)),
                        identify: None,
                    }),
                    P2pConnectionAction::Incoming(
                        P2pConnectionIncomingAction::FinalizePendingLibp2p { .. },
                    ) => {
                        state.peers.entry(*peer_id).or_insert_with(|| P2pPeerState {
                            is_libp2p: true,
                            dial_opts: None,
                            // correct status later set in the child reducer.
                            status: P2pPeerStatus::Disconnected { time: meta.time() },
                            identify: None,
                        })
                    }
                    _ => match state.peers.get_mut(peer_id) {
                        Some(v) => v,
                        None => return,
                    },
                };
                p2p_connection_reducer(peer, my_id, meta.with_action(action));
            }
            P2pAction::Disconnection(action) => match action {
                P2pDisconnectionAction::Init { .. } => {}
                P2pDisconnectionAction::Finish { peer_id } => {
                    if state
                        .network
                        .scheduler
                        .connections
                        .iter()
                        .any(|(_addr, conn_state)| conn_state.peer_id() == Some(peer_id))
                    {
                        // still have other connections
                        return;
                    }
                    let Some(peer) = state.peers.get_mut(peer_id) else {
                        return;
                    };
                    peer.status = P2pPeerStatus::Disconnected { time: meta.time() };
                }
            },
            P2pAction::Peer(action) => {
                p2p_peer_reducer(&mut state, meta.with_action(action));
            }
            P2pAction::Channels(action) => {
                let Some(peer_id) = action.peer_id() else {
                    return;
                };
                let Some(peer) = state.get_ready_peer_mut(peer_id) else {
                    return;
                };
                peer.channels.reducer(meta.with_action(action));
            }
            P2pAction::Discovery(action) => {
                p2p_discovery_reducer(&mut state, meta.with_action(action));
            }
            P2pAction::Identify(action) => match action {
                crate::identify::P2pIdentifyAction::NewRequest { .. } => {}
                crate::identify::P2pIdentifyAction::UpdatePeerInformation { peer_id, info } => {
                    if let Some(peer) = state.peers.get_mut(peer_id) {
                        peer.identify = Some(info.clone());
                    } else {
                        unreachable!()
                    }
                }
            },
            P2pAction::Network(action) => {
                let limits = state.config.limits.clone();
                state.network.reducer(meta.with_action(action), &limits)
            }
        }
    }
}
