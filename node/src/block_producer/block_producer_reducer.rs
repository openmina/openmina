use std::sync::Arc;

use ledger::{
    proofs::transaction::transaction_snark::CONSTRAINT_CONSTANTS,
    scan_state::currency::{Amount, Signed},
};
use mina_p2p_messages::{
    bigint::BigInt,
    v2::{
        ConsensusGlobalSlotStableV1, ConsensusProofOfStakeDataConsensusStateValueStableV2,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        DataHashLibStateHashStableV1, LedgerProofProdStableV2, MinaBaseEpochLedgerValueStableV1,
        MinaBaseEpochSeedStableV1, MinaBaseProtocolConstantsCheckedValueStableV1,
        MinaBlockBlockStableV2, MinaBlockHeaderStableV2, MinaStateBlockchainStateValueStableV2,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2,
        StagedLedgerDiffBodyStableV1, StateBodyHash, StateHash, UnsignedExtendedUInt32StableV1,
    },
};
use openmina_core::block::{ArcBlockWithHash, BlockWithHash};

use super::{
    BlockProducerAction, BlockProducerActionWithMetaRef, BlockProducerCurrentState,
    BlockProducerEnabled, BlockProducerState,
};

impl BlockProducerState {
    pub fn reducer(
        &mut self,
        action: BlockProducerActionWithMetaRef<'_>,
        best_chain: &[ArcBlockWithHash],
    ) {
        self.with_mut((), move |state| state.reducer(action, best_chain))
    }
}

impl BlockProducerEnabled {
    pub fn reducer(
        &mut self,
        action: BlockProducerActionWithMetaRef<'_>,
        best_chain: &[ArcBlockWithHash],
    ) {
        let (action, meta) = action.split();
        match action {
            BlockProducerAction::VrfEvaluator(action) => {
                self.vrf_evaluator.reducer(meta.with_action(action))
            }
            BlockProducerAction::BestTipUpdate(action) => {
                self.vrf_evaluator.current_best_tip_slot = action
                    .best_tip
                    .block
                    .header
                    .protocol_state
                    .body
                    .consensus_state
                    .curr_global_slot
                    .slot_number
                    .as_u32();

                // set the genesis timestamp on the first best tip update
                // TODO: move/remove once we can generate the genesis block
                if self.vrf_evaluator.genesis_timestamp == redux::Timestamp::ZERO {
                    self.vrf_evaluator.genesis_timestamp = action.best_tip.genesis_timestamp();
                }
            }
            BlockProducerAction::WonSlotSearch(_) => {}
            BlockProducerAction::WonSlot(action) => {
                self.current = BlockProducerCurrentState::WonSlot {
                    time: meta.time(),
                    won_slot: action.won_slot.clone(),
                };
            }
            BlockProducerAction::WonSlotDiscard(action) => {
                if let Some(won_slot) = self.current.won_slot() {
                    self.current = BlockProducerCurrentState::WonSlotDiscarded {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                        reason: action.reason.clone(),
                    };
                }
            }
            BlockProducerAction::WonSlotWait(_) => {
                if let Some(won_slot) = self.current.won_slot() {
                    self.current = BlockProducerCurrentState::WonSlotWait {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                    };
                }
            }
            BlockProducerAction::WonSlotProduceInit(_) => {
                if let Some(won_slot) = self.current.won_slot() {
                    let Some(chain) = best_chain.last().map(|best_tip| {
                        if best_tip.global_slot() == won_slot.global_slot() {
                            // We are producing block which replaces current best tip
                            // instead of extending it.
                            best_chain[..(best_chain.len() - 1)].to_vec()
                        } else {
                            best_chain.to_vec()
                        }
                    }) else {
                        return;
                    };
                    self.current = BlockProducerCurrentState::WonSlotProduceInit {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                        chain,
                    };
                }
            }
            BlockProducerAction::StagedLedgerDiffCreateInit(_) => {}
            BlockProducerAction::StagedLedgerDiffCreatePending(_) => {
                let BlockProducerCurrentState::WonSlotProduceInit {
                    won_slot, chain, ..
                } = &mut self.current
                else {
                    return;
                };
                self.current = BlockProducerCurrentState::StagedLedgerDiffCreatePending {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                    chain: std::mem::take(chain),
                    transactions: (),
                };
            }
            BlockProducerAction::StagedLedgerDiffCreateSuccess(action) => {
                let BlockProducerCurrentState::StagedLedgerDiffCreatePending {
                    won_slot,
                    chain,
                    ..
                } = &mut self.current
                else {
                    return;
                };
                self.current = BlockProducerCurrentState::StagedLedgerDiffCreateSuccess {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                    chain: std::mem::take(chain),
                    diff: action.diff.clone(),
                    diff_hash: action.diff_hash.clone(),
                    staged_ledger_hash: action.staged_ledger_hash.clone(),
                    emitted_ledger_proof: action.emitted_ledger_proof.clone(),
                };
            }
            BlockProducerAction::BlockUnprovenBuild(_) => {
                let BlockProducerCurrentState::StagedLedgerDiffCreateSuccess {
                    won_slot,
                    chain,
                    diff,
                    diff_hash,
                    staged_ledger_hash,
                    emitted_ledger_proof,
                    ..
                } = &mut self.current
                else {
                    return;
                };
                let Some(pred_block) = chain.last() else {
                    return;
                };

                let pred_consensus_state = &pred_block.header().protocol_state.body.consensus_state;
                let pred_blockchain_state =
                    &pred_block.header().protocol_state.body.blockchain_state;

                let genesis_ledger_hash = &pred_blockchain_state.genesis_ledger_hash;

                let block_timestamp = won_slot.timestamp();
                let pred_global_slot = pred_consensus_state.curr_global_slot.clone();
                let curr_global_slot = won_slot.global_slot.clone();
                let global_slot_since_genesis =
                    won_slot.global_slot_since_genesis(pred_block.global_slot_diff());
                let (pred_epoch, _) = to_epoch_and_slot(&pred_global_slot);
                let (next_epoch, next_slot) = to_epoch_and_slot(&curr_global_slot);
                let has_ancestor_in_same_checkpoint_window =
                    in_same_checkpoint_window(&pred_global_slot, &curr_global_slot);

                let block_stake_winner = won_slot.delegator.0.clone();
                let vrf_truncated_output = won_slot.vrf_output.clone();
                let vrf_hash = won_slot.vrf_hash.to_fp().unwrap();
                let block_creator = self.config.pub_key.clone();
                let coinbase_receiver = self.config.coinbase_receiver().clone();
                let proposed_protocol_version_opt = self.config.proposed_protocol_version.clone();

                let ledger_proof_statement = ledger_proof_statement_from_emitted_proof(
                    emitted_ledger_proof.as_ref(),
                    &pred_blockchain_state.ledger_proof_statement,
                );

                let supply_increase = emitted_ledger_proof.as_ref().map_or(Signed::zero(), |v| {
                    Signed::from(&v.statement.supply_increase)
                });

                let total_currency = {
                    let (amount, overflowed) =
                        Amount::from(pred_consensus_state.total_currency.clone())
                            .add_signed_flagged(supply_increase);
                    if overflowed {
                        todo!("error");
                    }
                    amount
                };

                let (staking_epoch_data, next_epoch_data, epoch_count) = {
                    let next_staking_ledger =
                        if pred_block.snarked_ledger_hash() == genesis_ledger_hash {
                            pred_consensus_state.next_epoch_data.ledger.clone()
                        } else {
                            MinaBaseEpochLedgerValueStableV1 {
                                hash: pred_block.snarked_ledger_hash().clone(),
                                total_currency: (&total_currency).into(),
                            }
                        };
                    let (staking_data, next_data, epoch_count) = if next_epoch > pred_epoch {
                        let staking_data =
                            next_to_staking_epoch_data(&pred_consensus_state.next_epoch_data);
                        let next_data =
                            ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
                                seed: pred_consensus_state.next_epoch_data.seed.clone(),
                                ledger: next_staking_ledger,
                                start_checkpoint: pred_block.hash().clone(),
                                // comment from Mina repo: (* TODO: We need to make sure issue #2328 is properly addressed. *)
                                lock_checkpoint: empty_state_hash(),
                                epoch_length: UnsignedExtendedUInt32StableV1(1.into()),
                            };
                        let epoch_count = UnsignedExtendedUInt32StableV1(
                            (pred_consensus_state.epoch_count.as_u32() + 1).into(),
                        );
                        (staking_data, next_data, epoch_count)
                    } else {
                        assert_eq!(pred_epoch, next_epoch);
                        let mut next_data = pred_consensus_state.next_epoch_data.clone();
                        next_data.epoch_length = UnsignedExtendedUInt32StableV1(
                            (next_data.epoch_length.as_u32() + 1).into(),
                        );
                        (
                            pred_consensus_state.staking_epoch_data.clone(),
                            next_data,
                            pred_consensus_state.epoch_count.clone(),
                        )
                    };

                    let next_data = if in_seed_update_range(next_slot, pred_block.constants()) {
                        let old_seed = next_data.seed.to_fp().unwrap();
                        let new_seed =
                            ledger::hash_with_kimchi("MinaEpochSeed", &[old_seed, vrf_hash]);
                        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
                            seed: MinaBaseEpochSeedStableV1(new_seed.into()).into(),
                            lock_checkpoint: pred_block.hash().clone(),
                            ..next_data
                        }
                    } else {
                        next_data
                    };

                    (staking_data, next_data, epoch_count)
                };

                let (min_window_density, sub_window_densities) = {
                    // TODO(binier): when should this be false?
                    let incr_window = true;
                    let pred_sub_window_densities = &pred_consensus_state.sub_window_densities;

                    let pred_global_sub_window =
                        global_sub_window(&pred_global_slot, pred_block.constants());
                    let next_global_sub_window =
                        global_sub_window(&curr_global_slot, pred_block.constants());

                    let pred_relative_sub_window = relative_sub_window(pred_global_sub_window);
                    let next_relative_sub_window = relative_sub_window(next_global_sub_window);

                    let is_same_global_sub_window =
                        pred_global_sub_window == next_global_sub_window;
                    let are_windows_overlapping = pred_global_sub_window
                        + CONSTRAINT_CONSTANTS.sub_windows_per_window as u32
                        >= next_global_sub_window;

                    let current_sub_window_densities = pred_sub_window_densities
                        .iter()
                        .enumerate()
                        .map(|(i, density)| (i as u32, density.as_u32()))
                        .map(|(i, density)| {
                            let gt_pred_sub_window = i > pred_relative_sub_window;
                            let lt_next_sub_window = i < next_relative_sub_window;
                            let within_range =
                                if pred_relative_sub_window < next_relative_sub_window {
                                    gt_pred_sub_window && lt_next_sub_window
                                } else {
                                    gt_pred_sub_window || lt_next_sub_window
                                };
                            if is_same_global_sub_window || are_windows_overlapping && !within_range
                            {
                                density
                            } else {
                                0
                            }
                        })
                        .collect::<Vec<_>>();

                    let grace_period_end = grace_period_end(pred_block.constants());
                    let min_window_density = if is_same_global_sub_window
                        || curr_global_slot.slot_number.as_u32() < grace_period_end
                    {
                        pred_consensus_state.min_window_density.clone()
                    } else {
                        let cur_density = current_sub_window_densities.iter().sum();
                        let min_density = pred_consensus_state
                            .min_window_density
                            .as_u32()
                            .min(cur_density);
                        UnsignedExtendedUInt32StableV1(min_density.into())
                    };

                    let next_sub_window_densities = current_sub_window_densities
                        .into_iter()
                        .enumerate()
                        .map(|(i, density)| (i as u32, density))
                        .map(|(i, density)| {
                            let is_next_sub_window = i == next_relative_sub_window;
                            if is_next_sub_window {
                                let density = if is_same_global_sub_window {
                                    density
                                } else {
                                    0
                                };
                                if incr_window {
                                    density + 1
                                } else {
                                    density
                                }
                            } else {
                                density
                            }
                        })
                        .map(|v| UnsignedExtendedUInt32StableV1(v.into()))
                        .collect();

                    (min_window_density, next_sub_window_densities)
                };

                let consensus_state = ConsensusProofOfStakeDataConsensusStateValueStableV2 {
                    blockchain_length: UnsignedExtendedUInt32StableV1(
                        (pred_block.height() + 1).into(),
                    ),
                    epoch_count,
                    min_window_density,
                    sub_window_densities,
                    last_vrf_output: vrf_truncated_output,
                    total_currency: (&total_currency).into(),
                    curr_global_slot,
                    global_slot_since_genesis,
                    staking_epoch_data,
                    next_epoch_data,
                    has_ancestor_in_same_checkpoint_window,
                    block_stake_winner,
                    block_creator,
                    coinbase_receiver,
                    // TODO(binier): Staged_ledger.can_apply_supercharged_coinbase_exn
                    supercharge_coinbase: false,
                };

                let protocol_state = MinaStateProtocolStateValueStableV2 {
                    previous_state_hash: pred_block.hash().clone(),
                    body: MinaStateProtocolStateBodyValueStableV2 {
                        genesis_state_hash: pred_block
                            .header()
                            .protocol_state
                            .body
                            .genesis_state_hash
                            .clone(),
                        constants: pred_block.header().protocol_state.body.constants.clone(),
                        blockchain_state: MinaStateBlockchainStateValueStableV2 {
                            staged_ledger_hash: staged_ledger_hash.clone(),
                            genesis_ledger_hash: genesis_ledger_hash.clone(),
                            ledger_proof_statement,
                            timestamp: block_timestamp,
                            body_reference: diff_hash.clone(),
                        },
                        consensus_state,
                    },
                };
                let body_hash = protocol_state.body.hash();
                let hash = StateHash::from_hashes(&protocol_state.previous_state_hash, &body_hash);

                // TODO(binier): test
                let chain_proof_len = pred_block.constants().delta.as_u32() as usize;
                let delta_block_chain_proof = match chain_proof_len {
                    0 => (hash.clone(), Vec::new()),
                    chain_proof_len => {
                        let mut iter = chain.iter().rev().take(chain_proof_len).rev();
                        let first_hash = iter
                            .next()
                            .map_or_else(|| hash.clone(), |b| b.hash().clone());
                        let body_hashes = iter
                            .map(|b| b.header().protocol_state.body.hash())
                            .chain(std::iter::once(body_hash))
                            .map(StateBodyHash::from)
                            .collect::<Vec<_>>();
                        (first_hash, body_hashes)
                    }
                };

                let block = MinaBlockBlockStableV2 {
                    header: MinaBlockHeaderStableV2 {
                        protocol_state,
                        protocol_state_proof: (*ledger::dummy::dummy_blockchain_proof()).clone(),
                        delta_block_chain_proof,
                        current_protocol_version: pred_block
                            .header()
                            .current_protocol_version
                            .clone(),
                        proposed_protocol_version_opt,
                    },
                    body: StagedLedgerDiffBodyStableV1 {
                        staged_ledger_diff: diff.clone(),
                    },
                };

                self.current = BlockProducerCurrentState::BlockUnprovenBuilt {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                    chain: std::mem::take(chain),
                    block: BlockWithHash {
                        hash,
                        block: Arc::new(block),
                    },
                }
            }
            BlockProducerAction::BlockProduced(_) => {
                if let BlockProducerCurrentState::BlockUnprovenBuilt {
                    won_slot,
                    chain,
                    block,
                    ..
                } = &mut self.current
                {
                    self.current = BlockProducerCurrentState::Produced {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                        chain: std::mem::take(chain),
                        block: block.clone(),
                    };
                }
            }
            BlockProducerAction::BlockInject(_) => {}
            BlockProducerAction::BlockInjected(_) => {
                if let BlockProducerCurrentState::Produced {
                    won_slot,
                    chain,
                    block,
                    ..
                } = &mut self.current
                {
                    self.current = BlockProducerCurrentState::Injected {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                        chain: std::mem::take(chain),
                        block: block.clone(),
                    };
                }
            }
        }
    }
}

fn to_epoch_and_slot(global_slot: &ConsensusGlobalSlotStableV1) -> (u32, u32) {
    let epoch = global_slot.slot_number.as_u32() / global_slot.slots_per_epoch.as_u32();
    let slot = global_slot.slot_number.as_u32() % global_slot.slots_per_epoch.as_u32();
    (epoch, slot)
}

fn next_to_staking_epoch_data(
    data: &ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
) -> ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
        seed: data.seed.clone(),
        ledger: data.ledger.clone(),
        start_checkpoint: data.start_checkpoint.clone(),
        lock_checkpoint: data.lock_checkpoint.clone(),
        epoch_length: data.epoch_length.clone(),
    }
}

fn ledger_proof_statement_from_emitted_proof(
    emitted_ledger_proof: Option<&LedgerProofProdStableV2>,
    pred_proof_statement: &MinaStateBlockchainStateValueStableV2LedgerProofStatement,
) -> MinaStateBlockchainStateValueStableV2LedgerProofStatement {
    match emitted_ledger_proof.map(|proof| &proof.statement) {
        None => pred_proof_statement.clone(),
        Some(stmt) => MinaStateBlockchainStateValueStableV2LedgerProofStatement {
            source: stmt.source.clone(),
            target: stmt.target.clone(),
            connecting_ledger_left: stmt.connecting_ledger_left.clone(),
            connecting_ledger_right: stmt.connecting_ledger_right.clone(),
            supply_increase: stmt.supply_increase.clone(),
            fee_excess: stmt.fee_excess.clone(),
            sok_digest: (),
        },
    }
}

fn empty_state_hash() -> StateHash {
    DataHashLibStateHashStableV1(BigInt::zero()).into()
}

fn in_seed_update_range(
    slot: u32,
    constants: &MinaBaseProtocolConstantsCheckedValueStableV1,
) -> bool {
    let third_epoch = constants.slots_per_epoch.as_u32() / 3;
    assert_eq!(constants.slots_per_epoch.as_u32(), third_epoch * 3);
    slot < third_epoch * 2
}

fn in_same_checkpoint_window(
    slot1: &ConsensusGlobalSlotStableV1,
    slot2: &ConsensusGlobalSlotStableV1,
) -> bool {
    checkpoint_window(slot1) == checkpoint_window(slot2)
}

fn checkpoint_window(slot: &ConsensusGlobalSlotStableV1) -> u32 {
    slot.slot_number.as_u32() / checkpoint_window_size_in_slots()
}

fn days_to_ms(days: u64) -> u64 {
    days * 24 * 60 * 60 * 1000
}

fn checkpoint_window_size_in_slots() -> u32 {
    let one_year_ms = days_to_ms(365);
    let slots_per_year = one_year_ms / CONSTRAINT_CONSTANTS.block_window_duration_ms;
    let size_in_slots = slots_per_year / 12;
    assert_eq!(slots_per_year % 12, 0);
    size_in_slots as u32
}

fn grace_period_end(constants: &MinaBaseProtocolConstantsCheckedValueStableV1) -> u32 {
    let slots = {
        const NUM_DAYS: u64 = 3;
        let n_days_ms = days_to_ms(NUM_DAYS);
        let n_days = n_days_ms / CONSTRAINT_CONSTANTS.block_window_duration_ms;
        (n_days as u32).min(constants.slots_per_epoch.as_u32())
    };
    match CONSTRAINT_CONSTANTS.fork.as_ref() {
        None => slots,
        Some(fork) => slots + fork.previous_global_slot.as_u32(),
    }
}

fn global_sub_window(
    slot: &ConsensusGlobalSlotStableV1,
    constants: &MinaBaseProtocolConstantsCheckedValueStableV1,
) -> u32 {
    slot.slot_number.as_u32() / constants.slots_per_sub_window.as_u32()
}

fn relative_sub_window(global_sub_window: u32) -> u32 {
    global_sub_window % CONSTRAINT_CONSTANTS.sub_windows_per_window as u32
}
