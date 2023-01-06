use super::{
    P2pRpcOutgoingAction, P2pRpcOutgoingActionWithMetaRef, P2pRpcOutgoingState,
    P2pRpcOutgoingStats, P2pRpcOutgoingStatus,
};

impl P2pRpcOutgoingState {
    pub fn reducer(&mut self, action: P2pRpcOutgoingActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pRpcOutgoingAction::Init(action) => {
                self.add(P2pRpcOutgoingStatus::Init {
                    time: meta.time(),
                    request: action.request.clone(),
                    requestor: action.requestor.clone(),
                });
                let stats = self.stats.entry(action.request.kind()).or_insert_with(|| {
                    P2pRpcOutgoingStats {
                        last_requested: meta.time(),
                    }
                });
                stats.last_requested = meta.time();
            }
            P2pRpcOutgoingAction::Pending(action) => {
                if let Some(req) = self.get_mut(action.rpc_id) {
                    *req = match req {
                        P2pRpcOutgoingStatus::Init {
                            request, requestor, ..
                        } => P2pRpcOutgoingStatus::Pending {
                            time: meta.time(),
                            request: std::mem::take(request),
                            requestor: std::mem::take(requestor),
                        },
                        _ => return,
                    };
                }
            }
            P2pRpcOutgoingAction::Received(action) => {
                if let Some(req) = self.get_mut(action.rpc_id) {
                    *req = match req {
                        P2pRpcOutgoingStatus::Pending {
                            request, requestor, ..
                        } => P2pRpcOutgoingStatus::Received {
                            time: meta.time(),
                            request: std::mem::take(request),
                            requestor: std::mem::take(requestor),
                            response: action.response.clone(),
                        },
                        _ => return,
                    };
                }
            }
            P2pRpcOutgoingAction::Error(action) => {
                if let Some(req) = self.get_mut(action.rpc_id) {
                    *req = match req {
                        P2pRpcOutgoingStatus::Pending {
                            request, requestor, ..
                        } => P2pRpcOutgoingStatus::Error {
                            time: meta.time(),
                            request: std::mem::take(request),
                            requestor: std::mem::take(requestor),
                            response: None,
                            error: action.error.clone(),
                        },
                        P2pRpcOutgoingStatus::Received {
                            request,
                            requestor,
                            response,
                            ..
                        } => P2pRpcOutgoingStatus::Error {
                            time: meta.time(),
                            request: std::mem::take(request),
                            requestor: std::mem::take(requestor),
                            response: Some(std::mem::take(response)),
                            error: action.error.clone(),
                        },
                        _ => return,
                    };
                }
            }
            P2pRpcOutgoingAction::Success(action) => {
                if let Some(req) = self.get_mut(action.rpc_id) {
                    *req = match req {
                        P2pRpcOutgoingStatus::Received {
                            request,
                            requestor,
                            response,
                            ..
                        } => P2pRpcOutgoingStatus::Success {
                            time: meta.time(),
                            request: std::mem::take(request),
                            requestor: std::mem::take(requestor),
                            response: std::mem::take(response),
                        },
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
