use redux::ActionMeta;

use super::{SnarkUserCommandVerifyEffectfulAction, SnarkUserCommandVerifyService};

impl SnarkUserCommandVerifyEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkUserCommandVerifyService,
    {
        match self {
            SnarkUserCommandVerifyEffectfulAction::Init {
                req_id,
                commands,
                verifier_index,
                verifier_srs,
            } => {
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, commands);
            }
        }
    }
}
