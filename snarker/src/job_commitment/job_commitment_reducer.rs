use super::{
    JobCommitment, JobCommitmentAction, JobCommitmentActionWithMetaRef, JobCommitmentsState,
};

impl JobCommitmentsState {
    pub fn reducer(&mut self, action: JobCommitmentActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
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
            JobCommitmentAction::CheckTimeouts(_) => {
                self.last_check_timeouts = meta.time();
            }
            JobCommitmentAction::Timeout(a) => {
                self.remove(&a.job_id);
            }
        }
    }
}
