use super::{
    SnarkUserCommandVerifyAction, SnarkUserCommandVerifyActionWithMetaRef,
    SnarkUserCommandVerifyState, SnarkUserCommandVerifyStatus,
};

impl SnarkUserCommandVerifyState {
    pub fn reducer(&mut self, action: SnarkUserCommandVerifyActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkUserCommandVerifyAction::Init {
                commands, sender, ..
            } => {
                self.jobs.add(SnarkUserCommandVerifyStatus::Init {
                    time: meta.time(),
                    commands: commands.clone(),
                    sender: sender.clone(),
                });
            }
            SnarkUserCommandVerifyAction::Pending { req_id } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
                    *req = match req {
                        SnarkUserCommandVerifyStatus::Init {
                            commands, sender, ..
                        } => SnarkUserCommandVerifyStatus::Pending {
                            time: meta.time(),
                            commands: std::mem::take(commands),
                            sender: std::mem::take(sender),
                        },
                        _ => return,
                    };
                }
            }
            SnarkUserCommandVerifyAction::Error { req_id, error } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
                    *req = match req {
                        SnarkUserCommandVerifyStatus::Pending {
                            commands, sender, ..
                        } => SnarkUserCommandVerifyStatus::Error {
                            time: meta.time(),
                            commands: std::mem::take(commands),
                            sender: std::mem::take(sender),
                            error: error.clone(),
                        },
                        _ => return,
                    };
                }
            }
            SnarkUserCommandVerifyAction::Success { req_id } => {
                if let Some(req) = self.jobs.get_mut(*req_id) {
                    *req = match req {
                        SnarkUserCommandVerifyStatus::Pending {
                            commands, sender, ..
                        } => SnarkUserCommandVerifyStatus::Success {
                            time: meta.time(),
                            commands: std::mem::take(commands),
                            sender: std::mem::take(sender),
                        },
                        _ => return,
                    };
                }
            }
            SnarkUserCommandVerifyAction::Finish { req_id } => {
                self.jobs.remove(*req_id);
            }
        }
    }
}
