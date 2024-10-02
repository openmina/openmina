use crate::{
    channels::P2pChannelsState, connection::P2pConnectionState,
    disconnection::P2pDisconnectedState, P2pAction, P2pActionWithMetaRef, P2pNetworkState,
    P2pPeerState, P2pState,
};
use openmina_core::{bug_condition, Substate};

impl P2pState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, Self>,
        action: P2pActionWithMetaRef<'_>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let Ok(state) = state_context.get_substate_mut() else {
            bug_condition!("no P2pState");
            return Ok(());
        };
        let (action, meta) = action.split();

        match action {
            P2pAction::Initialization(_) => {
                // noop
                Ok(())
            }
            P2pAction::Connection(action) => {
                P2pConnectionState::reducer(state_context, meta.with_action(action))
            }
            P2pAction::Disconnection(action) => {
                P2pDisconnectedState::reducer(state_context, meta.with_action(action))
            }
            P2pAction::Peer(action) => P2pPeerState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(action),
            ),
            P2pAction::Channels(action) => {
                P2pChannelsState::reducer(state_context, meta.with_action(action))
            }
            P2pAction::Identify(_action) => {
                #[cfg(feature = "p2p-libp2p")]
                Self::identify_reducer(state_context, meta.with_action(_action))?;
                Ok(())
            }
            P2pAction::Network(_action) => {
                #[cfg(feature = "p2p-libp2p")]
                {
                    let limits = state.config.limits;
                    P2pNetworkState::reducer(
                        Substate::from_compatible_substate(state_context),
                        meta.with_action(_action),
                        &limits,
                    )?;
                }
                Ok(())
            }
            P2pAction::ConnectionEffectful(_)
            | P2pAction::DisconnectionEffectful(_)
            | P2pAction::ChannelsEffectful(_) => {
                // effectful
                Ok(())
            }
        }
    }
}
