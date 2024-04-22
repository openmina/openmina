use super::{
    SnarkBlockVerifyAction, SnarkBlockVerifyActionWithMetaRef, SnarkBlockVerifyState,
    SnarkBlockVerifyStatus,
};

impl SnarkBlockVerifyState {
    pub fn reducer(&mut self, action: SnarkBlockVerifyActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkBlockVerifyAction::Init { block, .. } => {
                self.jobs.add(SnarkBlockVerifyStatus::Init {
                    time: meta.time(),
                    block: block.clone(),
                });
            }
            SnarkBlockVerifyAction::Pending {
                req_id,
                verify_success_cb,
            } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
                    *req = match req {
                        SnarkBlockVerifyStatus::Init { block, .. } => {
                            SnarkBlockVerifyStatus::Pending {
                                time: meta.time(),
                                block: block.clone(),
                                verify_success_cb: verify_success_cb.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkBlockVerifyAction::Error { req_id, error, .. } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
                    *req = match req {
                        SnarkBlockVerifyStatus::Pending { block, .. } => {
                            SnarkBlockVerifyStatus::Error {
                                time: meta.time(),
                                block: block.clone(),
                                error: error.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkBlockVerifyAction::Success { req_id, .. } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
                    *req = match req {
                        SnarkBlockVerifyStatus::Pending {
                            block,
                            time,
                            verify_success_cb,
                        } => SnarkBlockVerifyStatus::Success {
                            time: meta.time(),
                            block: block.clone(),
                            verify_success_cb: verify_success_cb.clone(),
                        },
                        _ => return,
                    };
                }
            }
            SnarkBlockVerifyAction::Finish { req_id, .. } => {
                self.jobs.remove(*req_id);
            }
        }
    }
}
