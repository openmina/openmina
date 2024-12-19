use super::{stream::P2pNetworkIdentifyStreamState, P2pNetworkIdentifyAction};
use crate::P2pLimits;
use openmina_core::Substate;
use redux::ActionWithMeta;

impl super::P2pNetworkIdentifyState {
    pub fn reducer<Action, State>(
        state_context: Substate<Action, State, Self>,
        action: ActionWithMeta<P2pNetworkIdentifyAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        match action {
            P2pNetworkIdentifyAction::Stream(action) => P2pNetworkIdentifyStreamState::reducer(
                state_context,
                meta.with_action(action),
                limits,
            ),
        }
    }
}
