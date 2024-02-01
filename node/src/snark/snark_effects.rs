use crate::consensus::ConsensusAction;
use crate::snark_pool::candidate::SnarkPoolCandidateAction;
use crate::snark_pool::SnarkPoolAction;
use crate::{Service, Store};

use super::block_verify::SnarkBlockVerifyAction;
use super::work_verify::SnarkWorkVerifyAction;
use super::{SnarkAction, SnarkActionWithMeta};

pub fn snark_effects<S: Service>(store: &mut Store<S>, action: SnarkActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        SnarkAction::BlockVerify(a) => {
            match a {
                SnarkBlockVerifyAction::Success { req_id } => {
                    let req = store.state().snark.block_verify.jobs.get(req_id);
                    let Some(req) = req else { return };
                    store.dispatch(ConsensusAction::BlockSnarkVerifySuccess {
                        hash: req.block().hash_ref().clone(),
                    });
                }
                SnarkBlockVerifyAction::Init { .. } => {}
                SnarkBlockVerifyAction::Pending { .. } => {}
                SnarkBlockVerifyAction::Error { .. } => {}
                SnarkBlockVerifyAction::Finish { .. } => {}
            }
            a.effects(&meta, store);
        }
        SnarkAction::WorkVerify(a) => {
            match a {
                SnarkWorkVerifyAction::Error { req_id, .. } => {
                    let req = store.state().snark.work_verify.jobs.get(req_id);
                    let Some(req) = req else { return };
                    let sender = req.sender().parse().unwrap();

                    store.dispatch(SnarkPoolCandidateAction::WorkVerifyError {
                        peer_id: sender,
                        verify_id: req_id,
                    });
                }
                SnarkWorkVerifyAction::Success { req_id } => {
                    let req = store.state().snark.work_verify.jobs.get(req_id);
                    let Some(req) = req else { return };
                    let sender = req.sender().parse().unwrap();
                    let batch = req.batch().to_vec();

                    store.dispatch(SnarkPoolCandidateAction::WorkVerifySuccess {
                        peer_id: sender,
                        verify_id: req_id,
                    });
                    for snark in batch {
                        store.dispatch(SnarkPoolAction::WorkAdd { snark, sender });
                    }
                }
                SnarkWorkVerifyAction::Init { .. } => {}
                SnarkWorkVerifyAction::Pending { .. } => {}
                SnarkWorkVerifyAction::Finish { .. } => {}
            }
            a.effects(&meta, store);
        }
    }
}
