//! # Node Crate
//!
//! The node crate combines all state machines of the Mina blockchain node into
//! one unified state machine using a Redux-style architecture.
//!
//! ## Core Architecture
//!
//! | Component   | Location           | Purpose                                      |
//! |-------------|--------------------|----------------------------------------------|
//! | Actions     | [`Action`]         | Events that trigger state changes            |
//! | Effects     | [`effects()`]      | Side-effects and service calls               |
//! | Reducers    | [`reducer()`]      | Functions that mutate state                  |
//! | Services    | [`service`]        | I/O and heavy computation (separate threads) |
//! | State       | [`State`]          | Centralized, immutable data structure        |
//!
//! ## Execution Flow
//!
//! ```text
//! Event arrives
//! -> Dispatch Action
//! -> Check Enabling Condition
//! -> Reducer (mutate state)
//! -> Effects (side-effects)
//! -> Service callbacks
//! -> Loop
//! ```
//!
//! ## Key Components
//!
//! | Component              | Module                     | Purpose                             |
//! |------------------------|----------------------------|-------------------------------------|
//! | Block Producer         | [`block_producer`]         | Block creation on won slots         |
//! | Event Source           | [`event_source`]           | External event ingestion            |
//! | External SNARK Worker  | [`external_snark_worker`]  | External SNARK worker management    |
//! | Ledger                 | [`ledger`]                 | Account state and transactions      |
//! | Logger                 | [`logger`]                 | Logging utilities                   |
//! | P2P                    | [`p2p`]                    | Networking layer                    |
//! | Recorder               | [`recorder`]               | Action recording for replay         |
//! | RPC                    | [`rpc`]                    | JSON-RPC API                        |
//! | SNARK Pool             | [`snark_pool`]             | Proof work management               |
//! | Stats                  | [`stats`]                  | Statistics tracking                 |
//! | Transaction Pool       | [`transaction_pool`]       | Mempool for pending transactions    |
//! | Transition Frontier    | [`transition_frontier`]    | Blockchain consensus and best chain |
//! | Watched Accounts       | [`watched_accounts`]       | Account monitoring                  |

#![allow(clippy::if_same_then_else)]

extern crate graphannis_malloc_size_of as malloc_size_of;
extern crate graphannis_malloc_size_of_derive as malloc_size_of_derive;

pub use mina_core as core;

#[macro_use]
mod action;
pub use action::*;

mod action_kind;
pub use action_kind::ActionKind;

pub mod config;
pub use config::*;

mod state;
pub use state::{P2p, State, Substate};

mod reducer;
pub use reducer::reducer;

mod effects;
pub use effects::effects;

pub mod service;
pub use service::Service;

pub mod account;

pub mod recorder;
pub mod stats;

pub mod block_producer;
pub mod block_producer_effectful;
pub mod daemon_json;
pub mod event_source;
pub mod external_snark_worker;
pub mod external_snark_worker_effectful;
pub mod ledger;
pub mod ledger_effectful;
pub mod logger;
pub mod p2p;
pub mod rpc;
pub mod rpc_effectful;
pub mod snark;
pub mod snark_pool;
pub mod transaction_pool;
pub mod transition_frontier;
pub mod watched_accounts;

pub type Store<S> = redux::Store<State, S, Action>;
pub type Effects<S> = redux::Effects<State, S, Action>;
