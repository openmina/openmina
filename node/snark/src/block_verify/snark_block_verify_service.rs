use std::sync::Arc;

use mina_p2p_messages::v2::MinaBlockHeaderStableV2;

use crate::VerifierIndex;

use super::SnarkBlockVerifyId;

pub trait SnarkBlockVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        block: &MinaBlockHeaderStableV2,
    );
}
