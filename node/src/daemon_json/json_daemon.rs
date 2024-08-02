use ledger::scan_state::currency::Slot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Daemon {
    txpool_max_size: Option<usize>,
    peer_list_url: Option<String>,
    slot_tx_end: Option<u32>,
    slot_chain_end: Option<u32>,
}

impl Daemon {
    pub const DEFAULT: Daemon = Daemon {
        txpool_max_size: Some(3000),
        peer_list_url: None,
        slot_tx_end: None,
        slot_chain_end: None,
    };

    pub fn tx_pool_max_size(&self) -> usize {
        self.txpool_max_size
            .unwrap_or(Self::DEFAULT.txpool_max_size.unwrap())
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
