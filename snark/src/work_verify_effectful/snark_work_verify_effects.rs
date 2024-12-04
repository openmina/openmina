use redux::ActionMeta;

use super::{SnarkWorkVerifyEffectfulAction, SnarkWorkVerifyService};

impl SnarkWorkVerifyEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkWorkVerifyService,
    {
        match self {
            SnarkWorkVerifyEffectfulAction::Init {
                req_id,
                batch,
                verifier_srs,
                verifier_index,
            } => {
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, batch);
            }
        }
    }
}
