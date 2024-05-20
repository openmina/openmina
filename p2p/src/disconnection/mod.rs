mod p2p_disconnection_state;
pub use p2p_disconnection_state::*;

mod p2p_disconnection_actions;
pub use p2p_disconnection_actions::*;

mod p2p_disconnection_effects;

mod p2p_disconnection_service;
pub use p2p_disconnection_service::*;

use serde::{Deserialize, Serialize};

use crate::{channels::ChannelId, connection::RejectionReason};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, thiserror::Error)]
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

    #[error("transition frontier RPC timeout")]
    TransitionFrontierRpcTimeout,

    #[error("failed to verify snark pool diff")]
    SnarkPoolVerifyError,

    #[error("duplicate connection")]
    DuplicateConnection,

    #[error("select error")]
    SelectError,
}
