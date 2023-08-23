use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use serde::{Deserialize, Serialize};

use super::SnarkJobId;

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct SnarkInfo {
    pub job_id: SnarkJobId,
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}
