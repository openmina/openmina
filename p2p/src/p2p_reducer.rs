use url::Host;

use crate::connection::incoming::{IncomingSignalingMethod, P2pConnectionIncomingAction};
use crate::connection::outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts};
use crate::connection::{p2p_connection_reducer, P2pConnectionAction, P2pConnectionState};
use crate::disconnection::P2pDisconnectionAction;
use crate::discovery::{P2pDiscoveryAction, P2pDiscoveryInitAction, P2pDiscoverySuccessAction};
use crate::peer::p2p_peer_reducer;
use crate::webrtc::{HttpSignalingInfo, SignalingMethod};
use crate::{P2pAction, P2pActionWithMetaRef, P2pPeerState, P2pPeerStatus, P2pState};

impl P2pState {
    pub fn reducer(&mut self, action: P2pActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pAction::Connection(action) => {
                let Some(peer_id) = action.peer_id() else {
                    return;
                };
                let peer = match action {
                    P2pConnectionAction::Outgoing(P2pConnectionOutgoingAction::Init(v)) => {
                        self.peers.entry(*peer_id).or_insert_with(|| P2pPeerState {
                            dial_opts: Some(v.opts.clone()),
                            status: P2pPeerStatus::Connecting(P2pConnectionState::outgoing_init(
                                &v.opts,
                            )),
                        })
                    }
                    P2pConnectionAction::Incoming(P2pConnectionIncomingAction::Init(v)) => {
                        self.peers.entry(*peer_id).or_insert_with(|| P2pPeerState {
                            dial_opts: {
                                Host::parse(&v.opts.offer.host).ok().map(|host| {
                                    let signaling = match v.opts.signaling {
                                        IncomingSignalingMethod::Http => {
                                            SignalingMethod::Http(HttpSignalingInfo {
                                                host,
                                                port: v.opts.offer.listen_port,
                                            })
                                        }
                                    };
                                    P2pConnectionOutgoingInitOpts::WebRTC {
                                        peer_id: *peer_id,
                                        signaling,
                                    }
                                })
                            },
                            status: P2pPeerStatus::Connecting(P2pConnectionState::incoming_init(
                                &v.opts,
                            )),
                        })
                    }
                    P2pConnectionAction::Incoming(P2pConnectionIncomingAction::Libp2pReceived(
                        _,
                    )) => {
                        self.peers.entry(*peer_id).or_insert_with(|| P2pPeerState {
                            dial_opts: None,
                            // correct status later set in the child reducer.
                            status: P2pPeerStatus::Disconnected { time: meta.time() },
                        })
                    }
                    _ => match self.peers.get_mut(peer_id) {
                        Some(v) => v,
                        None => return,
                    },
                };
                p2p_connection_reducer(peer, meta.with_action(action));
            }
            P2pAction::Disconnection(action) => match action {
                P2pDisconnectionAction::Init(_) => {}
                P2pDisconnectionAction::Finish(a) => {
                    let Some(peer) = self.peers.get_mut(&a.peer_id) else {
                        return;
                    };
                    peer.status = P2pPeerStatus::Disconnected { time: meta.time() };
                }
            },
            P2pAction::Peer(action) => {
                p2p_peer_reducer(self, meta.with_action(action));
            }
            P2pAction::Channels(action) => {
                let Some(peer_id) = action.peer_id() else {
                    return;
                };
                let Some(peer) = self.get_ready_peer_mut(peer_id) else {
                    return;
                };
                peer.channels.reducer(meta.with_action(action));
            }
            P2pAction::Discovery(action) => {
                match action {
                    P2pDiscoveryAction::Init(P2pDiscoveryInitAction { peer_id }) => {
                        let Some(peer) = self.get_ready_peer_mut(peer_id) else {
                            return;
                        };
                        peer.last_asked_initial_peers = Some(meta.time());
                    }
                    P2pDiscoveryAction::Success(P2pDiscoverySuccessAction { peers, peer_id }) => {
                        if let Some(peer) = self.get_ready_peer_mut(peer_id) {
                            peer.last_received_initial_peers = Some(meta.time());
                        };
                        self.known_peers.extend(peers.iter().filter_map(|msg| {
                            P2pConnectionOutgoingInitOpts::try_from_mina_rpc(msg)
                        }));
                    }
                    P2pDiscoveryAction::Timeout(_) => {}
                }
            }
        }
    }
}
