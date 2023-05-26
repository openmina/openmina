mod p2p_channels_snark_job_commitment_state;
pub use p2p_channels_snark_job_commitment_state::*;

mod p2p_channels_snark_job_commitment_actions;
pub use p2p_channels_snark_job_commitment_actions::*;

mod p2p_channels_snark_job_commitment_reducer;
pub use p2p_channels_snark_job_commitment_reducer::*;

mod p2p_channels_snark_job_commitment_effects;
pub use p2p_channels_snark_job_commitment_effects::*;

use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::{LedgerHash, MinaBaseSignatureStableV1, NonZeroCurvePoint};
use redux::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkJobCommitmentPropagationChannelMsg {
    /// Request next commitments upto the `limit`.
    ///
    /// - Must not be sent until peer sends `WillSend` message for the
    ///   previous request and until peer has fulfilled it.
    GetNext { limit: u8 },
    /// Amount of commitments which will proceed this message.
    ///
    /// - Can only be sent, if peer has sent `GetNext` and we haven't
    ///   responded with `WillSend` yet.
    /// - Can't be bigger than limit set by `GetNext`.
    /// - Amount of promised commitments must be delivered.
    WillSend { count: u8 },
    /// Promise/Commitments from the snark worker to produce a proof.
    Commitment(SnarkJobCommitment),
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct SnarkJobCommitment {
    /// Timestamp in milliseconds.
    /// TODO(binier): have to use i64, because binprot doesn't support u64.
    timestamp: i64,
    pub job_id: SnarkJobId,
    pub snarker: NonZeroCurvePoint,
    // pub signature: MinaBaseSignatureStableV1,
}

impl SnarkJobCommitment {
    pub fn new(timestamp: u64, job_id: SnarkJobId, snarker: NonZeroCurvePoint) -> Self {
        Self {
            timestamp: timestamp as i64,
            job_id,
            snarker,
            // TODO(binier): SEC have the snarkers sign the commitment.
            // signature: todo!(),
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        Timestamp::new(self.timestamp as u64 * 1_000_000)
    }
}

#[derive(
    BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
pub enum SnarkJobId {
    One(LedgerHashTransition),
    Two(LedgerHashTransition, LedgerHashTransition),
}

#[derive(
    BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
pub struct LedgerHashTransition {
    pub source: LedgerHashTransitionPasses,
    pub target: LedgerHashTransitionPasses,
}

#[derive(
    BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
pub struct LedgerHashTransitionPasses {
    pub first_pass_ledger: LedgerHash,
    pub second_pass_ledger: LedgerHash,
}
