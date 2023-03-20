use super::{
    P2pConnectionOutgoingAction, P2pConnectionOutgoingActionWithMetaRef, P2pConnectionOutgoingState,
};

impl P2pConnectionOutgoingState {
    pub fn reducer(&mut self, action: P2pConnectionOutgoingActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionOutgoingAction::RandomInit(_) => {}
            P2pConnectionOutgoingAction::Init(content) => {
                *self = Self::Init {
                    time: meta.time(),
                    opts: content.opts.clone(),
                    rpc_id: content.rpc_id,
                };
            }
            P2pConnectionOutgoingAction::Reconnect(action) => {
                *self = Self::Init {
                    time: meta.time(),
                    opts: action.opts.clone(),
                    rpc_id: action.rpc_id,
                };
            }
            P2pConnectionOutgoingAction::OfferSdpCreatePending(_) => {
                if let Self::Init { opts, rpc_id, .. } = self {
                    *self = Self::OfferSdpCreatePending {
                        time: meta.time(),
                        opts: opts.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::OfferSdpCreateSuccess(action) => {
                if let Self::OfferSdpCreatePending { opts, rpc_id, .. } = self {
                    *self = Self::OfferSdpCreateSuccess {
                        time: meta.time(),
                        opts: opts.clone(),
                        sdp: action.sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::OfferReady(action) => {
                if let Self::OfferSdpCreateSuccess { opts, rpc_id, .. } = self {
                    *self = Self::OfferReady {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: action.offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::OfferSendSuccess(_) => {
                if let Self::OfferReady {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::OfferSendSuccess {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::AnswerRecvPending(_) => {
                if let Self::OfferSendSuccess {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::AnswerRecvPending {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::AnswerRecvError(action) => {
                if let Self::AnswerRecvPending {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::AnswerRecvError {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        error: action.error.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::AnswerRecvSuccess(action) => {
                if let Self::AnswerRecvPending {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::AnswerRecvSuccess {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        answer: action.answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::FinalizePending(_) => {
                if let Self::AnswerRecvSuccess {
                    opts,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::FinalizePending {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::FinalizeError(action) => {
                if let Self::FinalizePending {
                    opts,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::FinalizeError {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        error: action.error.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::FinalizeSuccess(_) => {
                if let Self::FinalizePending {
                    opts,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = self
                {
                    *self = Self::FinalizeSuccess {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::Error(action) => {
                let rpc_id = self.rpc_id();
                *self = Self::Error {
                    time: meta.time(),
                    error: action.error.clone(),
                    rpc_id,
                };
            }
            P2pConnectionOutgoingAction::Success(_) => {
                let rpc_id = self.rpc_id();
                *self = Self::Success {
                    time: meta.time(),
                    rpc_id,
                };
            }
        }
    }
}
