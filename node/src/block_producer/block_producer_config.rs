use mina_p2p_messages::v2::{NonZeroCurvePoint, ProtocolVersionStableV1};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerConfig {
    pub pub_key: NonZeroCurvePoint,
    pub custom_coinbase_receiver: Option<NonZeroCurvePoint>,
    pub proposed_protocol_version: Option<ProtocolVersionStableV1>,
}

impl BlockProducerConfig {
    pub fn coinbase_receiver(&self) -> &NonZeroCurvePoint {
        self.custom_coinbase_receiver
            .as_ref()
            .unwrap_or(&self.pub_key)
    }
}
