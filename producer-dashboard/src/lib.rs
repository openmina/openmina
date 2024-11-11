pub mod archive;
pub mod config;
pub mod evaluator;
pub mod node;
pub mod rpc;
pub mod storage;

use std::sync::Arc;

pub use archive::ArchiveConnector;

#[cfg(test)]
pub use archive::ArchiveConnectorForTest;

use node::NodeData;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum StakingToolError {
    #[error("Empty graphql response")]
    EmptyGraphqlResponse,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Node offline")]
    NodeOffline,
}

pub type NodeStatus = Arc<RwLock<NodeData>>;
