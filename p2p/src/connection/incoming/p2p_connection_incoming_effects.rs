use redux::ActionMeta;

#[cfg(feature = "p2p-libp2p")]
use crate::disconnection::{P2pDisconnectionAction, P2pDisconnectionReason};
#[cfg(feature = "p2p-libp2p")]
use crate::P2pNetworkSchedulerAction;
use crate::{connection::P2pConnectionService, webrtc};
use crate::{peer::P2pPeerAction, ConnectionAddr};

use super::{P2pConnectionIncomingAction, P2pConnectionIncomingError};

impl P2pConnectionIncomingAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pConnectionIncomingAction::Init { opts, .. } => {
                let peer_id = opts.peer_id;
                store.service().incoming_init(peer_id, *opts.offer);
                store.dispatch(P2pConnectionIncomingAction::AnswerSdpCreatePending { peer_id });
            }
            P2pConnectionIncomingAction::AnswerSdpCreateError { peer_id, error } => {
                store.dispatch(P2pConnectionIncomingAction::Error {
                    peer_id,
                    error: P2pConnectionIncomingError::SdpCreateError(error),
                });
            }
            P2pConnectionIncomingAction::AnswerSdpCreateSuccess { peer_id, sdp } => {
                let answer = Box::new(webrtc::Answer {
                    sdp,
                    identity_pub_key: store.state().config.identity_pub_key.clone(),
                    target_peer_id: peer_id,
                });
                store.dispatch(P2pConnectionIncomingAction::AnswerReady { peer_id, answer });
            }
            P2pConnectionIncomingAction::AnswerReady { peer_id, answer } => {
                store.service().set_answer(peer_id, *answer);
            }
            P2pConnectionIncomingAction::AnswerSendSuccess { peer_id } => {
                store.dispatch(P2pConnectionIncomingAction::FinalizePending { peer_id });
            }
            P2pConnectionIncomingAction::FinalizeError { peer_id, error } => {
                store.dispatch(P2pConnectionIncomingAction::Error {
                    peer_id,
                    error: P2pConnectionIncomingError::FinalizeError(error),
                });
            }
            P2pConnectionIncomingAction::FinalizeSuccess { peer_id } => {
                store.dispatch(P2pConnectionIncomingAction::Success { peer_id });
            }
            P2pConnectionIncomingAction::Timeout { peer_id } => {
                #[cfg(feature = "p2p-libp2p")]
                if let Some((addr, _)) = store
                    .state()
                    .network
                    .scheduler
                    .connections
                    .iter()
                    .find(|(_, state)| state.peer_id().is_some_and(|id| *id == peer_id))
                {
                    store.dispatch(P2pNetworkSchedulerAction::Disconnect {
                        addr: *addr,
                        reason: P2pDisconnectionReason::Timeout,
                    });
                }

                store.dispatch(P2pConnectionIncomingAction::Error {
                    peer_id,
                    error: P2pConnectionIncomingError::Timeout,
                });
            }
            P2pConnectionIncomingAction::Success { peer_id } => {
                store.dispatch(P2pPeerAction::Ready {
                    peer_id,
                    incoming: true,
                });
            }
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pConnectionIncomingAction::FinalizePendingLibp2p { .. } => {}
            #[cfg(feature = "p2p-libp2p")]
            P2pConnectionIncomingAction::FinalizePendingLibp2p { peer_id, addr } => {
                use super::P2pConnectionIncomingState;
                use crate::connection::RejectionReason;
                use openmina_core::{debug, error, warn};
                let Some(peer_state) = store.state().peers.get(&peer_id) else {
                    error!(_meta.time(); "no peer state for incoming connection from: {peer_id}");
                    return;
                };

                if let Some(P2pConnectionIncomingState::FinalizePendingLibp2p {
                    close_duplicates,
                    ..
                }) = peer_state
                    .status
                    .as_connecting()
                    .and_then(|connecting| connecting.as_incoming())
                {
                    if let Err(reason) = store.state().libp2p_incoming_accept(peer_id) {
                        warn!(_meta.time(); node_id = display(store.state().my_id()), summary = "rejecting incoming conection", peer_id = display(peer_id), reason = display(&reason));
                        store.dispatch(P2pDisconnectionAction::Init {
                            peer_id,
                            reason: P2pDisconnectionReason::Libp2pIncomingRejected(reason),
                        });
                    } else {
                        debug!(_meta.time(); "accepting incoming conection from {peer_id}");
                        if !close_duplicates.is_empty() {
                            let duplicates = store
                                .state()
                                .network
                                .scheduler
                                .connections
                                .keys()
                                .filter(
                                    |ConnectionAddr {
                                         sock_addr,
                                         incoming,
                                     }| {
                                        *incoming
                                            && sock_addr != &addr
                                            && close_duplicates.contains(sock_addr)
                                    },
                                )
                                .cloned()
                                .collect::<Vec<_>>();
                            for addr in duplicates {
                                warn!(_meta.time(); node_id = display(store.state().my_id()), summary = "closing duplicate connection", addr = display(addr));
                                store.dispatch(P2pNetworkSchedulerAction::Disconnect {
                                    addr,
                                    reason: P2pDisconnectionReason::Libp2pIncomingRejected(
                                        RejectionReason::AlreadyConnected,
                                    ),
                                });
                            }
                        }
                    }
                } else {
                    warn!(_meta.time(); node_id = display(store.state().my_id()), summary = "rejecting incoming conection as duplicate", peer_id = display(peer_id));
                    store.dispatch(P2pNetworkSchedulerAction::Disconnect {
                        addr: ConnectionAddr {
                            sock_addr: addr,
                            incoming: true,
                        },
                        reason: P2pDisconnectionReason::Libp2pIncomingRejected(
                            RejectionReason::AlreadyConnected,
                        ),
                    });
                }
            }
            P2pConnectionIncomingAction::Libp2pReceived { peer_id: _peer_id } => {
                #[cfg(feature = "p2p-libp2p")]
                store.dispatch(P2pPeerAction::Ready {
                    peer_id: _peer_id,
                    incoming: true,
                });
            }
            P2pConnectionIncomingAction::AnswerSdpCreatePending { .. } => {}
            P2pConnectionIncomingAction::FinalizePending { .. } => {}
            P2pConnectionIncomingAction::Error { .. } => {}
        }
    }
}
