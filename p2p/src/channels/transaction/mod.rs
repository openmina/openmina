mod p2p_channels_transaction_state;
pub use p2p_channels_transaction_state::*;

mod p2p_channels_transaction_actions;
pub use p2p_channels_transaction_actions::*;

mod p2p_channels_transaction_reducer;

use binprot_derive::{BinProtRead, BinProtWrite};
pub use openmina_core::transaction::{Transaction, TransactionHash, TransactionInfo};
use serde::{Deserialize, Serialize};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPropagationChannelMsg {
    /// Request next transactions upto the `limit`.
    ///
    /// - Must not be sent until peer sends `WillSend` message for the
    ///   previous request and until peer has fulfilled it.
    GetNext { limit: u8 },
    /// Amount of transactions which will proceed this message.
    ///
    /// - Can only be sent, if peer has sent `GetNext` and we haven't
    ///   responded with `WillSend` yet.
    /// - Can't be bigger than limit set by `GetNext`.
    /// - Amount of promised transactions must be delivered.
    WillSend { count: u8 },
    /// Transaction.
    Transaction(TransactionInfo),
}
