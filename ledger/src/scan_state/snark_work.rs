pub mod spec {
    use crate::scan_state::{
        scan_state::transaction_snark::{LedgerProof, Statement},
        transaction_logic::transaction_witness::TransactionWitness,
    };

    pub enum Work {
        Transition((Box<Statement>, TransactionWitness)),
        Merge((Statement, Box<(LedgerProof, LedgerProof)>)),
    }
}
