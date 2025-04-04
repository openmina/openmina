//! Block producer configuration module.
//! Defines the configuration parameters for block production.

use mina_p2p_messages::v2::{NonZeroCurvePoint, ProtocolVersionStableV2};
use serde::{Deserialize, Serialize};

/// Configuration for the block producer.
/// Contains the public key for block production and optional settings.
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

    /// Returns the coinbase receiver public key.
    ///
    /// If a custom coinbase receiver is set, returns that key.
    /// Otherwise, returns the block producer's public key.
    pub fn coinbase_receiver(&self) -> &NonZeroCurvePoint {
        self.custom_coinbase_receiver
            .as_ref()
            .unwrap_or(&self.pub_key)
    }
}
