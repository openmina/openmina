use redux::ActionMeta;

use super::{SnarkUserCommandVerifyAction, SnarkUserCommandVerifyService};

impl SnarkUserCommandVerifyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkUserCommandVerifyService,
        SnarkUserCommandVerifyAction: redux::EnablingCondition<S>,
    {
        match self {
            SnarkUserCommandVerifyAction::Init {
                req_id, commands, ..
            } => {
                let verifier_index = store.state().work_verify.verifier_index.clone();
                let verifier_srs = store.state().work_verify.verifier_srs.clone();
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, commands);
                store.dispatch(SnarkUserCommandVerifyAction::Pending { req_id });
            }
            SnarkUserCommandVerifyAction::Error { req_id, .. } => {
                store.dispatch(SnarkUserCommandVerifyAction::Finish { req_id });
            }
            SnarkUserCommandVerifyAction::Success { req_id } => {
                store.dispatch(SnarkUserCommandVerifyAction::Finish { req_id });
            }
            SnarkUserCommandVerifyAction::Pending { .. } => {}
            SnarkUserCommandVerifyAction::Finish { .. } => {}
        }
    }
}
