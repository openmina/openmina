use serde::{Deserialize, Serialize};

use super::SnarkWorkSpecError;

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
pub enum ExternalSnarkWorkerError {
    #[error("error decoding binprot: {_0}")]
    BinprotError(String),
    #[error("I/O error: {_0}")]
    IOError(String),
    #[error("timeout starting external worker")]
    StartTimeout,
    #[error("other error: {_0}")]
    Error(String),
    #[error("snark worker is not running")]
    NotRunning,
    #[error("snark worker is busy")]
    Busy,
    /// Protocol logic is broken
    #[error("redux logic is broken: {_0}")]
    Broken(String),
}

#[derive(Clone, Debug, derive_more::From, Serialize, Deserialize, thiserror::Error)]
pub enum ExternalSnarkWorkerWorkError {
    #[error("invalid snark work specification: {_0}")]
    WorkSpecError(SnarkWorkSpecError),
    #[error("snark worker error: {_0}")]
    WorkerError(ExternalSnarkWorkerError),
    #[error("work is cancelled")]
    Cancelled,
    #[error("error producing snark: {_0}")]
    Error(String),
}
