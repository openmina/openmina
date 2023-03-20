mod p2p_connection_incoming_state;
pub use p2p_connection_incoming_state::*;

mod p2p_connection_incoming_actions;
pub use p2p_connection_incoming_actions::*;

mod p2p_connection_incoming_reducer;
pub use p2p_connection_incoming_reducer::*;

mod p2p_connection_incoming_effects;
pub use p2p_connection_incoming_effects::*;

use serde::{Deserialize, Serialize};

// TODO(binier): maybe move to `crate::webrtc`?
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IncomingSignalingMethod {
    Http,
}
