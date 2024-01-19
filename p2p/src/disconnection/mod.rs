mod p2p_disconnection_state;
pub use p2p_disconnection_state::*;

mod p2p_disconnection_actions;
pub use p2p_disconnection_actions::*;

mod p2p_disconnection_effects;
pub use p2p_disconnection_effects::*;

mod p2p_disconnection_service;
pub use p2p_disconnection_service::*;

use serde::{Deserialize, Serialize};

use crate::{channels::ChannelId, connection::webrtc::RejectionReason};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pDisconnectionReason {
    P2pChannelMsgUnexpected(ChannelId),
    P2pChannelSendFailed(String),
    P2pChannelReceiveFailed(String),
    P2pChannelClosed(ChannelId),
    Libp2pIncomingRejected(RejectionReason),

    TransitionFrontierRpcTimeout,

    SnarkPoolVerifyError,
}
