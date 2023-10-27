mod config;
pub use config::{NodeTestingConfig, RustNodeTestingConfig};

use node::event_source::{Event, EventSourceNewEventAction};
use node::p2p::connection::outgoing::{
    P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts,
};
use node::p2p::webrtc::SignalingMethod;
use node::{Action, CheckTimeoutsAction, State, Store};
use redux::EnablingCondition;

use crate::service::{NodeTestingService, PendingEventId};

pub struct Node {
    store: Store<NodeTestingService>,
}

impl Node {
    pub fn new(store: Store<NodeTestingService>) -> Self {
        Self { store }
    }

    fn service(&self) -> &NodeTestingService {
        &self.store.service
    }

    fn service_mut(&mut self) -> &mut NodeTestingService {
        &mut self.store.service
    }

    pub fn dial_addr(&self) -> P2pConnectionOutgoingInitOpts {
        let peer_id = self.store.state().p2p.my_id();
        if self.service().rust_to_rust_use_webrtc() {
            let port = self.store.state().p2p.config.listen_port;
            let signaling = SignalingMethod::Http(([127, 0, 0, 1], port).into());
            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, signaling }
        } else {
            let opts = P2pConnectionOutgoingInitLibp2pOpts {
                peer_id,
                host: node::p2p::webrtc::Host::Ipv4([127, 0, 0, 1].into()),
                port: self.store.state().p2p.config.libp2p_port.unwrap(),
            };
            P2pConnectionOutgoingInitOpts::LibP2P(opts)
        }
    }

    pub fn state(&self) -> &State {
        self.store.state()
    }

    pub fn pending_events(&mut self) -> impl Iterator<Item = (PendingEventId, &Event)> {
        self.pending_events_with_state().1
    }

    pub fn pending_events_with_state(
        &mut self,
    ) -> (&State, impl Iterator<Item = (PendingEventId, &Event)>) {
        (self.store.state.get(), self.store.service.pending_events())
    }

    fn dispatch<T>(&mut self, action: T) -> bool
    where
        T: Into<Action> + EnablingCondition<State>,
    {
        self.store.dispatch(action)
    }

    pub fn dispatch_event(&mut self, event: Event) -> bool {
        self.dispatch(EventSourceNewEventAction { event })
    }

    pub fn check_timeouts(&mut self) {
        self.dispatch(CheckTimeoutsAction {});
    }

    pub fn advance_time(&mut self, by_nanos: u64) {
        self.store.service.advance_time(by_nanos)
    }

    pub async fn wait_for_event_and_dispatch(&mut self, event_pattern: &str) -> bool {
        let event_id = self
            .service_mut()
            .pending_events()
            .find(|(_, event)| event.to_string().starts_with(event_pattern))
            .map(|(id, _)| id);
        let event_id = match event_id {
            Some(id) => Some(id),
            None => loop {
                let (id, event) = match self.service_mut().next_pending_event().await {
                    Some(v) => v,
                    None => break None,
                };
                if event.to_string().starts_with(event_pattern) {
                    break Some(id);
                }
            },
        };

        if let Some(id) = event_id {
            let event = self.service_mut().take_pending_event(id).unwrap();
            return self.dispatch_event(event);
        }
        false
    }
}
