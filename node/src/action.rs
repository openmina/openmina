use p2p::P2pEffectfulAction;
use serde::{Deserialize, Serialize};

pub type ActionWithMeta = redux::ActionWithMeta<Action>;
pub type ActionWithMetaRef<'a> = redux::ActionWithMeta<&'a Action>;

pub use crate::block_producer::BlockProducerAction;
pub use crate::block_producer_effectful::BlockProducerEffectfulAction;
pub use crate::event_source::EventSourceAction;
pub use crate::external_snark_worker::ExternalSnarkWorkerAction;
use crate::external_snark_worker_effectful::ExternalSnarkWorkerEffectfulAction;
pub use crate::ledger::LedgerAction;
use crate::ledger_effectful::LedgerEffectfulAction;
use crate::p2p::callbacks::P2pCallbacksAction;
pub use crate::p2p::P2pAction;
pub use crate::rpc::RpcAction;
use crate::rpc_effectful::RpcEffectfulAction;
pub use crate::snark::SnarkAction;
pub use crate::snark_pool::SnarkPoolAction;
pub use crate::snark_pool::SnarkPoolEffectfulAction;
pub use crate::transaction_pool::TransactionPoolAction;
use crate::transaction_pool::TransactionPoolEffectfulAction;
pub use crate::transition_frontier::TransitionFrontierAction;
pub use crate::watched_accounts::WatchedAccountsAction;

pub trait ActionKindGet {
    fn kind(&self) -> crate::ActionKind;
}

// Static limit for size of [`Action`] set to 512 bytes, if [`Action`] size is bigger code won't compile
// compile error: "attempt to compute `0_usize - 1_usize`, which would overflow"
static_assertions::const_assert!(std::mem::size_of::<Action>() <= 512);

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
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

impl redux::EnablingCondition<crate::State> for Action {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            Action::CheckTimeouts(a) => a.is_enabled(state, time),
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
