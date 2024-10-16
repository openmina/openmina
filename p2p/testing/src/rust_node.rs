use std::{
    pin::Pin,
    task::{ready, Context, Poll},
    time::Duration,
};

use futures::Stream;
use p2p::{P2pAction, P2pEvent, P2pLimits, P2pState, P2pTimeouts, PeerId};
use redux::{Effects, EnablingCondition, Reducer, SubStore};
use tokio::sync::mpsc;

use crate::{
    cluster::{Listener, PeerIdConfig},
    event::RustNodeEvent,
    redux::{Action, IdleAction, State, Store},
    service::ClusterService,
    test_node::TestNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustNodeId(pub(super) usize);

#[derive(Debug, Default, Clone)]
pub struct RustNodeConfig {
    pub peer_id: PeerIdConfig,
    pub initial_peers: Vec<Listener>,
    pub timeouts: P2pTimeouts,
    pub limits: P2pLimits,
    pub discovery: bool,
    pub override_fn: Option<Effects<State, ClusterService, Action>>,
    pub override_reducer: Option<Reducer<State, Action>>,
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

    pub fn with_limits(mut self, limits: P2pLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_discovery(mut self, discovery: bool) -> Self {
        self.discovery = discovery;
        self
    }

    pub fn with_override(mut self, override_fn: Effects<State, ClusterService, Action>) -> Self {
        self.override_fn = Some(override_fn);
        self
    }

    pub fn with_override_reducer(mut self, override_fn: Reducer<State, Action>) -> Self {
        self.override_reducer = Some(override_fn);
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

    pub(super) fn idle(&mut self, duration: Duration) -> RustNodeEvent {
        self.store.service.advance_time(duration);
        self.store.dispatch(IdleAction);
        self.store
            .service
            .rust_node_event()
            .unwrap_or(RustNodeEvent::Idle)
    }

    pub fn state(&self) -> &P2pState {
        &self.store.state().0
    }

    fn next_stored_event(&mut self) -> Option<RustNodeEvent> {
        self.store.service.rust_node_event()
    }

    fn poll_event_receiver(&mut self, cx: &mut Context<'_>) -> Poll<Option<RustNodeEvent>> {
        let event = ready!(Pin::new(&mut self.event_receiver).poll_recv(cx));
        Poll::Ready(event.map(|event| {
            self.dispatch_event(event.clone());
            RustNodeEvent::P2p { event }
        }))
    }

    pub(crate) fn dispatch_event(&mut self, event: P2pEvent) -> RustNodeEvent {
        super::redux::event_effect(&mut self.store, event.clone());
        RustNodeEvent::P2p { event }
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
    type Item = RustNodeEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(event) = this.next_stored_event() {
            Poll::Ready(Some(event))
        } else {
            this.poll_event_receiver(cx)
        }
    }
}
