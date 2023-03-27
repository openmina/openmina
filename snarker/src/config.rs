use serde::{Deserialize, Serialize};

pub use crate::p2p::P2pConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub p2p: P2pConfig,
}
