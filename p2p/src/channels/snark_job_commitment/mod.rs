mod p2p_channels_snark_job_commitment_state;
pub use p2p_channels_snark_job_commitment_state::*;

mod p2p_channels_snark_job_commitment_actions;
pub use p2p_channels_snark_job_commitment_actions::*;

mod p2p_channels_snark_job_commitment_reducer;
pub use p2p_channels_snark_job_commitment_reducer::*;

mod p2p_channels_snark_job_commitment_effects;
pub use p2p_channels_snark_job_commitment_effects::*;

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};
use shared::snark::SnarkJobCommitment;

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
