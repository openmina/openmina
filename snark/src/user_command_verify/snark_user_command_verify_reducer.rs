use openmina_core::{Substate, SubstateAccess};
use redux::EnablingCondition;

use crate::user_command_verify_effectful::SnarkUserCommandVerifyEffectfulAction;

use super::{
    SnarkUserCommandVerifyAction, SnarkUserCommandVerifyActionWithMetaRef,
    SnarkUserCommandVerifyState, SnarkUserCommandVerifyStatus,
};

pub fn reducer<State, Action>(
    mut state: Substate<Action, State, SnarkUserCommandVerifyState>,
    action: SnarkUserCommandVerifyActionWithMetaRef<'_>,
) where
    State: SubstateAccess<SnarkUserCommandVerifyState> + SubstateAccess<crate::SnarkState>,
    Action: From<SnarkUserCommandVerifyAction>
        + From<SnarkUserCommandVerifyEffectfulAction>
        + From<redux::AnyAction>
        + EnablingCondition<State>,
{
    let (action, meta) = action.split();
    match action {
        SnarkUserCommandVerifyAction::Init {
            commands, req_id, ..
        } => {
            let substate = state.get_substate_mut().unwrap();

            substate.jobs.add(SnarkUserCommandVerifyStatus::Init {
                time: meta.time(),
                commands: commands.clone(),
            });

            // Dispatch
            let verifier_index = substate.verifier_index.clone();
            let verifier_srs = substate.verifier_srs.clone();
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkUserCommandVerifyEffectfulAction::Init {
                req_id: *req_id,
                commands: commands.clone(),
                verifier_index,
                verifier_srs,
            });
            dispatcher.push(SnarkUserCommandVerifyAction::Pending { req_id: *req_id });
        }
        // TODO
        #[allow(unreachable_code)]
        SnarkUserCommandVerifyAction::Pending { req_id } => {
            let substate = state.get_substate_mut().unwrap();

            if let Some(req) = substate.jobs.get_mut(*req_id) {
                *req = match req {
                    SnarkUserCommandVerifyStatus::Init { .. } => {
                        SnarkUserCommandVerifyStatus::Pending {
                            time: meta.time(),
                            commands: todo!(),
                            // commands: std::mem::take(commands),
                        }
                    }
                    _ => return,
                };
            }
        }
        SnarkUserCommandVerifyAction::Error { req_id, error } => {
            let substate = state.get_substate_mut().unwrap();

            if let Some(req) = substate.jobs.get_mut(*req_id) {
                if let SnarkUserCommandVerifyStatus::Pending { commands, .. } = req {
                    *req = SnarkUserCommandVerifyStatus::Error {
                        time: meta.time(),
                        commands: std::mem::take(commands),
                        error: error.clone(),
                    }
                }
            }

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkUserCommandVerifyAction::Finish { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Success { req_id } => {
            let substate = state.get_substate_mut().unwrap();

            if let Some(req) = substate.jobs.get_mut(*req_id) {
                if let SnarkUserCommandVerifyStatus::Pending { commands, .. } = req {
                    *req = SnarkUserCommandVerifyStatus::Success {
                        time: meta.time(),
                        commands: std::mem::take(commands),
                    };
                }
            }

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkUserCommandVerifyAction::Finish { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Finish { req_id } => {
            let substate = state.get_substate_mut().unwrap();
            substate.jobs.remove(*req_id);
        }
    }
}
