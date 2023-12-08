use redux::ActionMeta;

use super::{
    P2pListenClosedAction, P2pListenErrorAction, P2pListenExpiredAction, P2pListenNewAction,
};

impl P2pListenNewAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, _store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        // TODO run Kademlia expose
    }
}

impl P2pListenExpiredAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, _store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        // TODO run Kademlia expose
    }
}

impl P2pListenErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, _store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        // TODO run Kademlia expose
    }
}

impl P2pListenClosedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, _store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        // TODO run Kademlia expose
    }
}
