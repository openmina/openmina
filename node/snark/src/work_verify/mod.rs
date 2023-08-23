mod snark_work_verify_state;
pub use snark_work_verify_state::*;

mod snark_work_verify_actions;
pub use snark_work_verify_actions::*;

mod snark_work_verify_reducer;
pub use snark_work_verify_reducer::*;

mod snark_work_verify_effects;
pub use snark_work_verify_effects::*;

mod snark_work_verify_service;
pub use snark_work_verify_service::*;

use serde::{Deserialize, Serialize};

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnarkWorkVerifyIdType;
impl shared::requests::RequestIdType for SnarkWorkVerifyIdType {
    fn request_id_type() -> &'static str {
        "SnarkWorkVerifyId"
    }
}

pub type SnarkWorkVerifyId = shared::requests::RequestId<SnarkWorkVerifyIdType>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkWorkVerifyError {
    VerificationFailed,
    ValidatorThreadCrashed,
}
