mod snark_user_command_verify_effectful_actions;
pub use snark_user_command_verify_effectful_actions::*;

mod snark_user_command_verify_effects;

mod snark_user_command_verify_service;
pub use snark_user_command_verify_service::*;

use serde::{Deserialize, Serialize};

pub struct SnarkUserCommandVerifyIdType;
impl openmina_core::requests::RequestIdType for SnarkUserCommandVerifyIdType {
    fn request_id_type() -> &'static str {
        "SnarkUserCommandVerifyId"
    }
}

pub type SnarkUserCommandVerifyId =
    openmina_core::requests::RequestId<SnarkUserCommandVerifyIdType>;

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum SnarkUserCommandVerifyError {
    #[error("verification failed")]
    VerificationFailed,
    #[error("validator thread crashed")]
    ValidatorThreadCrashed,
}
