pub trait ProofConstants {
    /// Number of public inputs
    const PRIMARY_LEN: usize;
    /// Number of witness values
    const AUX_LEN: usize;
    const PREVIOUS_CHALLENGES: usize;
    const ROWS: usize;
}

pub struct RegularTransactionProof {}
pub struct WrapProof {}
pub struct WrapBlockProof {}
pub struct MergeProof {}
pub struct BlockProof {}

impl ProofConstants for RegularTransactionProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 94386;
    const PREVIOUS_CHALLENGES: usize = 0;
    const ROWS: usize = 17794;
}

impl ProofConstants for WrapProof {
    const PRIMARY_LEN: usize = 40;
    const AUX_LEN: usize = 179491;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 15122;
}

impl ProofConstants for WrapBlockProof {
    const PRIMARY_LEN: usize = 40;
    const AUX_LEN: usize = 179248;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 14657;
}

impl ProofConstants for MergeProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 352536;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 29010;
}

impl ProofConstants for BlockProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 339034;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 34811;
}
