use super::{
    JobCommitment, JobCommitmentAction, JobCommitmentActionWithMetaRef, JobCommitmentsState,
};

impl JobCommitmentsState {
    pub fn reducer(&mut self, action: JobCommitmentActionWithMetaRef<'_>) {
        let (action, _meta) = action.split();
        match action {
            JobCommitmentAction::Create(_) => {}
            JobCommitmentAction::Add(a) => {
                self.insert(JobCommitment {
                    commitment: a.commitment.clone(),
                    sender: a.sender,
                });
            }
            JobCommitmentAction::P2pSendAll(_) => {}
            JobCommitmentAction::P2pSend(_) => {}
        }
    }
}
