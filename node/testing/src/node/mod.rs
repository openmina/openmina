mod config;
pub use config::{NodeTestingConfig, RustNodeTestingConfig};

use node::event_source::{Event, EventSourceNewEventAction};
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
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

    fn service(&mut self) -> &mut NodeTestingService {
        &mut self.store.service
    }

    pub fn dial_addr(&self) -> P2pConnectionOutgoingInitOpts {
        let peer_id = self.store.state().p2p.config.identity_pub_key.peer_id();
        let port = self.store.service.http_port();
        let signaling = SignalingMethod::Http(([127, 0, 0, 1], port).into());
        P2pConnectionOutgoingInitOpts::WebRTC { peer_id, signaling }
    }

    pub fn pending_events(&mut self) -> impl Iterator<Item = (PendingEventId, &Event)> {
        self.service().pending_events()
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
            .service()
            .pending_events()
            .find(|(_, event)| event.to_string().starts_with(event_pattern))
            .map(|(id, _)| id);
        let event_id = match event_id {
            Some(id) => Some(id),
            None => loop {
                let (id, event) = match self.service().next_pending_event().await {
                    Some(v) => v,
                    None => break None,
                };
                if event.to_string().starts_with(event_pattern) {
                    break Some(id);
                }
            },
        };

        if let Some(id) = event_id {
            let event = self.service().take_pending_event(id).unwrap();
            return self.dispatch_event(event);
        }
        false
    }
}
