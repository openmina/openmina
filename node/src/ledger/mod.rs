mod ledger_config;
pub use ledger_config::*;

mod ledger_service;
pub use ledger_service::*;

pub use ledger::AccountIndex as LedgerAccountIndex;
pub use ledger::Address as LedgerAddress;

use mina_p2p_messages::v2::LedgerHash;

pub const LEDGER_DEPTH: usize = 35;

lazy_static::lazy_static! {
    /// Array size needs to be changed when the tree's depth change
    static ref LEDGER_HASH_EMPTIES: [LedgerHash; LEDGER_DEPTH + 1] = {
        use mina_p2p_messages::v2::MinaBaseLedgerHash0StableV1;
        use ledger::TreeVersion;

        std::array::from_fn(|i| {
            let hash = ledger::V2::empty_hash_at_height(LEDGER_DEPTH - i);
            MinaBaseLedgerHash0StableV1(hash.into()).into()
        })
    };
}

pub fn ledger_empty_hash_at_depth(depth: usize) -> LedgerHash {
    LEDGER_HASH_EMPTIES.get(depth).unwrap().clone()
}
