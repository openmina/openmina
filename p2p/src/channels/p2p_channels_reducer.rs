use super::{P2pChannelsAction, P2pChannelsActionWithMetaRef, P2pChannelsState};

impl P2pChannelsState {
    pub fn reducer(&mut self, action: P2pChannelsActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsAction::SnarkJobCommitment(action) => {
                self.snark_job_commitment.reducer(meta.with_action(action));
            }
        }
    }
}
