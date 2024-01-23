use std::sync::{Arc, Mutex};

use openmina_core::snark::Snark;

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkWorkVerifyId;

pub trait SnarkWorkVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
        work: Vec<Snark>,
    );
}
