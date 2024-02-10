use super::{
    P2pConnectionOutgoingAction, P2pConnectionOutgoingActionWithMetaRef, P2pConnectionOutgoingState,
};

impl P2pConnectionOutgoingState {
    pub fn reducer(&mut self, action: P2pConnectionOutgoingActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionOutgoingAction::RandomInit => {}
            P2pConnectionOutgoingAction::Init { opts, rpc_id } => {
                *self = Self::Init {
                    time: meta.time(),
                    opts: opts.clone(),
                    rpc_id: *rpc_id,
                };
            }
            P2pConnectionOutgoingAction::Reconnect { opts, rpc_id } => {
                *self = Self::Init {
                    time: meta.time(),
                    opts: opts.clone(),
                    rpc_id: *rpc_id,
                };
            }
            P2pConnectionOutgoingAction::OfferSdpCreatePending { .. } => {
                if let Self::Init { opts, rpc_id, .. } = self {
                    *self = Self::OfferSdpCreatePending {
                        time: meta.time(),
                        opts: opts.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::OfferSdpCreateError { .. } => {}
            P2pConnectionOutgoingAction::OfferSdpCreateSuccess { sdp, .. } => {
                if let Self::OfferSdpCreatePending { opts, rpc_id, .. } = self {
                    *self = Self::OfferSdpCreateSuccess {
                        time: meta.time(),
                        opts: opts.clone(),
                        sdp: sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::OfferReady { offer, .. } => {
                if let Self::OfferSdpCreateSuccess { opts, rpc_id, .. } = self {
                    *self = Self::OfferReady {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::OfferSendSuccess { .. } => {
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
            P2pConnectionOutgoingAction::AnswerRecvPending { .. } => {
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
            P2pConnectionOutgoingAction::AnswerRecvError { .. } => {}
            P2pConnectionOutgoingAction::AnswerRecvSuccess { answer, .. } => {
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
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionOutgoingAction::FinalizePending { .. } => match self {
                Self::Init { opts, rpc_id, .. } => {
                    *self = Self::FinalizePending {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: None,
                        answer: None,
                        rpc_id: rpc_id.take(),
                    };
                }
                Self::AnswerRecvSuccess {
                    opts,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } => {
                    *self = Self::FinalizePending {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: Some(offer.clone()),
                        answer: Some(answer.clone()),
                        rpc_id: rpc_id.take(),
                    };
                }
                _ => {}
            },
            P2pConnectionOutgoingAction::FinalizeError { .. } => {}
            P2pConnectionOutgoingAction::FinalizeSuccess { .. } => {
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
            P2pConnectionOutgoingAction::Timeout { .. } => {}
            P2pConnectionOutgoingAction::Error { error, .. } => {
                let rpc_id = self.rpc_id();
                *self = Self::Error {
                    time: meta.time(),
                    error: error.clone(),
                    rpc_id,
                };
            }
            P2pConnectionOutgoingAction::Success { .. } => {
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
