use crate::connection::incoming::P2pConnectionIncomingAction;
use crate::connection::outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts};
use crate::connection::{p2p_connection_reducer, P2pConnectionAction, P2pConnectionState};
use crate::disconnection::P2pDisconnectionAction;
use crate::discovery::{P2pDiscoveryAction, P2pDiscoverySuccessAction};
use crate::peer::p2p_peer_reducer;
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
                            dial_opts: None,
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
            P2pAction::Discovery(action) => match action {
                P2pDiscoveryAction::Init(_) => {}
                P2pDiscoveryAction::Success(P2pDiscoverySuccessAction { peers, .. }) => {
                    self.known_peers.extend(peers.iter().filter_map(|peer| {
                        let peer_id_str = String::try_from(&peer.peer_id.0).ok()?;
                        let peer_id = peer_id_str.parse::<libp2p::PeerId>().ok()?;
                        if peer_id.as_ref().code() == 0x12 {
                            // the peer_id is not supported
                            return None;
                        }
                        let opts = P2pConnectionOutgoingInitOpts::LibP2P {
                            peer_id: peer_id.into(),
                            maddr: {
                                use libp2p::{multiaddr::Protocol, Multiaddr};
                                use std::net::IpAddr;

                                let host = String::try_from(&peer.host).ok()?;
                                let mut a = Multiaddr::from(host.parse::<IpAddr>().ok()?);
                                a.push(Protocol::Tcp(peer.libp2p_port.0 as u16));
                                a.push(Protocol::P2p(peer_id.into()));
                                a
                            },
                        };
                        Some((peer_id.into(), opts))
                    }));
                }
                P2pDiscoveryAction::Timeout(_) => {}
            },
        }
    }
}
