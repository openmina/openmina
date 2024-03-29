use redux::ActionMeta;

use crate::disconnection::{P2pDisconnectionAction, P2pDisconnectionReason};
use crate::peer::P2pPeerAction;
use crate::{connection::P2pConnectionService, webrtc};

use super::{P2pConnectionIncomingAction, P2pConnectionIncomingError};

impl P2pConnectionIncomingAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pConnectionIncomingAction::Init { opts, .. } => {
                let peer_id = opts.peer_id;
                store.service().incoming_init(peer_id, opts.offer);
                store.dispatch(P2pConnectionIncomingAction::AnswerSdpCreatePending { peer_id });
            }
            P2pConnectionIncomingAction::AnswerSdpCreateError { peer_id, error } => {
                store.dispatch(P2pConnectionIncomingAction::Error {
                    peer_id,
                    error: P2pConnectionIncomingError::SdpCreateError(error),
                });
            }
            P2pConnectionIncomingAction::AnswerSdpCreateSuccess { peer_id, sdp } => {
                let answer = webrtc::Answer {
                    sdp,
                    identity_pub_key: store.state().config.identity_pub_key.clone(),
                    target_peer_id: peer_id,
                };
                store.dispatch(P2pConnectionIncomingAction::AnswerReady { peer_id, answer });
            }
            P2pConnectionIncomingAction::AnswerReady { peer_id, answer } => {
                store.service().set_answer(peer_id, answer);
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
            P2pConnectionIncomingAction::Libp2pReceived { peer_id } => {
                if let Err(err) = store.state().libp2p_incoming_accept(peer_id) {
                    store.dispatch(P2pDisconnectionAction::Init {
                        peer_id,
                        reason: P2pDisconnectionReason::Libp2pIncomingRejected(err),
                    });
                } else {
                    store.dispatch(P2pPeerAction::Ready {
                        peer_id,
                        incoming: true,
                    });
                }
            }
            P2pConnectionIncomingAction::AnswerSdpCreatePending { .. } => {}
            P2pConnectionIncomingAction::FinalizePending { .. } => {}
            P2pConnectionIncomingAction::Error { .. } => {}
        }
    }
}
