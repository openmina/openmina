use redux::ActionMeta;

use super::{SnarkUserCommandVerifyEffectfulAction, SnarkUserCommandVerifyService};

impl SnarkUserCommandVerifyEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkUserCommandVerifyService,
    {
        match self {
            Self::Init {
                req_id, commands, ..
            } => {
                let verifier_index = store.state().work_verify.verifier_index.clone();
                let verifier_srs = store.state().work_verify.verifier_srs.clone();
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, commands);
            }
        }
    }
}
