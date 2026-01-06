use crate::p2p::P2pEffectfulAction;
use mina_core::bug_condition;
use serde::{Deserialize, Serialize};

pub type ActionWithMeta = redux::ActionWithMeta<Action>;
pub type ActionWithMetaRef<'a> = redux::ActionWithMeta<&'a Action>;

pub use crate::{
    block_producer::BlockProducerAction,
    block_producer_effectful::BlockProducerEffectfulAction,
    event_source::EventSourceAction,
    external_snark_worker::ExternalSnarkWorkerAction,
    ledger::LedgerAction,
    p2p::P2pAction,
    rpc::RpcAction,
    snark::SnarkAction,
    snark_pool::{SnarkPoolAction, SnarkPoolEffectfulAction},
    transaction_pool::TransactionPoolAction,
    transition_frontier::TransitionFrontierAction,
    watched_accounts::WatchedAccountsAction,
};
use crate::{
    external_snark_worker_effectful::ExternalSnarkWorkerEffectfulAction,
    ledger_effectful::LedgerEffectfulAction, p2p::callbacks::P2pCallbacksAction,
    rpc_effectful::RpcEffectfulAction, transaction_pool::TransactionPoolEffectfulAction,
};

pub trait ActionKindGet {
    fn kind(&self) -> crate::ActionKind;
}

// Static limit for size of [`Action`] set to 512 bytes, if [`Action`] size is bigger code won't compile
// compile error: "attempt to compute `0_usize - 1_usize`, which would overflow"
static_assertions::const_assert!(std::mem::size_of::<Action>() <= 512);

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
    CheckInvalidPeersAction(CheckInvalidPeersAction),
    EventSource(EventSourceAction),

    P2p(P2pAction),
    P2pEffectful(P2pEffectfulAction),
    P2pCallbacks(P2pCallbacksAction),

    Ledger(LedgerAction),
    LedgerEffects(LedgerEffectfulAction),
    Snark(SnarkAction),
    TransitionFrontier(TransitionFrontierAction),
    SnarkPool(SnarkPoolAction),
    SnarkPoolEffect(SnarkPoolEffectfulAction),
    TransactionPool(TransactionPoolAction),
    TransactionPoolEffect(TransactionPoolEffectfulAction),
    ExternalSnarkWorker(ExternalSnarkWorkerAction),
    ExternalSnarkWorkerEffects(ExternalSnarkWorkerEffectfulAction),
    BlockProducer(BlockProducerAction),
    BlockProducerEffectful(BlockProducerEffectfulAction),
    Rpc(RpcAction),
    RpcEffectful(RpcEffectfulAction),

    WatchedAccounts(WatchedAccountsAction),
}

impl Action {
    pub fn kind(&self) -> crate::ActionKind {
        ActionKindGet::kind(self)
    }
}

/// Checks for timeouts and dispatches other time dependant actions.
///
/// Gets called repeatedly, so it's effects should be as light as possible.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckTimeoutsAction {}

impl redux::EnablingCondition<crate::State> for CheckTimeoutsAction {}

/// Checks if node has been started with invalid peers, and depending on feature flags exits/logs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckInvalidPeersAction {}

impl redux::EnablingCondition<crate::State> for CheckInvalidPeersAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        /// This is starting block height, loaded from state, if node doesn't connect to any other nodes this will be best height
        const STARTING_BLOCK_HEIGHT: u32 = 296_372;
        const GRACE_PERIOD: u64 = 60 * 1_000 * 1_000 * 1000;

        let Some(grace_period) = state.start_time().checked_add(GRACE_PERIOD) else {
            bug_condition!("Failed to add duration");
            return false;
        };
        if grace_period > time {
            return false;
        }

        let Some(p2p_state) = state.p2p.ready() else {
            return false;
        };

        let Some(kad_state) = p2p_state.network.scheduler.discovery_state() else {
            return false;
        };

        let has_bootstrapped = kad_state.has_bootstrapped && p2p_state.ready_peers().is_empty();
        if !has_bootstrapped {
            return false;
        }

        let Some(tip) = &state.transition_frontier.best_tip_breadcrumb() else {
            return false;
        };

        tip.height() == STARTING_BLOCK_HEIGHT
    }
}

impl redux::EnablingCondition<crate::State> for Action {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            Action::CheckTimeouts(a) => a.is_enabled(state, time),
            Action::CheckInvalidPeersAction(a) => a.is_enabled(state, time),
            Action::EventSource(a) => a.is_enabled(state, time),
            Action::P2p(a) => match a {
                P2pAction::Initialization(a) => a.is_enabled(state, time),
                other => state
                    .p2p
                    .ready()
                    .is_some_and(|p2p| other.is_enabled(p2p, time)),
            },
            Action::P2pEffectful(a) => state
                .p2p
                .ready()
                .is_some_and(|state| a.is_enabled(state, time)),
            Action::Ledger(a) => a.is_enabled(state, time),
            Action::LedgerEffects(a) => a.is_enabled(state, time),
            Action::Snark(a) => a.is_enabled(&state.snark, time),
            Action::TransitionFrontier(a) => a.is_enabled(state, time),
            Action::SnarkPool(a) => a.is_enabled(state, time),
            Action::SnarkPoolEffect(a) => a.is_enabled(state, time),
            Action::ExternalSnarkWorker(a) => a.is_enabled(state, time),
            Action::ExternalSnarkWorkerEffects(a) => a.is_enabled(state, time),
            Action::BlockProducer(a) => a.is_enabled(state, time),
            Action::BlockProducerEffectful(a) => a.is_enabled(state, time),
            Action::Rpc(a) => a.is_enabled(state, time),
            Action::WatchedAccounts(a) => a.is_enabled(state, time),
            Action::TransactionPool(a) => a.is_enabled(state, time),
            Action::TransactionPoolEffect(a) => a.is_enabled(state, time),
            Action::P2pCallbacks(a) => a.is_enabled(state, time),
            Action::RpcEffectful(a) => a.is_enabled(state, time),
        }
    }
}

impl From<redux::AnyAction> for Action {
    fn from(action: redux::AnyAction) -> Self {
        match action.0.downcast() {
            Ok(action) => *action,
            Err(action) => Self::P2p(*action.downcast().expect("Downcast failed")),
        }
    }
}
