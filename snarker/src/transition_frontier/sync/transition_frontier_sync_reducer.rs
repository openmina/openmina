use std::collections::{BTreeMap, VecDeque};

use shared::block::ArcBlockWithHash;

use crate::TransitionFrontierConfig;

use super::{
    ledger::{
        snarked::TransitionFrontierSyncLedgerSnarkedState, TransitionFrontierSyncLedgerState,
    },
    PeerRpcState, TransitionFrontierSyncAction, TransitionFrontierSyncActionWithMetaRef,
    TransitionFrontierSyncBlockState, TransitionFrontierSyncState,
};

impl TransitionFrontierSyncState {
    pub fn reducer(
        &mut self,
        action: TransitionFrontierSyncActionWithMetaRef<'_>,
        config: &TransitionFrontierConfig,
        best_chain: &[ArcBlockWithHash],
    ) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncAction::Init(a) => {
                *self = Self::Init {
                    time: meta.time(),
                    best_tip: a.best_tip.clone(),
                    root_block: a.root_block.clone(),
                    blocks_inbetween: a.blocks_inbetween.clone(),
                };
            }
            // TODO(binier): refactor
            TransitionFrontierSyncAction::BestTipUpdate(a) => match self {
                Self::RootLedgerPending {
                    best_tip,
                    blocks_inbetween,
                    root_ledger,
                    ..
                } => {
                    root_ledger.update_block(meta.time(), a.root_block.clone());

                    *best_tip = a.best_tip.clone();
                    *blocks_inbetween = a.blocks_inbetween.clone();
                }
                Self::BlocksPending { chain, .. } => {
                    let mut applied_blocks: BTreeMap<_, _> =
                        best_chain.iter().map(|b| (&b.hash, b)).collect();

                    let old_chain = VecDeque::from(std::mem::take(chain));
                    let old_root = old_chain.front().and_then(|b| b.block()).unwrap().clone();
                    let new_root = &a.root_block;

                    let old_chain_has_new_root_applied = old_chain
                        .iter()
                        .find(|b| b.block_hash() == &new_root.hash)
                        .map_or(false, |b| b.is_apply_pending() || b.is_apply_success());

                    if applied_blocks.contains_key(&new_root.hash) || old_chain_has_new_root_applied
                    {
                        let mut old_block_states: BTreeMap<_, _> = old_chain
                            .into_iter()
                            .map(|b| (b.block_hash().clone(), b))
                            .collect();

                        let mut push_block = |hash, maybe_block: Option<&ArcBlockWithHash>| {
                            chain.push({
                                if let Some(old_state) =
                                    old_block_states.remove(hash).filter(|old_state| {
                                        old_state.block().is_some() || maybe_block.is_none()
                                    })
                                {
                                    old_state
                                } else if let Some(block) = applied_blocks.remove(hash) {
                                    TransitionFrontierSyncBlockState::ApplySuccess {
                                        time: meta.time(),
                                        block: block.clone(),
                                    }
                                } else if let Some(block) = maybe_block {
                                    TransitionFrontierSyncBlockState::FetchSuccess {
                                        time: meta.time(),
                                        block: block.clone(),
                                    }
                                } else {
                                    TransitionFrontierSyncBlockState::FetchPending {
                                        time: meta.time(),
                                        block_hash: hash.clone(),
                                        attempts: Default::default(),
                                    }
                                }
                            })
                        };

                        push_block(&a.root_block.hash, Some(&a.root_block));
                        for hash in &a.blocks_inbetween {
                            push_block(hash, None);
                        }
                        push_block(&a.best_tip.hash, Some(&a.best_tip));
                    } else {
                        let cur_best_root = best_chain.first();
                        let root_ledger = if old_root.snarked_ledger_hash()
                            == new_root.snarked_ledger_hash()
                            || cur_best_root.map_or(false, |cur| {
                                cur.snarked_ledger_hash() == new_root.snarked_ledger_hash()
                            }) {
                            TransitionFrontierSyncLedgerSnarkedState::Success {
                                time: meta.time(),
                                block: new_root.clone(),
                            }
                            .into()
                        } else {
                            TransitionFrontierSyncLedgerSnarkedState::pending(
                                meta.time(),
                                new_root.clone(),
                            )
                            .into()
                        };
                        *self = Self::RootLedgerPending {
                            time: meta.time(),
                            best_tip: a.best_tip.clone(),
                            blocks_inbetween: a.blocks_inbetween.clone(),
                            root_ledger,
                        };
                    }
                }
                Self::Synced { time, .. } => {
                    let applied_blocks: BTreeMap<_, _> =
                        best_chain.iter().map(|b| (&b.hash, b)).collect();

                    let old_root = best_chain.first().unwrap();
                    let new_root = &a.root_block;

                    if applied_blocks.contains_key(&new_root.hash) {
                        let chain = std::iter::once(a.root_block.hash())
                            .chain(&a.blocks_inbetween)
                            .chain(std::iter::once(a.best_tip.hash()))
                            .map(|hash| match applied_blocks.get(hash) {
                                Some(&block) => TransitionFrontierSyncBlockState::ApplySuccess {
                                    time: *time,
                                    block: block.clone(),
                                },
                                None if hash == a.best_tip.hash() => {
                                    TransitionFrontierSyncBlockState::FetchSuccess {
                                        time: meta.time(),
                                        block: a.best_tip.clone(),
                                    }
                                }
                                None => TransitionFrontierSyncBlockState::FetchPending {
                                    time: meta.time(),
                                    block_hash: hash.clone(),
                                    attempts: Default::default(),
                                },
                            })
                            .collect::<Vec<_>>();
                        *self = Self::BlocksPending {
                            time: meta.time(),
                            chain,
                        };
                    } else {
                        let root_ledger =
                            if old_root.snarked_ledger_hash() == new_root.snarked_ledger_hash() {
                                TransitionFrontierSyncLedgerSnarkedState::Success {
                                    time: meta.time(),
                                    block: new_root.clone(),
                                }
                                .into()
                            } else {
                                TransitionFrontierSyncLedgerSnarkedState::pending(
                                    meta.time(),
                                    new_root.clone(),
                                )
                                .into()
                            };
                        *self = Self::RootLedgerPending {
                            time: meta.time(),
                            best_tip: a.best_tip.clone(),
                            blocks_inbetween: a.blocks_inbetween.clone(),
                            root_ledger,
                        }
                    }
                }
                _ => return,
            },
            TransitionFrontierSyncAction::LedgerRootPending(_) => {
                if let Self::Init {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    ..
                } = self
                {
                    *self = Self::RootLedgerPending {
                        time: meta.time(),
                        best_tip: best_tip.clone(),
                        root_ledger: TransitionFrontierSyncLedgerState::Init {
                            time: meta.time(),
                            block: root_block.clone(),
                        },
                        blocks_inbetween: std::mem::take(blocks_inbetween),
                    };
                }
            }
            TransitionFrontierSyncAction::LedgerRootSuccess(_) => {
                if let Self::RootLedgerPending {
                    best_tip,
                    blocks_inbetween,
                    root_ledger,
                    ..
                } = self
                {
                    *self = Self::RootLedgerSuccess {
                        time: meta.time(),
                        best_tip: best_tip.clone(),
                        root_block: root_ledger.block().clone(),
                        blocks_inbetween: std::mem::take(blocks_inbetween),
                    };
                }
            }
            TransitionFrontierSyncAction::BlocksPending(_) => {
                let Self::RootLedgerSuccess {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    ..
                } = self else { return };
                let (best_tip, root_block) = (best_tip.clone(), root_block.clone());
                let blocks_inbetween = std::mem::take(blocks_inbetween);

                let mut applied_blocks: BTreeMap<_, _> =
                    best_chain.iter().map(|b| (&b.hash, b)).collect();

                let mut chain = Vec::with_capacity(config.k());

                // TODO(binier): maybe time should be when we originally
                // applied this block? Same for below.

                // Root block is always applied since we have reconstructed it
                // in previous steps.
                chain.push(TransitionFrontierSyncBlockState::ApplySuccess {
                    time: meta.time(),
                    block: root_block,
                });

                chain.extend(blocks_inbetween.into_iter().map(|block_hash| {
                    if let Some(block) = applied_blocks.remove(&block_hash) {
                        TransitionFrontierSyncBlockState::ApplySuccess {
                            time: meta.time(),
                            block: (*block).clone(),
                        }
                    } else {
                        TransitionFrontierSyncBlockState::FetchPending {
                            time: meta.time(),
                            block_hash,
                            attempts: Default::default(),
                        }
                    }
                }));

                chain.push(TransitionFrontierSyncBlockState::FetchSuccess {
                    time: meta.time(),
                    block: best_tip,
                });

                *self = Self::BlocksPending {
                    time: meta.time(),
                    chain,
                };
            }
            TransitionFrontierSyncAction::BlocksPeersQuery(_) => {}
            TransitionFrontierSyncAction::BlocksPeerQueryInit(a) => {
                let Some(block_state) = self.block_state_mut(&a.hash) else { return };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else { return };
                attempts.insert(a.peer_id.clone(), PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierSyncAction::BlocksPeerQueryRetry(a) => {
                let Some(block_state) = self.block_state_mut(&a.hash) else { return };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else { return };
                attempts.insert(a.peer_id.clone(), PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierSyncAction::BlocksPeerQueryPending(a) => {
                let Some(block_state) = self.block_state_mut(&a.hash) else { return };
                let Some(peer_state) = block_state.fetch_pending_from_peer_mut(&a.peer_id) else { return };
                *peer_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: a.rpc_id,
                };
            }
            TransitionFrontierSyncAction::BlocksPeerQueryError(a) => {
                let Self::BlocksPending { chain, .. } = self else { return };
                let Some(peer_state) = chain.iter_mut()
                    .find_map(|b| b.fetch_pending_from_peer_mut(&a.peer_id))
                    else { return };
                *peer_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: a.rpc_id,
                    error: a.error.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksPeerQuerySuccess(a) => {
                let Some(block_state) = self.block_state_mut(&a.response.hash) else { return };
                let Some(peer_state) = block_state.fetch_pending_from_peer_mut(&a.peer_id) else { return };
                *peer_state = PeerRpcState::Success {
                    time: meta.time(),
                    block: a.response.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksFetchSuccess(a) => {
                let Some(block_state) = self.block_state_mut(&a.hash) else { return };
                let Some(block) = block_state.fetch_pending_fetched_block() else { return };
                *block_state = TransitionFrontierSyncBlockState::FetchSuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksNextApplyInit(_) => {}
            TransitionFrontierSyncAction::BlocksNextApplyPending(a) => {
                let Some(block_state) = self.block_state_mut(&a.hash) else { return };
                let Some(block) = block_state.block() else { return };

                *block_state = TransitionFrontierSyncBlockState::ApplyPending {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksNextApplySuccess(a) => {
                let Some(block_state) = self.block_state_mut(&a.hash) else { return };
                let Some(block) = block_state.block() else { return };

                *block_state = TransitionFrontierSyncBlockState::ApplySuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksSuccess(_) => {
                let Self::BlocksPending { chain, .. } = self else { return };
                let chain = std::mem::take(chain)
                    .into_iter()
                    .rev()
                    .take(config.k() + 1)
                    .rev()
                    .filter_map(|v| v.take_block())
                    .collect();

                *self = Self::BlocksSuccess {
                    time: meta.time(),
                    chain,
                };
            }
            TransitionFrontierSyncAction::Ledger(a) => match self {
                Self::RootLedgerPending { root_ledger, .. } => {
                    root_ledger.reducer(meta.with_action(a));
                }
                _ => {}
            },
        }
    }
}
