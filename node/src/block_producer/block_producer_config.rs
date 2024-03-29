use mina_p2p_messages::v2::{NonZeroCurvePoint, ProtocolVersionStableV2};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerConfig {
    pub pub_key: NonZeroCurvePoint,
    pub custom_coinbase_receiver: Option<NonZeroCurvePoint>,
    pub proposed_protocol_version: Option<ProtocolVersionStableV2>,
}

impl BlockProducerConfig {
    pub fn new(pub_key: NonZeroCurvePoint) -> Self {
        Self {
            pub_key,
            custom_coinbase_receiver: None,
            proposed_protocol_version: None,
        }
    }

    pub fn coinbase_receiver(&self) -> &NonZeroCurvePoint {
        self.custom_coinbase_receiver
            .as_ref()
            .unwrap_or(&self.pub_key)
    }
}
