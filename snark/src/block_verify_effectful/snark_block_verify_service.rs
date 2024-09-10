use std::sync::Arc;

use crate::{block_verify::VerifiableBlockWithHash, VerifierIndex, VerifierSRS};

use super::SnarkBlockVerifyId;

pub trait SnarkBlockVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        block: VerifiableBlockWithHash,
    );
}
