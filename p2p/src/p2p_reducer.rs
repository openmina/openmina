use openmina_core::log::error;

use crate::connection::libp2p::P2pConnectionLibP2pAction;
use crate::connection::webrtc::P2pConnectionWebRTCAction;
use crate::connection::{p2p_connection_reducer, P2pConnectionAction};
use crate::disconnection::P2pDisconnectionAction;
use crate::peer::p2p_peer_reducer;
use crate::{P2pAction, P2pActionWithMetaRef, P2pPeerState, P2pPeerStatus, P2pState};

impl P2pState {
    pub fn reducer(&mut self, action: P2pActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pAction::Listen(action) => {
                self.listeners.reducer(meta.with_action(action));
            }
            P2pAction::Connection(action) => {
                let Some(peer_id) = action.peer_id() else {
                    error!(meta.time(); "empty peer id for action {action:?}");
                    return;
                };
                let peer_state = match action {
                    P2pConnectionAction::LibP2p(P2pConnectionLibP2pAction::Incoming(action)) => {
                        self.peers
                            .entry(peer_id.clone())
                            .or_insert_with(|| P2pPeerState::new_libp2p_incoming())
                    }
                    P2pConnectionAction::WebRTC(P2pConnectionWebRTCAction::Incoming(action)) => {
                        self.peers
                            .entry(peer_id.clone())
                            .or_insert_with(|| P2pPeerState::new_webrtc_incoming())
                    }
                    P2pConnectionAction::LibP2p(P2pConnectionLibP2pAction::Outgoing(_))
                    | P2pConnectionAction::WebRTC(P2pConnectionWebRTCAction::Outgoing(_)) => {
                        let Some(peer_state) = self.peers.get_mut(&peer_id) else {
                            error!(meta.time(); "non-existing peer {}", peer_id);
                            return;
                        };
                        peer_state
                    }
                };
                let Some(peer_state) = self.peers.get_mut(peer_id) else {
                    error!(meta.time(); "non-existing peer {}", peer_id);
                    return;
                };
                p2p_connection_reducer(peer_state, meta.with_action(action));
            }
            P2pAction::Disconnection(action) => match action {
                P2pDisconnectionAction::Init(_) => {}
                P2pDisconnectionAction::Finish(a) => {
                    let Some(peer) = self.peers.get_mut(&a.peer_id) else {
                        error!(meta.time(); "non-existing peer {}", a.peer_id);
                        return;
                    };
                    match peer {
                        P2pPeerState::Default => {
                            error!(meta.time(); "peer {} is not initialized", a.peer_id);
                        }
                        P2pPeerState::WebRTC(s) => {
                            s.status = P2pPeerStatus::Disconnected { time: meta.time() };
                        }
                        P2pPeerState::Libp2p(s) => {
                            s.status = P2pPeerStatus::Disconnected { time: meta.time() };
                        }
                    }
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
                    error!(meta.time(); "non-existing peer {}", peer_id);
                    return;
                };
                peer.channels.reducer(meta.with_action(action));
            }
            P2pAction::Discovery(action) => {
                self.kademlia.reducer(meta.with_action(action));
            }
        }
    }
}
