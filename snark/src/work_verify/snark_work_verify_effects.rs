use redux::ActionMeta;

use super::{SnarkWorkVerifyAction, SnarkWorkVerifyService};

impl SnarkWorkVerifyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkWorkVerifyService,
        SnarkWorkVerifyAction: redux::EnablingCondition<S>,
    {
        match self {
            SnarkWorkVerifyAction::Init { req_id, batch, .. } => {
                let verifier_index = store.state().work_verify.verifier_index.clone();
                let verifier_srs = store.state().work_verify.verifier_srs.clone();
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, batch);
                store.dispatch(SnarkWorkVerifyAction::Pending { req_id });
            }
            SnarkWorkVerifyAction::Error { req_id, .. } => {
                store.dispatch(SnarkWorkVerifyAction::Finish { req_id });
            }
            SnarkWorkVerifyAction::Success { req_id } => {
                store.dispatch(SnarkWorkVerifyAction::Finish { req_id });
            }
            SnarkWorkVerifyAction::Pending { .. } => {}
            SnarkWorkVerifyAction::Finish { .. } => {}
        }
    }
}
