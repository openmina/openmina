use redux::ActionMeta;

use super::{SnarkBlockVerifyAction, SnarkBlockVerifyService};

impl SnarkBlockVerifyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkBlockVerifyService,
        SnarkBlockVerifyAction: redux::EnablingCondition<S>,
    {
        match self {
            SnarkBlockVerifyAction::Init { req_id, block, .. } => {
                let verifier_index = store.state().block_verify.verifier_index.clone();
                let verifier_srs = store.state().block_verify.verifier_srs.clone();
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, block);
                store.dispatch(SnarkBlockVerifyAction::Pending { req_id });
            }
            SnarkBlockVerifyAction::Error { req_id, .. } => {
                store.dispatch(SnarkBlockVerifyAction::Finish { req_id });
            }
            SnarkBlockVerifyAction::Success { req_id, .. } => {
                store.dispatch(SnarkBlockVerifyAction::Finish { req_id });
            }
            SnarkBlockVerifyAction::Pending { .. } => {}
            SnarkBlockVerifyAction::Finish { .. } => {}
        }
    }
}
