use super::{
    P2pChannelsTransactionAction, P2pChannelsTransactionActionWithMetaRef,
    P2pChannelsTransactionState, TransactionPropagationState,
};

impl P2pChannelsTransactionState {
    pub fn reducer(&mut self, action: P2pChannelsTransactionActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsTransactionAction::Init { .. } => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsTransactionAction::Pending { .. } => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsTransactionAction::Ready { .. } => {
                *self = Self::Ready {
                    time: meta.time(),
                    local: TransactionPropagationState::WaitingForRequest { time: meta.time() },
                    remote: TransactionPropagationState::WaitingForRequest { time: meta.time() },
                    next_send_index: 0,
                };
            }
            P2pChannelsTransactionAction::RequestSend { limit, .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                *local = TransactionPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
            }
            P2pChannelsTransactionAction::PromiseReceived { promised_count, .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                let TransactionPropagationState::Requested {
                    requested_limit, ..
                } = &local
                else {
                    return;
                };
                *local = TransactionPropagationState::Responding {
                    time: meta.time(),
                    requested_limit: *requested_limit,
                    promised_count: *promised_count,
                    current_count: 0,
                };
            }
            P2pChannelsTransactionAction::Received { .. } => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                let TransactionPropagationState::Responding {
                    promised_count,
                    current_count,
                    ..
                } = local
                else {
                    return;
                };

                *current_count += 1;

                if current_count >= promised_count {
                    *local = TransactionPropagationState::Responded {
                        time: meta.time(),
                        count: *current_count,
                    };
                }
            }
            P2pChannelsTransactionAction::RequestReceived { limit, .. } => {
                let Self::Ready { remote, .. } = self else {
                    return;
                };
                *remote = TransactionPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
            }
            P2pChannelsTransactionAction::ResponseSend {
                transactions,
                last_index,
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

                let count = transactions.len() as u8;
                if count == 0 {
                    return;
                }

                *remote = TransactionPropagationState::Responded {
                    time: meta.time(),
                    count,
                };
            }
            P2pChannelsTransactionAction::Libp2pReceived { .. }
            | P2pChannelsTransactionAction::Libp2pBroadcast { .. } => {}
        }
    }
}
