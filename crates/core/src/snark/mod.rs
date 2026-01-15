mod snark_job_id;
pub use snark_job_id::SnarkJobId;

mod snark_job_commitment;
pub use snark_job_commitment::SnarkJobCommitment;

mod snark_info;
pub use snark_info::SnarkInfo;

#[allow(clippy::module_inception)]
mod snark;
pub use snark::Snark;

mod snark_cmp;
pub use snark_cmp::SnarkCmp;

use mina_p2p_messages::v2::NonZeroCurvePoint;

pub fn tie_breaker_hash(job_id: &SnarkJobId, snarker: &NonZeroCurvePoint) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(job_id.source.first_pass_ledger.to_bytes());
    hasher.update(job_id.source.second_pass_ledger.to_bytes());
    hasher.update(job_id.target.first_pass_ledger.to_bytes());
    hasher.update(job_id.target.second_pass_ledger.to_bytes());
    hasher.update(snarker.x.to_bytes());
    hasher.finalize().into()
}
