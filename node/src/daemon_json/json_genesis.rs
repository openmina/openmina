use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use mina_p2p_messages::v2::{
    BlockTimeTimeStableV1, MinaBaseProtocolConstantsCheckedValueStableV1,
    UnsignedExtendedUInt32StableV1,
};
use openmina_core::constants::PROTOCOL_CONSTANTS;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genesis {
    k: Option<u32>,
    slots_per_epoch: Option<u32>,
    slots_per_sub_window: Option<u32>,
    grace_period_slots: Option<u32>,
    delta: Option<u32>,
    #[serde_as(as = "Option<Rfc3339>")]
    genesis_state_timestamp: Option<OffsetDateTime>,
}

macro_rules! value_or_protocol_default {
    ($name:ident, $ty:ty) => {
        pub fn $name(&self) -> $ty {
            self.$name.map_or(PROTOCOL_CONSTANTS.$name, Into::into)
        }
    };
}

impl Genesis {
    value_or_protocol_default!(k, UnsignedExtendedUInt32StableV1);
    value_or_protocol_default!(slots_per_epoch, UnsignedExtendedUInt32StableV1);
    value_or_protocol_default!(slots_per_sub_window, UnsignedExtendedUInt32StableV1);
    value_or_protocol_default!(grace_period_slots, UnsignedExtendedUInt32StableV1);
    value_or_protocol_default!(delta, UnsignedExtendedUInt32StableV1);
    value_or_protocol_default!(genesis_state_timestamp, BlockTimeTimeStableV1);

    pub fn protocol_constants(&self) -> MinaBaseProtocolConstantsCheckedValueStableV1 {
        MinaBaseProtocolConstantsCheckedValueStableV1 {
            k: self.k(),
            slots_per_epoch: self.slots_per_epoch(),
            slots_per_sub_window: self.slots_per_sub_window(),
            grace_period_slots: self.grace_period_slots(),
            delta: self.delta(),
            genesis_state_timestamp: self.genesis_state_timestamp(),
        }
    }
}
