use std::time::Duration;

use openmina_core::{bug_condition, pseudo_rng, Substate};
use rand::prelude::*;
use redux::ActionWithMeta;

use crate::{
    disconnection_effectful::P2pDisconnectionEffectfulAction, P2pNetworkSchedulerAction,
    P2pPeerAction, P2pPeerStatus, P2pState,
};

use super::{P2pDisconnectedState, P2pDisconnectionAction, P2pDisconnectionReason};

/// Do not disconnect peer for this duration just for freeing up peer space.
const FORCE_PEER_STABLE_FOR: Duration = Duration::from_secs(90);

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
            P2pDisconnectionAction::RandomTry => {
                p2p_state.last_random_disconnection_try = meta.time();
                if p2p_state.config.limits.max_stable_peers()
                    >= p2p_state.ready_peers_iter().count()
                {
                    return Ok(());
                }
                let mut rng = pseudo_rng(meta.time());

                let peer_id = p2p_state
                    .ready_peers_iter()
                    .filter(|(_, s)| s.connected_for(meta.time()) > FORCE_PEER_STABLE_FOR)
                    .map(|(id, _)| *id)
                    .choose(&mut rng);

                if let Some(peer_id) = peer_id {
                    let dispatcher = state_context.into_dispatcher();
                    dispatcher.push(P2pDisconnectionAction::Init {
                        peer_id,
                        reason: P2pDisconnectionReason::FreeUpSpace,
                    });
                }
                Ok(())
            }
            P2pDisconnectionAction::Init { peer_id, reason } => {
                let Some(peer) = p2p_state.peers.get_mut(&peer_id) else {
                    bug_condition!("Invalid state for: `P2pDisconnectionAction::Init`");
                    return Ok(());
                };
                peer.status = P2pPeerStatus::Disconnecting { time: meta.time() };

                #[cfg(feature = "p2p-libp2p")]
                if peer.is_libp2p() {
                    let connections = p2p_state
                        .network
                        .scheduler
                        .connections
                        .iter()
                        .filter(|(_, conn_state)| conn_state.peer_id() == Some(&peer_id))
                        .map(|(addr, _)| *addr)
                        .collect::<Vec<_>>();

                    let dispatcher = state_context.into_dispatcher();
                    for addr in connections {
                        dispatcher.push(P2pNetworkSchedulerAction::Disconnect {
                            addr,
                            reason: reason.clone(),
                        });
                    }

                    dispatcher.push(P2pDisconnectionAction::Finish { peer_id });
                    return Ok(());
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pDisconnectionEffectfulAction::Init { peer_id });
                Ok(())
            }
            P2pDisconnectionAction::PeerClosed { peer_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pDisconnectionEffectfulAction::Init { peer_id });
                Ok(())
            }
            P2pDisconnectionAction::FailedCleanup { peer_id } => {
                let Some(peer) = p2p_state.peers.get_mut(&peer_id) else {
                    bug_condition!("Invalid state for: `P2pDisconnectionAction::FailedCleanup`");
                    return Ok(());
                };
                peer.status = P2pPeerStatus::Disconnecting { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pDisconnectionEffectfulAction::Init { peer_id });
                Ok(())
            }
            P2pDisconnectionAction::Finish { peer_id } => {
                let Some(peer) = p2p_state.peers.get_mut(&peer_id) else {
                    bug_condition!("Invalid state for: `P2pDisconnectionAction::Finish`");
                    return Ok(());
                };
                if peer.is_libp2p()
                    && p2p_state
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
