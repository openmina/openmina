mod snark_work_verify_state;
pub use snark_work_verify_state::*;

mod snark_work_verify_actions;
pub use snark_work_verify_actions::*;

mod snark_work_verify_reducer;


mod snark_work_verify_effects;


mod snark_work_verify_service;
pub use snark_work_verify_service::*;

use serde::{Deserialize, Serialize};

pub struct SnarkWorkVerifyIdType;
impl openmina_core::requests::RequestIdType for SnarkWorkVerifyIdType {
    fn request_id_type() -> &'static str {
        "SnarkWorkVerifyId"
    }
}

pub type SnarkWorkVerifyId = openmina_core::requests::RequestId<SnarkWorkVerifyIdType>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkWorkVerifyError {
    VerificationFailed,
    ValidatorThreadCrashed,
}
