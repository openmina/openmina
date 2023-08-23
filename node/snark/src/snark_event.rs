use serde::{Deserialize, Serialize};

use super::block_verify::{SnarkBlockVerifyError, SnarkBlockVerifyId};
use super::work_verify::{SnarkWorkVerifyError, SnarkWorkVerifyId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkEvent {
    BlockVerify(SnarkBlockVerifyId, Result<(), SnarkBlockVerifyError>),
    WorkVerify(SnarkWorkVerifyId, Result<(), SnarkWorkVerifyError>),
}
