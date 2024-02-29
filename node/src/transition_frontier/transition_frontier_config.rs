use mina_p2p_messages::v2::{
    BlockTimeTimeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};
use serde::{Deserialize, Serialize};

use crate::ProtocolConstants;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierConfig {
    pub protocol_constants: ProtocolConstants,
}

impl TransitionFrontierConfig {
    pub fn k(&self) -> usize {
        self.protocol_constants.k.0.as_u32() as usize
    }
}

impl Default for TransitionFrontierConfig {
    fn default() -> Self {
        // TODO(binier): better way.
        TransitionFrontierConfig {
            protocol_constants: ProtocolConstants {
                k: 290.into(),
                slots_per_epoch: 7140.into(),
                slots_per_sub_window: 7.into(),
                grace_period_slots: 0.into(),
                delta: 0.into(),
                genesis_state_timestamp: BlockTimeTimeStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(0.into()),
                ),
            },
        }
    }
}
