use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use mina_p2p_messages::{
    number::Number,
    v2::{BlockTimeTimeStableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genesis {
    genesis_state_timestamp: String,
}

impl Genesis {
    pub fn genesis_state_timestamp(&self) -> Result<BlockTimeTimeStableV1, time::error::Parse> {
        OffsetDateTime::parse(&self.genesis_state_timestamp, &Rfc3339).map(|dt| {
            BlockTimeTimeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(Number(
                dt.unix_timestamp() as u64,
            )))
        })
    }
}
