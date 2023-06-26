use serde::{Deserialize, Serialize};

use super::LedgerConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerState {
    pub config: LedgerConfig,
}

impl LedgerState {
    pub fn new(config: LedgerConfig) -> Self {
        Self { config }
    }
}
