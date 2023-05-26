use serde::{Deserialize, Serialize};

use crate::account::AccountPublicKey;
pub use crate::job_commitment::JobCommitmentsConfig;
pub use crate::p2p::P2pConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub p2p: P2pConfig,
    pub snarker: SnarkerConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkerConfig {
    pub public_key: AccountPublicKey,
    pub job_commitments: JobCommitmentsConfig,
}
