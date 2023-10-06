mod p2p_channels_snark_state;
pub use p2p_channels_snark_state::*;

mod p2p_channels_snark_actions;
pub use p2p_channels_snark_actions::*;

mod p2p_channels_snark_reducer;
pub use p2p_channels_snark_reducer::*;

mod p2p_channels_snark_effects;
pub use p2p_channels_snark_effects::*;

use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};
use openmina_core::snark::SnarkInfo;
use serde::{Deserialize, Serialize};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkPropagationChannelMsg {
    /// Request next snarks upto the `limit`.
    ///
    /// - Must not be sent until peer sends `WillSend` message for the
    ///   previous request and until peer has fulfilled it.
    GetNext { limit: u8 },
    /// Amount of snarks which will proceed this message.
    ///
    /// - Can only be sent, if peer has sent `GetNext` and we haven't
    ///   responded with `WillSend` yet.
    /// - Can't be bigger than limit set by `GetNext`.
    /// - Amount of promised snarks must be delivered.
    WillSend { count: u8 },
    /// Snark.
    Snark(SnarkInfo),
}
