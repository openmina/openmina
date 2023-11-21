use std::sync::Arc;

use openmina_core::snark::Snark;

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkWorkVerifyId;

/// Snark work verification is a CPU intensive operation that can run for
/// long periods, for this reason it has to be implemented as a service.
/// 
/// When called, the `verify_init` function spawns a `rayon` task to perform
/// the transaction snark proof verification. When verification is completed
/// (either with success or error) the service reports back the result as a
/// `SnarkEvent::WorkVerify` event.
///  
pub trait SnarkWorkVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        work: Vec<Snark>,
    );
}
