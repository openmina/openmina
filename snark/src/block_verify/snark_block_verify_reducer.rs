use openmina_core::{Substate, SubstateAccess};
use redux::EnablingCondition;

use crate::block_verify_effectful::SnarkBlockVerifyEffectfulAction;

use super::{
    SnarkBlockVerifyAction, SnarkBlockVerifyActionWithMetaRef, SnarkBlockVerifyState,
    SnarkBlockVerifyStatus,
};

pub fn reducer<State, Action>(
    mut state_context: Substate<Action, State, SnarkBlockVerifyState>,
    action: SnarkBlockVerifyActionWithMetaRef<'_>,
) where
    State: SubstateAccess<SnarkBlockVerifyState> + SubstateAccess<crate::SnarkState>,
    Action: From<SnarkBlockVerifyAction>
        + From<SnarkBlockVerifyEffectfulAction>
        + From<redux::AnyAction>
        + EnablingCondition<State>,
{
    let Ok(state) = state_context.get_substate_mut() else {
        // TODO: log or propagate
        return;
    };
    let (action, meta) = action.split();

    match action {
        SnarkBlockVerifyAction::Init {
            block,
            req_id,
            on_success,
            ..
        } => {
            state.jobs.add(SnarkBlockVerifyStatus::Init {
                time: meta.time(),
                block: block.clone(),
                on_success: on_success.clone(),
            });

            // Dispatch
            let verifier_index = state.verifier_index.clone();
            let verifier_srs = state.verifier_srs.clone();
            let dispatcher = state_context.into_dispatcher();
            dispatcher.push(SnarkBlockVerifyEffectfulAction::Init {
                req_id: *req_id,
                block: block.clone(),
                verifier_index,
                verifier_srs,
            });
            dispatcher.push(SnarkBlockVerifyAction::Pending { req_id: *req_id });
        }
        SnarkBlockVerifyAction::Pending { req_id, .. } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                *req = match req {
                    SnarkBlockVerifyStatus::Init {
                        block, on_success, ..
                    } => SnarkBlockVerifyStatus::Pending {
                        time: meta.time(),
                        block: block.clone(),
                        on_success: on_success.clone(),
                    },
                    _ => return,
                };
            }
        }
        SnarkBlockVerifyAction::Error { req_id, error, .. } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                if let SnarkBlockVerifyStatus::Pending { block, .. } = req {
                    *req = SnarkBlockVerifyStatus::Error {
                        time: meta.time(),
                        block: block.clone(),
                        error: error.clone(),
                    };
                }
            }

            // Dispatch
            let dispatcher = state_context.into_dispatcher();
            dispatcher.push(SnarkBlockVerifyAction::Finish { req_id: *req_id });
        }
        SnarkBlockVerifyAction::Success { req_id, .. } => {
            let callback_and_arg = state.jobs.get_mut(*req_id).and_then(|req| {
                if let SnarkBlockVerifyStatus::Pending {
                    block, on_success, ..
                } = req
                {
                    let callback = on_success.clone();
                    let block_hash = block.hash_ref().clone();
                    *req = SnarkBlockVerifyStatus::Success {
                        time: meta.time(),
                        block: block.clone(),
                    };
                    Some((callback, block_hash))
                } else {
                    None
                }
            });

            // Dispatch
            let dispatcher = state_context.into_dispatcher();

            if let Some((callback, block_hash)) = callback_and_arg {
                dispatcher.push_callback(callback, block_hash);
            }

            dispatcher.push(SnarkBlockVerifyAction::Finish { req_id: *req_id });
        }
        SnarkBlockVerifyAction::Finish { req_id, .. } => {
            state.jobs.remove(*req_id);
        }
    }
}
