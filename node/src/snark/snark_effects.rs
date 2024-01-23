use crate::consensus::ConsensusBlockSnarkVerifySuccessAction;
use crate::snark_pool::candidate::SnarkPoolCandidateAction;
use crate::snark_pool::SnarkPoolWorkAddAction;
use crate::{Service, Store};

use super::block_verify::SnarkBlockVerifyAction;
use super::work_verify::SnarkWorkVerifyAction;
use super::{SnarkAction, SnarkActionWithMeta};

pub fn snark_effects<S: Service>(store: &mut Store<S>, action: SnarkActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        SnarkAction::BlockVerify(a) => match a {
            SnarkBlockVerifyAction::Init(a) => {
                a.effects(&meta, store);
            }
            SnarkBlockVerifyAction::Pending(_) => {}
            SnarkBlockVerifyAction::Error(a) => {
                a.effects(&meta, store);
            }
            SnarkBlockVerifyAction::Success(a) => {
                let req = store.state().snark.block_verify.jobs.get(a.req_id);
                let Some(req) = req else { return };
                store.dispatch(ConsensusBlockSnarkVerifySuccessAction {
                    hash: req.block().hash_ref().clone(),
                });
                a.effects(&meta, store);
            }
            SnarkBlockVerifyAction::Finish(_) => {}
        },
        SnarkAction::WorkVerify(a) => match a {
            SnarkWorkVerifyAction::Init(a) => {
                a.effects(&meta, store);
            }
            SnarkWorkVerifyAction::Pending(_) => {}
            SnarkWorkVerifyAction::Error(a) => {
                let req = store.state().snark.work_verify.jobs.get(a.req_id);
                let Some(req) = req else { return };
                let sender = req.sender().parse().unwrap();

                store.dispatch(SnarkPoolCandidateAction::WorkVerifyError {
                    peer_id: sender,
                    verify_id: a.req_id,
                });
                a.effects(&meta, store);
            }
            SnarkWorkVerifyAction::Success(a) => {
                let req = store.state().snark.work_verify.jobs.get(a.req_id);
                let Some(req) = req else { return };
                let sender = req.sender().parse().unwrap();
                let batch = req.batch().to_vec();

                store.dispatch(SnarkPoolCandidateAction::WorkVerifySuccess {
                    peer_id: sender,
                    verify_id: a.req_id,
                });
                for snark in batch {
                    store.dispatch(SnarkPoolWorkAddAction { snark, sender });
                }
                a.effects(&meta, store);
            }
            SnarkWorkVerifyAction::Finish(_) => {}
        },
    }
}
