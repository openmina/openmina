use redux::ActionWithMeta;

use super::*;

impl P2pConnectionWebRTCOutgoingState {
    pub fn reducer(&mut self, action: ActionWithMeta<&'_ P2pConnectionWebRTCOutgoingAction>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionWebRTCOutgoingAction::Init(content) => {
                *self = Self::Init {
                    time: meta.time(),
                    rpc_id: content.rpc_id,
                };
            }
            P2pConnectionWebRTCOutgoingAction::OfferSdpCreatePending(_) => {
                if let Self::Init { rpc_id, .. } = self {
                    *self = Self::OfferSdpCreatePending {
                        time: meta.time(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCOutgoingAction::OfferSdpCreateError(_) => {}
            P2pConnectionWebRTCOutgoingAction::OfferSdpCreateSuccess(action) => {
                if let Self::OfferSdpCreatePending { rpc_id, .. } = self {
                    *self = Self::OfferSdpCreateSuccess {
                        time: meta.time(),
                        sdp: action.sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCOutgoingAction::OfferReady(action) => {
                if let Self::OfferSdpCreateSuccess { rpc_id, .. } = self {
                    *self = Self::OfferReady {
                        time: meta.time(),
                        offer: action.offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCOutgoingAction::OfferSendSuccess(_) => {
                if let Self::OfferReady { offer, rpc_id, .. } = self {
                    *self = Self::OfferSendSuccess {
                        time: meta.time(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCOutgoingAction::AnswerRecvPending(_) => {
                if let Self::OfferSendSuccess { offer, rpc_id, .. } = self {
                    *self = Self::AnswerRecvPending {
                        time: meta.time(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCOutgoingAction::AnswerRecvError(_) => {}
            P2pConnectionWebRTCOutgoingAction::AnswerRecvSuccess(action) => {
                if let Self::AnswerRecvPending { offer, rpc_id, .. } = self {
                    *self = Self::AnswerRecvSuccess {
                        time: meta.time(),
                        offer: offer.clone(),
                        answer: action.answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCOutgoingAction::FinalizePending(_) => match self {
                Self::Init { rpc_id, .. } => {
                    *self = Self::FinalizePending {
                        time: meta.time(),
                        offer: None,
                        answer: None,
                        rpc_id: rpc_id.take(),
                    };
                }
                Self::AnswerRecvSuccess {
                    offer,
                    answer,
                    rpc_id,
                    ..
                } => {
                    *self = Self::FinalizePending {
                        time: meta.time(),
                        offer: Some(offer.clone()),
                        answer: Some(answer.clone()),
                        rpc_id: rpc_id.take(),
                    };
                }
                _ => {}
            },
            P2pConnectionWebRTCOutgoingAction::FinalizeError(_) => {}
            P2pConnectionWebRTCOutgoingAction::FinalizeSuccess(_) => {
                if let Self::FinalizePending {
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::FinalizeSuccess {
                        time: meta.time(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCOutgoingAction::Timeout(_) => {}
            P2pConnectionWebRTCOutgoingAction::Error(action) => {
                let rpc_id = self.rpc_id();
                *self = Self::Error {
                    time: meta.time(),
                    error: action.error.clone(),
                    rpc_id,
                };
            }
            P2pConnectionWebRTCOutgoingAction::Success(_) => {
                if let Self::FinalizeSuccess {
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::Success {
                        time: meta.time(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
        }
    }
}
