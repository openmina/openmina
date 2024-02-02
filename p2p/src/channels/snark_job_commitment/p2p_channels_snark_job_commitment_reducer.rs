use super::{
    P2pChannelsSnarkJobCommitmentAction, P2pChannelsSnarkJobCommitmentActionWithMetaRef,
    P2pChannelsSnarkJobCommitmentState, SnarkJobCommitmentPropagationState,
};

impl P2pChannelsSnarkJobCommitmentState {
    pub fn reducer(&mut self, action: P2pChannelsSnarkJobCommitmentActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsSnarkJobCommitmentAction::Init { .. } => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsSnarkJobCommitmentAction::Pending { .. } => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsSnarkJobCommitmentAction::Ready { .. } => {
                *self = Self::Ready {
                    time: meta.time(),
                    local: SnarkJobCommitmentPropagationState::WaitingForRequest {
                        time: meta.time(),
                    },
                    remote: SnarkJobCommitmentPropagationState::WaitingForRequest {
                        time: meta.time(),
                    },
                    next_send_index: 0,
                };
            }
            P2pChannelsSnarkJobCommitmentAction::RequestSend { limit, .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                *local = SnarkJobCommitmentPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
            }
            P2pChannelsSnarkJobCommitmentAction::PromiseReceived { promised_count, .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                let SnarkJobCommitmentPropagationState::Requested {
                    requested_limit, ..
                } = &local
                else {
                    return;
                };

                *local = SnarkJobCommitmentPropagationState::Responding {
                    time: meta.time(),
                    requested_limit: *requested_limit,
                    promised_count: *promised_count,
                    current_count: 0,
                };
            }
            P2pChannelsSnarkJobCommitmentAction::Received { .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                let SnarkJobCommitmentPropagationState::Responding {
                    promised_count,
                    current_count,
                    ..
                } = local
                else {
                    return;
                };

                *current_count += 1;

                if current_count >= promised_count {
                    *local = SnarkJobCommitmentPropagationState::Responded {
                        time: meta.time(),
                        count: *current_count,
                    };
                }
            }
            P2pChannelsSnarkJobCommitmentAction::RequestReceived { limit, .. } => {
                let Self::Ready { remote, .. } = self else {
                    return;
                };
                *remote = SnarkJobCommitmentPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
            }
            P2pChannelsSnarkJobCommitmentAction::ResponseSend {
                last_index,
                commitments,
                ..
            } => {
                let Self::Ready {
                    remote,
                    next_send_index,
                    ..
                } = self
                else {
                    return;
                };
                *next_send_index = last_index + 1;

                let count = commitments.len() as u8;
                if count == 0 {
                    return;
                }

                *remote = SnarkJobCommitmentPropagationState::Responded {
                    time: meta.time(),
                    count,
                };
            }
        }
    }
}
