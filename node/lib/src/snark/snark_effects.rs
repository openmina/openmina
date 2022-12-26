use crate::consensus::ConsensusBlockSnarkVerifySuccessAction;
use crate::{Service, Store};

use super::block_verify::SnarkBlockVerifyAction;
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
    }
}
