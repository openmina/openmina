use std::net::SocketAddr;

use redux::ActionMeta;

use crate::connection::{P2pConnectionErrorResponse, P2pConnectionState};
use crate::peer::P2pPeerAction;
use crate::webrtc::Host;
use crate::{connection::P2pConnectionService, webrtc};
use crate::{is_old_libp2p, P2pNetworkKadRequestAction, P2pNetworkSchedulerAction, P2pPeerStatus};

use super::libp2p_opts::P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError;
use super::{
    P2pConnectionOutgoingAction, P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts,
    P2pConnectionOutgoingState,
};

impl P2pConnectionOutgoingAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pConnectionOutgoingAction::RandomInit => {
                #[cfg(feature = "p2p-libp2p")]
                {
                    let peers = store.state().initial_unused_peers();
                    let picked_peer = store.service().random_pick(&peers);
                    store.dispatch(P2pConnectionOutgoingAction::Init {
                        opts: picked_peer,
                        rpc_id: None,
                    });
                }
                #[cfg(not(feature = "p2p-libp2p"))]
                {
                    let peers = store.state().disconnected_peers().collect::<Vec<_>>();
                    let picked_peer = store.service().random_pick(&peers);
                    store.dispatch(P2pConnectionOutgoingAction::Reconnect {
                        opts: picked_peer,
                        rpc_id: None,
                    });
                }
            }
            P2pConnectionOutgoingAction::Init { opts, .. }
            | P2pConnectionOutgoingAction::Reconnect { opts, .. } => {
                let peer_id = *opts.peer_id();
                if let P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) = &opts {
                    if is_old_libp2p() {
                        store.service().outgoing_init(opts.clone());
                    } else {
                        match SocketAddr::try_from(libp2p_opts) {
                            Ok(addr) => {
                                store.dispatch(P2pNetworkSchedulerAction::OutgoingConnect { addr });
                            }
                            Err(
                                P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError::Unresolved(
                                    _name,
                                ),
                            ) => {
                                // TODO: initiate name resolution
                                openmina_core::warn!(meta.time(); "name resolution needed to connect to {}", opts);
                            }
                        }
                    }
                    store.dispatch(P2pConnectionOutgoingAction::FinalizePending { peer_id });
                } else {
                    store.dispatch(P2pConnectionOutgoingAction::OfferSdpCreatePending { peer_id });
                }
            }
            P2pConnectionOutgoingAction::OfferSdpCreateError { peer_id, error } => {
                store.dispatch(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::SdpCreateError(error),
                });
            }
            P2pConnectionOutgoingAction::OfferSdpCreateSuccess { peer_id, sdp } => {
                let offer = webrtc::Offer {
                    sdp,
                    identity_pub_key: store.state().config.identity_pub_key.clone(),
                    target_peer_id: peer_id,
                    // TODO(vlad9486): put real address
                    host: Host::Ipv4([127, 0, 0, 1].into()),
                    listen_port: store.state().config.listen_port,
                };
                store.dispatch(P2pConnectionOutgoingAction::OfferReady { peer_id, offer });
            }
            P2pConnectionOutgoingAction::OfferReady { peer_id, offer } => {
                let (state, service) = store.state_and_service();
                let Some(peer) = state.peers.get(&peer_id) else {
                    return;
                };
                let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::OfferReady { opts, .. },
                )) = &peer.status
                else {
                    return;
                };
                let signaling_method = match opts {
                    P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => signaling,
                    #[allow(unreachable_patterns)]
                    _ => return,
                };
                match signaling_method {
                    webrtc::SignalingMethod::Http(_) | webrtc::SignalingMethod::Https(_) => {
                        let Some(url) = signaling_method.http_url() else {
                            return;
                        };
                        service.http_signaling_request(url, offer);
                    }
                }
                store.dispatch(P2pConnectionOutgoingAction::OfferSendSuccess { peer_id });
            }
            P2pConnectionOutgoingAction::OfferSendSuccess { peer_id } => {
                store.dispatch(P2pConnectionOutgoingAction::AnswerRecvPending { peer_id });
            }
            P2pConnectionOutgoingAction::AnswerRecvError { peer_id, error } => {
                store.dispatch(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: match error {
                        P2pConnectionErrorResponse::Rejected(reason) => {
                            P2pConnectionOutgoingError::Rejected(reason)
                        }
                        P2pConnectionErrorResponse::InternalError => {
                            P2pConnectionOutgoingError::RemoteInternalError
                        }
                    },
                });
            }
            P2pConnectionOutgoingAction::AnswerRecvSuccess { peer_id, answer } => {
                store.service().set_answer(peer_id, answer.clone());
                store.dispatch(P2pConnectionOutgoingAction::FinalizePending { peer_id });
            }
            P2pConnectionOutgoingAction::FinalizeError { peer_id, error } => {
                store.dispatch(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::FinalizeError(error),
                });
            }
            P2pConnectionOutgoingAction::FinalizeSuccess { peer_id } => {
                store.dispatch(P2pConnectionOutgoingAction::Success { peer_id });
            }
            P2pConnectionOutgoingAction::Timeout { peer_id } => {
                store.dispatch(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::Timeout,
                });
            }
            P2pConnectionOutgoingAction::Success { peer_id } => {
                store.dispatch(P2pPeerAction::Ready {
                    peer_id,
                    incoming: false,
                });
            }
            P2pConnectionOutgoingAction::Error { peer_id, error } => {
                if store
                    .state()
                    .network
                    .scheduler
                    .discovery_state()
                    .and_then(|discovery_state| discovery_state.request(&peer_id))
                    .is_some()
                {
                    store.dispatch(P2pNetworkKadRequestAction::Error {
                        peer_id,
                        error: format!("{error:?}"),
                    });
                }
            }
            _ => {}
        }
    }
}
