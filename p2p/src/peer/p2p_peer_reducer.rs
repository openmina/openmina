use openmina_core::{bug_condition, Substate};
use redux::{ActionWithMeta, Timestamp};

use crate::{P2pPeerState, P2pPeerStatus, P2pPeerStatusReady, P2pState};

use super::P2pPeerAction;

impl P2pPeerState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pPeerAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let p2p_state = state_context.get_substate_mut()?;
        let (action, meta) = action.split();

        match action {
            P2pPeerAction::Discovered { peer_id, dial_opts } => {
                // TODO: add bound to peers
                let peer_state = p2p_state
                    .peers
                    .entry(*peer_id)
                    .or_insert_with(|| P2pPeerState {
                        is_libp2p: true,
                        dial_opts: dial_opts.clone(),
                        identify: None,
                        status: P2pPeerStatus::Disconnected {
                            time: Timestamp::ZERO,
                        },
                    });

                if let Some(dial_opts) = dial_opts {
                    peer_state.dial_opts.get_or_insert(dial_opts.clone());
                }
                Ok(())
            }
            P2pPeerAction::Ready { peer_id, incoming } => {
                let Some(peer) = p2p_state.peers.get_mut(peer_id) else {
                    return Ok(());
                };
                peer.status = P2pPeerStatus::Ready(P2pPeerStatusReady::new(
                    *incoming,
                    meta.time(),
                    &p2p_state.config.enabled_channels,
                ));

                Ok(())
            }
            P2pPeerAction::BestTipUpdate { peer_id, best_tip } => {
                let Some(peer) = p2p_state.get_ready_peer_mut(peer_id) else {
                    bug_condition!("Peer state not found for `P2pPeerAction::BestTipUpdate`");
                    return Ok(());
                };
                peer.best_tip = Some(best_tip.clone());

                Ok(())
            }
        }
    }
}
