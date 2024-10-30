use crate::{
    block_producer::BlockProducerCurrentState,
    ledger::write::{LedgerWriteAction, LedgerWriteRequest},
    BlockProducerAction, Store,
};
use mina_p2p_messages::v2::{
    BlockchainSnarkBlockchainStableV2, ConsensusStakeProofStableV2,
    MinaStateSnarkTransitionValueStableV2, ProverExtendBlockchainInputStableV2,
};
use openmina_node_account::AccountSecretKey;
use redux::ActionWithMeta;

use super::BlockProducerEffectfulAction;

pub fn block_producer_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: ActionWithMeta<BlockProducerEffectfulAction>,
) {
    let (action, meta) = action.split();

    match action {
        BlockProducerEffectfulAction::VrfEvaluator(a) => {
            a.effects(&meta, store);
        }
        BlockProducerEffectfulAction::WonSlot { won_slot } => {
            if let Some(stats) = store.service.stats() {
                stats.block_producer().scheduled(meta.time(), &won_slot);
            }
            if !store.dispatch(BlockProducerAction::WonSlotWait) {
                store.dispatch(BlockProducerAction::WonSlotProduceInit);
            }
        }
        BlockProducerEffectfulAction::StagedLedgerDiffCreateInit => {
            if let Some(stats) = store.service.stats() {
                stats
                    .block_producer()
                    .staged_ledger_diff_create_start(meta.time());
            }
            let state = store.state.get();
            let Some((won_slot, pred_block, producer, coinbase_receiver)) = None.or_else(|| {
                let pred_block = state.block_producer.current_parent_chain()?.last()?;
                let won_slot = state.block_producer.current_won_slot()?;
                let config = state.block_producer.config()?;
                Some((
                    won_slot,
                    pred_block,
                    &config.pub_key,
                    config.coinbase_receiver(),
                ))
            }) else {
                return;
            };

            let completed_snarks = state
                .snark_pool
                .completed_snarks_iter()
                .map(|snark| (snark.job_id(), snark.clone()))
                .collect();
            // TODO(binier)
            let supercharge_coinbase = true;
            // We want to know if this is a new epoch to decide which staking ledger to use
            // (staking epoch ledger or next epoch ledger).
            let is_new_epoch = won_slot.epoch()
                > pred_block
                    .header()
                    .protocol_state
                    .body
                    .consensus_state
                    .epoch_count
                    .as_u32();

            let transactions_by_fee = state.block_producer.pending_transactions();

            store.dispatch(LedgerWriteAction::Init {
                request: LedgerWriteRequest::StagedLedgerDiffCreate {
                    pred_block: pred_block.clone(),
                    global_slot_since_genesis: won_slot
                        .global_slot_since_genesis(pred_block.global_slot_diff()),
                    is_new_epoch,
                    producer: producer.clone(),
                    delegator: won_slot.delegator.0.clone(),
                    coinbase_receiver: coinbase_receiver.clone(),
                    completed_snarks,
                    supercharge_coinbase,
                    transactions_by_fee,
                },
                on_init: redux::callback!(
                    on_staged_ledger_diff_create_init(_request: LedgerWriteRequest) -> crate::Action {
                        BlockProducerAction::StagedLedgerDiffCreatePending
                    }
                ),
            });
        }
        BlockProducerEffectfulAction::StagedLedgerDiffCreateSuccess => {
            if let Some(stats) = store.service.stats() {
                stats
                    .block_producer()
                    .staged_ledger_diff_create_end(meta.time());
            }
            store.dispatch(BlockProducerAction::BlockUnprovenBuild);
        }
        BlockProducerEffectfulAction::BlockUnprovenBuild => {
            if let Some(stats) = store.service.stats() {
                let bp = &store.state.get().block_producer;
                if let Some((block_hash, block)) = bp.with(None, |bp| match &bp.current {
                    BlockProducerCurrentState::BlockUnprovenBuilt {
                        block, block_hash, ..
                    } => Some((block_hash, block)),
                    _ => None,
                }) {
                    stats
                        .block_producer()
                        .produced(meta.time(), block_hash, block);
                }
            }

            store.dispatch(BlockProducerAction::BlockProveInit);
        }
        BlockProducerEffectfulAction::BlockProveInit => {
            let service = &mut store.service;

            if let Some(stats) = service.stats() {
                stats.block_producer().proof_create_start(meta.time());
            }
            let Some((block_hash, input)) = store.state.get().block_producer.with(None, |bp| {
                let BlockProducerCurrentState::BlockUnprovenBuilt {
                    won_slot,
                    chain,
                    emitted_ledger_proof,
                    pending_coinbase_update,
                    pending_coinbase_witness,
                    stake_proof_sparse_ledger,
                    block,
                    block_hash,
                    ..
                } = &bp.current
                else {
                    return None;
                };

                let pred_block = chain.last()?;

                let producer_public_key = block
                    .protocol_state
                    .body
                    .consensus_state
                    .block_creator
                    .clone();

                let input = Box::new(ProverExtendBlockchainInputStableV2 {
                    chain: BlockchainSnarkBlockchainStableV2 {
                        state: pred_block.header().protocol_state.clone(),
                        proof: pred_block.header().protocol_state_proof.clone(),
                    },
                    next_state: block.protocol_state.clone(),
                    block: MinaStateSnarkTransitionValueStableV2 {
                        blockchain_state: block.protocol_state.body.blockchain_state.clone(),
                        consensus_transition: block
                            .protocol_state
                            .body
                            .consensus_state
                            .curr_global_slot_since_hard_fork
                            .slot_number
                            .clone(),
                        pending_coinbase_update: pending_coinbase_update.clone(),
                    },
                    ledger_proof: emitted_ledger_proof.as_ref().map(|proof| (**proof).clone()),
                    prover_state: ConsensusStakeProofStableV2 {
                        delegator: won_slot.delegator.1.into(),
                        delegator_pk: won_slot.delegator.0.clone(),
                        coinbase_receiver_pk: block
                            .protocol_state
                            .body
                            .consensus_state
                            .coinbase_receiver
                            .clone(),
                        ledger: stake_proof_sparse_ledger.clone(),
                        // it is replaced with correct keys in the service.
                        producer_private_key: AccountSecretKey::genesis_producer().into(),
                        producer_public_key,
                    },
                    pending_coinbase: pending_coinbase_witness.clone(),
                });
                Some((block_hash.clone(), input))
            }) else {
                return;
            };
            service.prove(block_hash, input);
            store.dispatch(BlockProducerAction::BlockProvePending);
        }
        BlockProducerEffectfulAction::BlockProveSuccess => {
            if let Some(stats) = store.service.stats() {
                stats.block_producer().proof_create_end(meta.time());
            }
            store.dispatch(BlockProducerAction::BlockProduced);
        }
        BlockProducerEffectfulAction::WonSlotDiscard { reason } => {
            if let Some(stats) = store.service.stats() {
                stats.block_producer().discarded(meta.time(), reason);
            }
            store.dispatch(BlockProducerAction::WonSlotSearch);
        }
    }
}
