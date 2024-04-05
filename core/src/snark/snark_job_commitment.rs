use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};
use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use super::SnarkJobId;

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct SnarkJobCommitment {
    timestamp: u64,
    pub job_id: SnarkJobId,
    pub fee: CurrencyFeeStableV1,
    pub snarker: NonZeroCurvePoint,
    // pub signature: MinaBaseSignatureStableV1,
}

impl SnarkJobCommitment {
    pub fn new(
        timestamp: u64,
        job_id: SnarkJobId,
        fee: CurrencyFeeStableV1,
        snarker: NonZeroCurvePoint,
    ) -> Self {
        Self {
            timestamp,
            job_id,
            fee,
            snarker,
            // TODO(binier): SEC have the snarkers sign the commitment.
            // signature: todo!(),
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        Timestamp::new(self.timestamp as u64 * 1_000_000)
    }

    pub fn tie_breaker_hash(&self) -> [u8; 32] {
        super::tie_breaker_hash(&self.job_id, &self.snarker)
    }
}
