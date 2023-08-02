use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerError {
    BinprotError(String),
    IOError(String),
    NotRunning,
    Busy,
    /// Protocol logic is broken
    Broken(String),
}


#[derive(Clone, Debug, thiserror::Error, derive_more::From, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerWorkError {
    #[error("snark work error: {_0}")]
    Error(String)
}
