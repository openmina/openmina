use serde::{Deserialize, Serialize};

pub type ActionWithMeta = redux::ActionWithMeta<Action>;

pub use crate::event_source::EventSourceAction;
pub use crate::p2p::P2pAction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
    EventSource(EventSourceAction),

    P2p(P2pAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckTimeoutsAction {}

impl redux::EnablingCondition<crate::State> for CheckTimeoutsAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

impl From<CheckTimeoutsAction> for crate::Action {
    fn from(a: CheckTimeoutsAction) -> Self {
        Self::CheckTimeouts(a)
    }
}
