use super::{
    SnarkWorkVerifyAction, SnarkWorkVerifyActionWithMetaRef, SnarkWorkVerifyState,
    SnarkWorkVerifyStatus,
};

impl SnarkWorkVerifyState {
    pub fn reducer(&mut self, action: SnarkWorkVerifyActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkWorkVerifyAction::Init(action) => {
                self.jobs.add(SnarkWorkVerifyStatus::Init {
                    time: meta.time(),
                    batch: action.batch.clone(),
                    sender: action.sender.clone(),
                });
            }
            SnarkWorkVerifyAction::Pending(action) => {
                if let Some(req) = self.jobs.get_mut(action.req_id) {
                    *req = match req {
                        SnarkWorkVerifyStatus::Init { batch, sender, .. } => {
                            SnarkWorkVerifyStatus::Pending {
                                time: meta.time(),
                                batch: std::mem::take(batch),
                                sender: std::mem::take(sender),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkWorkVerifyAction::Error(action) => {
                if let Some(req) = self.jobs.get_mut(action.req_id) {
                    *req = match req {
                        SnarkWorkVerifyStatus::Pending { batch, sender, .. } => {
                            SnarkWorkVerifyStatus::Error {
                                time: meta.time(),
                                batch: std::mem::take(batch),
                                sender: std::mem::take(sender),
                                error: action.error.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkWorkVerifyAction::Success(action) => {
                if let Some(req) = self.jobs.get_mut(action.req_id) {
                    *req = match req {
                        SnarkWorkVerifyStatus::Pending { batch, sender, .. } => {
                            SnarkWorkVerifyStatus::Success {
                                time: meta.time(),
                                batch: std::mem::take(batch),
                                sender: std::mem::take(sender),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkWorkVerifyAction::Finish(action) => {
                self.jobs.remove(action.req_id);
            }
        }
    }
}
