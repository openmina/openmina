use std::sync::Arc;

use openmina_core::snark::Snark;

use crate::{TransactionVerifier, VerifierSRS};

use super::SnarkWorkVerifyId;

pub trait SnarkWorkVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: TransactionVerifier,
        verifier_srs: Arc<VerifierSRS>,
        work: Vec<Snark>,
    );
}
