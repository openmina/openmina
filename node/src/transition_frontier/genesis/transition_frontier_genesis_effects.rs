use ledger::dummy::dummy_blockchain_proof;
use mina_p2p_messages::v2;
use openmina_core::error;
use p2p::P2pInitializeAction;
use redux::ActionMeta;

use crate::account::AccountSecretKey;
use crate::{transition_frontier::genesis::empty_pending_coinbase, Store};

use super::{
    TransitionFrontierGenesisAction, TransitionFrontierGenesisService,
    TransitionFrontierGenesisState,
};

impl TransitionFrontierGenesisAction {
    pub fn effects<S>(&self, meta: &ActionMeta, store: &mut Store<S>)
    where
        S: redux::Service + TransitionFrontierGenesisService,
    {
        match self {
            TransitionFrontierGenesisAction::LedgerLoadInit => {
                let config = &store.state().transition_frontier.config.genesis;
                store.service.load_genesis(config.clone());
                store.dispatch(TransitionFrontierGenesisAction::LedgerLoadPending);
            }
            TransitionFrontierGenesisAction::LedgerLoadPending => {}
            TransitionFrontierGenesisAction::LedgerLoadSuccess { .. } => {
                store.dispatch(TransitionFrontierGenesisAction::Produce);
            }
            TransitionFrontierGenesisAction::Produce => {
                if store.state().p2p.ready().is_none() {
                    let TransitionFrontierGenesisState::Produced { genesis, .. } =
                        &store.state().transition_frontier.genesis
                    else {
                        error!(meta.time(); "incorrect state: {:?}", store.state().transition_frontier.genesis);
                        return;
                    };
                    use openmina_core::{constants, ChainId};
                    let genesis_state_hash = genesis.hash();
                    let chain_id = ChainId::compute(
                        constants::CONSTRAINT_SYSTEM_DIGESTS.as_slice(),
                        &genesis_state_hash,
                        &genesis.body.constants,
                        constants::PROTOCOL_TRANSACTION_VERSION,
                        constants::PROTOCOL_NETWORK_VERSION,
                        &v2::UnsignedExtendedUInt32StableV1::from(constants::TX_POOL_MAX_SIZE),
                    );
                    store.dispatch(P2pInitializeAction::Initialize { chain_id });
                }
                store.dispatch(TransitionFrontierGenesisAction::ProveInit);
            }
            TransitionFrontierGenesisAction::ProveInit => {
                let TransitionFrontierGenesisState::Produced {
                    negative_one,
                    genesis,
                    genesis_producer_stake_proof,
                    ..
                } = &store.state.get().transition_frontier.genesis
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

                store.service.prove(block_hash, input.into());
                store.dispatch(TransitionFrontierGenesisAction::ProvePending);
            }
            TransitionFrontierGenesisAction::ProvePending => {}
            TransitionFrontierGenesisAction::ProveSuccess { .. } => {}
        }
    }
}
