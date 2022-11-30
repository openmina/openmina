use super::{
    SnarkBlockVerifyAction, SnarkBlockVerifyActionWithMetaRef, SnarkBlockVerifyState,
    SnarkBlockVerifyStatus,
};

impl SnarkBlockVerifyState {
    pub fn reducer(&mut self, action: SnarkBlockVerifyActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkBlockVerifyAction::Init(action) => {
                self.jobs.add(SnarkBlockVerifyStatus::Init {
                    time: meta.time(),
                    block: action.block.clone(),
                });
            }
            SnarkBlockVerifyAction::Pending(action) => {
                if let Some(req) = self.jobs.get_mut(action.req_id) {
                    *req = match req {
                        SnarkBlockVerifyStatus::Init { block, .. } => {
                            SnarkBlockVerifyStatus::Pending {
                                time: meta.time(),
                                block: block.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkBlockVerifyAction::Error(action) => {
                if let Some(req) = self.jobs.get_mut(action.req_id) {
                    *req = match req {
                        SnarkBlockVerifyStatus::Pending { block, .. } => {
                            SnarkBlockVerifyStatus::Error {
                                time: meta.time(),
                                block: block.clone(),
                                error: action.error.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkBlockVerifyAction::Success(action) => {
                if let Some(req) = self.jobs.get_mut(action.req_id) {
                    *req = match req {
                        SnarkBlockVerifyStatus::Pending { block, .. } => {
                            SnarkBlockVerifyStatus::Success {
                                time: meta.time(),
                                block: block.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkBlockVerifyAction::Finish(action) => {
                self.jobs.remove(action.req_id);
            }
        }
    }
}
