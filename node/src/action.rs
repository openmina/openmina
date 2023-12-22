use serde::{Deserialize, Serialize};

pub type ActionWithMeta = redux::ActionWithMeta<Action>;
pub type ActionWithMetaRef<'a> = redux::ActionWithMeta<&'a Action>;

pub use crate::consensus::ConsensusAction;
pub use crate::event_source::EventSourceAction;
use crate::external_snark_worker::ExternalSnarkWorkerAction;
pub use crate::p2p::P2pAction;
pub use crate::rpc::RpcAction;
pub use crate::snark::SnarkAction;
pub use crate::snark_pool::SnarkPoolAction;
pub use crate::transition_frontier::TransitionFrontierAction;
pub use crate::watched_accounts::WatchedAccountsAction;

pub trait ActionKindGet {
    fn kind(&self) -> crate::ActionKind;
}

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
    EventSource(EventSourceAction),

    P2p(P2pAction),
    Snark(SnarkAction),
    Consensus(ConsensusAction),
    TransitionFrontier(TransitionFrontierAction),
    SnarkPool(SnarkPoolAction),
    Rpc(RpcAction),
    ExternalSnarkWorker(ExternalSnarkWorkerAction),

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

macro_rules! impl_into_global_action {
    ($selector:ident : $($action:ty),* $(,)?) => {
        $(
            impl From<$action> for crate::Action {
                fn from(value: $action) -> Self {
                    Self::$selector(value.into())
                }
            }
        )*
    };
}

#[cfg(feature = "replay")]
impl redux::EnablingCondition<crate::State> for Action {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}
