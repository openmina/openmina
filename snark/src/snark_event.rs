use ledger::scan_state::transaction_logic::valid;
use serde::{Deserialize, Serialize};

use super::block_verify::{SnarkBlockVerifyError, SnarkBlockVerifyId};
use super::work_verify::{SnarkWorkVerifyError, SnarkWorkVerifyId};
use crate::user_command_verify::SnarkUserCommandVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkEvent {
    BlockVerify(SnarkBlockVerifyId, Result<(), SnarkBlockVerifyError>),
    WorkVerify(SnarkWorkVerifyId, Result<(), SnarkWorkVerifyError>),
    UserCommandVerify(
        SnarkUserCommandVerifyId,
        Vec<Result<valid::UserCommand, String>>,
    ),
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
            Self::UserCommandVerify(id, res) => {
                let n_failed = res.iter().filter(|res| res.is_err()).count();
                let n_success = res.len() - n_failed;
                write!(
                    f,
                    "UserCommandVerify, {id}, n_success={} n_failed={}",
                    n_success, n_failed
                )
            }
        }
    }
}
