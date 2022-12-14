use binprot::BinProtWrite;
use mina_p2p_messages::v2::{
    TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    TransactionSnarkScanStateTransactionWithWitnessStableV2,
};

use super::parallel_scan::{ParallelScan, SpacePartition};

type LedgerProofWithSokMessage = TransactionSnarkScanStateLedgerProofWithSokMessageStableV2;
type TransactionWithWitness = TransactionSnarkScanStateTransactionWithWitnessStableV2;

type AvailableJob =
    super::parallel_scan::AvailableJob<TransactionWithWitness, LedgerProofWithSokMessage>;

struct ScanState {
    state: ParallelScan<TransactionWithWitness, LedgerProofWithSokMessage>,
}

impl ScanState {
    pub fn hash(&self) {
        // let mut buffer = Vec::with_capacity(32 * 1024);
        // let mut buffer2 = Vec::with_capacity(32 * 1024);

        // self.state.hash(|proof| {
        //     buffer.clear();

        //     // let mut bytes = Vec::with_capacity(10000);
        //     proof.binprot_write(&mut buffer).unwrap();
        //     buffer.as_slice()
        // }, |transaction| {
        //     buffer2.clear();
        //     // let mut buffer = Vec::with_capacity(10000);
        //     transaction.binprot_write(&mut buffer2).unwrap();
        //     buffer2.as_slice()
        // });

        // self.state.hash(|proof| {
        //     let mut bytes = Vec::with_capacity(10000);
        //     proof.binprot_write(&mut bytes).unwrap();
        //     bytes
        // }, |transaction| {
        //     let mut bytes = Vec::with_capacity(10000);
        //     transaction.binprot_write(&mut bytes).unwrap();
        //     bytes
        // });
    }
}
