mod snark_block_verify_effectful_actions;
pub use snark_block_verify_effectful_actions::*;

mod snark_block_verify_effects;

mod snark_block_verify_service;
pub use snark_block_verify_service::*;

use serde::{Deserialize, Serialize};

pub struct SnarkBlockVerifyIdType;
impl openmina_core::requests::RequestIdType for SnarkBlockVerifyIdType {
    fn request_id_type() -> &'static str {
        "SnarkBlockVerifyId"
    }
}

pub type SnarkBlockVerifyId = openmina_core::requests::RequestId<SnarkBlockVerifyIdType>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkBlockVerifyError {
    AccumulatorCheckFailed,
    VerificationFailed,
    ValidatorThreadCrashed,
}
