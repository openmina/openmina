use serde::{Deserialize, Serialize};

pub type ActionWithMeta = redux::ActionWithMeta<Action>;
pub type ActionWithMetaRef<'a> = redux::ActionWithMeta<&'a Action>;

pub use crate::event_source::EventSourceAction;
pub use crate::job_commitment::JobCommitmentAction;
pub use crate::p2p::P2pAction;
pub use crate::rpc::RpcAction;

pub trait ActionKindGet {
    fn kind(&self) -> crate::ActionKind;
}

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
    EventSource(EventSourceAction),

    P2p(P2pAction),
    JobCommitment(JobCommitmentAction),
    Rpc(RpcAction),
}

impl Action {
    pub fn kind(&self) -> crate::ActionKind {
        ActionKindGet::kind(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckTimeoutsAction {}

impl redux::EnablingCondition<crate::State> for CheckTimeoutsAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}
