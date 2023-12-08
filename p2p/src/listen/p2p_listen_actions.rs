use redux::EnablingCondition;
use serde::{Serialize, Deserialize};

use crate::{P2pListenerId, P2pState, P2pAction};

pub type P2pListenActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pListenAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pListenAction {
    New(P2pListenNewAction),
    Expired(P2pListenExpiredAction),
    Error(P2pListenErrorAction),
    Closed(P2pListenClosedAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pListenNewAction {
    pub listener_id: P2pListenerId,
    pub addr: libp2p::Multiaddr,
}

impl EnablingCondition<P2pState> for P2pListenNewAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &P2pState) -> bool {
        true
    }
}

impl From<P2pListenNewAction> for P2pAction {
    fn from(value: P2pListenNewAction) -> Self {
        P2pListenAction::from(value).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pListenExpiredAction {
    pub listener_id: P2pListenerId,
    pub addr: libp2p::Multiaddr,
}

impl EnablingCondition<P2pState> for P2pListenExpiredAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &P2pState) -> bool {
        true
    }
}

impl From<P2pListenExpiredAction> for P2pAction {
    fn from(value: P2pListenExpiredAction) -> Self {
        P2pListenAction::from(value).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pListenErrorAction {
    pub listener_id: P2pListenerId,
    pub error: String,
}

impl EnablingCondition<P2pState> for P2pListenErrorAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &P2pState) -> bool {
        true
    }
}

impl From<P2pListenErrorAction> for P2pAction {
    fn from(value: P2pListenErrorAction) -> Self {
        P2pListenAction::from(value).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pListenClosedAction {
    pub listener_id: P2pListenerId,
    pub error: Option<String>,
}

impl EnablingCondition<P2pState> for P2pListenClosedAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &P2pState) -> bool {
        true
    }
}

impl From<P2pListenClosedAction> for P2pAction {
    fn from(value: P2pListenClosedAction) -> Self {
        P2pListenAction::from(value).into()
    }
}
