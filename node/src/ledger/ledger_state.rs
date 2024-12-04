use super::{read::LedgerReadState, write::LedgerWriteState, LedgerConfig};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LedgerState {
    pub write: LedgerWriteState,
    pub read: LedgerReadState,
}

impl LedgerState {
    pub fn new(_config: LedgerConfig) -> Self {
        Self::default()
    }
}
