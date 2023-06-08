mod action;
pub use action::*;

mod action_kind;
pub use action_kind::ActionKind;

mod config;
pub use config::*;

mod state;
pub use state::State;

mod reducer;
pub use reducer::reducer;

mod effects;
pub use effects::effects;

pub mod service;
pub use service::Service;

pub mod account;
pub mod consensus;
pub mod event_source;
pub mod job_commitment;
pub mod logger;
pub mod p2p;
pub mod rpc;
pub mod snark;
pub mod watched_accounts;

pub type Store<S> = redux::Store<State, S, Action>;

pub struct Snarker<Serv> {
    store: Store<Serv>,
}

impl<Serv: Service> Snarker<Serv> {
    pub fn new(initial_state: State, service: Serv) -> Self {
        let store = Store::new(
            reducer,
            effects,
            service,
            redux::SystemTime::now(),
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

impl<Serv> Clone for Snarker<Serv>
where
    Serv: Clone,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}
