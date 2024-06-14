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
        SnarkUserCommandVerifyAction::Receive { commands, nonce } => {
            todo!()
        },
        SnarkUserCommandVerifyAction::Init {
            commands,
            sender,
            req_id,
            ..
        } => {
            state.jobs.add(SnarkUserCommandVerifyStatus::Init {
                time: meta.time(),
                commands: commands.clone(),
                sender: sender.clone(),
            });

            // Dispatch
            let verifier_index = state.verifier_index.clone();
            let verifier_srs = state.verifier_srs.clone();
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkUserCommandVerifyEffectfulAction::Init {
                req_id: *req_id,
                sender: sender.clone(),
                commands: commands.clone(),
                verifier_index,
                verifier_srs,
            });
            dispatcher.push(SnarkUserCommandVerifyAction::Pending { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Pending { req_id } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                *req = match req {
                    SnarkUserCommandVerifyStatus::Init {
                        commands, sender, ..
                    } => SnarkUserCommandVerifyStatus::Pending {
                        time: meta.time(),
                        commands: std::mem::take(commands),
                        sender: std::mem::take(sender),
                    },
                    _ => return,
                };
            }
        }
        SnarkUserCommandVerifyAction::Error { req_id, error } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                if let SnarkUserCommandVerifyStatus::Pending {
                    commands, sender, ..
                } = req
                {
                    *req = SnarkUserCommandVerifyStatus::Error {
                        time: meta.time(),
                        commands: std::mem::take(commands),
                        sender: std::mem::take(sender),
                        error: error.clone(),
                    }
                }
            }

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkUserCommandVerifyAction::Finish { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Success { req_id } => {
            if let Some(req) = state.jobs.get_mut(*req_id) {
                if let SnarkUserCommandVerifyStatus::Pending {
                    commands, sender, ..
                } = req
                {
                    *req = SnarkUserCommandVerifyStatus::Success {
                        time: meta.time(),
                        commands: std::mem::take(commands),
                        sender: std::mem::take(sender),
                    };
                }
            }

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkUserCommandVerifyAction::Finish { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Finish { req_id } => {
            state.jobs.remove(*req_id);
        }
    }
}
