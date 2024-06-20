use super::{
    P2pConnectionIncomingAction, P2pConnectionIncomingActionWithMetaRef, P2pConnectionIncomingState,
};

impl P2pConnectionIncomingState {
    pub fn reducer(&mut self, action: P2pConnectionIncomingActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionIncomingAction::Init { opts, rpc_id } => {
                *self = Self::Init {
                    time: meta.time(),
                    signaling: opts.signaling.clone(),
                    offer: opts.offer.clone(),
                    rpc_id: *rpc_id,
                };
            }
            P2pConnectionIncomingAction::AnswerSdpCreatePending { .. } => {
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
            P2pConnectionIncomingAction::AnswerSdpCreateError { .. } => {}
            P2pConnectionIncomingAction::AnswerSdpCreateSuccess { sdp, .. } => {
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
                        sdp: sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionIncomingAction::AnswerReady { answer, .. } => {
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
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                }
            }
            P2pConnectionIncomingAction::AnswerSendSuccess { .. } => {
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
            P2pConnectionIncomingAction::FinalizePending { .. } => {
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
            P2pConnectionIncomingAction::FinalizeError { .. } => {}
            P2pConnectionIncomingAction::FinalizeSuccess { .. } => {
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
            P2pConnectionIncomingAction::Timeout { .. } => {}
            P2pConnectionIncomingAction::Error { error, .. } => {
                let rpc_id = self.rpc_id();
                *self = Self::Error {
                    time: meta.time(),
                    error: error.clone(),
                    rpc_id,
                };
            }
            P2pConnectionIncomingAction::Success { .. } => {
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
            P2pConnectionIncomingAction::FinalizePendingLibp2p { .. } => {
                // handled in the parent reducer.
            }
            P2pConnectionIncomingAction::Libp2pReceived { .. } =>
            {
                #[cfg(feature = "p2p-libp2p")]
                if let Self::FinalizePendingLibp2p { time, .. } = self {
                    *self = Self::Libp2pReceived { time: *time };
                }
            }
        }
    }
}
