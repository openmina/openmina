use redux::ActionMeta;

use super::{
    SnarkWorkVerifyErrorAction, SnarkWorkVerifyFinishAction, SnarkWorkVerifyInitAction,
    SnarkWorkVerifyPendingAction, SnarkWorkVerifyService, SnarkWorkVerifySuccessAction,
};

impl SnarkWorkVerifyInitAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkWorkVerifyService,
        SnarkWorkVerifyPendingAction: redux::EnablingCondition<S>,
    {
        let req_id = self.req_id;
        let verifier_index = store.state().work_verify.verifier_index.clone();
        let verifier_srs = store.state().work_verify.verifier_srs.clone();
        store
            .service()
            .verify_init(req_id, verifier_index, verifier_srs, self.batch.clone());
        store.dispatch(SnarkWorkVerifyPendingAction { req_id });
    }
}

impl SnarkWorkVerifyErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkWorkVerifyService,
        SnarkWorkVerifyFinishAction: redux::EnablingCondition<S>,
    {
        let req_id = self.req_id;
        store.dispatch(SnarkWorkVerifyFinishAction { req_id });
    }
}

impl SnarkWorkVerifySuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkWorkVerifyService,
        SnarkWorkVerifyFinishAction: redux::EnablingCondition<S>,
    {
        let req_id = self.req_id;
        store.dispatch(SnarkWorkVerifyFinishAction { req_id });
    }
}
