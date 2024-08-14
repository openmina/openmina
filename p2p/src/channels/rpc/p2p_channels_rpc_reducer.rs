use std::collections::VecDeque;

use super::{
    P2pChannelsRpcAction, P2pChannelsRpcActionWithMetaRef, P2pChannelsRpcState, P2pRpcId,
    P2pRpcLocalState, P2pRpcRemotePendingRequestState, P2pRpcRemoteState,
    MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS,
};

impl P2pChannelsRpcState {
    pub fn reducer(
        &mut self,
        action: P2pChannelsRpcActionWithMetaRef<'_>,
        next_local_rpc_id: &mut P2pRpcId,
    ) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsRpcAction::Init { .. } => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsRpcAction::Pending { .. } => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsRpcAction::Ready { .. } => {
                *self = Self::Ready {
                    time: meta.time(),
                    local: P2pRpcLocalState::WaitingForRequest { time: meta.time() },
                    remote: P2pRpcRemoteState {
                        pending_requests: VecDeque::with_capacity(
                            MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS,
                        ),
                        last_responded: redux::Timestamp::ZERO,
                    },
                };
            }
            P2pChannelsRpcAction::RequestSend { id, request, .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                *next_local_rpc_id += 1;
                *local = P2pRpcLocalState::Requested {
                    time: meta.time(),
                    id: *id,
                    request: request.clone(),
                };
            }
            P2pChannelsRpcAction::Timeout { .. } => {}
            P2pChannelsRpcAction::ResponseReceived { .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                let P2pRpcLocalState::Requested { id, request, .. } = local else {
                    return;
                };
                *local = P2pRpcLocalState::Responded {
                    time: meta.time(),
                    id: *id,
                    request: std::mem::take(request),
                };
            }
            P2pChannelsRpcAction::RequestReceived { id, request, .. } => {
                let Self::Ready { remote, .. } = self else {
                    return;
                };
                remote
                    .pending_requests
                    .push_back(P2pRpcRemotePendingRequestState {
                        time: meta.time(),
                        id: *id,
                        request: (**request).clone(),
                        is_pending: false,
                    });
            }
            P2pChannelsRpcAction::ResponsePending { id, .. } => {
                let Self::Ready { remote, .. } = self else {
                    return;
                };
                if let Some(req) = remote.pending_requests.iter_mut().find(|r| r.id == *id) {
                    req.is_pending = true;
                }
            }
            P2pChannelsRpcAction::ResponseSend { id, .. } => {
                let Self::Ready { remote, .. } = self else {
                    return;
                };
                if let Some(pos) = remote.pending_requests.iter().position(|r| r.id == *id) {
                    remote.pending_requests.remove(pos);
                    remote.last_responded = meta.time();
                }
            }
        }
    }
}
