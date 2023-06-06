use serde::{Deserialize, Serialize};

use crate::block_verify::{SnarkBlockVerifyError, SnarkBlockVerifyId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkEvent {
    BlockVerify(SnarkBlockVerifyId, Result<(), SnarkBlockVerifyError>),
}
