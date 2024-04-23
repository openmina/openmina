use redux::ActionMeta;

use crate::ledger::write::{LedgerWriteAction, LedgerWriteRequest};
use crate::p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};
use crate::Store;

use super::TransitionFrontierSyncLedgerStagedAction;

impl TransitionFrontierSyncLedgerStagedAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        match self {
            TransitionFrontierSyncLedgerStagedAction::PartsFetchPending => {
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit => {
                let state = store.state();
                let Some(staged_ledger) =
                    None.or_else(|| state.transition_frontier.sync.ledger()?.staged())
                else {
                    return;
                };
                let block_hash = staged_ledger.target().staged.block_hash.clone();

                let ready_peers = staged_ledger
                    .filter_available_peers(state.p2p.ready_rpc_peers_iter())
                    .collect::<Vec<_>>();

                for (peer_id, rpc_id) in ready_peers {
                    // TODO(binier): maybe
                    if store.dispatch(P2pChannelsRpcAction::RequestSend {
                        peer_id,
                        id: rpc_id,
                        request: P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                            block_hash.clone(),
                        ),
                    }) {
                        store.dispatch(
                            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending {
                                peer_id,
                                rpc_id,
                            },
                        );
                        break;
                    }
                }
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError { .. } => {
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess {
                peer_id,
                parts,
                ..
            } => {
                if !store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsPeerValid {
                    sender: peer_id,
                }) {
                    store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid {
                        sender: peer_id,
                        parts,
                    });
                }
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid { .. } => {
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerValid { sender } => {
                store.dispatch(
                    TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess { sender },
                );
            }
            TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess { .. } => {
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::ReconstructInit);
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty => {
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::ReconstructInit);
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructInit => {
                let ledger_state = store.state().transition_frontier.sync.ledger();
                let Some((target, parts)) =
                    ledger_state.and_then(|s| s.staged()?.target_with_parts())
                else {
                    return;
                };
                let snarked_ledger_hash = target.snarked_ledger_hash.clone();
                let parts = parts.cloned();

                if store.dispatch(LedgerWriteAction::Init {
                    request: LedgerWriteRequest::StagedLedgerReconstruct {
                        snarked_ledger_hash,
                        parts,
                    },
                }) {
                    store.dispatch(TransitionFrontierSyncLedgerStagedAction::ReconstructPending);
                }
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess { .. } => {
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::Success);
            }
            _ => {}
        }
    }
}
