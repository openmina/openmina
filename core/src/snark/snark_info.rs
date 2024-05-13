use mina_p2p_messages::binprot::macros::{BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use serde::{Deserialize, Serialize};

use super::SnarkJobId;

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct SnarkInfo {
    pub job_id: SnarkJobId,
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

impl SnarkInfo {
    pub fn tie_breaker_hash(&self) -> [u8; 32] {
        super::tie_breaker_hash(&self.job_id, &self.prover)
    }
}
