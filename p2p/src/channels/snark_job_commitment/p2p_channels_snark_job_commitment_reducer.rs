use super::{
    P2pChannelsSnarkJobCommitmentAction, P2pChannelsSnarkJobCommitmentActionWithMetaRef,
    P2pChannelsSnarkJobCommitmentState, SnarkJobCommitmentPropagationState,
};

impl P2pChannelsSnarkJobCommitmentState {
    pub fn reducer(&mut self, action: P2pChannelsSnarkJobCommitmentActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsSnarkJobCommitmentAction::Init(_) => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsSnarkJobCommitmentAction::Pending(_) => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsSnarkJobCommitmentAction::Ready(_) => {
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
            P2pChannelsSnarkJobCommitmentAction::RequestSend(action) => {
                let Self::Ready { local, .. } = self else { return };
                *local = SnarkJobCommitmentPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: action.limit,
                };
            }
            P2pChannelsSnarkJobCommitmentAction::PromiseReceived(action) => {
                let Self::Ready { local, .. } = self else { return };
                let SnarkJobCommitmentPropagationState::Requested { requested_limit, .. } = &local else { return };

                *local = SnarkJobCommitmentPropagationState::Responding {
                    time: meta.time(),
                    requested_limit: *requested_limit,
                    promised_count: action.promised_count,
                    current_count: 0,
                };
            }
            P2pChannelsSnarkJobCommitmentAction::Received(_) => {
                let Self::Ready { local, .. } = self else { return };
                let SnarkJobCommitmentPropagationState::Responding { promised_count, current_count, .. } = local else { return };

                *current_count += 1;

                if current_count >= promised_count {
                    *local = SnarkJobCommitmentPropagationState::Responded {
                        time: meta.time(),
                        count: *current_count,
                    };
                }
            }
            P2pChannelsSnarkJobCommitmentAction::RequestReceived(action) => {
                let Self::Ready { remote, .. } = self else { return };

                *remote = SnarkJobCommitmentPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: action.limit,
                };
            }
            P2pChannelsSnarkJobCommitmentAction::ResponseSend(action) => {
                let Self::Ready { remote, next_send_index, .. } = self else { return };

                *next_send_index = action.last_index + 1;
                *remote = SnarkJobCommitmentPropagationState::Responded {
                    time: meta.time(),
                    count: action.commitments.len() as u8,
                };
            }
        }
    }
}
