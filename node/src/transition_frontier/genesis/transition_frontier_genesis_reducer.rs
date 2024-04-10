use ledger::scan_state::transaction_logic::local_state::LocalState;
use mina_p2p_messages::v2;
use openmina_core::{
    block::{genesis::genesis_and_negative_one_protocol_states, BlockWithHash},
    constants::PROTOCOL_VERSION,
};
use openmina_node_account::AccountSecretKey;

use crate::block_producer::calc_epoch_seed;

use super::{
    empty_block_body, empty_block_body_hash, empty_pending_coinbase_hash,
    TransitionFrontierGenesisAction, TransitionFrontierGenesisActionWithMetaRef,
    TransitionFrontierGenesisState,
};

impl TransitionFrontierGenesisState {
    pub fn reducer(&mut self, action: TransitionFrontierGenesisActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierGenesisAction::LedgerLoadInit => {}
            TransitionFrontierGenesisAction::LedgerLoadPending => {
                *self = Self::LedgerLoadPending { time: meta.time() };
            }
            TransitionFrontierGenesisAction::LedgerLoadSuccess { data } => {
                *self = Self::LedgerLoadSuccess {
                    time: meta.time(),
                    data: data.clone(),
                };
            }
            TransitionFrontierGenesisAction::Produce => {
                let TransitionFrontierGenesisState::LedgerLoadSuccess { data, .. } = self else {
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
                *self = Self::Produced {
                    time: meta.time(),
                    negative_one,
                    genesis,
                };
            }
            TransitionFrontierGenesisAction::ProveInit => {}
            TransitionFrontierGenesisAction::ProvePending => {
                let TransitionFrontierGenesisState::Produced {
                    negative_one,
                    genesis,
                    ..
                } = self
                else {
                    return;
                };
                *self = Self::ProvePending {
                    time: meta.time(),
                    negative_one: negative_one.clone(),
                    genesis: genesis.clone(),
                };
            }
            TransitionFrontierGenesisAction::ProveSuccess { proof } => {
                let TransitionFrontierGenesisState::ProvePending { genesis, .. } = self else {
                    return;
                };
                *self = Self::ProveSuccess {
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
