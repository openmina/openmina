use ledger::scan_state::currency::Slot;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Daemon {
    txpool_max_size: Option<u32>,
    peer_list_url: Option<String>,
    slot_tx_end: Option<u32>,
    slot_chain_end: Option<u32>,
}

impl Daemon {
    pub fn tx_pool_max_size(&self) -> u32 {
        self.txpool_max_size.unwrap_or(3000)
    }

    pub fn peer_list_url(&self) -> Option<String> {
        self.peer_list_url.clone()
    }

    pub fn slot_tx_end(&self) -> Option<Slot> {
        self.slot_tx_end.map(Slot::from_u32)
    }

    pub fn slot_chain_end(&self) -> Option<Slot> {
        self.slot_chain_end.map(Slot::from_u32)
    }
}
