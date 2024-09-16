use std::sync::Arc;

use node::NodeData;
use tokio::sync::RwLock;

// Re-export modules
pub mod archive;
pub mod config;
pub mod evaluator;
pub mod node;
pub mod rpc;
pub mod storage;

pub type NodeStatus = Arc<RwLock<NodeData>>;

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
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
