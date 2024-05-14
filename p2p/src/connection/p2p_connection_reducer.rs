use std::net::{IpAddr, SocketAddr};

use multiaddr::Protocol;

use crate::{P2pPeerState, P2pPeerStatus, PeerId};

use super::{
    incoming::{P2pConnectionIncomingAction, P2pConnectionIncomingState},
    outgoing::{
        P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState,
    },
    P2pConnectionAction, P2pConnectionActionWithMetaRef, P2pConnectionState,
};

pub fn p2p_connection_reducer(
    state: &mut P2pPeerState,
    my_id: PeerId,
    action: P2pConnectionActionWithMetaRef<'_>,
) {
    let (action, meta) = action.split();
    match action {
        P2pConnectionAction::Outgoing(action) => {
            if let P2pConnectionOutgoingAction::Reconnect { opts, rpc_id }
            | P2pConnectionOutgoingAction::Init { opts, rpc_id } = action
            {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Init {
                        time: meta.time(),
                        opts: opts.clone(),
                        rpc_id: *rpc_id,
                    },
                ));
            }
            let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(state)) = &mut state.status
            else {
                return;
            };
            state.reducer(meta.with_action(action));
        }
        P2pConnectionAction::Incoming(action) => {
            if let P2pConnectionIncomingAction::Init { opts, rpc_id } = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::Init {
                        time: meta.time(),
                        signaling: opts.signaling.clone(),
                        offer: opts.offer.clone(),
                        rpc_id: *rpc_id,
                    },
                ))
            } else if let P2pConnectionIncomingAction::FinalizePendingLibp2p {
                peer_id, addr, ..
            } = action
            {
                let incoming_state = match &state.status {
                    // No duplicate connection
                    P2pPeerStatus::Disconnected { .. } => {
                        Some(P2pConnectionIncomingState::FinalizePendingLibp2p {
                            addr: *addr,
                            close_duplicates: Vec::new(),
                            time: meta.time(),
                        })
                    }
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(_))
                        if &my_id < peer_id =>
                    {
                        // connection from lesser peer_id to greater one is kept in favour of the opposite one (incoming in this case)
                        None
                    }
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(_)) => {
                        let mut close_duplicates = Vec::new();
                        if let Some(identify) = state.identify.as_ref() {
                            close_duplicates.extend(identify.listen_addrs.iter().filter_map(
                                |maddr| {
                                    let mut iter = maddr.iter();
                                    let ip: IpAddr = match iter.next()? {
                                        Protocol::Ip4(ip4) => ip4.into(),
                                        Protocol::Ip6(ip6) => ip6.into(),
                                        _ => return None,
                                    };
                                    let port = match iter.next()? {
                                        Protocol::Tcp(port) => port,
                                        _ => return None,
                                    };
                                    Some(SocketAddr::from((ip, port)))
                                },
                            ))
                        }
                        if let Some(P2pConnectionOutgoingInitOpts::LibP2P(libp2p)) =
                            state.dial_opts.as_ref()
                        {
                            match libp2p.try_into() {
                                Ok(addr) if !close_duplicates.contains(&addr) => {
                                    close_duplicates.push(addr)
                                }
                                _ => {}
                            }
                        };
                        Some(P2pConnectionIncomingState::FinalizePendingLibp2p {
                            addr: *addr,
                            close_duplicates,
                            time: meta.time(),
                        })
                    }
                    _ => None,
                };
                if let Some(incoming_state) = incoming_state {
                    state.status =
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(incoming_state));
                }
            }
            let P2pPeerStatus::Connecting(P2pConnectionState::Incoming(state)) = &mut state.status
            else {
                return;
            };
            state.reducer(meta.with_action(action));
        }
    }
}
