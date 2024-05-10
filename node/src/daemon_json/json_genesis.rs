use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use mina_p2p_messages::{
    number::Number,
    v2::{
        BlockTimeTimeStableV1, MinaBaseProtocolConstantsCheckedValueStableV1,
        UnsignedExtendedUInt32StableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};
use openmina_core::constants::PROTOCOL_CONSTANTS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genesis {
    k: Option<u32>,
    slots_per_epoch: Option<u32>,
    slots_per_sub_window: Option<u32>,
    grace_period_slots: Option<u32>,
    delta: Option<u32>,
    genesis_state_timestamp: String,
}

impl Genesis {
    pub fn k(&self) -> UnsignedExtendedUInt32StableV1 {
        self.k
            .map(|k| UnsignedExtendedUInt32StableV1(Number(k as u32)))
            .unwrap_or(PROTOCOL_CONSTANTS.k.clone())
    }

    pub fn slots_per_epoch(&self) -> UnsignedExtendedUInt32StableV1 {
        self.slots_per_epoch
            .map(|slots_per_epoch| UnsignedExtendedUInt32StableV1(Number(slots_per_epoch as u32)))
            .unwrap_or(PROTOCOL_CONSTANTS.slots_per_epoch.clone())
    }

    pub fn slots_per_sub_window(&self) -> UnsignedExtendedUInt32StableV1 {
        self.slots_per_sub_window
            .map(|slots_per_sub_window| {
                UnsignedExtendedUInt32StableV1(Number(slots_per_sub_window as u32))
            })
            .unwrap_or(PROTOCOL_CONSTANTS.slots_per_sub_window.clone())
    }

    pub fn grace_period_slots(&self) -> UnsignedExtendedUInt32StableV1 {
        self.grace_period_slots
            .map(|grace_period_slots| {
                UnsignedExtendedUInt32StableV1(Number(grace_period_slots as u32))
            })
            .unwrap_or(PROTOCOL_CONSTANTS.grace_period_slots.clone())
    }

    pub fn delta(&self) -> UnsignedExtendedUInt32StableV1 {
        self.delta
            .map(|delta| UnsignedExtendedUInt32StableV1(Number(delta as u32)))
            .unwrap_or(PROTOCOL_CONSTANTS.delta.clone())
    }

    pub fn genesis_state_timestamp(&self) -> Result<BlockTimeTimeStableV1, time::error::Parse> {
        OffsetDateTime::parse(&self.genesis_state_timestamp, &Rfc3339).map(|dt| {
            BlockTimeTimeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(Number(
                dt.unix_timestamp_nanos() as u64,
            )))
        })
    }

    pub fn protocol_constants(
        &self,
    ) -> Result<MinaBaseProtocolConstantsCheckedValueStableV1, time::error::Parse> {
        Ok(MinaBaseProtocolConstantsCheckedValueStableV1 {
            k: self.k(),
            slots_per_epoch: self.slots_per_epoch(),
            slots_per_sub_window: self.slots_per_sub_window(),
            grace_period_slots: self.grace_period_slots(),
            delta: self.delta(),
            genesis_state_timestamp: self.genesis_state_timestamp()?,
        })
    }
}
