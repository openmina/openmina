use serde::{Deserialize, Serialize};

use super::read::{LedgerReadId, LedgerReadResponse};
use super::write::LedgerWriteResponse;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerEvent {
    Write(LedgerWriteResponse),
    Read(LedgerReadId, LedgerReadResponse),
}

impl std::fmt::Display for LedgerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ledger, ")?;

        match self {
            Self::Write(resp) => {
                write!(f, "Write, {:?}", resp.kind())?;
                match resp {
                    LedgerWriteResponse::StagedLedgerReconstruct {
                        staged_ledger_hash,
                        result,
                    } => {
                        write!(f, ", {staged_ledger_hash}, {}", res_kind_str(result))
                    }
                    LedgerWriteResponse::StagedLedgerDiffCreate {
                        pred_block_hash,
                        global_slot_since_genesis,
                        result,
                    } => {
                        write!(
                            f,
                            ", {pred_block_hash}, {}, {}",
                            global_slot_since_genesis.as_u32(),
                            res_kind_str(result)
                        )
                    }
                    LedgerWriteResponse::BlockApply { block_hash, result } => {
                        write!(f, ", {block_hash}, {}", res_kind_str(result))
                    }
                    LedgerWriteResponse::Commit { best_tip_hash, .. } => {
                        write!(f, ", {best_tip_hash}")
                    }
                }
            }
            Self::Read(id, resp) => {
                write!(f, "Read, {:?}, {id}", resp.kind())
            }
        }
    }
}

fn res_kind_str<T, E>(res: &Result<T, E>) -> &'static str {
    match res {
        Err(_) => "Err",
        Ok(_) => "Ok",
    }
}
