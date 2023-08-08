use serde::{Deserialize, Serialize};

use super::SnarkWorkSpecError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerError {
    BinprotError(String),
    IOError(String),
    Error(String),
    NotRunning,
    Busy,
    /// Protocol logic is broken
    Broken(String),
}

#[derive(Clone, Debug, derive_more::From, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerWorkError {
    WorkSpecError(SnarkWorkSpecError),
    WorkerError(ExternalSnarkWorkerError),
    Cancelled,
    Error(String),
}
