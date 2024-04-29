use redux::ActionMeta;

use super::{SnarkWorkVerifyEffectfulAction, SnarkWorkVerifyService};

impl SnarkWorkVerifyEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkWorkVerifyService,
    {
        match self {
            Self::Init { req_id, batch, .. } => {
                let verifier_index = store.state().work_verify.verifier_index.clone();
                let verifier_srs = store.state().work_verify.verifier_srs.clone();
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, batch);
            }
        }
    }
}
