use node::{event_source::Event, p2p::connection::outgoing::P2pConnectionOutgoingInitOpts};
use serde::{Deserialize, Serialize};

use crate::cluster::{ClusterNodeId, ClusterOcamlNodeId};
use crate::node::{NodeTestingConfig, OcamlStep};

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
    /// Create a new node, start it and add it to the cluster.
    AddNode {
        config: NodeTestingConfig,
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
    Ocaml {
        node_id: ClusterOcamlNodeId,
        step: OcamlStep,
    },
}

#[derive(Serialize, Deserialize, derive_more::From, Debug, Clone)]
pub enum ListenerNode {
    Rust(ClusterNodeId),
    Ocaml(ClusterOcamlNodeId),
    Custom(P2pConnectionOutgoingInitOpts),
}
