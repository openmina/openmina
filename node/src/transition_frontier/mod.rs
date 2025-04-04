//! The Transition Frontier module manages the blockchain's current state and history,
//! serving as the core component for maintaining consensus in the Mina blockchain.
//! [Ref: doc:/Users/lukasimrich/Code/GitHub/openmina/docs/components-overview.md#Transition Frontier (0.95)]
//!
//! ## Core Responsibilities
//!
//! - Processes and validates incoming blocks from the network [Ref: file:openmina/node/src/transition_frontier/candidate/mod.rs (0.9)]
//! - Applies validated blocks to update the blockchain state [Ref: file:openmina/node/src/transition_frontier/transition_frontier_reducer.rs (0.9)]
//! - Maintains the best chain from root to tip according to consensus rules [Ref: item:openmina/node/src/transition_frontier/transition_frontier_state.rs::TransitionFrontierState::best_chain (1.0)]
//! - Implements Ouroboros Samasika consensus protocol fork-choice rules (using both short-range and long-range rules) to select the canonical chain [Ref: item:openmina/node/src/transition_frontier/candidate/transition_frontier_candidate_state.rs::TransitionFrontierCandidateState::cmp (0.95)]
//! - Manages synchronization with the network when the local state falls behind [Ref: file:openmina/node/src/transition_frontier/sync/mod.rs (0.95)]
//! - Provides basic archiving capabilities for historical blockchain data (note: currently in preliminary implementation state) [Ref: file:openmina/node/src/transition_frontier/archive/mod.rs (0.9)]
//!
//! ## Key Concepts
//!
//! - **Best Chain**: The current canonical chain from root to tip, selected according to consensus rules [Ref: item:openmina/node/src/transition_frontier/transition_frontier_state.rs::TransitionFrontierState::best_chain (1.0)]
//! - **Root Block**: The oldest block kept in the transition frontier, serving as the foundation for the chain [Ref: item:openmina/node/src/transition_frontier/transition_frontier_state.rs::TransitionFrontierState::root (0.95)]
//! - **Best Tip**: The most recent block in the best chain, representing the current blockchain state [Ref: item:openmina/node/src/transition_frontier/transition_frontier_state.rs::TransitionFrontierState::best_tip (1.0)]
//! - **Candidate Blocks**: Blocks that have been received and validated but may not yet be part of the best chain [Ref: file:openmina/node/src/transition_frontier/candidate/mod.rs (0.95)]
//! - **Synchronization States**: Different states during the synchronization process, including Idle, Init, BlockFetchPending, BlockApplyPending, various LedgerPending states (StakingLedgerPending, NextEpochLedgerPending, RootLedgerPending), BlocksSuccess, CommitPending, CommitSuccess, and Synced [Ref: item:openmina/node/src/transition_frontier/sync/transition_frontier_sync_state.rs::TransitionFrontierSyncState (0.95)]
//! - **Blacklist**: Blocks that had valid proofs but failed application or other validations [Ref: item:openmina/node/src/transition_frontier/transition_frontier_state.rs::TransitionFrontierState::blacklist (0.9)]
//!
//! ## Interactions
//!
//! - Receives blocks from the P2P network for validation and potential inclusion in the chain [Ref: multi:(transition_frontier_actions.rs, candidate/transition_frontier_candidate_actions.rs) (0.85)]
//! - Provides the best tip information to other components like Block Producer [Ref: file:openmina/node/src/transition_frontier/transition_frontier_effects.rs (0.8)]
//! - Triggers updates to the Transaction Pool based on chain reorganizations by maintaining a chain_diff field that tracks differences between old and new best chains [Ref: item:openmina/node/src/transition_frontier/transition_frontier_state.rs::TransitionFrontierState::maybe_make_chain_diff (0.9)]
//! - Interacts with the Ledger module for applying blocks and updating the ledger state [Ref: file:openmina/node/src/transition_frontier/sync/ledger/mod.rs (0.9)]
//!
//! ## Submodules
//!
//! - **archive**: Provides functionality for archiving historical blockchain data beyond what is kept in the active transition frontier [Ref: file:openmina/node/src/transition_frontier/archive/mod.rs (1.0)]
//! - **candidate**: Manages candidate blocks that may become part of the best chain, including validation, verification, and consensus-based ordering [Ref: file:openmina/node/src/transition_frontier/candidate/mod.rs (1.0)]
//! - **genesis**: Handles the generation and management of the genesis block, which serves as the foundation for all subsequent blocks [Ref: file:openmina/node/src/transition_frontier/genesis/mod.rs (1.0)]
//! - **genesis_effectful**: Handles side effects related to genesis block generation and proving, including interactions with external services [Ref: file:openmina/node/src/transition_frontier/genesis_effectful/mod.rs (1.0)]
//! - **sync**: Manages the synchronization of the transition frontier with the network, including block fetching, validation, and ledger state synchronization [Ref: file:openmina/node/src/transition_frontier/sync/mod.rs (1.0)]

pub mod archive;
pub mod candidate;
pub mod genesis;
pub mod genesis_effectful;
pub mod sync;

mod transition_frontier_config;
pub use transition_frontier_config::*;

mod transition_frontier_state;
pub use transition_frontier_state::*;

mod transition_frontier_actions;
pub use transition_frontier_actions::*;

mod transition_frontier_reducer;

mod transition_frontier_effects;
pub use transition_frontier_effects::*;
