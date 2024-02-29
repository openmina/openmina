use std::collections::{BTreeMap, VecDeque};

use mina_p2p_messages::v2::StateHash;
use openmina_core::block::ArcBlockWithHash;

use crate::TransitionFrontierConfig;

use super::{
    ledger::{
        snarked::TransitionFrontierSyncLedgerSnarkedState, SyncLedgerTarget, SyncLedgerTargetKind,
        TransitionFrontierSyncLedgerState,
    },
    PeerRpcState, TransitionFrontierSyncAction, TransitionFrontierSyncActionWithMetaRef,
    TransitionFrontierSyncBlockState, TransitionFrontierSyncLedgerPending,
    TransitionFrontierSyncState,
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
            TransitionFrontierSyncAction::Init {
                best_tip,
                root_block,
                blocks_inbetween,
            } => {
                *self = Self::Init {
                    time: meta.time(),
                    best_tip: best_tip.clone(),
                    root_block: root_block.clone(),
                    blocks_inbetween: blocks_inbetween.clone(),
                };
            }
            // TODO(binier): refactor
            TransitionFrontierSyncAction::BestTipUpdate {
                best_tip,
                root_block,
                blocks_inbetween,
            } => match self {
                Self::StakingLedgerPending(state)
                | Self::NextEpochLedgerPending(state)
                | Self::RootLedgerPending(state) => {
                    state.time = meta.time();
                    state.root_block = root_block.clone();
                    state.blocks_inbetween = blocks_inbetween.clone();
                    let old_best_tip = std::mem::replace(&mut state.best_tip, best_tip.clone());

                    let staking_epoch_target = SyncLedgerTarget::staking_epoch(best_tip);
                    let next_epoch_target = SyncLedgerTarget::next_epoch(best_tip, root_block);

                    let new_target = if let Self::StakingLedgerPending(state) = self {
                        state
                            .ledger
                            .update_target(meta.time(), staking_epoch_target);
                        None
                    } else if let Self::NextEpochLedgerPending(state) = self {
                        if old_best_tip.staking_epoch_ledger_hash()
                            != old_best_tip.staking_epoch_ledger_hash()
                        {
                            Some((state, staking_epoch_target))
                        } else {
                            if let Some(next_epoch_target) = next_epoch_target {
                                state.ledger.update_target(meta.time(), next_epoch_target);
                            }
                            None
                        }
                    } else if let Self::RootLedgerPending(state) = self {
                        if old_best_tip.staking_epoch_ledger_hash()
                            != old_best_tip.staking_epoch_ledger_hash()
                        {
                            Some((state, staking_epoch_target))
                        } else if let Some(next_epoch_target) = next_epoch_target.filter(|_| {
                            old_best_tip.next_epoch_ledger_hash()
                                != best_tip.next_epoch_ledger_hash()
                        }) {
                            Some((state, next_epoch_target))
                        } else {
                            state
                                .ledger
                                .update_target(meta.time(), SyncLedgerTarget::root(root_block));
                            None
                        }
                    } else {
                        return;
                    };

                    let Some((state, new_target)) = new_target else {
                        return;
                    };
                    let new_target_kind = new_target.kind;
                    state.ledger =
                        TransitionFrontierSyncLedgerSnarkedState::pending(meta.time(), new_target)
                            .into();
                    *self = match new_target_kind {
                        SyncLedgerTargetKind::StakingEpoch => {
                            Self::StakingLedgerPending(state.clone())
                        }
                        SyncLedgerTargetKind::NextEpoch => {
                            Self::NextEpochLedgerPending(state.clone())
                        }
                        SyncLedgerTargetKind::Root => Self::RootLedgerPending(state.clone()),
                    };
                }
                Self::BlocksPending {
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    ..
                } => {
                    let mut applied_blocks: BTreeMap<_, _> =
                        best_chain.iter().map(|b| (&b.hash, b)).collect();

                    let old_chain = VecDeque::from(std::mem::take(chain));
                    let old_root = old_chain.front().and_then(|b| b.block()).unwrap().clone();
                    let old_best_tip = old_chain.back().and_then(|b| b.block()).unwrap().clone();
                    let new_root = root_block;
                    let new_best_tip = best_tip;

                    let old_chain_has_new_root_applied = old_chain
                        .iter()
                        .find(|b| b.block_hash() == &new_root.hash)
                        .map_or(false, |b| b.is_apply_success());

                    if applied_blocks.contains_key(&new_root.hash) || old_chain_has_new_root_applied
                    {
                        if old_chain_has_new_root_applied {
                            root_snarked_ledger_updates.extend_with_needed(
                                new_root,
                                old_chain.iter().filter_map(|s| s.block()),
                            );
                        }

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

                        push_block(new_root.hash(), Some(new_root));
                        for hash in blocks_inbetween {
                            push_block(hash, None);
                        }
                        push_block(new_best_tip.hash(), Some(new_best_tip));

                        needed_protocol_states.extend(old_block_states.into_iter().filter_map(
                            |(hash, s)| {
                                Some((hash, s.take_block()?.block.header.protocol_state.clone()))
                            },
                        ));
                    } else {
                        let cur_best_root = best_chain.first();
                        let cur_best_tip = best_chain.last();
                        *self = next_required_ledger_to_sync(
                            meta.time(),
                            cur_best_tip,
                            cur_best_root,
                            &old_best_tip,
                            &old_root,
                            new_best_tip,
                            new_root,
                            blocks_inbetween,
                        );
                    }
                }
                Self::Synced { time, .. } => {
                    let applied_blocks: BTreeMap<_, _> =
                        best_chain.iter().map(|b| (&b.hash, b)).collect();

                    let old_best_tip = best_chain.last().unwrap();
                    let old_root = best_chain.first().unwrap();
                    let new_best_tip = best_tip;
                    let new_root = root_block;

                    if applied_blocks.contains_key(&new_root.hash) {
                        let chain = std::iter::once(root_block.hash())
                            .chain(blocks_inbetween)
                            .chain(std::iter::once(new_best_tip.hash()))
                            .map(|hash| match applied_blocks.get(hash) {
                                Some(&block) => TransitionFrontierSyncBlockState::ApplySuccess {
                                    time: *time,
                                    block: block.clone(),
                                },
                                None if hash == new_best_tip.hash() => {
                                    TransitionFrontierSyncBlockState::FetchSuccess {
                                        time: meta.time(),
                                        block: new_best_tip.clone(),
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
                            root_snarked_ledger_updates: Default::default(),
                            needed_protocol_states: Default::default(),
                        };
                    } else {
                        *self = next_required_ledger_to_sync(
                            meta.time(),
                            None,
                            None,
                            old_best_tip,
                            old_root,
                            new_best_tip,
                            new_root,
                            blocks_inbetween,
                        );
                    }
                }
                _ => return,
            },
            TransitionFrontierSyncAction::LedgerStakingPending => {
                if let Self::Init {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    ..
                } = self
                {
                    *self = Self::StakingLedgerPending(TransitionFrontierSyncLedgerPending {
                        time: meta.time(),
                        best_tip: best_tip.clone(),
                        root_block: root_block.clone(),
                        blocks_inbetween: std::mem::take(blocks_inbetween),
                        ledger: TransitionFrontierSyncLedgerState::Init {
                            time: meta.time(),
                            target: SyncLedgerTarget::staking_epoch(best_tip),
                        },
                    });
                }
            }
            TransitionFrontierSyncAction::LedgerStakingSuccess => {
                if let Self::StakingLedgerPending(state) = self {
                    let TransitionFrontierSyncLedgerState::Success {
                        needed_protocol_states,
                        ..
                    } = &mut state.ledger
                    else {
                        return;
                    };
                    *self = Self::StakingLedgerSuccess {
                        time: meta.time(),
                        best_tip: state.best_tip.clone(),
                        root_block: state.root_block.clone(),
                        blocks_inbetween: std::mem::take(&mut state.blocks_inbetween),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::LedgerNextEpochPending => {
                let (best_tip, root_block, blocks_inbetween) = match self {
                    Self::Init {
                        best_tip,
                        root_block,
                        blocks_inbetween,
                        ..
                    }
                    | Self::StakingLedgerSuccess {
                        best_tip,
                        root_block,
                        blocks_inbetween,
                        ..
                    } => (best_tip, root_block, blocks_inbetween),
                    _ => return,
                };
                let Some(target) = SyncLedgerTarget::next_epoch(best_tip, root_block) else {
                    return;
                };
                *self = Self::NextEpochLedgerPending(TransitionFrontierSyncLedgerPending {
                    time: meta.time(),
                    best_tip: best_tip.clone(),
                    root_block: root_block.clone(),
                    blocks_inbetween: std::mem::take(blocks_inbetween),
                    ledger: TransitionFrontierSyncLedgerState::Init {
                        time: meta.time(),
                        target,
                    },
                });
            }
            TransitionFrontierSyncAction::LedgerNextEpochSuccess => {
                if let Self::NextEpochLedgerPending(state) = self {
                    let TransitionFrontierSyncLedgerState::Success {
                        needed_protocol_states,
                        ..
                    } = &mut state.ledger
                    else {
                        return;
                    };
                    *self = Self::NextEpochLedgerSuccess {
                        time: meta.time(),
                        best_tip: state.best_tip.clone(),
                        root_block: state.root_block.clone(),
                        blocks_inbetween: std::mem::take(&mut state.blocks_inbetween),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::LedgerRootPending => {
                let (best_tip, root_block, blocks_inbetween) = match self {
                    Self::Init {
                        best_tip,
                        root_block,
                        blocks_inbetween,
                        ..
                    }
                    | Self::StakingLedgerSuccess {
                        best_tip,
                        root_block,
                        blocks_inbetween,
                        ..
                    }
                    | Self::NextEpochLedgerSuccess {
                        best_tip,
                        root_block,
                        blocks_inbetween,
                        ..
                    } => (best_tip, root_block, blocks_inbetween),
                    _ => return,
                };
                *self = Self::RootLedgerPending(TransitionFrontierSyncLedgerPending {
                    time: meta.time(),
                    best_tip: best_tip.clone(),
                    root_block: root_block.clone(),
                    blocks_inbetween: std::mem::take(blocks_inbetween),
                    ledger: TransitionFrontierSyncLedgerState::Init {
                        time: meta.time(),
                        target: SyncLedgerTarget::root(root_block),
                    },
                });
            }
            TransitionFrontierSyncAction::LedgerRootSuccess => {
                if let Self::RootLedgerPending(state) = self {
                    let TransitionFrontierSyncLedgerState::Success {
                        needed_protocol_states,
                        ..
                    } = &mut state.ledger
                    else {
                        return;
                    };
                    *self = Self::RootLedgerSuccess {
                        time: meta.time(),
                        best_tip: state.best_tip.clone(),
                        root_block: state.root_block.clone(),
                        blocks_inbetween: std::mem::take(&mut state.blocks_inbetween),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::BlocksPending => {
                let Self::RootLedgerSuccess {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    needed_protocol_states,
                    ..
                } = self
                else {
                    return;
                };
                let (best_tip, root_block) = (best_tip.clone(), root_block.clone());
                let blocks_inbetween = std::mem::take(blocks_inbetween);
                let root_block_height = root_block.height();

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

                // TODO(binier): can only happen if best_tip is genesis.
                // TMP until we don't have genesis reconstruction logic
                // without relying on peers for it.
                if root_block_height != best_tip.height() {
                    chain.push(TransitionFrontierSyncBlockState::FetchSuccess {
                        time: meta.time(),
                        block: best_tip,
                    });
                }

                *self = Self::BlocksPending {
                    time: meta.time(),
                    chain,
                    root_snarked_ledger_updates: Default::default(),
                    needed_protocol_states: std::mem::take(needed_protocol_states),
                };
            }
            TransitionFrontierSyncAction::BlocksPeersQuery => {}
            TransitionFrontierSyncAction::BlocksPeerQueryInit { hash, peer_id } => {
                let Some(block_state) = self.block_state_mut(hash) else {
                    return;
                };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else {
                    return;
                };
                attempts.insert(peer_id.clone(), PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierSyncAction::BlocksPeerQueryRetry { hash, peer_id } => {
                let Some(block_state) = self.block_state_mut(hash) else {
                    return;
                };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else {
                    return;
                };
                attempts.insert(peer_id.clone(), PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierSyncAction::BlocksPeerQueryPending {
                hash,
                peer_id,
                rpc_id,
            } => {
                let Some(block_state) = self.block_state_mut(hash) else {
                    return;
                };
                let Some(peer_state) = block_state.fetch_pending_from_peer_mut(peer_id) else {
                    return;
                };
                *peer_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncAction::BlocksPeerQueryError {
                peer_id,
                rpc_id,
                error,
            } => {
                let Self::BlocksPending { chain, .. } = self else {
                    return;
                };
                let Some(peer_state) = chain
                    .iter_mut()
                    .find_map(|b| b.fetch_pending_from_peer_mut(peer_id))
                else {
                    return;
                };
                *peer_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: error.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksPeerQuerySuccess {
                peer_id, response, ..
            } => {
                let Some(block_state) = self.block_state_mut(&response.hash) else {
                    return;
                };
                let Some(peer_state) = block_state.fetch_pending_from_peer_mut(peer_id) else {
                    return;
                };
                *peer_state = PeerRpcState::Success {
                    time: meta.time(),
                    block: response.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksFetchSuccess { hash } => {
                let Some(block_state) = self.block_state_mut(hash) else {
                    return;
                };
                let Some(block) = block_state.fetch_pending_fetched_block() else {
                    return;
                };
                *block_state = TransitionFrontierSyncBlockState::FetchSuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksNextApplyInit => {}
            TransitionFrontierSyncAction::BlocksNextApplyPending { hash } => {
                let Some(block_state) = self.block_state_mut(hash) else {
                    return;
                };
                let Some(block) = block_state.block() else {
                    return;
                };

                *block_state = TransitionFrontierSyncBlockState::ApplyPending {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksNextApplySuccess { hash } => {
                let Some(block_state) = self.block_state_mut(hash) else {
                    return;
                };
                let Some(block) = block_state.block() else {
                    return;
                };

                *block_state = TransitionFrontierSyncBlockState::ApplySuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksSuccess => {
                let Self::BlocksPending {
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    ..
                } = self
                else {
                    return;
                };
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
                    root_snarked_ledger_updates: std::mem::take(root_snarked_ledger_updates),
                    needed_protocol_states: std::mem::take(needed_protocol_states),
                };
            }
            TransitionFrontierSyncAction::Ledger(a) => {
                if let Some(ledger) = self.ledger_mut() {
                    ledger.reducer(meta.with_action(a));
                }
            }
        }
    }
}

fn next_required_ledger_to_sync(
    time: redux::Timestamp,
    cur_best_tip: Option<&ArcBlockWithHash>,
    cur_best_root: Option<&ArcBlockWithHash>,
    old_best_tip: &ArcBlockWithHash,
    old_root: &ArcBlockWithHash,
    new_best_tip: &ArcBlockWithHash,
    new_root: &ArcBlockWithHash,
    new_blocks_inbetween: &Vec<StateHash>,
) -> TransitionFrontierSyncState {
    let next_epoch_target = SyncLedgerTarget::next_epoch(new_best_tip, new_root);

    let (kind, ledger) = if old_best_tip.staking_epoch_ledger_hash()
        != new_best_tip.staking_epoch_ledger_hash()
        && cur_best_tip.map_or(true, |cur| {
            cur.staking_epoch_ledger_hash() != new_best_tip.staking_epoch_ledger_hash()
        }) {
        let ledger = TransitionFrontierSyncLedgerSnarkedState::pending(
            time,
            SyncLedgerTarget::staking_epoch(new_best_tip),
        )
        .into();
        (SyncLedgerTargetKind::StakingEpoch, ledger)
    } else if old_best_tip.staking_epoch_ledger_hash() != new_best_tip.staking_epoch_ledger_hash()
        && cur_best_tip.map_or(true, |cur| {
            cur.staking_epoch_ledger_hash() != new_best_tip.staking_epoch_ledger_hash()
        })
        && next_epoch_target.is_some()
    {
        let ledger =
            TransitionFrontierSyncLedgerSnarkedState::pending(time, next_epoch_target.unwrap())
                .into();
        (SyncLedgerTargetKind::NextEpoch, ledger)
    } else if old_root.snarked_ledger_hash() == new_root.snarked_ledger_hash()
        || cur_best_root.map_or(false, |cur| {
            cur.snarked_ledger_hash() == new_root.snarked_ledger_hash()
        })
    {
        let ledger = TransitionFrontierSyncLedgerSnarkedState::Success {
            time,
            target: SyncLedgerTarget::root(new_root),
        }
        .into();
        (SyncLedgerTargetKind::Root, ledger)
    } else {
        let ledger = TransitionFrontierSyncLedgerSnarkedState::pending(
            time,
            SyncLedgerTarget::root(new_root),
        )
        .into();
        (SyncLedgerTargetKind::Root, ledger)
    };

    let state = TransitionFrontierSyncLedgerPending {
        time,
        best_tip: new_best_tip.clone(),
        root_block: new_root.clone(),
        blocks_inbetween: new_blocks_inbetween.clone(),
        ledger,
    };
    match kind {
        SyncLedgerTargetKind::StakingEpoch => {
            TransitionFrontierSyncState::StakingLedgerPending(state)
        }
        SyncLedgerTargetKind::NextEpoch => {
            TransitionFrontierSyncState::NextEpochLedgerPending(state)
        }
        SyncLedgerTargetKind::Root => TransitionFrontierSyncState::RootLedgerPending(state),
    }
}
