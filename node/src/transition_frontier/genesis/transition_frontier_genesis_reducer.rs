use ledger::scan_state::transaction_logic::local_state::LocalState;
use mina_p2p_messages::v2;
use openmina_core::{
    block::{genesis::genesis_and_negative_one_protocol_states, BlockWithHash},
    constants::PROTOCOL_VERSION,
    error,
};
use p2p::P2pInitializeAction;

use crate::account::AccountSecretKey;
use crate::block_producer::calc_epoch_seed;

use super::{
    empty_block_body, empty_block_body_hash, empty_pending_coinbase_hash,
    TransitionFrontierGenesisAction, TransitionFrontierGenesisActionWithMetaRef,
    TransitionFrontierGenesisState,
};

impl TransitionFrontierGenesisState {
    pub fn reducer(
        mut state: crate::Substate<Self>,
        action: TransitionFrontierGenesisActionWithMetaRef<'_>,
    ) {
        let (action, meta) = action.split();
        let state_ref = &*state;
        match action {
            TransitionFrontierGenesisAction::LedgerLoadInit => {}
            TransitionFrontierGenesisAction::LedgerLoadPending => {
                *state = Self::LedgerLoadPending { time: meta.time() };
            }
            TransitionFrontierGenesisAction::LedgerLoadSuccess { data } => {
                *state = Self::LedgerLoadSuccess {
                    time: meta.time(),
                    data: data.clone(),
                };

                // Dispatch
                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransitionFrontierGenesisAction::Produce);
            }
            TransitionFrontierGenesisAction::Produce => {
                let Self::LedgerLoadSuccess { data, .. } = state_ref else {
                    return;
                };

                let genesis_vrf = ::vrf::genesis_vrf().unwrap();
                let genesis_vrf_hash = genesis_vrf.hash();

                let (negative_one, genesis) = genesis_and_negative_one_protocol_states(
                    data.constants.clone(),
                    data.ledger_hash.clone(),
                    data.total_currency.clone(),
                    AccountSecretKey::genesis_producer().public_key().into(),
                    empty_pending_coinbase_hash(),
                    (&LocalState::dummy()).into(),
                    empty_block_body_hash(),
                    genesis_vrf.into(),
                    calc_epoch_seed(&v2::EpochSeed::zero(), genesis_vrf_hash),
                );
                *state = Self::Produced {
                    time: meta.time(),
                    negative_one,
                    genesis,
                    genesis_producer_stake_proof: data.genesis_producer_stake_proof.clone(),
                };

                // Dispatch
                let (dispatcher, global_state) = state.into_dispatcher_and_state();
                if global_state.p2p.ready().is_none() {
                    let TransitionFrontierGenesisState::Produced { genesis, .. } =
                        &global_state.transition_frontier.genesis
                    else {
                        error!(meta.time(); "incorrect state: {:?}", global_state.transition_frontier.genesis);
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
                    dispatcher.push(P2pInitializeAction::Initialize { chain_id });
                }
                dispatcher.push(TransitionFrontierGenesisAction::ProveInit);
            }
            TransitionFrontierGenesisAction::ProveInit => {}
            TransitionFrontierGenesisAction::ProvePending => {
                let Self::Produced {
                    negative_one,
                    genesis,
                    genesis_producer_stake_proof,
                    ..
                } = state_ref
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
                let Self::ProvePending { genesis, .. } = state_ref else {
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
