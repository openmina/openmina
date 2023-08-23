use std::sync::Arc;

use shared::snark::Snark;

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkWorkVerifyId;

pub trait SnarkWorkVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        work: Vec<Snark>,
    );
}
