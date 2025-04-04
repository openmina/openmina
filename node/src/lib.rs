#![allow(clippy::if_same_then_else)]

extern crate graphannis_malloc_size_of as malloc_size_of;
extern crate graphannis_malloc_size_of_derive as malloc_size_of_derive;

pub use openmina_core as core;

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
pub mod error_sink;
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
