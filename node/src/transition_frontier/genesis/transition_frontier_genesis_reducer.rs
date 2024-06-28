use crate::{
    account::AccountSecretKey, block_producer::calc_epoch_seed,
    transition_frontier::genesis_effectful::TransitionFrontierGenesisEffectfulAction,
};
use ledger::{
    dummy::dummy_blockchain_proof, scan_state::transaction_logic::local_state::LocalState,
};
use mina_p2p_messages::v2;
use openmina_core::{
    block::{genesis::genesis_and_negative_one_protocol_states, BlockWithHash},
    constants::PROTOCOL_VERSION,
    error,
};
use p2p::P2pInitializeAction;

use super::{
    empty_block_body, empty_block_body_hash, empty_pending_coinbase, empty_pending_coinbase_hash,
    TransitionFrontierGenesisAction, TransitionFrontierGenesisActionWithMetaRef,
    TransitionFrontierGenesisState,
};

impl TransitionFrontierGenesisState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierGenesisActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierGenesisAction::LedgerLoadInit => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let config = global_state.transition_frontier.config.genesis.clone();

                dispatcher.push(TransitionFrontierGenesisAction::LedgerLoadPending);
                dispatcher
                    .push(TransitionFrontierGenesisEffectfulAction::LedgerLoadInit { config });
            }
            TransitionFrontierGenesisAction::LedgerLoadPending => {
                *state = Self::LedgerLoadPending { time: meta.time() };
            }
            TransitionFrontierGenesisAction::LedgerLoadSuccess { data } => {
                *state = Self::LedgerLoadSuccess {
                    time: meta.time(),
                    data: data.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                // TODO(refactor): before this is dispatched genesis inject must be dispatched
                dispatcher.push(TransitionFrontierGenesisAction::Produce);
            }
            TransitionFrontierGenesisAction::Produce => {
                let Self::LedgerLoadSuccess { data, .. } = state else {
                    return;
                };

                let genesis_vrf = ::vrf::genesis_vrf(data.staking_epoch_seed.clone()).unwrap();
                let genesis_vrf_hash = genesis_vrf.hash();

                let (negative_one, genesis) = genesis_and_negative_one_protocol_states(
                    data.constants.clone(),
                    data.genesis_ledger_hash.clone(),
                    data.genesis_total_currency.clone(),
                    data.staking_epoch_ledger_hash.clone(),
                    data.staking_epoch_total_currency.clone(),
                    data.next_epoch_ledger_hash.clone(),
                    data.next_epoch_total_currency.clone(),
                    AccountSecretKey::genesis_producer().public_key().into(),
                    empty_pending_coinbase_hash(),
                    (&LocalState::dummy()).into(),
                    empty_block_body_hash(),
                    genesis_vrf.into(),
                    data.staking_epoch_seed.clone(),
                    data.next_epoch_seed.clone(),
                    calc_epoch_seed(&data.next_epoch_seed, genesis_vrf_hash), //data.next_epoch_seed.clone(),
                );

                *state = Self::Produced {
                    time: meta.time(),
                    negative_one,
                    genesis,
                    genesis_producer_stake_proof: data.genesis_producer_stake_proof.clone(),
                };

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if global_state.p2p.ready().is_none() {
                    let TransitionFrontierGenesisState::Produced { genesis, .. } =
                        &global_state.transition_frontier.genesis
                    else {
                        error!(meta.time(); "incorrect state: {:?}", global_state.transition_frontier.genesis);
                        return;
                    };
                    use openmina_core::{constants, ChainId};
                    let genesis_state_hash = genesis.hash();
                    let constraint_system_digests =
                        openmina_core::NetworkConfig::global().constraint_system_digests;
                    let chain_id = ChainId::compute(
                        constraint_system_digests,
                        &genesis_state_hash,
                        &genesis.body.constants,
                        constants::PROTOCOL_TRANSACTION_VERSION,
                        constants::PROTOCOL_NETWORK_VERSION,
                        &v2::UnsignedExtendedUInt32StableV1::from(constants::TX_POOL_MAX_SIZE),
                    );
                    dispatcher.push(P2pInitializeAction::Initialize { chain_id });
                }
                dispatcher.push(TransitionFrontierGenesisAction::ProveInit);
            }
            TransitionFrontierGenesisAction::ProveInit => {
                let TransitionFrontierGenesisState::Produced {
                    negative_one,
                    genesis,
                    genesis_producer_stake_proof,
                    ..
                } = state
                else {
                    return;
                };

                let block_hash = genesis.hash();
                let producer_pk = genesis.body.consensus_state.block_creator.clone();
                let delegator_pk = genesis.body.consensus_state.block_stake_winner.clone();

                let input = v2::ProverExtendBlockchainInputStableV2 {
                    chain: v2::BlockchainSnarkBlockchainStableV2 {
                        state: negative_one.clone(),
                        proof: (*dummy_blockchain_proof()).clone(),
                    },
                    next_state: genesis.clone(),
                    block: v2::MinaStateSnarkTransitionValueStableV2 {
                        blockchain_state: genesis.body.blockchain_state.clone(),
                        consensus_transition: genesis
                            .body
                            .consensus_state
                            .curr_global_slot_since_hard_fork
                            .slot_number
                            .clone(),
                        pending_coinbase_update: v2::MinaBasePendingCoinbaseUpdateStableV1::zero(),
                    },
                    ledger_proof: None,
                    prover_state: v2::ConsensusStakeProofStableV2 {
                        delegator: v2::MinaBaseAccountIndexStableV1(0u64.into()),
                        delegator_pk,
                        coinbase_receiver_pk: genesis
                            .body
                            .consensus_state
                            .coinbase_receiver
                            .clone(),
                        producer_public_key: producer_pk,
                        producer_private_key: AccountSecretKey::genesis_producer().into(),
                        ledger: genesis_producer_stake_proof.clone(),
                    },
                    pending_coinbase: v2::MinaBasePendingCoinbaseWitnessStableV2 {
                        pending_coinbases: (&empty_pending_coinbase()).into(),
                        is_new_stack: true,
                    },
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(TransitionFrontierGenesisAction::ProvePending);
                dispatcher.push(TransitionFrontierGenesisEffectfulAction::ProveInit {
                    block_hash,
                    input: input.into(),
                });
            }
            TransitionFrontierGenesisAction::ProvePending => {
                let Self::Produced {
                    negative_one,
                    genesis,
                    genesis_producer_stake_proof,
                    ..
                } = state
                else {
                    return;
                };

                *state = Self::ProvePending {
                    time: meta.time(),
                    negative_one: negative_one.clone(),
                    genesis: genesis.clone(),
                    genesis_producer_stake_proof: genesis_producer_stake_proof.clone(),
                };
            }
            TransitionFrontierGenesisAction::ProveSuccess { proof } => {
                let Self::ProvePending { genesis, .. } = state else {
                    return;
                };

                *state = Self::ProveSuccess {
                    time: meta.time(),
                    genesis: BlockWithHash::new(
                        v2::MinaBlockBlockStableV2 {
                            header: v2::MinaBlockHeaderStableV2 {
                                protocol_state: genesis.clone(),
                                protocol_state_proof: (**proof).clone(),
                                delta_block_chain_proof: (
                                    genesis.hash(),
                                    std::iter::empty().collect(),
                                ),
                                current_protocol_version: PROTOCOL_VERSION.clone(),
                                proposed_protocol_version_opt: None,
                            },
                            body: v2::StagedLedgerDiffBodyStableV1 {
                                staged_ledger_diff: empty_block_body(),
                            },
                        }
                        .into(),
                    ),
                };
            }
        }
    }
}
