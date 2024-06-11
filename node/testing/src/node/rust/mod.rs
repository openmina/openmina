mod config;
pub use config::*;

mod event;
pub use event::*;

use node::event_source::EventSourceAction;
use node::p2p::connection::outgoing::{
    P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts,
};
use node::p2p::webrtc::SignalingMethod;
use node::p2p::PeerId;
use node::service::P2pDisconnectionService;
use node::{Action, CheckTimeoutsAction, State, Store};
use redux::EnablingCondition;
use temp_dir::TempDir;

use crate::cluster::ClusterNodeId;
use crate::service::{DynEffects, NodeTestingService, PendingEventId};

pub struct Node {
    work_dir: TempDir,
    config: RustNodeTestingConfig,
    store: Store<NodeTestingService>,
}

impl Drop for Node {
    fn drop(&mut self) {
        std::mem::replace(&mut self.work_dir, TempDir::new().unwrap()).leak();
    }
}

impl Node {
    pub fn new(
        work_dir: TempDir,
        config: RustNodeTestingConfig,
        store: Store<NodeTestingService>,
    ) -> Self {
        Self {
            work_dir,
            config,
            store,
        }
    }

    pub fn work_dir(&self) -> &TempDir {
        &self.work_dir
    }

    pub fn config(&self) -> &RustNodeTestingConfig {
        &self.config
    }

    pub fn service(&self) -> &NodeTestingService {
        &self.store.service
    }

    fn service_mut(&mut self) -> &mut NodeTestingService {
        &mut self.store.service
    }

    pub fn set_dyn_effects(&mut self, effects: DynEffects) {
        self.service_mut().set_dyn_effects(effects)
    }

    pub fn remove_dyn_effects(&mut self) -> Option<DynEffects> {
        self.service_mut().remove_dyn_effects()
    }

    pub fn dial_addr(&self) -> P2pConnectionOutgoingInitOpts {
        let peer_id = self.store.state().p2p.my_id();
        if self.service().rust_to_rust_use_webrtc() {
            let port = self.store.state().p2p.config().listen_port;
            let signaling = SignalingMethod::Http(([127, 0, 0, 1], port).into());
            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, signaling }
        } else {
            let opts = P2pConnectionOutgoingInitLibp2pOpts {
                peer_id,
                host: node::p2p::webrtc::Host::Ipv4([127, 0, 0, 1].into()),
                port: self.store.state().p2p.config().libp2p_port.unwrap(),
            };
            P2pConnectionOutgoingInitOpts::LibP2P(opts)
        }
    }

    pub fn state(&self) -> &State {
        self.store.state()
    }

    pub fn node_id(&self) -> ClusterNodeId {
        self.service().node_id()
    }

    pub fn peer_id(&self) -> PeerId {
        self.state().p2p.my_id()
    }

    pub fn pending_events(&mut self, poll: bool) -> impl Iterator<Item = (PendingEventId, &Event)> {
        self.pending_events_with_state(poll).1
    }

    pub fn pending_events_with_state(
        &mut self,
        poll: bool,
    ) -> (&State, impl Iterator<Item = (PendingEventId, &Event)>) {
        (
            self.store.state.get(),
            self.store.service.pending_events(poll),
        )
    }

    fn dispatch<T>(&mut self, action: T) -> bool
    where
        T: Into<Action> + EnablingCondition<State>,
    {
        self.store.dispatch(action)
    }

    pub fn dispatch_event(&mut self, event: Event) -> bool {
        self.dispatch(EventSourceAction::NewEvent { event })
    }

    pub fn get_pending_event(&self, event_id: PendingEventId) -> Option<&Event> {
        self.service().get_pending_event(event_id)
    }

    pub fn take_pending_event(&mut self, event_id: PendingEventId) -> Option<Event> {
        self.service_mut().take_pending_event(event_id)
    }

    pub fn take_event_and_dispatch(&mut self, event_id: PendingEventId) -> bool {
        let event = self.service_mut().take_pending_event(event_id).unwrap();
        self.dispatch_event(event)
    }

    pub fn check_timeouts(&mut self) {
        self.dispatch(CheckTimeoutsAction {});
    }

    pub fn advance_time(&mut self, by_nanos: u64) {
        self.store.service.advance_time(by_nanos)
    }

    pub async fn wait_for_next_pending_event(&mut self) -> Option<(PendingEventId, &Event)> {
        self.service_mut().next_pending_event().await
    }

    pub async fn wait_for_event(&mut self, event_pattern: &str) -> Option<PendingEventId> {
        let readonly_rpcs = self
            .service_mut()
            .pending_events(false)
            .filter(|(_, event)| {
                matches!(
                    NonDeterministicEvent::new(event).as_deref(),
                    Some(NonDeterministicEvent::RpcReadonly(..))
                )
            })
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        for event_id in readonly_rpcs {
            self.take_event_and_dispatch(event_id);
        }

        let event_id = self
            .service_mut()
            .pending_events(false)
            .find(|(_, event)| event.to_string().starts_with(event_pattern))
            .map(|(id, _)| id);
        match event_id {
            Some(id) => Some(id),
            None => loop {
                let (id, event) = match self.service_mut().next_pending_event().await {
                    Some(v) => v,
                    None => break None,
                };
                if event.to_string().starts_with(event_pattern) {
                    break Some(id);
                } else if matches!(
                    NonDeterministicEvent::new(event).as_deref(),
                    Some(NonDeterministicEvent::RpcReadonly(..))
                ) {
                    self.take_event_and_dispatch(id);
                }
            },
        }
    }

    pub async fn wait_for_event_and_dispatch(&mut self, event_pattern: &str) -> bool {
        if let Some(id) = self.wait_for_event(event_pattern).await {
            return self.take_event_and_dispatch(id);
        }
        false
    }

    pub fn p2p_disconnect(&mut self, peer_id: PeerId) {
        self.service_mut().disconnect(peer_id)
    }
}
