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
        store.service().verify_init(req_id, &self.block);
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
