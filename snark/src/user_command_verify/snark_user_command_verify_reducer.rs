use openmina_core::{bug_condition, Substate, SubstateAccess};
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
            commands,
            req_id,
            from_rpc,
            on_success,
            on_error,
        } => {
            let substate = state.get_substate_mut().unwrap();

            substate.jobs.add(SnarkUserCommandVerifyStatus::Init {
                time: meta.time(),
                commands: commands.clone(),
                from_rpc: *from_rpc,
                on_success: on_success.clone(),
                on_error: on_error.clone(),
            });

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push(SnarkUserCommandVerifyEffectfulAction::Init {
                req_id: *req_id,
                commands: commands.clone(),
            });
            dispatcher.push(SnarkUserCommandVerifyAction::Pending { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Pending { req_id } => {
            let substate = state.get_substate_mut().unwrap();

            let Some(req) = substate.jobs.get_mut(*req_id) else {
                bug_condition!("State for job not found in SnarkUserCommandVerifyAction::Pending");
                return;
            };
            let SnarkUserCommandVerifyStatus::Init {
                commands,
                from_rpc,
                on_success,
                on_error,
                ..
            } = req
            else {
                bug_condition!("Unexpected state in SnarkUserCommandVerifyAction::Pending");
                return;
            };

            *req = SnarkUserCommandVerifyStatus::Pending {
                time: meta.time(),
                commands: std::mem::take(commands),
                from_rpc: std::mem::take(from_rpc),
                on_success: on_success.clone(),
                on_error: on_error.clone(),
            };
        }
        SnarkUserCommandVerifyAction::Error { req_id, error } => {
            let substate = state.get_substate_mut().unwrap();

            let Some(req) = substate.jobs.get_mut(*req_id) else {
                bug_condition!("State for job not found in SnarkUserCommandVerifyAction::Error");
                return;
            };
            let SnarkUserCommandVerifyStatus::Pending { commands, .. } = req else {
                bug_condition!("Unexpected state in SnarkUserCommandVerifyAction::Error");
                return;
            };

            *req = SnarkUserCommandVerifyStatus::Error {
                time: meta.time(),
                commands: std::mem::take(commands),
                error: error.clone(),
            };

            // Dispatch
            let dispatcher = state.into_dispatcher();
            // TODO: dispatch on error callback
            dispatcher.push(SnarkUserCommandVerifyAction::Finish { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Success { req_id, commands } => {
            let substate = state.get_substate_mut().unwrap();
            let Some(req) = substate.jobs.get_mut(*req_id) else {
                bug_condition!("State for job not found in SnarkUserCommandVerifyAction::Success");
                return;
            };
            let SnarkUserCommandVerifyStatus::Pending {
                from_rpc,
                on_success,
                ..
            } = req
            else {
                bug_condition!("Unexpected state in SnarkUserCommandVerifyAction::Success");
                return;
            };

            let from_rpc = std::mem::take(from_rpc);
            let commands: Vec<ledger::scan_state::transaction_logic::valid::UserCommand> =
                commands.clone();
            let on_success = on_success.clone();

            *req = SnarkUserCommandVerifyStatus::Success {
                time: meta.time(),
                commands: commands.clone(), // std::mem::take(commands),
            };

            // Dispatch
            let dispatcher = state.into_dispatcher();
            dispatcher.push_callback(on_success, (*req_id, commands, from_rpc));
            dispatcher.push(SnarkUserCommandVerifyAction::Finish { req_id: *req_id });
        }
        SnarkUserCommandVerifyAction::Finish { req_id } => {
            let substate = state.get_substate_mut().unwrap();
            substate.jobs.remove(*req_id);
        }
    }
}
