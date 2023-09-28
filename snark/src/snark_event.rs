use serde::{Deserialize, Serialize};

use super::block_verify::{SnarkBlockVerifyError, SnarkBlockVerifyId};
use super::work_verify::{SnarkWorkVerifyError, SnarkWorkVerifyId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkEvent {
    BlockVerify(SnarkBlockVerifyId, Result<(), SnarkBlockVerifyError>),
    WorkVerify(SnarkWorkVerifyId, Result<(), SnarkWorkVerifyError>),
}

fn res_kind<T, E>(res: &Result<T, E>) -> &'static str {
    match res {
        Err(_) => "Err",
        Ok(_) => "Ok",
    }
}

impl std::fmt::Display for SnarkEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Snark, ")?;
        match self {
            Self::BlockVerify(id, res) => {
                write!(f, "BlockVerify, {id}, {}", res_kind(res))
            }
            Self::WorkVerify(id, res) => {
                write!(f, "WorkVerify, {id}, {}", res_kind(res))
            }
        }
    }
}
