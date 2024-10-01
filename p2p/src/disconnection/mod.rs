mod p2p_disconnection_state;
pub use p2p_disconnection_state::*;

mod p2p_disconnection_actions;
pub use p2p_disconnection_actions::*;

mod p2p_disconnection_reducer;

use serde::{Deserialize, Serialize};

use crate::{
    channels::{rpc::P2pRpcKind, streaming_rpc::P2pStreamingRpcKind, ChannelId},
    connection::RejectionReason,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, thiserror::Error)]
pub enum P2pDisconnectionReason {
    #[error("message is unexpected for channel {0}")]
    P2pChannelMsgUnexpected(ChannelId),
    #[error("failed to send message to channel: {0}")]
    P2pChannelSendFailed(String),
    #[error("failed to receive message from channel: {0}")]
    P2pChannelReceiveFailed(String),
    #[error("channel {0} is closed")]
    P2pChannelClosed(ChannelId),
    #[error("connection is rejected: {0}")]
    Libp2pIncomingRejected(RejectionReason),
    #[error("transition frontier RPC({0:?}) timeout")]
    TransitionFrontierRpcTimeout(P2pRpcKind),
    #[error("transition frontier streaming RPC({0:?}) timeout")]
    TransitionFrontierStreamingRpcTimeout(P2pStreamingRpcKind),
    #[error("received num accounts rejected")]
    TransitionFrontierSyncLedgerSnarkedNumAccountsRejected,
    #[error("failed to verify snark pool diff")]
    SnarkPoolVerifyError,
    #[error("duplicate connection")]
    DuplicateConnection,
    #[error("timeout")]
    Timeout,
    #[error("rpc protocol not supported")]
    Unsupported,
}
