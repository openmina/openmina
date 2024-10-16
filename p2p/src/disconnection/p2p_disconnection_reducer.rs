use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    disconnection_effectful::P2pDisconnectionEffectfulAction, P2pNetworkSchedulerAction,
    P2pPeerAction, P2pPeerStatus, P2pState,
};

use super::{P2pDisconnectedState, P2pDisconnectionAction};

impl P2pDisconnectedState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<P2pDisconnectionAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;

        match action {
            P2pDisconnectionAction::Init { peer_id, reason } => {
                #[cfg(feature = "p2p-libp2p")]
                if p2p_state.is_libp2p_peer(&peer_id) {
                    if let Some((&addr, _)) = p2p_state
                        .network
                        .scheduler
                        .connections
                        .iter()
                        .find(|(_, conn_state)| conn_state.peer_id() == Some(&peer_id))
                    {
                        let Some(peer) = p2p_state.peers.get_mut(&peer_id) else {
                            bug_condition!("Invalid state for: `P2pDisconnectionAction::Finish`");
                            return Ok(());
                        };
                        peer.status = P2pPeerStatus::Disconnecting { time: meta.time() };

                        let dispatcher = state_context.into_dispatcher();
                        dispatcher.push(P2pNetworkSchedulerAction::Disconnect { addr, reason });
                        dispatcher.push(P2pDisconnectionAction::Finish { peer_id });
                    }
                    return Ok(());
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pDisconnectionEffectfulAction::Init { peer_id });
                Ok(())
            }
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pDisconnectionAction::Finish { peer_id } => {
                let Some(peer) = p2p_state.peers.get_mut(&peer_id) else {
                    bug_condition!("Invalid state for: `P2pDisconnectionAction::Finish`");
                    return Ok(());
                };
                peer.status = P2pPeerStatus::Disconnected { time: meta.time() };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                dispatcher.push(P2pPeerAction::Remove { peer_id });

                if let Some(callback) = &p2p_state.callbacks.on_p2p_disconnection_finish {
                    dispatcher.push_callback(callback.clone(), peer_id);
                }
                Ok(())
            }
            #[cfg(feature = "p2p-libp2p")]
            P2pDisconnectionAction::Finish { peer_id } => {
                if p2p_state
                    .network
                    .scheduler
                    .connections
                    .iter()
                    .any(|(_addr, conn_state)| {
                        conn_state.peer_id() == Some(&peer_id) && conn_state.closed.is_none()
                    })
                {
                    return Ok(());
                }

                let Some(peer) = p2p_state.peers.get_mut(&peer_id) else {
                    bug_condition!("Invalid state for: `P2pDisconnectionAction::Finish`");
                    return Ok(());
                };
                peer.status = P2pPeerStatus::Disconnected { time: meta.time() };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                dispatcher.push(P2pPeerAction::Remove { peer_id });

                if let Some(callback) = &p2p_state.callbacks.on_p2p_disconnection_finish {
                    dispatcher.push_callback(callback.clone(), peer_id);
                }

                Ok(())
            }
        }
    }
}
