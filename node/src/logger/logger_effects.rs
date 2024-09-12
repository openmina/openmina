use openmina_core::log::inner::field::{display, DisplayValue};
use openmina_core::log::inner::Value;
use openmina_core::log::{time_to_str, ActionEvent, EventContext};
use p2p::PeerId;

use crate::p2p::channels::P2pChannelsAction;
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::network::P2pNetworkAction;
use crate::p2p::P2pAction;
use crate::snark::SnarkAction;
use crate::{
    Action, ActionWithMetaRef, BlockProducerAction, Service, Store, TransitionFrontierAction,
};

struct ActionLoggerContext {
    time: redux::Timestamp,
    time_str: String,
    node_id: DisplayValue<PeerId>,
}

impl ActionLoggerContext {
    fn new(time: redux::Timestamp, node_id: PeerId) -> Self {
        ActionLoggerContext {
            time,
            time_str: time_to_str(time),
            node_id: display(node_id),
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
}

pub fn logger_effects<S: Service>(store: &Store<S>, action: ActionWithMetaRef<'_>) {
    let (action, meta) = action.split();
    let context = ActionLoggerContext::new(meta.time(), store.state().p2p.my_id());

    match action {
        Action::P2p(action) => match action {
            P2pAction::Initialization(action) => action.action_event(&context),
            P2pAction::Connection(action) => match action {
                P2pConnectionAction::Outgoing(action) => action.action_event(&context),
                P2pConnectionAction::Incoming(action) => action.action_event(&context),
            },
            P2pAction::Disconnection(action) => action.action_event(&context),
            P2pAction::Discovery(action) => action.action_event(&context),
            P2pAction::Identify(action) => action.action_event(&context),
            P2pAction::Channels(action) => match action {
                P2pChannelsAction::MessageReceived(action) => action.action_event(&context),
                P2pChannelsAction::BestTip(action) => action.action_event(&context),
                P2pChannelsAction::Transaction(action) => action.action_event(&context),
                P2pChannelsAction::Snark(action) => action.action_event(&context),
                P2pChannelsAction::SnarkJobCommitment(action) => action.action_event(&context),
                P2pChannelsAction::Rpc(action) => action.action_event(&context),
                P2pChannelsAction::StreamingRpc(action) => action.action_event(&context),
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
                P2pNetworkAction::Pubsub(action) => action.action_event(&context),
                P2pNetworkAction::Identify(action) => action.action_event(&context),
            },
        },
        Action::ExternalSnarkWorker(action) => action.action_event(&context),
        Action::SnarkPool(action) => action.action_event(&context),
        Action::Snark(SnarkAction::WorkVerify(a)) => a.action_event(&context),
        Action::Consensus(a) => a.action_event(&context),
        Action::TransitionFrontier(a) => match a {
            TransitionFrontierAction::Synced { .. } => {
                let tip = store.state().transition_frontier.best_tip().unwrap();

                if store.state().block_producer.is_produced_by_me(tip) {
                    openmina_core::action_info!(
                        context,
                        kind = "BlockProducerBlockIntegrated",
                        summary = "produced block integrated into frontier",
                        block_hash = tip.hash().to_string(),
                        block_height = tip.height(),
                    );
                }

                openmina_core::action_info!(
                    context,
                    kind = action.kind().to_string(),
                    summary = "transition frontier synced",
                    block_hash = tip.hash().to_string(),
                    block_height = tip.height(),
                );
            }
            TransitionFrontierAction::SyncFailed { best_tip, error } => {
                openmina_core::action_error!(
                    context,
                    kind = action.kind().to_string(),
                    summary = "transition frontier failed to sync",
                    block_hash = best_tip.hash().to_string(),
                    block_height = best_tip.height(),
                    error = error.to_string(),
                );
            }
            a => a.action_event(&context),
        },
        Action::BlockProducer(a) => match a {
            BlockProducerAction::BlockProduced => {
                let block = store.state().block_producer.produced_block().unwrap();
                openmina_core::action_info!(
                    context,
                    kind = action.kind().to_string(),
                    summary = "produced a block",
                    block_hash = block.hash().to_string(),
                    block_height = block.height(),
                );
            }
            BlockProducerAction::BlockInjected => {
                let block = store.state().transition_frontier.sync.best_tip().unwrap();
                openmina_core::action_info!(
                    context,
                    kind = action.kind().to_string(),
                    summary = "produced block injected",
                    block_hash = block.hash().to_string(),
                    block_height = block.height(),
                );
            }
            a => a.action_event(&context),
        },
        Action::Rpc(a) => a.action_event(&context),
        Action::TransactionPool(a) => a.action_event(&context),
        _ => {}
    }
}
