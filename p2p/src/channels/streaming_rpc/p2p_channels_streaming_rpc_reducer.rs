use openmina_core::bug_condition;

use super::{
    staged_ledger_parts::{StagedLedgerPartsReceiveProgress, StagedLedgerPartsSendProgress},
    P2pChannelsStreamingRpcAction, P2pChannelsStreamingRpcActionWithMetaRef,
    P2pChannelsStreamingRpcState, P2pStreamingRpcId, P2pStreamingRpcLocalState,
    P2pStreamingRpcRemoteState, P2pStreamingRpcRequest, P2pStreamingRpcResponseFull,
    P2pStreamingRpcSendProgress,
};

impl P2pChannelsStreamingRpcState {
    pub fn reducer(
        &mut self,
        action: P2pChannelsStreamingRpcActionWithMetaRef<'_>,
        next_local_rpc_id: &mut P2pStreamingRpcId,
    ) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsStreamingRpcAction::Init { .. } => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsStreamingRpcAction::Pending { .. } => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsStreamingRpcAction::Ready { .. } => {
                *self = Self::Ready {
                    time: meta.time(),
                    local: P2pStreamingRpcLocalState::WaitingForRequest { time: meta.time() },
                    remote: P2pStreamingRpcRemoteState::WaitingForRequest { time: meta.time() },
                    remote_last_responded: redux::Timestamp::ZERO,
                };
            }
            P2pChannelsStreamingRpcAction::RequestSend { id, request, .. } => {
                let Self::Ready { local, .. } = self else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
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
            }
            P2pChannelsStreamingRpcAction::Timeout { .. } => {}
            P2pChannelsStreamingRpcAction::ResponseNextPartGet { .. } => {
                let Self::Ready {
                    local: P2pStreamingRpcLocalState::Requested { progress, .. },
                    ..
                } = self
                else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
                };

                if !progress.set_next_pending(meta.time()) {
                    bug_condition!("progress state already pending: {progress:?}");
                }

                if !progress.is_part_pending() {
                    bug_condition!("progress state is not pending {:?}", progress);
                }
            }
            P2pChannelsStreamingRpcAction::ResponsePartReceived { response, .. } => {
                let Self::Ready {
                    local: P2pStreamingRpcLocalState::Requested { progress, .. },
                    ..
                } = self
                else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
                };
                if !progress.update(meta.time(), response) {
                    bug_condition!("progress response mismatch! {progress:?}\n{response:?}");
                }
            }
            P2pChannelsStreamingRpcAction::ResponseReceived { .. } => {
                let Self::Ready { local, .. } = self else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
                };
                let P2pStreamingRpcLocalState::Requested { id, request, .. } = local else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
                };
                *local = P2pStreamingRpcLocalState::Responded {
                    time: meta.time(),
                    id: *id,
                    request: std::mem::take(request),
                };
            }
            P2pChannelsStreamingRpcAction::RequestReceived { id, request, .. } => {
                let Self::Ready { remote, .. } = self else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
                };
                *remote = P2pStreamingRpcRemoteState::Requested {
                    time: meta.time(),
                    id: *id,
                    request: request.clone(),
                    progress: StagedLedgerPartsSendProgress::LedgerGetIdle { time: meta.time() }
                        .into(),
                };
            }
            P2pChannelsStreamingRpcAction::ResponsePending { .. } => {
                let Self::Ready {
                    remote:
                        P2pStreamingRpcRemoteState::Requested {
                            request, progress, ..
                        },
                    ..
                } = self
                else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
                };
                match &**request {
                    P2pStreamingRpcRequest::StagedLedgerParts(_) => {
                        *progress =
                            StagedLedgerPartsSendProgress::LedgerGetPending { time: meta.time() }
                                .into();
                    }
                }
            }
            P2pChannelsStreamingRpcAction::ResponseSendInit { response, .. } => {
                let Self::Ready {
                    remote:
                        P2pStreamingRpcRemoteState::Requested {
                            request, progress, ..
                        },
                    ..
                } = self
                else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
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
            }
            P2pChannelsStreamingRpcAction::ResponsePartNextSend { .. } => {}
            P2pChannelsStreamingRpcAction::ResponsePartSend { .. } => {
                let Self::Ready {
                    remote: P2pStreamingRpcRemoteState::Requested { progress, .. },
                    ..
                } = self
                else {
                    bug_condition!("{:?} with state {:?}", action, self);
                    return;
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
                                return;
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
            }
            P2pChannelsStreamingRpcAction::ResponseSent { id, .. } => {
                let (remote, request) = match self {
                    Self::Ready { remote, .. } => match remote {
                        P2pStreamingRpcRemoteState::Requested { request, .. } => {
                            let request = std::mem::take(request);
                            (remote, request)
                        }
                        _ => {
                            bug_condition!("{:?} with state {:?}", action, self);
                            return;
                        }
                    },
                    _ => {
                        bug_condition!("{:?} with state {:?}", action, self);
                        return;
                    }
                };
                *remote = P2pStreamingRpcRemoteState::Responded {
                    time: meta.time(),
                    id: *id,
                    request,
                };
            }
        }
    }
}
