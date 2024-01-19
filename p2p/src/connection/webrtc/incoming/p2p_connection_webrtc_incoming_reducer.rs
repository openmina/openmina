use redux::ActionWithMeta;

use super::*;

impl P2pConnectionWebRTCIncomingState {
    pub fn reducer(&mut self, action: ActionWithMeta<&'_ P2pConnectionWebRTCIncomingAction>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionWebRTCIncomingAction::Init(content) => {
                *self = Self::Init {
                    time: meta.time(),
                    signaling: content.opts.signaling.clone(),
                    offer: content.opts.offer.clone(),
                    rpc_id: content.rpc_id,
                };
            }
            P2pConnectionWebRTCIncomingAction::AnswerSdpCreatePending(_) => {
                if let Self::Init {
                    signaling,
                    offer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::AnswerSdpCreatePending {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCIncomingAction::AnswerSdpCreateError(_) => {}
            P2pConnectionWebRTCIncomingAction::AnswerSdpCreateSuccess(action) => {
                if let Self::AnswerSdpCreatePending {
                    signaling,
                    offer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::AnswerSdpCreateSuccess {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        sdp: action.sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCIncomingAction::AnswerReady(action) => {
                if let Self::AnswerSdpCreateSuccess {
                    signaling,
                    offer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::AnswerReady {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: action.answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCIncomingAction::AnswerSendSuccess(_) => {
                if let Self::AnswerReady {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::AnswerSendSuccess {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCIncomingAction::FinalizePending(_) => {
                if let Self::AnswerSendSuccess {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::FinalizePending {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCIncomingAction::FinalizeError(_) => {}
            P2pConnectionWebRTCIncomingAction::FinalizeSuccess(_) => {
                if let Self::FinalizePending {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::FinalizeSuccess {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCIncomingAction::Timeout(_) => {}
            P2pConnectionWebRTCIncomingAction::Error(action) => {
                let rpc_id = self.rpc_id();
                *self = Self::Error {
                    time: meta.time(),
                    error: action.error.clone(),
                    rpc_id,
                };
            }
            P2pConnectionWebRTCIncomingAction::Success(_) => {
                if let Self::FinalizeSuccess {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::Success {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionWebRTCIncomingAction::Libp2pReceived(_) => {
                // handled in the parent reducer.
            }
        }
    }
}
