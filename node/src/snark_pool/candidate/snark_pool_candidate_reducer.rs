use super::{
    SnarkPoolCandidateAction, SnarkPoolCandidateActionWithMetaRef, SnarkPoolCandidatesState,
};

impl SnarkPoolCandidatesState {
    pub fn reducer(&mut self, action: SnarkPoolCandidateActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkPoolCandidateAction::InfoReceived { peer_id, info } => {
                self.info_received(meta.time(), *peer_id, info.clone());
            }
            SnarkPoolCandidateAction::WorkFetchAll => {}
            SnarkPoolCandidateAction::WorkFetchInit { .. } => {}
            SnarkPoolCandidateAction::WorkFetchPending {
                peer_id,
                job_id,
                rpc_id,
            } => {
                self.work_fetch_pending(meta.time(), peer_id, job_id, *rpc_id);
            }
            SnarkPoolCandidateAction::WorkReceived { peer_id, work } => {
                self.work_received(meta.time(), *peer_id, work.clone());
            }
            SnarkPoolCandidateAction::WorkVerifyNext => {}
            SnarkPoolCandidateAction::WorkVerifyPending {
                peer_id,
                job_ids,
                verify_id,
            } => {
                self.verify_pending(meta.time(), peer_id, *verify_id, job_ids);
            }
            SnarkPoolCandidateAction::WorkVerifyError { peer_id, verify_id } => {
                self.verify_result(meta.time(), peer_id, *verify_id, Err(()));
            }
            SnarkPoolCandidateAction::WorkVerifySuccess {
                peer_id, verify_id, ..
            } => {
                self.verify_result(meta.time(), peer_id, *verify_id, Ok(()));
            }
            SnarkPoolCandidateAction::PeerPrune { peer_id } => {
                self.peer_remove(*peer_id);
            }
        }
    }
}
