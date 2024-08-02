use snark::work_verify::SnarkWorkVerifyAction;

use crate::{snark_pool::candidate::SnarkPoolCandidateAction, Service, SnarkPoolAction, Store};

use super::{SnarkAction, SnarkActionWithMeta};

pub fn snark_effects<S: Service>(store: &mut Store<S>, action: SnarkActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        SnarkAction::BlockVerify(_) => {}
        SnarkAction::BlockVerifyEffect(a) => {
            a.effects(&meta, store);
        }
        SnarkAction::WorkVerify(a) => match a {
            // TODO(tizoc): handle this logic with the on_error callback passed on the Init action
            SnarkWorkVerifyAction::Error { req_id, .. } => {
                let req = store.state().snark.work_verify.jobs.get(req_id);
                let Some(req) = req else { return };
                let sender = req.sender().parse().unwrap();

                store.dispatch(SnarkPoolCandidateAction::WorkVerifyError {
                    peer_id: sender,
                    verify_id: req_id,
                });
            }
            // TODO(tizoc): handle this logic with the on_success callback passed on the Init action
            SnarkWorkVerifyAction::Success { req_id } => {
                let req = store.state().snark.work_verify.jobs.get(req_id);
                let Some(req) = req else { return };
                let sender = req.sender().parse().unwrap();
                let batch = req.batch().to_vec();

                store.dispatch(SnarkPoolCandidateAction::WorkVerifySuccess {
                    peer_id: sender,
                    verify_id: req_id,
                    batch,
                });
            }
            SnarkWorkVerifyAction::Init { .. } => {}
            SnarkWorkVerifyAction::Pending { .. } => {}
            SnarkWorkVerifyAction::Finish { .. } => {}
        },
        SnarkAction::WorkVerifyEffect(a) => {
            a.effects(&meta, store);
        }
    }
}
