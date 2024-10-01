use std::collections::VecDeque;

use openmina_core::{block::BlockWithHash, bug_condition, error, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::rpc_effectful::P2pChannelsRpcEffectfulAction, P2pNetworkRpcAction, P2pPeerAction,
    P2pState,
};

use super::{
    P2pChannelsRpcAction, P2pChannelsRpcState, P2pRpcLocalState, P2pRpcRemotePendingRequestState,
    P2pRpcRemoteState, P2pRpcResponse, MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS,
};

impl P2pChannelsRpcState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsRpcAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;
        let peer_id = *action.peer_id();
        let is_libp2p = p2p_state.is_libp2p_peer(&peer_id);
        let peer_state = &mut p2p_state
            .get_ready_peer_mut(&peer_id)
            .ok_or_else(|| format!("Peer state not found for: {action:?}"))?
            .channels;

        let next_local_rpc_id = &mut peer_state.next_local_rpc_id;
        let rpc_state = &mut peer_state.rpc;

        match action {
            P2pChannelsRpcAction::Init { .. } => {
                *rpc_state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsRpcEffectfulAction::Init { peer_id });
                Ok(())
            }
            P2pChannelsRpcAction::Pending { .. } => {
                *rpc_state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsRpcAction::Ready { .. } => {
                *rpc_state = Self::Ready {
                    time: meta.time(),
                    local: P2pRpcLocalState::WaitingForRequest { time: meta.time() },
                    remote: P2pRpcRemoteState {
                        pending_requests: VecDeque::with_capacity(
                            MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS,
                        ),
                        last_responded: redux::Timestamp::ZERO,
                    },
                };
                Ok(())
            }
            P2pChannelsRpcAction::RequestSend {
                id,
                request,
                on_init,
                ..
            } => {
                let Self::Ready { local, .. } = rpc_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsRpcAction::RequestSend`, state: {:?}",
                        rpc_state
                    );
                    return Ok(());
                };
                *next_local_rpc_id += 1;
                *local = P2pRpcLocalState::Requested {
                    time: meta.time(),
                    id: *id,
                    request: request.clone(),
                };

                let dispatcher = state_context.into_dispatcher();

                #[cfg(feature = "p2p-libp2p")]
                if is_libp2p {
                    if let Some((query, data)) =
                        super::libp2p::internal_request_into_libp2p(*request.clone(), *id)
                    {
                        dispatcher.push(P2pNetworkRpcAction::OutgoingQuery {
                            peer_id,
                            query,
                            data,
                        });
                    }
                    if let Some(on_init) = on_init {
                        dispatcher.push_callback(on_init.clone(), (peer_id, *id, *request.clone()));
                    }

                    return Ok(());
                }

                dispatcher.push(P2pChannelsRpcEffectfulAction::RequestSend {
                    peer_id,
                    id: *id,
                    request: request.clone(),
                    on_init: on_init.clone(),
                });
                Ok(())
            }
            P2pChannelsRpcAction::Timeout { .. } => Ok(()),
            P2pChannelsRpcAction::ResponseReceived { response, .. } => {
                let Self::Ready { local, .. } = rpc_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsRpcAction::ResponseReceived`, state: {:?}",
                        rpc_state
                    );
                    return Ok(());
                };
                let P2pRpcLocalState::Requested { id, request, .. } = local else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsRpcAction::ResponseReceived`, state: {:?}",
                        rpc_state
                    );
                    return Ok(());
                };
                *local = P2pRpcLocalState::Responded {
                    time: meta.time(),
                    id: *id,
                    request: std::mem::take(request),
                };

                let dispatcher = state_context.into_dispatcher();
                if let Some(P2pRpcResponse::BestTipWithProof(resp)) = response.as_deref() {
                    let Ok(best_tip) = BlockWithHash::try_new(resp.best_tip.clone()) else {
                        error!(meta.time(); "P2pChannelsRpcAction::ResponseReceived: Invalid bigint in block");
                        return Ok(());
                    };

                    dispatcher.push(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
                }
                Ok(())
            }
            P2pChannelsRpcAction::RequestReceived { id, request, .. } => {
                let Self::Ready { remote, .. } = rpc_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsRpcAction::RequestReceived`, state: {:?}",
                        rpc_state
                    );
                    return Ok(());
                };
                remote
                    .pending_requests
                    .push_back(P2pRpcRemotePendingRequestState {
                        time: meta.time(),
                        id: *id,
                        request: (**request).clone(),
                        is_pending: false,
                    });
                Ok(())
            }
            P2pChannelsRpcAction::ResponsePending { id, .. } => {
                let Self::Ready { remote, .. } = rpc_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsRpcAction::ResponsePending`, state: {:?}",
                        rpc_state
                    );
                    return Ok(());
                };
                if let Some(req) = remote.pending_requests.iter_mut().find(|r| r.id == *id) {
                    req.is_pending = true;
                }
                Ok(())
            }
            P2pChannelsRpcAction::ResponseSend { id, response, .. } => {
                let Self::Ready { remote, .. } = rpc_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsRpcAction::ResponseSend`, state: {:?}",
                        rpc_state
                    );
                    return Ok(());
                };

                if let Some(pos) = remote.pending_requests.iter().position(|r| r.id == *id) {
                    remote.pending_requests.remove(pos);
                    remote.last_responded = meta.time();
                }

                let dispatcher = state_context.into_dispatcher();

                #[cfg(feature = "p2p-libp2p")]
                if is_libp2p {
                    if let Some(response) = response {
                        if let Some((response, data)) =
                            super::libp2p::internal_response_into_libp2p(*response.clone(), *id)
                        {
                            dispatcher.push(P2pNetworkRpcAction::OutgoingResponse {
                                peer_id,
                                response,
                                data,
                            });
                        }
                    }

                    return Ok(());
                }

                dispatcher.push(P2pChannelsRpcEffectfulAction::ResponseSend {
                    peer_id,
                    id: *id,
                    response: response.clone(),
                });
                Ok(())
            }
        }
    }
}
