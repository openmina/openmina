use redux::ActionMeta;

use super::{
    SnarkWorkVerifyAction, SnarkWorkVerifyService, SnarkWorkVerifyState, SnarkWorkVerifyStatus,
};

impl SnarkWorkVerifyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::SnarkStore<S>,
        Store::Service: SnarkWorkVerifyService,
        SnarkWorkVerifyAction: redux::EnablingCondition<S>,
    {
        match self {
            SnarkWorkVerifyAction::Init {
                req_id,
                batch,
                verify_success_cb,
                verify_error_cb,
                ..
            } => {
                let verifier_index = store.state().work_verify.verifier_index.clone();
                let verifier_srs = store.state().work_verify.verifier_srs.clone();
                store
                    .service()
                    .verify_init(req_id, verifier_index, verifier_srs, batch);
                store.dispatch(SnarkWorkVerifyAction::Pending {
                    req_id,
                    verify_success_cb,
                    verify_error_cb,
                });
            }
            SnarkWorkVerifyAction::Error { req_id, .. } => {
                let req = store.state().work_verify.jobs.get(req_id);
                let Some(SnarkWorkVerifyStatus::Error {
                    sender,
                    verify_error_cb,
                    ..
                }) = req
                else {
                    return;
                };

                store.dispatch_callback(verify_error_cb.clone(), (sender.clone(), req_id));
                store.dispatch(SnarkWorkVerifyAction::Finish { req_id });
            }
            SnarkWorkVerifyAction::Success { req_id } => {
                let req = store.state().work_verify.jobs.get(req_id);
                let Some(SnarkWorkVerifyStatus::Success {
                    batch,
                    sender,
                    verify_success_cb,
                    ..
                }) = req
                else {
                    return;
                };

                store.dispatch_callback(
                    verify_success_cb.clone(),
                    (sender.clone(), req_id, batch.clone()),
                );
                store.dispatch(SnarkWorkVerifyAction::Finish { req_id });
            }
            SnarkWorkVerifyAction::Pending { .. } => {}
            SnarkWorkVerifyAction::Finish { .. } => {}
        }
    }
}
