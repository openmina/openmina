use super::{
    P2pChannelsSnarkJobCommitmentAction, P2pChannelsSnarkJobCommitmentActionWithMetaRef,
    P2pChannelsSnarkJobCommitmentState,
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
                *self = Self::Ready { time: meta.time() };
            }
        }
    }
}
