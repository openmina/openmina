use openmina_core::{Substate, SubstateAccess};
use redux::EnablingCondition;

use crate::{
    block_verify::{SnarkBlockVerifyAction, SnarkBlockVerifyState},
    block_verify_effectful::SnarkBlockVerifyEffectfulAction,
    work_verify::{SnarkWorkVerifyAction, SnarkWorkVerifyState},
    work_verify_effectful::SnarkWorkVerifyEffectfulAction,
};

use super::{SnarkAction, SnarkActionWithMetaRef, SnarkState};

impl SnarkState {
    pub fn reducer<State, Action>(
        state: Substate<Action, State, Self>,
        action: SnarkActionWithMetaRef<'_>,
    ) where
        State: SubstateAccess<Self>
            + SubstateAccess<SnarkWorkVerifyState>
            + SubstateAccess<SnarkBlockVerifyState>,
        Action: From<SnarkBlockVerifyAction>
            + From<SnarkBlockVerifyEffectfulAction>
            + From<SnarkWorkVerifyAction>
            + From<SnarkWorkVerifyEffectfulAction>
            + From<redux::AnyAction>
            + EnablingCondition<State>,
    {
        let (action, meta) = action.split();
        match action {
            SnarkAction::BlockVerify(a) => crate::block_verify::reducer(
                Substate::from_compatible_substate(state),
                meta.with_action(a),
            ),
            SnarkAction::BlockVerifyEffect(_) => {}
            SnarkAction::WorkVerify(a) => crate::work_verify::reducer(
                Substate::from_compatible_substate(state),
                meta.with_action(a),
            ),
            SnarkAction::WorkVerifyEffect(_) => {}
        }
    }
}
