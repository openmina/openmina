use std::collections::BTreeMap;

use ledger::transaction_pool::diff::BestTipDiff;
use mina_p2p_messages::v2::{
    MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2, StateHash,
    TransactionHash,
};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use super::genesis::TransitionFrontierGenesisState;
use super::sync::TransitionFrontierSyncState;
use super::TransitionFrontierConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierState {
    pub config: TransitionFrontierConfig,
    /// Genesis block generation/proving state
    pub genesis: TransitionFrontierGenesisState,
    /// Current best known chain, from root of the transition frontier to best tip
    pub best_chain: Vec<ArcBlockWithHash>,
    /// Needed protocol states for applying transactions in the root
    /// scan state that we don't have in the `best_chain` list.
    pub needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    /// Transition frontier synchronization state
    pub sync: TransitionFrontierSyncState,

    /// Blocks which had valid proof but failed block application or
    /// other validations after it reached transition frontier.
    pub blacklist: BTreeMap<StateHash, u32>,
    /// The diff of `Self::best_chain` with the previous one
    pub chain_diff: Option<BestTipDiff>,
}

impl TransitionFrontierState {
    pub fn new(config: TransitionFrontierConfig) -> Self {
        Self {
            config,
            genesis: TransitionFrontierGenesisState::Idle,
            best_chain: Vec::with_capacity(290),
            needed_protocol_states: Default::default(),
            sync: TransitionFrontierSyncState::Idle,
            blacklist: Default::default(),
            chain_diff: None,
        }
    }

    pub fn best_tip(&self) -> Option<&ArcBlockWithHash> {
        self.best_chain.last()
    }

    pub fn root(&self) -> Option<&ArcBlockWithHash> {
        self.best_chain.first()
    }

    /// FIXME
    /// Note(adonagy): This can be expensive, keep a map with all the tx hashis in the best chain
    pub fn contains_transaction(&self, hash: &TransactionHash) -> bool {
        self.best_chain.iter().any(|block| {
            block
                .body()
                .transactions()
                .any(|transaction| transaction.hash().as_ref().ok() == Some(hash))
        })
    }

    /// Looks up state body by state hash.
    pub fn get_state_body(
        &self,
        hash: &StateHash,
    ) -> Option<&MinaStateProtocolStateBodyValueStableV2> {
        self.best_chain
            .iter()
            .find_map(|block_with_hash| {
                if &block_with_hash.hash == hash {
                    Some(&block_with_hash.block.header.protocol_state.body)
                } else {
                    None
                }
            })
            .or_else(|| {
                self.needed_protocol_states
                    .iter()
                    .find_map(|(block_hash, state)| {
                        if block_hash == hash {
                            Some(&state.body)
                        } else {
                            None
                        }
                    })
            })
    }

    /// Create a diff between the old best chain and the new one
    /// This is used to update the transaction pool
    pub fn maybe_make_chain_diff(&self, new_chain: &[ArcBlockWithHash]) -> Option<BestTipDiff> {
        let old_chain = self.best_chain.as_slice();
        let new_root = new_chain.first();

        if old_chain.last() == new_chain.last() {
            // Both chains are equal
            return None;
        }

        // Look for the new root in the old chain, get its index
        let new_chain_start_at = match new_root {
            None => None,
            Some(new_root) => old_chain
                .iter()
                .enumerate()
                .rev()
                .find(|(_index, block)| *block == new_root),
        };

        let (diff_old_chain, diff_new_chain) = match new_chain_start_at {
            None => {
                // The new chain has a root not present in the old chain,
                // so the diff is the 2 wholes chains
                (old_chain, new_chain)
            }
            Some((new_chain_start_at, _)) => {
                // `new_chain_start_at` is the index of `new_root` in `old_chain`
                let old_chain_advanced = &old_chain[new_chain_start_at..];

                // Common length
                let len = old_chain_advanced.len().min(new_chain.len());

                // Find the first different block, search from the end
                let diff_start_at = old_chain_advanced[..len]
                    .iter()
                    .rev()
                    .zip(new_chain[..len].iter().rev())
                    .position(|(old_block, new_block)| old_block == new_block)
                    .map(|index| len - index) // we started from the end
                    .unwrap(); // Never panics because we know there is the common root block

                let diff_old_chain = &old_chain_advanced[diff_start_at..];
                let diff_new_chain = &new_chain[diff_start_at..];

                (diff_old_chain, diff_new_chain)
            }
        };

        // Collect commands and convert them to type `WithStatus::<UserCommand>`
        let collect = |chain: &[ArcBlockWithHash]| {
            chain
                .iter()
                .flat_map(|block| block.body().commands_iter())
                .map(|cmd| {
                    use ledger::scan_state::transaction_logic::{UserCommand, WithStatus};
                    WithStatus::<UserCommand>::from(cmd).into_map(UserCommand::to_valid_unsafe)
                })
                .collect::<Vec<_>>()
        };

        let removed_commands = collect(diff_old_chain);
        let new_commands = collect(diff_new_chain);

        if removed_commands.is_empty() && new_commands.is_empty() {
            return None;
        }

        Some(BestTipDiff {
            new_commands,
            removed_commands,
            reorg_best_tip: false, // TODO: Unused for now
        })
    }
}
