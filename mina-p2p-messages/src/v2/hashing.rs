use super::generated;
use sha2::{
    digest::{generic_array::GenericArray, typenum::U32},
    Digest, Sha256,
};

impl generated::MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub fn sha256(&self) -> GenericArray<u8, U32> {
        let mut ledger_hash_bytes: [u8; 32] = [0; 32];

        ledger_hash_bytes.copy_from_slice(self.ledger_hash.as_ref());
        ledger_hash_bytes.reverse();

        let mut hasher = Sha256::new();
        hasher.update(ledger_hash_bytes);
        hasher.update(self.aux_hash.as_ref());
        hasher.update(self.pending_coinbase_aux.as_ref());

        hasher.finalize()
    }
}
