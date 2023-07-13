use std::collections::{BTreeMap, VecDeque};

use shared::block::ArcBlockWithHash;

use super::{
    sync::ledger::TransitionFrontierSyncLedgerState, PeerRpcState, TransitionFrontierAction,
    TransitionFrontierActionWithMetaRef, TransitionFrontierState, TransitionFrontierSyncBlockState,
    TransitionFrontierSyncState,
};

impl TransitionFrontierState {
    pub fn reducer(&mut self, action: TransitionFrontierActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierAction::SyncInit(a) => {
                self.sync = TransitionFrontierSyncState::Init {
                    time: meta.time(),
                    best_tip: a.best_tip.clone(),
                    root_block: a.root_block.clone(),
                    blocks_inbetween: a.blocks_inbetween.clone(),
                };
            }
            // TODO(binier): refactor
            TransitionFrontierAction::SyncBestTipUpdate(a) => match &mut self.sync {
                TransitionFrontierSyncState::RootLedgerSyncPending {
                    best_tip,
                    blocks_inbetween,
                    root_ledger,
                    ..
                } => {
                    match root_ledger {
                        TransitionFrontierSyncLedgerState::SnarkedLedgerSyncPending {
                            block,
                            ..
                        } => {
                            if block.snarked_ledger_hash() == a.root_block.snarked_ledger_hash() {
                                *block = a.root_block.clone();
                            } else {
                                *root_ledger = TransitionFrontierSyncLedgerState::Init {
                                    time: meta.time(),
                                    block: a.root_block.clone(),
                                };
                            }
                        }
                        TransitionFrontierSyncLedgerState::StagedLedgerReconstructPending {
                            block,
                            ..
                        } => {
                            if block.snarked_ledger_hash() == a.root_block.snarked_ledger_hash() {
                                *root_ledger =
                                    TransitionFrontierSyncLedgerState::SnarkedLedgerSyncSuccess {
                                        time: meta.time(),
                                        block: a.root_block.clone(),
                                    };
                            } else {
                                *root_ledger = TransitionFrontierSyncLedgerState::Init {
                                    time: meta.time(),
                                    block: a.root_block.clone(),
                                };
                            }
                        }
                        _ => {
                            // should be impossible.
                        }
                    }

                    *best_tip = a.best_tip.clone();
                    *blocks_inbetween = a.blocks_inbetween.clone();
                }
                TransitionFrontierSyncState::BlocksFetchAndApplyPending { chain, .. } => {
                    let mut applied_blocks: BTreeMap<_, _> =
                        self.best_chain.iter().map(|b| (&b.hash, b)).collect();

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
                        let cur_best_root = self.best_chain.first();
                        let root_ledger = if old_root.snarked_ledger_hash()
                            == new_root.snarked_ledger_hash()
                            || cur_best_root.map_or(false, |cur| {
                                cur.snarked_ledger_hash() == new_root.snarked_ledger_hash()
                            }) {
                            TransitionFrontierSyncLedgerState::SnarkedLedgerSyncSuccess {
                                time: meta.time(),
                                block: new_root.clone(),
                            }
                        } else {
                            TransitionFrontierSyncLedgerState::Init {
                                time: meta.time(),
                                block: new_root.clone(),
                            }
                        };
                        self.sync = TransitionFrontierSyncState::RootLedgerSyncPending {
                            time: meta.time(),
                            best_tip: a.best_tip.clone(),
                            blocks_inbetween: a.blocks_inbetween.clone(),
                            root_ledger,
                        }
                    }
                }
                TransitionFrontierSyncState::Synced { time, .. } => {
                    let mut applied_blocks: BTreeMap<_, _> =
                        self.best_chain.iter().map(|b| (&b.hash, b)).collect();

                    let old_root = self.best_chain.first().unwrap();
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
                        self.sync = TransitionFrontierSyncState::BlocksFetchAndApplyPending {
                            time: meta.time(),
                            chain,
                        };
                    } else {
                        let root_ledger =
                            if old_root.snarked_ledger_hash() == new_root.snarked_ledger_hash() {
                                TransitionFrontierSyncLedgerState::SnarkedLedgerSyncSuccess {
                                    time: meta.time(),
                                    block: new_root.clone(),
                                }
                            } else {
                                TransitionFrontierSyncLedgerState::Init {
                                    time: meta.time(),
                                    block: new_root.clone(),
                                }
                            };
                        self.sync = TransitionFrontierSyncState::RootLedgerSyncPending {
                            time: meta.time(),
                            best_tip: a.best_tip.clone(),
                            blocks_inbetween: a.blocks_inbetween.clone(),
                            root_ledger,
                        }
                    }
                }
                _ => return,
            },
            TransitionFrontierAction::RootLedgerSyncPending(_) => {
                if let TransitionFrontierSyncState::Init {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    ..
                } = &mut self.sync
                {
                    self.sync = TransitionFrontierSyncState::RootLedgerSyncPending {
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
            TransitionFrontierAction::RootLedgerSyncSuccess(_) => {
                if let TransitionFrontierSyncState::RootLedgerSyncPending {
                    best_tip,
                    blocks_inbetween,
                    root_ledger,
                    ..
                } = &mut self.sync
                {
                    self.sync = TransitionFrontierSyncState::RootLedgerSyncSuccess {
                        time: meta.time(),
                        best_tip: best_tip.clone(),
                        root_block: root_ledger.block().clone(),
                        blocks_inbetween: std::mem::take(blocks_inbetween),
                    };
                }
            }
            TransitionFrontierAction::SyncBlocksFetchAndApplyPending(_) => {
                let TransitionFrontierSyncState::RootLedgerSyncSuccess {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    ..
                } = &mut self.sync else { return };
                let (best_tip, root_block) = (best_tip.clone(), root_block.clone());
                let blocks_inbetween = std::mem::take(blocks_inbetween);

                let mut applied_blocks: BTreeMap<_, _> =
                    self.best_chain.iter().map(|b| (&b.hash, b)).collect();

                let mut chain = Vec::with_capacity(self.config.k());

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

                self.sync = TransitionFrontierSyncState::BlocksFetchAndApplyPending {
                    time: meta.time(),
                    chain,
                };
            }
            TransitionFrontierAction::SyncBlocksFetchAndApplyPeersQuery(_) => {}
            TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryInit(a) => {
                let Some(block_state) = self.sync.block_state_mut(&a.hash) else { return };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else { return };
                attempts.insert(a.peer_id.clone(), PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryRetry(a) => {
                let Some(block_state) = self.sync.block_state_mut(&a.hash) else { return };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else { return };
                attempts.insert(a.peer_id.clone(), PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryPending(a) => {
                let Some(block_state) = self.sync.block_state_mut(&a.hash) else { return };
                let Some(peer_state) = block_state.fetch_pending_from_peer_mut(&a.peer_id) else { return };
                *peer_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: a.rpc_id,
                };
            }
            TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryError(a) => {
                let TransitionFrontierSyncState::BlocksFetchAndApplyPending { chain, .. } = &mut self.sync else { return };
                let Some(peer_state) = chain.iter_mut()
                    .find_map(|b| b.fetch_pending_from_peer_mut(&a.peer_id))
                    else { return };
                *peer_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: a.rpc_id,
                    error: a.error.clone(),
                };
            }
            TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQuerySuccess(a) => {
                let Some(block_state) = self.sync.block_state_mut(&a.response.hash) else { return };
                let Some(peer_state) = block_state.fetch_pending_from_peer_mut(&a.peer_id) else { return };
                *peer_state = PeerRpcState::Success {
                    time: meta.time(),
                    block: a.response.clone(),
                };
            }
            TransitionFrontierAction::SyncBlockFetchSuccess(a) => {
                let Some(block_state) = self.sync.block_state_mut(&a.hash) else { return };
                let Some(block) = block_state.fetch_pending_fetched_block() else { return };
                *block_state = TransitionFrontierSyncBlockState::FetchSuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierAction::SyncBlockNextApplyInit(_) => {}
            TransitionFrontierAction::SyncBlockApplyPending(a) => {
                let Some(block_state) = self.sync.block_state_mut(&a.hash) else { return };
                let Some(block) = block_state.block() else { return };

                *block_state = TransitionFrontierSyncBlockState::ApplyPending {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierAction::SyncBlockApplySuccess(a) => {
                let Some(block_state) = self.sync.block_state_mut(&a.hash) else { return };
                let Some(block) = block_state.block() else { return };

                *block_state = TransitionFrontierSyncBlockState::ApplySuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierAction::SyncBlocksFetchAndApplySuccess(_) => {
                let TransitionFrontierSyncState::BlocksFetchAndApplyPending { chain, .. } = &mut self.sync else { return };
                let chain = std::mem::take(chain)
                    .into_iter()
                    .rev()
                    .take(self.config.k() + 1)
                    .rev()
                    .filter_map(|v| v.take_block())
                    .collect();

                self.sync = TransitionFrontierSyncState::BlocksFetchAndApplySuccess {
                    time: meta.time(),
                    chain,
                };
            }
            TransitionFrontierAction::Synced(_) => {
                let TransitionFrontierSyncState::BlocksFetchAndApplySuccess { chain, .. } = &mut self.sync else { return };
                self.best_chain = std::mem::take(chain);
                self.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
            }
            TransitionFrontierAction::SyncLedger(a) => match &mut self.sync {
                TransitionFrontierSyncState::RootLedgerSyncPending { root_ledger, .. } => {
                    root_ledger.reducer(meta.with_action(a));
                }
                _ => {}
            },
        }
    }
}
