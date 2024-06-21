#![allow(clippy::if_same_then_else)]

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
pub mod consensus;
pub mod daemon_json;
pub mod event_source;
pub mod external_snark_worker;
pub mod ledger;
pub mod logger;
pub mod p2p;
pub mod rpc;
pub mod snark;
pub mod snark_pool;
pub mod transition_frontier;
pub mod watched_accounts;

pub type Store<S> = redux::Store<State, S, Action>;
pub type Effects<S> = redux::Effects<State, S, Action>;

pub struct Node<Serv> {
    store: Store<Serv>,
}

impl<Serv: Service> Node<Serv> {
    pub fn new(
        initial_state: State,
        service: Serv,
        override_effects: Option<Effects<Serv>>,
    ) -> Self {
        let time_since_epoch = initial_state
            .time()
            .checked_sub(redux::Timestamp::ZERO)
            .unwrap();
        let store = Store::new(
            reducer,
            override_effects.unwrap_or(effects),
            service,
            redux::SystemTime::UNIX_EPOCH + time_since_epoch,
            initial_state,
        );

        Self { store }
    }

    pub fn store(&self) -> &Store<Serv> {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut Store<Serv> {
        &mut self.store
    }
}

impl<Serv> Clone for Node<Serv>
where
    Serv: Clone,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}

#[cfg(feature = "replay")]
pub mod replay_status {
    use std::sync::atomic::{AtomicBool, Ordering};

    static REPLAY_STATUS: AtomicBool = AtomicBool::new(false);

    pub fn toggle_enabled(value: bool) {
        REPLAY_STATUS.store(value, Ordering::Relaxed);
    }

    pub fn is_enabled() -> bool {
        REPLAY_STATUS.load(Ordering::Relaxed)
    }
}
