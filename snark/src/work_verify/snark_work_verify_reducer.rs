use openmina_core::{Substate, SubstateAccess};
use redux::EnablingCondition;

use crate::work_verify_effectful::SnarkWorkVerifyEffectfulAction;

use super::{
    SnarkWorkVerifyAction, SnarkWorkVerifyActionWithMetaRef, SnarkWorkVerifyState,
    SnarkWorkVerifyStatus,
};

pub fn reducer<State, Action>(
    mut state: Substate<Action, State, SnarkWorkVerifyState>,
    action: SnarkWorkVerifyActionWithMetaRef<'_>,
) where
    State: SubstateAccess<SnarkWorkVerifyState> + SubstateAccess<crate::SnarkState>,
    Action: From<SnarkWorkVerifyAction>
        + From<SnarkWorkVerifyEffectfulAction>
        + From<redux::AnyAction>
        + EnablingCondition<State>,
{
    let (action, meta) = action.split();
    match action {
        SnarkWorkVerifyAction::Init {
            batch,
            sender,
            req_id,
            ..
        } => {
            state.jobs.add(SnarkWorkVerifyStatus::Init {
                time: meta.time(),
                batch: batch.clone(),
                sender: sender.clone(),
            });

            // Dispatch
            let verifier_index = state.verifier_index.clone();
            let verifier_srs = state.verifier_srs.clone();
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkWorkVerifyEffectfulAction::Init {
                req_id: *req_id,
                sender: sender.clone(),
                batch: batch.clone(),
                verifier_index,
                verifier_srs,
            });
            dispatcher.push(SnarkWorkVerifyAction::Pending { req_id: *req_id });
        }
        SnarkWorkVerifyAction::Pending { req_id } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                *req = match req {
                    SnarkWorkVerifyStatus::Init { batch, sender, .. } => {
                        SnarkWorkVerifyStatus::Pending {
                            time: meta.time(),
                            batch: std::mem::take(batch),
                            sender: std::mem::take(sender),
                        }
                    }
                    _ => return,
                };
            }
        }
        SnarkWorkVerifyAction::Error { req_id, error } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                if let SnarkWorkVerifyStatus::Pending { batch, sender, .. } = req {
                    *req = SnarkWorkVerifyStatus::Error {
                        time: meta.time(),
                        batch: std::mem::take(batch),
                        sender: std::mem::take(sender),
                        error: error.clone(),
                    }
                }
            }

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkWorkVerifyAction::Finish { req_id: *req_id });
        }
        SnarkWorkVerifyAction::Success { req_id } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                if let SnarkWorkVerifyStatus::Pending { batch, sender, .. } = req {
                    *req = SnarkWorkVerifyStatus::Success {
                        time: meta.time(),
                        batch: std::mem::take(batch),
                        sender: std::mem::take(sender),
                    };
                }
            }

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkWorkVerifyAction::Finish { req_id: *req_id });
        }
        SnarkWorkVerifyAction::Finish { req_id } => {
            state.jobs.remove(*req_id);
        }
    }
}
