use mina_p2p_messages::v2::MinaBlockHeaderStableV2;

use super::SnarkBlockVerifyId;

pub trait SnarkBlockVerifyService: redux::Service {
    fn verify_init(&mut self, req_id: SnarkBlockVerifyId, block: &MinaBlockHeaderStableV2);
}
