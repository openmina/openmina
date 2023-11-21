use std::sync::Arc;

use crate::{VerifierIndex, VerifierSRS};

use super::{SnarkBlockVerifyId, VerifiableBlockWithHash};

/// Snark block verification is a CPU intensive operation that can run for
/// long periods, for this reason it has to be implemented as a service.
/// 
/// When called, the `verify_init` function spawns a `rayon` task to perform
/// the block proof verification. When verification is completed (either with
/// success or error) the service reports back the result as a
/// `SnarkEvent::BlockVerify` event.
///  
pub trait SnarkBlockVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        block: VerifiableBlockWithHash,
    );
}
