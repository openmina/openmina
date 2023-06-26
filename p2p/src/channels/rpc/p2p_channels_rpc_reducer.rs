use super::{
    P2pChannelsRpcAction, P2pChannelsRpcActionWithMetaRef, P2pChannelsRpcState, P2pRpcLocalState,
    P2pRpcRemotePendingRequestState, P2pRpcRemoteState, MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS,
};

impl P2pChannelsRpcState {
    pub fn reducer(&mut self, action: P2pChannelsRpcActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsRpcAction::Init(_) => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsRpcAction::Pending(_) => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsRpcAction::Ready(_) => {
                *self = Self::Ready {
                    time: meta.time(),
                    local: P2pRpcLocalState::WaitingForRequest { time: meta.time() },
                    remote: P2pRpcRemoteState {
                        pending_requests: Vec::with_capacity(
                            MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS,
                        ),
                    },
                    next_local_rpc_id: 0,
                };
            }
            P2pChannelsRpcAction::RequestSend(action) => {
                let Self::Ready { local, next_local_rpc_id, .. } = self else { return };
                *next_local_rpc_id += 1;
                *local = P2pRpcLocalState::Requested {
                    time: meta.time(),
                    id: action.id,
                    request: action.request.clone(),
                };
            }
            P2pChannelsRpcAction::Timeout(_) => {}
            P2pChannelsRpcAction::ResponseReceived(_) => {
                let Self::Ready { local, .. } = self else { return };

                *local = P2pRpcLocalState::Responded { time: meta.time() };
            }
            P2pChannelsRpcAction::RequestReceived(action) => {
                let Self::Ready { remote, .. } = self else { return };

                remote
                    .pending_requests
                    .push(P2pRpcRemotePendingRequestState {
                        time: meta.time(),
                        id: action.id,
                        request: action.request.clone(),
                    });
            }
            P2pChannelsRpcAction::ResponseSend(action) => {
                let Self::Ready { remote, .. } = self else { return };
                let Some(i) = remote.pending_requests.iter().position(|v| v.id == action.id) else { return };
                remote.pending_requests.remove(i);
            }
        }
    }
}
