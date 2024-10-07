use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{channels::streaming_rpc_effectful::P2pChannelsStreamingRpcEffectfulAction, P2pState};

use super::{
    staged_ledger_parts::{StagedLedgerPartsReceiveProgress, StagedLedgerPartsSendProgress},
    P2pChannelsStreamingRpcAction, P2pChannelsStreamingRpcState, P2pStreamingRpcLocalState,
    P2pStreamingRpcRemoteState, P2pStreamingRpcRequest, P2pStreamingRpcResponseFull,
    P2pStreamingRpcSendProgress,
};

impl P2pChannelsStreamingRpcState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsStreamingRpcAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let peer_id = *action.peer_id();
        let p2p_state = state_context.get_substate_mut()?;

        let channels_state = &mut p2p_state
            .get_ready_peer_mut(&peer_id)
            .ok_or_else(|| format!("Invalid state for: {action:?}"))?
            .channels;

        let next_local_rpc_id = &mut channels_state.next_local_rpc_id;
        let streaming_rpc_state = &mut channels_state.streaming_rpc;

        match action {
            P2pChannelsStreamingRpcAction::Init { .. } => {
                *streaming_rpc_state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsStreamingRpcEffectfulAction::Init { peer_id });
                Ok(())
            }
            P2pChannelsStreamingRpcAction::Pending { .. } => {
                *streaming_rpc_state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsStreamingRpcAction::Ready { .. } => {
                *streaming_rpc_state = Self::Ready {
                    time: meta.time(),
                    local: P2pStreamingRpcLocalState::WaitingForRequest { time: meta.time() },
                    remote: P2pStreamingRpcRemoteState::WaitingForRequest { time: meta.time() },
                    remote_last_responded: redux::Timestamp::ZERO,
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(callback) = &p2p_state.callbacks.on_p2p_channels_streaming_rpc_ready {
                    dispatcher.push_callback(callback.clone(), ());
                }
                Ok(())
            }
            P2pChannelsStreamingRpcAction::RequestSend {
                id,
                request,
                on_init,
                ..
            } => {
                let Self::Ready { local, .. } = streaming_rpc_state else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };

                *next_local_rpc_id += 1;
                *local = P2pStreamingRpcLocalState::Requested {
                    time: meta.time(),
                    id: *id,
                    request: request.clone(),
                    progress: match &**request {
                        P2pStreamingRpcRequest::StagedLedgerParts(_) => {
                            Into::into(StagedLedgerPartsReceiveProgress::BasePending {
                                time: meta.time(),
                            })
                        }
                    },
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsStreamingRpcEffectfulAction::RequestSend {
                    peer_id,
                    id: *id,
                    request: request.clone(),
                    on_init: on_init.clone(),
                });
                Ok(())
            }
            P2pChannelsStreamingRpcAction::Timeout { id, .. } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(callback) = &p2p_state.callbacks.on_p2p_channels_streaming_rpc_timeout {
                    dispatcher.push_callback(callback.clone(), (peer_id, *id));
                }

                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponseNextPartGet { id, .. } => {
                let Self::Ready {
                    local: P2pStreamingRpcLocalState::Requested { progress, .. },
                    ..
                } = streaming_rpc_state
                else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };

                if !progress.set_next_pending(meta.time()) {
                    bug_condition!("progress state already pending: {progress:?}");
                }

                if !progress.is_part_pending() {
                    bug_condition!("progress state is not pending {:?}", progress);
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(
                    P2pChannelsStreamingRpcEffectfulAction::ResponseNextPartGet {
                        peer_id,
                        id: *id,
                    },
                );
                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponsePartReceived { response, id, .. } => {
                let Self::Ready {
                    local: P2pStreamingRpcLocalState::Requested { progress, .. },
                    ..
                } = streaming_rpc_state
                else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };
                if !progress.update(meta.time(), response) {
                    bug_condition!("progress response mismatch! {progress:?}\n{response:?}");
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &P2pState = state.substate()?;
                let Some(peer) = state.get_ready_peer(&peer_id) else {
                    return Ok(());
                };

                if let Some(response) = peer.channels.streaming_rpc.local_done_response() {
                    dispatcher.push(P2pChannelsStreamingRpcAction::ResponseReceived {
                        peer_id,
                        id: *id,
                        response: Some(response),
                    });
                    return Ok(());
                }
                dispatcher
                    .push(P2pChannelsStreamingRpcAction::ResponseNextPartGet { peer_id, id: *id });
                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponseReceived {
                id: rpc_id,
                response,
                ..
            } => {
                let Self::Ready { local, .. } = streaming_rpc_state else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };
                let P2pStreamingRpcLocalState::Requested { id, request, .. } = local else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };
                *local = P2pStreamingRpcLocalState::Responded {
                    time: meta.time(),
                    id: *id,
                    request: std::mem::take(request),
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(callback) = &p2p_state
                    .callbacks
                    .on_p2p_channels_streaming_rpc_response_received
                {
                    dispatcher.push_callback(callback.clone(), (peer_id, *rpc_id, response.clone()))
                }

                Ok(())
            }
            P2pChannelsStreamingRpcAction::RequestReceived { id, request, .. } => {
                let Self::Ready { remote, .. } = streaming_rpc_state else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };
                *remote = P2pStreamingRpcRemoteState::Requested {
                    time: meta.time(),
                    id: *id,
                    request: request.clone(),
                    progress: StagedLedgerPartsSendProgress::LedgerGetIdle { time: meta.time() }
                        .into(),
                };
                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponsePending { .. } => {
                let Self::Ready {
                    remote:
                        P2pStreamingRpcRemoteState::Requested {
                            request, progress, ..
                        },
                    ..
                } = streaming_rpc_state
                else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };
                match &**request {
                    P2pStreamingRpcRequest::StagedLedgerParts(_) => {
                        *progress =
                            StagedLedgerPartsSendProgress::LedgerGetPending { time: meta.time() }
                                .into();
                    }
                }
                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponseSendInit { response, id, .. } => {
                let Self::Ready {
                    remote:
                        P2pStreamingRpcRemoteState::Requested {
                            request, progress, ..
                        },
                    ..
                } = streaming_rpc_state
                else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };
                match (&**request, response) {
                    (_, Some(P2pStreamingRpcResponseFull::StagedLedgerParts(data))) => {
                        *progress = StagedLedgerPartsSendProgress::LedgerGetSuccess {
                            time: meta.time(),
                            data: Some(data.clone()),
                        }
                        .into();
                    }
                    (P2pStreamingRpcRequest::StagedLedgerParts(_), None) => {
                        *progress =
                            StagedLedgerPartsSendProgress::Success { time: meta.time() }.into();
                    } // _ => todo!("unexpected response send call: {response:?}"),
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsStreamingRpcEffectfulAction::ResponseSendInit {
                    peer_id,
                    id: *id,
                    response: response.clone(),
                });
                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponsePartNextSend { id, .. } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &P2pState = state.substate()?;

                let Some(response) = state
                    .get_ready_peer(&peer_id)
                    .and_then(|peer| peer.channels.streaming_rpc.remote_next_msg().map(Box::new))
                else {
                    return Ok(());
                };

                dispatcher.push(P2pChannelsStreamingRpcAction::ResponsePartSend {
                    peer_id,
                    id: *id,
                    response,
                });

                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponsePartSend { id, response, .. } => {
                let Self::Ready {
                    remote: P2pStreamingRpcRemoteState::Requested { progress, .. },
                    ..
                } = streaming_rpc_state
                else {
                    bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                    return Ok(());
                };
                match progress {
                    P2pStreamingRpcSendProgress::StagedLedgerParts(progress) => {
                        *progress = match progress {
                            StagedLedgerPartsSendProgress::LedgerGetSuccess {
                                data: Some(data),
                                ..
                            } => StagedLedgerPartsSendProgress::BaseSent {
                                time: meta.time(),
                                data: data.clone(),
                            },
                            StagedLedgerPartsSendProgress::BaseSent { data, .. } => {
                                StagedLedgerPartsSendProgress::ScanStateBaseSent {
                                    time: meta.time(),
                                    data: data.clone(),
                                }
                            }
                            StagedLedgerPartsSendProgress::ScanStateBaseSent { data, .. } => {
                                StagedLedgerPartsSendProgress::PreviousIncompleteZkappUpdatesSent {
                                    time: meta.time(),
                                    data: data.clone(),
                                }
                            }
                            StagedLedgerPartsSendProgress::PreviousIncompleteZkappUpdatesSent {
                                data,
                                ..
                            } => StagedLedgerPartsSendProgress::ScanStateTreesSending {
                                time: meta.time(),
                                data: data.clone(),
                                tree_index: 0,
                            },
                            StagedLedgerPartsSendProgress::ScanStateTreesSending {
                                data,
                                tree_index,
                                ..
                            } => StagedLedgerPartsSendProgress::ScanStateTreesSending {
                                time: meta.time(),
                                data: data.clone(),
                                tree_index: *tree_index + 1,
                            },
                            progress => {
                                bug_condition!("unexpected state during `P2pStreamingRpcSendProgress::StagedLedgerParts`: {progress:?}");
                                return Ok(());
                            }
                        };

                        if let StagedLedgerPartsSendProgress::ScanStateTreesSending {
                            data,
                            tree_index,
                            ..
                        } = progress
                        {
                            let target_index = data.scan_state.scan_state.trees.1.len();
                            if *tree_index >= target_index {
                                *progress =
                                    StagedLedgerPartsSendProgress::Success { time: meta.time() };
                            }
                        }
                    }
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsStreamingRpcEffectfulAction::ResponsePartSend {
                    peer_id,
                    id: *id,
                    response: response.clone(),
                });
                Ok(())
            }
            P2pChannelsStreamingRpcAction::ResponseSent { id, .. } => {
                let (remote, request) = match streaming_rpc_state {
                    Self::Ready { remote, .. } => match remote {
                        P2pStreamingRpcRemoteState::Requested { request, .. } => {
                            let request = std::mem::take(request);
                            (remote, request)
                        }
                        _ => {
                            bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                            return Ok(());
                        }
                    },
                    _ => {
                        bug_condition!("{:?} with state {:?}", action, streaming_rpc_state);
                        return Ok(());
                    }
                };
                *remote = P2pStreamingRpcRemoteState::Responded {
                    time: meta.time(),
                    id: *id,
                    request,
                };

                Ok(())
            }
        }
    }
}
