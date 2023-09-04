mod p2p_channels_best_tip_state;
pub use p2p_channels_best_tip_state::*;

mod p2p_channels_best_tip_actions;
pub use p2p_channels_best_tip_actions::*;

mod p2p_channels_best_tip_reducer;
pub use p2p_channels_best_tip_reducer::*;

mod p2p_channels_best_tip_effects;
pub use p2p_channels_best_tip_effects::*;

use binprot_derive::{BinProtRead, BinProtWrite};
use openmina_core::block::ArcBlock;
use serde::{Deserialize, Serialize};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum BestTipPropagationChannelMsg {
    /// Request next commitments upto the `limit`.
    GetNext,
    BestTip(ArcBlock),
}
