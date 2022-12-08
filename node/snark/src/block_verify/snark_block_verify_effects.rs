use redux::ActionMeta;

use super::{
    SnarkBlockVerifyErrorAction, SnarkBlockVerifyFinishAction, SnarkBlockVerifyInitAction,
    SnarkBlockVerifyPendingAction, SnarkBlockVerifyService, SnarkBlockVerifySuccessAction,
};

impl SnarkBlockVerifyInitAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkBlockVerifyService,
        SnarkBlockVerifyPendingAction: redux::EnablingCondition<S>,
    {
        let req_id = self.req_id;
        let verifier_index = store.state().block_verify.verifier_index.clone();
        let verifier_srs = store.state().block_verify.verifier_srs.clone();
        store
            .service()
            .verify_init(req_id, verifier_index, verifier_srs, &self.block.header);
        store.dispatch(SnarkBlockVerifyPendingAction { req_id });
    }
}

impl SnarkBlockVerifyErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkBlockVerifyService,
        SnarkBlockVerifyFinishAction: redux::EnablingCondition<S>,
    {
        let req_id = self.req_id;
        store.dispatch(SnarkBlockVerifyFinishAction { req_id });
    }
}

impl SnarkBlockVerifySuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkBlockVerifyService,
        SnarkBlockVerifyFinishAction: redux::EnablingCondition<S>,
    {
        let req_id = self.req_id;
        store.dispatch(SnarkBlockVerifyFinishAction { req_id });
    }
}
