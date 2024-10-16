use openmina_core::Substate;
use redux::ActionWithMeta;

use super::{
    incoming::P2pConnectionIncomingState, outgoing::P2pConnectionOutgoingState,
    P2pConnectionAction, P2pConnectionState,
};
use crate::P2pState;

impl P2pConnectionState {
    pub fn reducer<Action, State>(
        state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<P2pConnectionAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();

        match action {
            P2pConnectionAction::Outgoing(action) => {
                P2pConnectionOutgoingState::reducer(state_context, meta.with_action(action))
            }
            P2pConnectionAction::Incoming(action) => {
                P2pConnectionIncomingState::reducer(state_context, meta.with_action(action))
            }
        }
    }
}
