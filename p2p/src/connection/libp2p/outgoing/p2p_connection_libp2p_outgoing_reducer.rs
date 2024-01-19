use openmina_core::error;
use redux::ActionWithMeta;

use super::*;

impl P2pConnectionLibP2pOutgoingState {
    pub fn reducer(&mut self, action: ActionWithMeta<&'_ P2pConnectionLibP2pOutgoingAction>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionLibP2pOutgoingAction::Init(action) => {
                *self = Self::Init(P2pConnectionLibP2pOutgoingInitState {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                });
            }
            P2pConnectionLibP2pOutgoingAction::FinalizePending(action) => {
                let Self::Init(P2pConnectionLibP2pOutgoingInitState { rpc_id, .. }) = self else {
                    error!(meta.time(); "incorrect state: {self:?}");
                    return;
                };
                *self = Self::FinalizePending(P2pConnectionLibP2pOutgoingFinalizePendingState {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                });
            }
            P2pConnectionLibP2pOutgoingAction::FinalizeSuccess(action) => {
                let Self::FinalizePending(P2pConnectionLibP2pOutgoingFinalizePendingState {
                    rpc_id,
                    ..
                }) = self
                else {
                    error!(meta.time(); "incorrect state: {self:?}");
                    return;
                };
                *self = Self::Success(P2pConnectionLibP2pOutgoingSuccessState {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                });
            }
            P2pConnectionLibP2pOutgoingAction::FinalizeError(action) => {
                let Self::FinalizePending(P2pConnectionLibP2pOutgoingFinalizePendingState {
                    rpc_id,
                    ..
                }) = self
                else {
                    error!(meta.time(); "incorrect state: {self:?}");
                    return;
                };
                *self = Self::Error(P2pConnectionLibP2pOutgoingErrorState {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: P2pConnectionLibP2pOutgoingError::FinalizeError(action.error.clone()),
                });
            }
            P2pConnectionLibP2pOutgoingAction::FinalizeTimeout(action) => {
                let Self::FinalizePending(P2pConnectionLibP2pOutgoingFinalizePendingState {
                    rpc_id,
                    ..
                }) = self
                else {
                    error!(meta.time(); "incorrect state: {self:?}");
                    return;
                };
                *self = Self::Error(P2pConnectionLibP2pOutgoingErrorState {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: P2pConnectionLibP2pOutgoingError::Timeout,
                });
            }
            P2pConnectionLibP2pOutgoingAction::Success(_) => {
                // noop
            }
            P2pConnectionLibP2pOutgoingAction::Error(_) => {
                // noop
            }
        }
    }
}
