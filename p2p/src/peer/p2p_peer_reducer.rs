use openmina_core::Substate;
use redux::{ActionWithMeta, Timestamp};

use crate::{
    channels::{
        best_tip::P2pChannelsBestTipAction, rpc::P2pChannelsRpcAction,
        snark::P2pChannelsSnarkAction, snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
        streaming_rpc::P2pChannelsStreamingRpcAction, transaction::P2pChannelsTransactionAction,
        ChannelId,
    },
    P2pPeerState, P2pPeerStatus, P2pPeerStatusReady, P2pState,
};

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

                let dispatcher = state_context.into_dispatcher();
                let peer_id = *peer_id;

                // Dispatches can be done without a loop, but inside we do
                // exhaustive matching so that we don't miss any channels.
                for id in ChannelId::iter_all() {
                    match id {
                        ChannelId::BestTipPropagation => {
                            dispatcher.push(P2pChannelsBestTipAction::Init { peer_id });
                        }
                        ChannelId::TransactionPropagation => {
                            dispatcher.push(P2pChannelsTransactionAction::Init { peer_id });
                        }
                        ChannelId::SnarkPropagation => {
                            dispatcher.push(P2pChannelsSnarkAction::Init { peer_id });
                        }
                        ChannelId::SnarkJobCommitmentPropagation => {
                            dispatcher.push(P2pChannelsSnarkJobCommitmentAction::Init { peer_id });
                        }
                        ChannelId::Rpc => {
                            dispatcher.push(P2pChannelsRpcAction::Init { peer_id });
                        }
                        ChannelId::StreamingRpc => {
                            dispatcher.push(P2pChannelsStreamingRpcAction::Init { peer_id });
                        }
                    }
                }

                Ok(())
            }
            P2pPeerAction::BestTipUpdate { peer_id, best_tip } => {
                let Some(peer) = p2p_state.get_ready_peer_mut(peer_id) else {
                    return Ok(());
                };
                peer.best_tip = Some(best_tip.clone());

                Ok(())
            }
        }
    }
}
