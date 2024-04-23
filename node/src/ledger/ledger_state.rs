use serde::{Deserialize, Serialize};

use super::read::LedgerReadState;
use super::write::LedgerWriteState;
use super::LedgerConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerState {
    pub write: LedgerWriteState,
    pub read: LedgerReadState,
}

impl LedgerState {
    pub fn new(_config: LedgerConfig) -> Self {
        Self {
            write: Default::default(),
            read: Default::default(),
        }
    }
}
