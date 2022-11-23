use super::{
    P2pRpcOutgoingAction, P2pRpcOutgoingActionWithMetaRef, P2pRpcOutgoingState,
    P2pRpcOutgoingStatus,
};

impl P2pRpcOutgoingState {
    pub fn reducer(&mut self, action: P2pRpcOutgoingActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pRpcOutgoingAction::Init(action) => {
                self.add(P2pRpcOutgoingStatus::Init {
                    time: meta.time(),
                    request: action.request.clone(),
                });
            }
            P2pRpcOutgoingAction::Pending(action) => {
                if let Some(req) = self.get_mut(action.rpc_id) {
                    *req = match req {
                        P2pRpcOutgoingStatus::Init { request, .. } => {
                            P2pRpcOutgoingStatus::Pending {
                                time: meta.time(),
                                request: std::mem::take(request),
                            }
                        }
                        _ => return,
                    };
                }
            }
            P2pRpcOutgoingAction::Error(action) => {
                if let Some(req) = self.get_mut(action.rpc_id) {
                    *req = match req {
                        P2pRpcOutgoingStatus::Pending { request, .. } => {
                            P2pRpcOutgoingStatus::Error {
                                time: meta.time(),
                                request: std::mem::take(request),
                                error: action.error.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            P2pRpcOutgoingAction::Success(action) => {
                if let Some(req) = self.get_mut(action.rpc_id) {
                    *req = match req {
                        P2pRpcOutgoingStatus::Pending { request, .. } => {
                            P2pRpcOutgoingStatus::Success {
                                time: meta.time(),
                                request: std::mem::take(request),
                                response: action.response.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            P2pRpcOutgoingAction::Finish(action) => {
                self.remove(action.rpc_id);
            }
        }
    }
}
