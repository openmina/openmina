use openmina_core::log::inner::field::{display, DisplayValue};
use openmina_core::log::inner::Value;
use openmina_core::log::{time_to_str, ActionEvent, EventContext};
use p2p::PeerId;

use crate::p2p::channels::P2pChannelsAction;
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::network::P2pNetworkAction;
use crate::p2p::P2pAction;
use crate::snark::SnarkAction;
use crate::{Action, ActionKind, ActionWithMetaRef, Service, Store};

struct ActionLoggerContext {
    time: redux::Timestamp,
    time_str: String,
    node_id: DisplayValue<PeerId>,
    kind: DisplayValue<ActionKind>,
}

impl ActionLoggerContext {
    fn new(time: redux::Timestamp, node_id: PeerId, kind: ActionKind) -> Self {
        ActionLoggerContext {
            time,
            time_str: time_to_str(time),
            node_id: display(node_id),
            kind: display(kind),
        }
    }
}

impl EventContext for ActionLoggerContext {
    fn timestamp(&self) -> redux::Timestamp {
        self.time
    }

    fn time(&self) -> &'_ dyn Value {
        &self.time_str
    }

    fn node_id(&self) -> &'_ dyn Value {
        &self.node_id
    }

    fn kind(&self) -> &'_ dyn Value {
        &self.kind
    }
}

pub fn logger_effects<S: Service>(store: &Store<S>, action: ActionWithMetaRef<'_>) {
    let (action, meta) = action.split();
    let context = ActionLoggerContext::new(meta.time(), store.state().p2p.my_id(), action.kind());

    match action {
        Action::P2p(action) => match action {
            P2pAction::Listen(action) => action.action_event(&context),
            P2pAction::Connection(action) => match action {
                P2pConnectionAction::Outgoing(action) => action.action_event(&context),
                P2pConnectionAction::Incoming(action) => action.action_event(&context),
            },
            P2pAction::Disconnection(action) => action.action_event(&context),
            P2pAction::Discovery(action) => action.action_event(&context),
            P2pAction::Identify(action) => match action {
                p2p::identify::P2pIdentifyAction::NewRequest { peer_id, .. } => {
                    openmina_core::log::debug!(
                        meta.time();
                        node_id = context.node_id,
                        kind = context.kind.to_string(),
                        summary = format!("peer_id: {peer_id}"),
                        peer_id = peer_id.to_string()
                    );
                }
                p2p::identify::P2pIdentifyAction::UpdatePeerInformation { peer_id, info } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = context.node_id,
                        kind = context.kind.to_string(),
                        summary = format!("peer_id: {peer_id} info: {:?}", info),
                        peer_id = peer_id.to_string()
                    );
                }
            },
            P2pAction::Channels(action) => match action {
                P2pChannelsAction::MessageReceived(action) => action.action_event(&context),
                P2pChannelsAction::BestTip(action) => action.action_event(&context),
                P2pChannelsAction::Snark(action) => action.action_event(&context),
                P2pChannelsAction::SnarkJobCommitment(action) => action.action_event(&context),
                P2pChannelsAction::Rpc(action) => action.action_event(&context),
            },
            P2pAction::Peer(action) => action.action_event(&context),
            P2pAction::Network(action) => match action {
                P2pNetworkAction::Scheduler(action) => action.action_event(&context),
                P2pNetworkAction::Pnet(action) => action.action_event(&context),
                P2pNetworkAction::Select(action) => action.action_event(&context),
                P2pNetworkAction::Noise(action) => action.action_event(&context),
                P2pNetworkAction::Yamux(action) => action.action_event(&context),
                P2pNetworkAction::Rpc(action) => action.action_event(&context),
                P2pNetworkAction::Kad(action) => action.action_event(&context),
                P2pNetworkAction::Identify(action) => action.action_event(&context),
            },
        },
        Action::ExternalSnarkWorker(action) => action.action_event(&context),
        Action::Snark(a) => match a {
            SnarkAction::WorkVerify(a) => a.action_event(&context),
            _ => {}
        },
        Action::TransitionFrontier(a) => a.action_event(&context),
        Action::BlockProducer(a) => a.action_event(&context),
        _ => {}
    }
}
