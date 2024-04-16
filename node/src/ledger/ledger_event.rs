use mina_p2p_messages::v2::LedgerHash;
use serde::{Deserialize, Serialize};
use std::fmt;

/// This type represents Events raised by the LedgerManager in response to
/// asynchronous requests. Functions making asynchronous requests will always
/// return `Result<(), String>` immediately, while the actual result of
/// computation will be delivered via one or more of these events.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LedgerEvent {
    LedgerReconstructSuccess(LedgerHash),
    LedgerReconstructError(String),
}

impl fmt::Display for LedgerEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LedgerEvent::LedgerReconstructSuccess(ledger_hash) => {
                write!(f, "LedgerReconstructSuccess: {}", ledger_hash)
            }
            LedgerEvent::LedgerReconstructError(msg) => {
                write!(f, "LedgerReconstructError: {}", msg)
            }
        }
    }
}
