use super::{
    RpcAction, RpcActionWithMetaRef, RpcRequest, RpcRequestExtraData, RpcRequestState,
    RpcRequestStatus, RpcState,
};

impl RpcState {
    pub fn reducer(&mut self, action: RpcActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            RpcAction::GlobalStateGet { .. } => {}
            RpcAction::ActionStatsGet { .. } => {}
            RpcAction::SyncStatsGet { .. } => {}
            RpcAction::BlockProducerStatsGet { .. } => {}
            RpcAction::MessageProgressGet { .. } => {}
            RpcAction::PeersGet { .. } => {}
            RpcAction::P2pConnectionOutgoingInit { rpc_id, opts } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::P2pConnectionOutgoing(opts.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                self.requests.insert(*rpc_id, rpc_state);
            }
            RpcAction::P2pConnectionOutgoingPending { rpc_id } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::P2pConnectionOutgoingError { rpc_id, error } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Error {
                    time: meta.time(),
                    error: format!("{:?}", error),
                };
            }
            RpcAction::P2pConnectionOutgoingSuccess { rpc_id } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
            }
            RpcAction::P2pConnectionIncomingInit { rpc_id, opts } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::P2pConnectionIncoming(opts.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                self.requests.insert(*rpc_id, rpc_state);
            }
            RpcAction::P2pConnectionIncomingPending { rpc_id } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::P2pConnectionIncomingRespond { .. } => {}
            RpcAction::P2pConnectionIncomingError { rpc_id, error } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Error {
                    time: meta.time(),
                    error: format!("{:?}", error),
                };
            }
            RpcAction::P2pConnectionIncomingSuccess { rpc_id } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
            }
            RpcAction::ScanStateSummaryGetInit { rpc_id, query } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::ScanStateSummaryGet(query.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                self.requests.insert(*rpc_id, rpc_state);
            }
            RpcAction::ScanStateSummaryLedgerGetInit { .. } => {}
            RpcAction::ScanStateSummaryGetPending { rpc_id, block } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
                rpc.data = RpcRequestExtraData::FullBlockOpt(block.clone());
            }
            RpcAction::ScanStateSummaryGetSuccess { rpc_id, .. } => {
                let Some(rpc) = self.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
            }
            RpcAction::SnarkPoolAvailableJobsGet { .. } => {}
            RpcAction::SnarkPoolJobGet { .. } => {}
            RpcAction::SnarkerConfigGet { .. } => {}
            RpcAction::SnarkerJobCommit { .. } => {}
            RpcAction::SnarkerJobSpec { .. } => {}
            RpcAction::SnarkerWorkersGet { .. } => {}
            RpcAction::HealthCheck { .. } => {}
            RpcAction::ReadinessCheck { .. } => {}
            RpcAction::DiscoveryRoutingTable { .. } => {}
            RpcAction::DiscoveryBoostrapStats { .. } => {}
            RpcAction::Finish { rpc_id } => {
                self.requests.remove(rpc_id);
            }
        }
    }
}
