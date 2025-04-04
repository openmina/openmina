//! # Block Producer Effectful Module
//!
//! This module implements the side effects layer for block production in the OpenMina node.
//! It separates effectful operations (external service interactions) from pure state management
//! in the block producer, following the Redux-inspired architecture pattern.
//! [Ref: doc:docs/components-overview.md#New Pattern (0.95)]
//!
//! ## Core Responsibilities
//!
//! - **VRF Evaluation**: Triggers the evaluation of slots using the VRF service to determine
//!   if the node can produce a block. [Ref: item:node/src/block_producer_effectful/vrf_evaluator_effectful/block_producer_vrf_evaluator_effectful_actions.rs::BlockProducerVrfEvaluatorEffectfulAction::EvaluateSlot (0.95)]
//! - **Staged Ledger Diff Creation**: Initiates and processes the creation of staged ledger diffs
//!   by interacting with the ledger service. [Ref: item:node/src/block_producer_effectful/block_producer_effectful_effects.rs::block_producer_effects::StagedLedgerDiffCreateInit (0.9)]
//! // FACT-CHECKER-WARNING: Documentation omits the BlockUnprovenBuild effect which is a critical step between staged ledger diff creation and block proving, as shown in the code and diagram.
//! - **Block Proving**: Manages the cryptographic proving process for blocks by interfacing with
//!   the prover service. [Ref: item:node/src/block_producer_effectful/block_producer_effectful_effects.rs::block_producer_effects::BlockProveInit (0.95)]
//! - **Statistics Tracking**: Records metrics about block production activities for monitoring
//!   and debugging. [Ref: multi:(node/src/block_producer_effectful/block_producer_effectful_effects.rs::BlockProducerEffectfulAction::BlockProduced, node/src/block_producer_effectful/vrf_evaluator_effectful/block_producer_vrf_evaluator_effectful_effects.rs::InitializeStats) (0.85)]
//! // FACT-CHECKER-WARNING: Documentation does not mention block injection, which is a critical responsibility shown in the block_producer module but not explicitly handled in the effectful module.
//!
//! ## Key Components
//!
//! - **BlockProducerEffectfulAction**: Defines the actions that trigger side effects in the block
//!   production process. [Ref: item:node/src/block_producer_effectful/block_producer_effectful_actions.rs::BlockProducerEffectfulAction (0.95)]
//! - **block_producer_effects**: The central handler function that processes effectful actions and
//!   translates them into service calls. [Ref: item:node/src/block_producer_effectful/block_producer_effectful_effects.rs::block_producer_effects (0.95)]
//! - **BlockProducerService**: Service interface for block production operations like proving blocks
//!   and accessing producer keypairs. [Ref: item:node/src/block_producer_effectful/block_producer_effectful_service.rs::BlockProducerService (0.95)]
//!
//! ## Critical Interactions
//!
//! - **Block Producer State Machine**: Dispatches actions to advance the block producer state machine
//!   based on the results of effectful operations. [Ref: multi:(node/src/block_producer_effectful/block_producer_effectful_effects.rs, node/src/block_producer/block_producer_state.rs::BlockProducerCurrentState) (0.9)]
//! - **Ledger Service**: Interacts with the ledger to create staged ledger diffs and process
//!   transactions. [Ref: item:node/src/block_producer_effectful/block_producer_effectful_effects.rs::LedgerWriteAction::Init (0.9)]
//! - **VRF Service**: Communicates with the VRF service to evaluate slots for block production
//!   eligibility. [Ref: item:node/src/block_producer_effectful/vrf_evaluator_effectful/block_producer_vrf_evaluator_effectful_effects.rs::BlockProducerVrfEvaluatorEffectfulAction::effects (0.9)]
//! - **Prover Service**: Interfaces with the cryptographic prover to generate zero-knowledge proofs
//!   for blocks. [Ref: item:node/src/block_producer_effectful/block_producer_effectful_service.rs::BlockProducerService::prove (0.95)]
//!
//! ## Submodules
//!
//! - **vrf_evaluator_effectful**: Handles VRF evaluation effects for determining block production
//!   eligibility through the leader election process. [Ref: file:node/src/block_producer_effectful/vrf_evaluator_effectful/mod.rs (0.95)]
//! - **block_producer_effectful_actions**: Defines actions that trigger side effects in the block
//!   production process. [Ref: file:node/src/block_producer_effectful/block_producer_effectful_actions.rs (0.95)]
//! - **block_producer_effectful_effects**: Implements effect handlers that translate actions into
//!   service calls. [Ref: file:node/src/block_producer_effectful/block_producer_effectful_effects.rs (0.95)]
//! - **block_producer_effectful_service**: Defines service interfaces for external interactions.
//!   [Ref: file:node/src/block_producer_effectful/block_producer_effectful_service.rs (0.95)]

pub mod vrf_evaluator_effectful;

mod block_producer_effectful_actions;
pub use block_producer_effectful_actions::*;

mod block_producer_effectful_effects;
pub use block_producer_effectful_effects::*;

mod block_producer_effectful_service;
pub use block_producer_effectful_service::*;
