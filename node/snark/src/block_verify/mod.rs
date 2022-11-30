mod snark_block_verify_state;
pub use snark_block_verify_state::*;

mod snark_block_verify_actions;
pub use snark_block_verify_actions::*;

mod snark_block_verify_reducer;
pub use snark_block_verify_reducer::*;

mod snark_block_verify_effects;
pub use snark_block_verify_effects::*;

mod snark_block_verify_service;
pub use snark_block_verify_service::*;

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnarkBlockVerifyIdType;
impl shared::requests::RequestIdType for SnarkBlockVerifyIdType {
    fn request_id_type() -> &'static str {
        "SnarkBlockVerifyId"
    }
}

pub type SnarkBlockVerifyId = shared::requests::RequestId<SnarkBlockVerifyIdType>;
