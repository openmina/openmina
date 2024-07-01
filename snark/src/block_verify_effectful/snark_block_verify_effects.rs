use redux::ActionMeta;

use super::{SnarkBlockVerifyEffectfulAction, SnarkBlockVerifyService};

impl SnarkBlockVerifyEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkBlockVerifyService,
    {
        match self {
            Self::Init {
                req_id,
                block,
                verifier_index,
                verifier_srs,
                ..
            } => {
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, block);
            }
        }
    }
}
