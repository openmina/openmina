use std::sync::Arc;

use mina_p2p_messages::v2::MinaBlockHeaderStableV2;

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkBlockVerifyId;

pub trait SnarkBlockVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        block: &MinaBlockHeaderStableV2,
    );
}
