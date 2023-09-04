use super::{
    SnarkPoolCandidateAction, SnarkPoolCandidateActionWithMetaRef, SnarkPoolCandidatesState,
};

impl SnarkPoolCandidatesState {
    pub fn reducer(&mut self, action: SnarkPoolCandidateActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkPoolCandidateAction::InfoReceived(a) => {
                self.info_received(meta.time(), a.peer_id, a.info.clone());
            }
            SnarkPoolCandidateAction::WorkFetchAll(_) => {}
            SnarkPoolCandidateAction::WorkFetchInit(_) => {}
            SnarkPoolCandidateAction::WorkFetchPending(a) => {
                self.work_fetch_pending(meta.time(), &a.peer_id, &a.job_id, a.rpc_id);
            }
            SnarkPoolCandidateAction::WorkReceived(a) => {
                self.work_received(meta.time(), a.peer_id, a.work.clone());
            }
            SnarkPoolCandidateAction::WorkVerifyNext(_) => {}
            SnarkPoolCandidateAction::WorkVerifyPending(a) => {
                self.verify_pending(meta.time(), &a.peer_id, a.verify_id, &a.job_ids);
            }
            SnarkPoolCandidateAction::WorkVerifyError(a) => {
                self.verify_result(meta.time(), &a.peer_id, a.verify_id, Err(()));
            }
            SnarkPoolCandidateAction::WorkVerifySuccess(a) => {
                self.verify_result(meta.time(), &a.peer_id, a.verify_id, Ok(()));
            }
            SnarkPoolCandidateAction::PeerPrune(a) => {
                self.peer_remove(a.peer_id);
            }
        }
    }
}
