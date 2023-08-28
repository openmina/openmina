use super::{
    BestTipPropagationState, P2pChannelsBestTipAction, P2pChannelsBestTipActionWithMetaRef,
    P2pChannelsBestTipState,
};

impl P2pChannelsBestTipState {
    pub fn reducer(&mut self, action: P2pChannelsBestTipActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsBestTipAction::Init(_) => {
                *self = Self::Init { time: meta.time() };
            }
            P2pChannelsBestTipAction::Pending(_) => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pChannelsBestTipAction::Ready(_) => {
                *self = Self::Ready {
                    time: meta.time(),
                    local: BestTipPropagationState::WaitingForRequest { time: meta.time() },
                    remote: BestTipPropagationState::WaitingForRequest { time: meta.time() },
                    last_sent: None,
                    last_received: None,
                };
            }
            P2pChannelsBestTipAction::RequestSend(_) => {
                let Self::Ready { local, .. } = self else {
                    return;
                };
                *local = BestTipPropagationState::Requested { time: meta.time() };
            }
            P2pChannelsBestTipAction::Received(action) => {
                let Self::Ready {
                    local,
                    last_received,
                    ..
                } = self
                else {
                    return;
                };

                *local = BestTipPropagationState::Responded { time: meta.time() };
                *last_received = Some(action.best_tip.clone());
            }
            P2pChannelsBestTipAction::RequestReceived(_) => {
                let Self::Ready { remote, .. } = self else {
                    return;
                };

                *remote = BestTipPropagationState::Requested { time: meta.time() };
            }
            P2pChannelsBestTipAction::ResponseSend(action) => {
                let Self::Ready {
                    remote, last_sent, ..
                } = self
                else {
                    return;
                };

                *remote = BestTipPropagationState::Responded { time: meta.time() };
                *last_sent = Some(action.best_tip.clone());
            }
        }
    }
}
