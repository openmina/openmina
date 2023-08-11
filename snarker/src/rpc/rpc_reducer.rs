use super::{
    RpcAction, RpcActionWithMetaRef, RpcRequest, RpcRequestState, RpcRequestStatus, RpcState,
};

impl RpcState {
    pub fn reducer(&mut self, action: RpcActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            RpcAction::GlobalStateGet(_) => {}
            RpcAction::ActionStatsGet(_) => {}
            RpcAction::SyncStatsGet(_) => {}
            RpcAction::P2pConnectionOutgoingInit(content) => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::P2pConnectionOutgoing(content.opts.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                };
                self.requests.insert(content.rpc_id, rpc_state);
            }
            RpcAction::P2pConnectionOutgoingPending(content) => {
                let Some(rpc) = self.requests.get_mut(&content.rpc_id) else { return };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::P2pConnectionOutgoingError(content) => {
                let Some(rpc) = self.requests.get_mut(&content.rpc_id) else { return };
                rpc.status = RpcRequestStatus::Error {
                    time: meta.time(),
                    error: format!("{:?}", content.error),
                };
            }
            RpcAction::P2pConnectionOutgoingSuccess(content) => {
                let Some(rpc) = self.requests.get_mut(&content.rpc_id) else { return };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
            }
            RpcAction::P2pConnectionIncomingInit(content) => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::P2pConnectionIncoming(content.opts.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                };
                self.requests.insert(content.rpc_id, rpc_state);
            }
            RpcAction::P2pConnectionIncomingPending(content) => {
                let Some(rpc) = self.requests.get_mut(&content.rpc_id) else { return };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::P2pConnectionIncomingRespond(_) => {}
            RpcAction::P2pConnectionIncomingError(content) => {
                let Some(rpc) = self.requests.get_mut(&content.rpc_id) else { return };
                rpc.status = RpcRequestStatus::Error {
                    time: meta.time(),
                    error: format!("{:?}", content.error),
                };
            }
            RpcAction::P2pConnectionIncomingSuccess(content) => {
                let Some(rpc) = self.requests.get_mut(&content.rpc_id) else { return };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
            }
            RpcAction::ScanStateSummaryGet(_) => {}
            RpcAction::SnarkPoolAvailableJobsGet(_) => {}
            RpcAction::SnarkPoolJobGet(_) => {}
            RpcAction::SnarkerConfigGet(_) => {}
            RpcAction::SnarkerJobCommit(_) => {}
            RpcAction::SnarkerJobSpec(_) => {}
            RpcAction::SnarkerWorkersGet(_) => {}
            RpcAction::Finish(action) => {
                self.requests.remove(&action.rpc_id);
            }
        }
    }
}
