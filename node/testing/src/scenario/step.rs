use node::{event_source::Event, p2p::connection::outgoing::P2pConnectionOutgoingInitOpts};
use serde::{Deserialize, Serialize};

use crate::cluster::ClusterNodeId;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ScenarioStep {
    /// Manually added/dispatched event.
    ManualEvent {
        node_id: ClusterNodeId,
        event: Box<Event>,
    },
    /// Event picked from pending events, which are triggered by actual service.
    ///
    /// Passed string is used as a pattern to pick event from pending events.
    Event {
        node_id: ClusterNodeId,
        event: String,
    },
    ConnectNodes {
        dialer: ClusterNodeId,
        listener: ListenerNode,
    },
    CheckTimeouts {
        node_id: ClusterNodeId,
    },
    /// Advance global time by passed nanoseconds.
    AdvanceTime {
        by_nanos: u64,
    },
    /// Advance time by passed nanoseconds for the node.
    AdvanceNodeTime {
        node_id: ClusterNodeId,
        by_nanos: u64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ListenerNode {
    Rust(ClusterNodeId),
    Custom(P2pConnectionOutgoingInitOpts),
}
