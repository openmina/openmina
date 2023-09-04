use std::sync::Arc;

use crate::{VerifierIndex, VerifierSRS};

use super::{SnarkBlockVerifyId, VerifiableBlockWithHash};

pub trait SnarkBlockVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        block: VerifiableBlockWithHash,
    );
}
