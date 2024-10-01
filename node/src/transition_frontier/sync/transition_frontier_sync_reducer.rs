use std::collections::{BTreeMap, VecDeque};

use mina_p2p_messages::v2::StateHash;
use openmina_core::block::{AppliedBlock, ArcBlockWithHash};

use super::{
    ledger::{
        snarked::TransitionFrontierSyncLedgerSnarkedState, SyncLedgerTarget, SyncLedgerTargetKind,
        TransitionFrontierSyncLedgerState,
    },
    PeerRpcState, TransitionFrontierRootSnarkedLedgerUpdates, TransitionFrontierSyncAction,
    TransitionFrontierSyncActionWithMetaRef, TransitionFrontierSyncBlockState,
    TransitionFrontierSyncLedgerPending, TransitionFrontierSyncState,
};

impl TransitionFrontierSyncState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierSyncActionWithMetaRef<'_>,
        best_chain: &[AppliedBlock],
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierSyncAction::Init {
                best_tip,
                root_block,
                blocks_inbetween,
            } => {
                *state = Self::Init {
                    time: meta.time(),
                    best_tip: best_tip.clone(),
                    root_block: root_block.clone(),
                    blocks_inbetween: blocks_inbetween.clone(),
                };
            }
            // TODO(binier): refactor
            TransitionFrontierSyncAction::BestTipUpdate {
                previous_root_snarked_ledger_hash: _,
                best_tip,
                root_block,
                blocks_inbetween,
            } => match state {
                Self::StakingLedgerPending(substate)
                | Self::NextEpochLedgerPending(substate)
                | Self::RootLedgerPending(substate) => {
                    substate.time = meta.time();
                    substate.blocks_inbetween.clone_from(blocks_inbetween);
                    let old_root_block =
                        std::mem::replace(&mut substate.root_block, root_block.clone());
                    let old_best_tip = std::mem::replace(&mut substate.best_tip, best_tip.clone());

                    let staking_epoch_target = SyncLedgerTarget::staking_epoch(best_tip);
                    let next_epoch_target = SyncLedgerTarget::next_epoch(best_tip, root_block);

                    let new_target = if let Self::StakingLedgerPending(substate) = state {
                        substate
                            .ledger
                            .update_target(meta.time(), staking_epoch_target);
                        None
                    } else if let Self::NextEpochLedgerPending(substate) = state {
                        if old_best_tip.staking_epoch_ledger_hash()
                            != best_tip.staking_epoch_ledger_hash()
                        {
                            Some((substate, staking_epoch_target))
                        } else {
                            if let Some(next_epoch_target) = next_epoch_target {
                                substate
                                    .ledger
                                    .update_target(meta.time(), next_epoch_target);
                            }
                            None
                        }
                    } else if let Self::RootLedgerPending(substate) = state {
                        if old_best_tip.staking_epoch_ledger_hash()
                            != best_tip.staking_epoch_ledger_hash()
                        {
                            Some((substate, staking_epoch_target))
                        } else if let Some(next_epoch_target) = next_epoch_target.filter(|_| {
                            old_best_tip.next_epoch_ledger_hash()
                                != best_tip.next_epoch_ledger_hash()
                        }) {
                            Some((substate, next_epoch_target))
                        } else if substate
                            .ledger
                            .staged()
                            .map_or(false, |s| s.is_parts_fetched())
                            && root_block.pred_hash() == old_root_block.hash()
                        {
                            // Optimization. Prevent changing staging ledger target,
                            // if the new root block is the extension of the old one.
                            // Since in such case, we can reuse the old reconstructed
                            // staged ledger to reconstruct the new one.
                            substate.root_block_updates.push(old_root_block);
                            None
                        } else {
                            substate.root_block_updates = Default::default();
                            substate
                                .ledger
                                .update_target(meta.time(), SyncLedgerTarget::root(root_block));
                            None
                        }
                    } else {
                        return;
                    };

                    let Some((substate, new_target)) = new_target else {
                        return;
                    };
                    let new_target_kind = new_target.kind;
                    substate.ledger =
                        TransitionFrontierSyncLedgerSnarkedState::pending(meta.time(), new_target)
                            .into();
                    *state = match new_target_kind {
                        SyncLedgerTargetKind::StakingEpoch => {
                            Self::StakingLedgerPending(substate.clone())
                        }
                        SyncLedgerTargetKind::NextEpoch => {
                            Self::NextEpochLedgerPending(substate.clone())
                        }
                        SyncLedgerTargetKind::Root => Self::RootLedgerPending(substate.clone()),
                    };
                }
                Self::BlocksPending {
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    ..
                } => {
                    let mut applied_blocks: BTreeMap<_, _> =
                        best_chain.iter().map(|b| (b.hash(), b)).collect();

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
                        *state = next_required_ledger_to_sync(
                            meta.time(),
                            cur_best_tip.map(AppliedBlock::block_with_hash),
                            cur_best_root.map(AppliedBlock::block_with_hash),
                            &old_best_tip,
                            &old_root,
                            new_best_tip,
                            new_root,
                            blocks_inbetween,
                        );
                    }
                }
                Self::CommitPending { .. } => {}
                Self::CommitSuccess { .. } => {}
                Self::Synced { time, .. } => {
                    let applied_blocks: BTreeMap<_, _> =
                        best_chain.iter().map(|b| (b.hash(), b)).collect();

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
                        *state = Self::BlocksPending {
                            time: meta.time(),
                            chain,
                            root_snarked_ledger_updates: Default::default(),
                            needed_protocol_states: Default::default(),
                        };
                    } else {
                        *state = next_required_ledger_to_sync(
                            meta.time(),
                            None,
                            None,
                            old_best_tip.block_with_hash(),
                            old_root.block_with_hash(),
                            new_best_tip,
                            new_root,
                            blocks_inbetween,
                        );
                    }
                }
                _ => (),
            },
            TransitionFrontierSyncAction::LedgerStakingPending => {
                if let Self::Init {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    ..
                } = state
                {
                    *state = Self::StakingLedgerPending(TransitionFrontierSyncLedgerPending {
                        time: meta.time(),
                        best_tip: best_tip.clone(),
                        root_block: root_block.clone(),
                        blocks_inbetween: std::mem::take(blocks_inbetween),
                        root_block_updates: Default::default(),
                        ledger: TransitionFrontierSyncLedgerState::Init {
                            time: meta.time(),
                            target: SyncLedgerTarget::staking_epoch(best_tip),
                        },
                    });
                }
            }
            TransitionFrontierSyncAction::LedgerStakingSuccess => {
                if let Self::StakingLedgerPending(substate) = state {
                    let TransitionFrontierSyncLedgerState::Success {
                        needed_protocol_states,
                        ..
                    } = &mut substate.ledger
                    else {
                        return;
                    };
                    *state = Self::StakingLedgerSuccess {
                        time: meta.time(),
                        best_tip: substate.best_tip.clone(),
                        root_block: substate.root_block.clone(),
                        blocks_inbetween: std::mem::take(&mut substate.blocks_inbetween),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::LedgerNextEpochPending => {
                let (best_tip, root_block, blocks_inbetween) = match state {
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
                *state = Self::NextEpochLedgerPending(TransitionFrontierSyncLedgerPending {
                    time: meta.time(),
                    best_tip: best_tip.clone(),
                    root_block: root_block.clone(),
                    blocks_inbetween: std::mem::take(blocks_inbetween),
                    root_block_updates: Default::default(),
                    ledger: TransitionFrontierSyncLedgerState::Init {
                        time: meta.time(),
                        target,
                    },
                });
            }
            TransitionFrontierSyncAction::LedgerNextEpochSuccess => {
                if let Self::NextEpochLedgerPending(substate) = state {
                    let TransitionFrontierSyncLedgerState::Success {
                        needed_protocol_states,
                        ..
                    } = &mut substate.ledger
                    else {
                        return;
                    };
                    *state = Self::NextEpochLedgerSuccess {
                        time: meta.time(),
                        best_tip: substate.best_tip.clone(),
                        root_block: substate.root_block.clone(),
                        blocks_inbetween: std::mem::take(&mut substate.blocks_inbetween),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::LedgerRootPending => {
                let (best_tip, root_block, blocks_inbetween) = match state {
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
                *state = Self::RootLedgerPending(TransitionFrontierSyncLedgerPending {
                    time: meta.time(),
                    best_tip: best_tip.clone(),
                    root_block: root_block.clone(),
                    blocks_inbetween: std::mem::take(blocks_inbetween),
                    root_block_updates: Default::default(),
                    ledger: TransitionFrontierSyncLedgerState::Init {
                        time: meta.time(),
                        target: SyncLedgerTarget::root(root_block),
                    },
                });
            }
            TransitionFrontierSyncAction::LedgerRootSuccess => {
                if let Self::RootLedgerPending(substate) = state {
                    let TransitionFrontierSyncLedgerState::Success {
                        needed_protocol_states,
                        ..
                    } = &mut substate.ledger
                    else {
                        return;
                    };
                    *state = Self::RootLedgerSuccess {
                        time: meta.time(),
                        best_tip: substate.best_tip.clone(),
                        root_block: substate.root_block.clone(),
                        blocks_inbetween: std::mem::take(&mut substate.blocks_inbetween),
                        root_block_updates: std::mem::take(&mut substate.root_block_updates),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::BlocksPending => {
                let Self::RootLedgerSuccess {
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    root_block_updates,
                    needed_protocol_states,
                    ..
                } = state
                else {
                    return;
                };
                let (best_tip, root_block) = (best_tip.clone(), root_block.clone());
                let blocks_inbetween = std::mem::take(blocks_inbetween);
                let root_block_updates = std::mem::take(root_block_updates);

                let mut root_snarked_ledger_updates =
                    TransitionFrontierRootSnarkedLedgerUpdates::default();
                let mut root_block_updates_iter = root_block_updates.windows(2);
                while let Some([old_root, new_root]) = root_block_updates_iter.next() {
                    root_snarked_ledger_updates
                        .extend_with_needed(new_root, std::iter::once(old_root));
                }
                root_snarked_ledger_updates
                    .extend_with_needed(&root_block, root_block_updates.iter().rev().take(1));

                let mut applied_blocks: BTreeMap<_, _> =
                    best_chain.iter().map(|b| (b.hash(), b)).collect();

                let k = best_tip.constants().k.as_u32() as usize;
                let mut chain = Vec::with_capacity(k + root_block_updates.len());

                // TODO(binier): maybe time should be when we originally
                // applied this block? Same for below.

                // Root block is always applied since we have reconstructed it
                // in previous steps.
                let mut root_block_updates_iter = root_block_updates.into_iter();
                // NOTE: flag for root block is always set to `false` because
                // it is not possible to recover this info from the staging ledger reconstruction,
                // so the value will not always be correct for the root block.
                // The `false` value is used to be compatible with:
                // https://github.com/MinaProtocol/mina/blob/e975835deab303c7f48b09ec2fd0e41ec649aef6/src/lib/transition_frontier/full_frontier/full_frontier.ml#L157-L160
                let root_block_just_emitted_a_proof = false;

                if let Some(reconstructed_root_block) = root_block_updates_iter.next() {
                    chain.push(TransitionFrontierSyncBlockState::ApplySuccess {
                        time: meta.time(),
                        block: AppliedBlock {
                            block: reconstructed_root_block.clone(),
                            just_emitted_a_proof: root_block_just_emitted_a_proof,
                        },
                    });
                    chain.extend(
                        root_block_updates_iter
                            .chain(std::iter::once(root_block))
                            .map(|block| TransitionFrontierSyncBlockState::FetchSuccess {
                                time: meta.time(),
                                block,
                            }),
                    );
                } else {
                    chain.push(TransitionFrontierSyncBlockState::ApplySuccess {
                        time: meta.time(),
                        block: AppliedBlock {
                            block: root_block,
                            just_emitted_a_proof: root_block_just_emitted_a_proof,
                        },
                    });
                }

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

                *state = Self::BlocksPending {
                    time: meta.time(),
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states: std::mem::take(needed_protocol_states),
                };
            }
            TransitionFrontierSyncAction::BlocksPeersQuery => {}
            TransitionFrontierSyncAction::BlocksPeerQueryInit { hash, peer_id } => {
                let Some(block_state) = state.block_state_mut(hash) else {
                    return;
                };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else {
                    return;
                };
                attempts.insert(*peer_id, PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierSyncAction::BlocksPeerQueryRetry { hash, peer_id } => {
                let Some(block_state) = state.block_state_mut(hash) else {
                    return;
                };
                let Some(attempts) = block_state.fetch_pending_attempts_mut() else {
                    return;
                };
                attempts.insert(*peer_id, PeerRpcState::Init { time: meta.time() });
            }
            TransitionFrontierSyncAction::BlocksPeerQueryPending {
                hash,
                peer_id,
                rpc_id,
            } => {
                let Some(block_state) = state.block_state_mut(hash) else {
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
                let Self::BlocksPending { chain, .. } = state else {
                    return;
                };
                let Some(peer_state) = chain.iter_mut().find_map(|b| {
                    b.fetch_pending_from_peer_mut(peer_id)
                        .filter(|peer_rpc_state| {
                            matches!(peer_rpc_state, PeerRpcState::Pending { .. })
                        })
                }) else {
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
                let Some(block_state) = state.block_state_mut(&response.hash) else {
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
                let Some(block_state) = state.block_state_mut(hash) else {
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
                let Some(block_state) = state.block_state_mut(hash) else {
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
            TransitionFrontierSyncAction::BlocksNextApplyError { hash, error } => {
                let Some(block_state) = state.block_state_mut(hash) else {
                    return;
                };
                let Some(block) = block_state.block() else {
                    return;
                };

                *block_state = TransitionFrontierSyncBlockState::ApplyError {
                    time: meta.time(),
                    block: block.clone(),
                    error: error.clone(),
                };
            }
            TransitionFrontierSyncAction::BlocksNextApplySuccess {
                hash,
                just_emitted_a_proof,
            } => {
                let Some(block_state) = state.block_state_mut(hash) else {
                    return;
                };
                let Some(block) = block_state.block() else {
                    return;
                };

                *block_state = TransitionFrontierSyncBlockState::ApplySuccess {
                    time: meta.time(),
                    block: AppliedBlock {
                        block: block.clone(),
                        just_emitted_a_proof: *just_emitted_a_proof,
                    },
                };
            }
            TransitionFrontierSyncAction::BlocksSuccess => {
                let Self::BlocksPending {
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    ..
                } = state
                else {
                    return;
                };
                let Some(k) = chain
                    .last()
                    .and_then(|v| v.block())
                    .map(|b| b.constants().k.as_u32() as usize)
                else {
                    return;
                };
                let mut needed_protocol_states = std::mem::take(needed_protocol_states);
                let start_i = chain.len().saturating_sub(k + 1);
                let mut iter = std::mem::take(chain)
                    .into_iter()
                    .filter_map(|v| v.take_applied_block());

                for _ in 0..start_i {
                    if let Some(b) = iter.next() {
                        needed_protocol_states
                            .insert(b.hash().clone(), b.header().protocol_state.clone());
                    }
                }

                *state = Self::BlocksSuccess {
                    time: meta.time(),
                    chain: iter.collect(),
                    root_snarked_ledger_updates: std::mem::take(root_snarked_ledger_updates),
                    needed_protocol_states,
                };
            }
            TransitionFrontierSyncAction::CommitInit => {}
            TransitionFrontierSyncAction::CommitPending => {
                if let Self::BlocksSuccess {
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    ..
                } = state
                {
                    *state = Self::CommitPending {
                        time: meta.time(),
                        chain: std::mem::take(chain),
                        root_snarked_ledger_updates: std::mem::take(root_snarked_ledger_updates),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::CommitSuccess { .. } => {
                if let Self::CommitPending {
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    ..
                } = state
                {
                    *state = Self::CommitSuccess {
                        time: meta.time(),
                        chain: std::mem::take(chain),
                        root_snarked_ledger_updates: std::mem::take(root_snarked_ledger_updates),
                        needed_protocol_states: std::mem::take(needed_protocol_states),
                    };
                }
            }
            TransitionFrontierSyncAction::Ledger(a) => {
                if state.ledger_mut().is_some() {
                    TransitionFrontierSyncLedgerState::reducer(
                        crate::Substate::from_compatible_substate(state_context),
                        meta.with_action(a),
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn next_required_ledger_to_sync(
    time: redux::Timestamp,
    cur_best_tip: Option<&ArcBlockWithHash>,
    cur_best_root: Option<&ArcBlockWithHash>,
    old_best_tip: &ArcBlockWithHash,
    old_root: &ArcBlockWithHash,
    new_best_tip: &ArcBlockWithHash,
    new_root: &ArcBlockWithHash,
    new_blocks_inbetween: &[StateHash],
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
    } else if old_best_tip.next_epoch_ledger_hash() != new_best_tip.next_epoch_ledger_hash()
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

    let substate = TransitionFrontierSyncLedgerPending {
        time,
        best_tip: new_best_tip.clone(),
        root_block: new_root.clone(),
        blocks_inbetween: new_blocks_inbetween.to_owned(),
        root_block_updates: Default::default(),
        ledger,
    };
    match kind {
        SyncLedgerTargetKind::StakingEpoch => {
            TransitionFrontierSyncState::StakingLedgerPending(substate)
        }
        SyncLedgerTargetKind::NextEpoch => {
            TransitionFrontierSyncState::NextEpochLedgerPending(substate)
        }
        SyncLedgerTargetKind::Root => TransitionFrontierSyncState::RootLedgerPending(substate),
    }
}
