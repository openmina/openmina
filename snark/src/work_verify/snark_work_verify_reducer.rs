use super::{
    SnarkWorkVerifyAction, SnarkWorkVerifyActionWithMetaRef, SnarkWorkVerifyState,
    SnarkWorkVerifyStatus,
};

impl SnarkWorkVerifyState {
    pub fn reducer(&mut self, action: SnarkWorkVerifyActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkWorkVerifyAction::Init { batch, sender, .. } => {
                self.jobs.add(SnarkWorkVerifyStatus::Init {
                    time: meta.time(),
                    batch: batch.clone(),
                    sender: sender.clone(),
                });
            }
            SnarkWorkVerifyAction::Pending { req_id } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
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
            SnarkWorkVerifyAction::Error { req_id, error } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
                    *req = match req {
                        SnarkWorkVerifyStatus::Pending { batch, sender, .. } => {
                            SnarkWorkVerifyStatus::Error {
                                time: meta.time(),
                                batch: std::mem::take(batch),
                                sender: std::mem::take(sender),
                                error: error.clone(),
                            }
                        }
                        _ => return,
                    };
                }
            }
            SnarkWorkVerifyAction::Success { req_id } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
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
            SnarkWorkVerifyAction::Finish { req_id } => {
                self.jobs.remove(*req_id);
            }
        }
    }
}
