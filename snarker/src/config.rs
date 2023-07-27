use serde::{Deserialize, Serialize};

use crate::account::AccountPublicKey;
pub use crate::ledger::LedgerConfig;
pub use crate::p2p::P2pConfig;
pub use crate::snark::SnarkConfig;
pub use crate::snark_pool::SnarkPoolConfig;
pub use crate::transition_frontier::TransitionFrontierConfig;
pub use mina_p2p_messages::v2::MinaBaseProtocolConstantsCheckedValueStableV1 as ProtocolConstants;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ledger: LedgerConfig,
    pub snark: SnarkConfig,
    pub p2p: P2pConfig,
    pub transition_frontier: TransitionFrontierConfig,
    pub snarker: SnarkerConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkerConfig {
    pub public_key: AccountPublicKey,
    pub job_commitments: SnarkPoolConfig,
}
