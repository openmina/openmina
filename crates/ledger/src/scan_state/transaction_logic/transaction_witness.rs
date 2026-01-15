use super::{Transaction, TransactionStatus};
use crate::{
    scan_state::{currency::Slot, pending_coinbase::Stack},
    sparse_ledger::SparseLedger,
};
use mina_p2p_messages::v2::MinaStateProtocolStateBodyValueStableV2;

/// <https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction_witness/transaction_witness.ml#L55>
#[derive(Debug)]
pub struct TransactionWitness {
    pub transaction: Transaction,
    pub first_pass_ledger: SparseLedger,
    pub second_pass_ledger: SparseLedger,
    pub protocol_state_body: MinaStateProtocolStateBodyValueStableV2,
    pub init_stack: Stack,
    pub status: TransactionStatus,
    pub block_global_slot: Slot,
}
