use super::{
    P2pChannelsSnarkAction, P2pChannelsSnarkActionWithMetaRef, P2pChannelsSnarkState,
    SnarkPropagationState,
};

impl P2pChannelsSnarkState {
    pub fn reducer(&mut self, action: P2pChannelsSnarkActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsSnarkAction::Init { .. } => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsSnarkAction::Pending { .. } => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsSnarkAction::Ready { .. } => {
                *self = Self::Ready {
                    time: meta.time(),
                    local: SnarkPropagationState::WaitingForRequest { time: meta.time() },
                    remote: SnarkPropagationState::WaitingForRequest { time: meta.time() },
                    next_send_index: 0,
                };
            }
            P2pChannelsSnarkAction::RequestSend { limit, .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                *local = SnarkPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
            }
            P2pChannelsSnarkAction::PromiseReceived { promised_count, .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                let SnarkPropagationState::Requested {
                    requested_limit, ..
                } = &local
                else {
                    return;
                };
                *local = SnarkPropagationState::Responding {
                    time: meta.time(),
                    requested_limit: *requested_limit,
                    promised_count: *promised_count,
                    current_count: 0,
                };
            }
            P2pChannelsSnarkAction::Received { .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                let SnarkPropagationState::Responding {
                    promised_count,
                    current_count,
                    ..
                } = local
                else {
                    return;
                };

                *current_count += 1;

                if current_count >= promised_count {
                    *local = SnarkPropagationState::Responded {
                        time: meta.time(),
                        count: *current_count,
                    };
                }
            }
            P2pChannelsSnarkAction::RequestReceived { limit, .. } => {
                let Self::Ready { remote, .. } = self else {
                    return;
                };
                *remote = SnarkPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
            }
            P2pChannelsSnarkAction::ResponseSend {
                snarks, last_index, ..
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

                let count = snarks.len() as u8;
                if count == 0 {
                    return;
                }

                *remote = SnarkPropagationState::Responded {
                    time: meta.time(),
                    count,
                };
            }
            P2pChannelsSnarkAction::Libp2pReceived { .. }
            | P2pChannelsSnarkAction::Libp2pBroadcast { .. } => {}
        }
    }
}
