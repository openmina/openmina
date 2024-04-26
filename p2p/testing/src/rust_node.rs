use std::{pin::Pin, time::Duration};

use futures::Stream;
use p2p::{P2pAction, P2pEvent, P2pState, P2pTimeouts, PeerId};
use redux::{EnablingCondition, SubStore};
use tokio::sync::mpsc;

use crate::{
    cluster::{Listener, PeerIdConfig},
    event::RustNodeEvent,
    redux::IdleAction,
    redux::Store,
    test_node::TestNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustNodeId(pub(super) usize);

#[derive(Debug, Default, Clone)]
pub struct RustNodeConfig {
    pub peer_id: PeerIdConfig,
    pub initial_peers: Vec<Listener>,
    pub timeouts: P2pTimeouts,
    pub discovery: bool,
}

impl RustNodeConfig {
    pub fn with_peer_id(mut self, peer_id: PeerIdConfig) -> Self {
        self.peer_id = peer_id;
        self
    }

    pub fn with_initial_peers<T>(mut self, initial_peers: T) -> Self
    where
        T: IntoIterator<Item = Listener>,
    {
        self.initial_peers = Vec::from_iter(initial_peers);
        self
    }

    pub fn with_timeouts(mut self, timeouts: P2pTimeouts) -> Self {
        self.timeouts = timeouts;
        self
    }

    pub fn with_discovery(mut self, discovery: bool) -> Self {
        self.discovery = discovery;
        self
    }
}

pub struct RustNode {
    store: Store,
    event_receiver: mpsc::UnboundedReceiver<P2pEvent>,
}

impl RustNode {
    pub(super) fn new(store: Store, event_receiver: mpsc::UnboundedReceiver<P2pEvent>) -> Self {
        RustNode {
            store,
            event_receiver,
        }
    }

    pub fn dispatch_action<A>(&mut self, action: A) -> bool
    where
        A: Into<P2pAction> + EnablingCondition<P2pState>,
    {
        SubStore::dispatch(&mut self.store, action)
    }

    pub(super) fn idle(&mut self, duration: Duration) {
        self.store.service.advance_time(duration);
        self.store.dispatch(IdleAction);
    }

    pub fn state(&self) -> &P2pState {
        &self.store.state().0
    }

    pub(crate) fn dispatch_event(&mut self, event: P2pEvent) -> bool {
        super::redux::event_effect(&mut self.store, event)
    }

    pub(crate) fn rust_node_event(&mut self) -> Option<RustNodeEvent> {
        self.store.service.rust_node_event()
    }
}

impl TestNode for RustNode {
    fn peer_id(&self) -> PeerId {
        self.state().my_id()
    }

    fn libp2p_port(&self) -> u16 {
        self.state()
            .config
            .libp2p_port
            .expect("port should be present")
    }
}

impl Stream for RustNode {
    type Item = P2pEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().event_receiver).poll_recv(cx)
    }
}
