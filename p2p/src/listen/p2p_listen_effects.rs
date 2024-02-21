use redux::ActionMeta;

use super::P2pListenAction;

impl P2pListenAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, _store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        // TODO run Kademlia expose
    }
}
