use crate::{
    connection::P2pConnectionState, disconnection::P2pDisconnectionAction, P2pAction,
    P2pActionWithMetaRef, P2pNetworkState, P2pPeerState, P2pPeerStatus, P2pState,
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
            P2pAction::Disconnection(action) => match action {
                P2pDisconnectionAction::Init { .. } => Ok(()),
                P2pDisconnectionAction::Finish { peer_id } => {
                    #[cfg(feature = "p2p-libp2p")]
                    if state
                        .network
                        .scheduler
                        .connections
                        .iter()
                        .any(|(_addr, conn_state)| conn_state.peer_id() == Some(peer_id))
                    {
                        // still have other connections
                        return Ok(());
                    }
                    let Some(peer) = state.peers.get_mut(peer_id) else {
                        return Ok(());
                    };
                    peer.status = P2pPeerStatus::Disconnected { time: meta.time() };
                    Ok(())
                }
            },
            P2pAction::Peer(action) => P2pPeerState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(action),
            ),
            P2pAction::Channels(action) => {
                let Some(peer_id) = action.peer_id() else {
                    return Ok(());
                };
                let is_libp2p = state.is_libp2p_peer(peer_id);
                let Some(peer) = state.get_ready_peer_mut(peer_id) else {
                    return Ok(());
                };
                peer.channels.reducer(meta.with_action(action), is_libp2p);
                Ok(())
            }
            P2pAction::Identify(_action) => {
                #[cfg(feature = "p2p-libp2p")]
                match _action {
                    crate::identify::P2pIdentifyAction::NewRequest { .. } => {}
                    crate::identify::P2pIdentifyAction::UpdatePeerInformation { peer_id, info } => {
                        if let Some(peer) = state.peers.get_mut(peer_id) {
                            peer.identify = Some(*info.clone());
                        }
                    }
                }
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
            P2pAction::ConnectionEffectful(_) => {
                // effectful
                Ok(())
            }
        }
    }
}
