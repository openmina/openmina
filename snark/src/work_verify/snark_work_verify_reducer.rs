use openmina_core::{bug_condition, Substate, SubstateAccess};
use redux::EnablingCondition;

use crate::work_verify_effectful::SnarkWorkVerifyEffectfulAction;

use super::{
    SnarkWorkVerifyAction, SnarkWorkVerifyActionWithMetaRef, SnarkWorkVerifyState,
    SnarkWorkVerifyStatus,
};

pub fn reducer<State, Action>(
    mut state_context: Substate<Action, State, SnarkWorkVerifyState>,
    action: SnarkWorkVerifyActionWithMetaRef<'_>,
) where
    State: SubstateAccess<SnarkWorkVerifyState> + SubstateAccess<crate::SnarkState>,
    Action: From<SnarkWorkVerifyAction>
        + From<SnarkWorkVerifyEffectfulAction>
        + From<redux::AnyAction>
        + EnablingCondition<State>,
{
    let Ok(state) = state_context.get_substate_mut() else {
        // TODO: log or propagate
        return;
    };
    let (action, meta) = action.split();

    match action {
        SnarkWorkVerifyAction::Init {
            batch,
            sender,
            req_id,
            on_error,
            on_success,
        } => {
            state.jobs.add(SnarkWorkVerifyStatus::Init {
                time: meta.time(),
                batch: batch.clone(),
                sender: sender.clone(),
                on_error: on_error.clone(),
                on_success: on_success.clone(),
            });

            // Dispatch
            let verifier_index = state.verifier_index.clone();
            let verifier_srs = state.verifier_srs.clone();
            let dispatcher = state_context.into_dispatcher();
            dispatcher.push(SnarkWorkVerifyEffectfulAction::Init {
                req_id: *req_id,
                batch: batch.clone(),
                verifier_index,
                verifier_srs,
            });
            dispatcher.push(SnarkWorkVerifyAction::Pending { req_id: *req_id });
        }
        SnarkWorkVerifyAction::Pending { req_id } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                *req = match req {
                    SnarkWorkVerifyStatus::Init {
                        batch,
                        sender,
                        on_error,
                        on_success,
                        ..
                    } => SnarkWorkVerifyStatus::Pending {
                        time: meta.time(),
                        batch: std::mem::take(batch),
                        sender: std::mem::take(sender),
                        on_error: on_error.clone(),
                        on_success: on_success.clone(),
                    },
                    _ => return,
                };
            }
        }
        SnarkWorkVerifyAction::Error { req_id, error } => {
            let Some(req) = state.jobs.get_mut(*req_id) else {
                bug_condition!(
                    "Invalid state for `SnarkWorkVerifyAction::Error` job not found with id: {}",
                    req_id
                );
                return;
            };
            let SnarkWorkVerifyStatus::Pending {
                batch,
                sender,
                on_error,
                ..
            } = req
            else {
                bug_condition!(
                    "Invalid state of `SnarkWorkVerifyStatus` for `SnarkWorkVerifyAction::Error`"
                );
                return;
            };
            let callback = on_error.clone();
            let sender = std::mem::take(sender);
            let batch = std::mem::take(batch);
            *req = SnarkWorkVerifyStatus::Error {
                time: meta.time(),
                batch: batch.clone(),
                sender: sender.clone(),
                error: error.clone(),
            };
            // Dispatch
            let dispatcher = state_context.into_dispatcher();
            dispatcher.push_callback(callback, (*req_id, sender, batch));
            dispatcher.push(SnarkWorkVerifyAction::Finish { req_id: *req_id });
        }
        SnarkWorkVerifyAction::Success { req_id } => {
            let Some(req) = state.jobs.get_mut(*req_id) else {
                bug_condition!(
                    "Invalid state for `SnarkWorkVerifyAction::Success` job not found with id: {}",
                    req_id
                );
                return;
            };
            let SnarkWorkVerifyStatus::Pending {
                batch,
                sender,
                on_success,
                ..
            } = req
            else {
                bug_condition!(
                    "Invalid state of `SnarkWorkVerifyStatus` for `SnarkWorkVerifyAction::Error`"
                );
                return;
            };

            let callback = on_success.clone();
            let sender = std::mem::take(sender);
            let batch = std::mem::take(batch);

            *req = SnarkWorkVerifyStatus::Success {
                time: meta.time(),
                batch: batch.clone(),
                sender: sender.clone(),
            };

            // Dispatch
            let dispatcher = state_context.into_dispatcher();
            dispatcher.push_callback(callback, (*req_id, sender, batch));
            dispatcher.push(SnarkWorkVerifyAction::Finish { req_id: *req_id });
        }
        SnarkWorkVerifyAction::Finish { req_id } => {
            state.jobs.remove(*req_id);
        }
    }
}
