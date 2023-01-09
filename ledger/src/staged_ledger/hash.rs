use ark_ff::{PrimeField, ToBytes};
use mina_hasher::Fp;
use sha2::{Digest, Sha256};

use crate::{scan_state::pending_coinbase::PendingCoinbase, ToInputs};

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L27
#[derive(Debug, PartialEq, Eq)]
pub struct AuxHash(pub [u8; 32]);

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L110
#[derive(Debug, PartialEq, Eq)]
pub struct PendingCoinbaseAux(pub [u8; 32]);

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L152
#[derive(Debug, PartialEq, Eq)]
pub struct NonStark {
    ledger_hash: Fp,
    aux_hash: AuxHash,
    pending_coinbase_aux: PendingCoinbaseAux,
}

impl NonStark {
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L182
    pub fn digest(&self) -> [u8; 32] {
        let Self {
            ledger_hash,
            aux_hash,
            pending_coinbase_aux,
        } = self;

        let mut sha: Sha256 = Sha256::new();

        let mut ledger_hash_bytes: [u8; 32] = <[u8; 32]>::default();

        let ledger_hash = ledger_hash.into_repr();
        ledger_hash.write(ledger_hash_bytes.as_mut_slice()).unwrap();

        sha.update(ledger_hash_bytes.as_slice());
        sha.update(aux_hash.0.as_slice());
        sha.update(pending_coinbase_aux.0.as_slice());

        sha.finalize().into()
    }
}

impl ToInputs for NonStark {
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L193
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        let digest = self.digest();
        inputs.append_bytes(digest.as_slice());
    }
}

/// Staged ledger hash has two parts
///
/// 1) merkle root of the pending coinbases
/// 2) ledger hash, aux hash, and the FIFO order of the coinbase stacks(Non snark).
///
/// Only part 1 is required for blockchain snark computation and therefore the
/// remaining fields of the staged ledger are grouped together as "Non_snark"
///
/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L259
#[derive(Debug, PartialEq, Eq)]
pub struct StagedLedgerHash {
    non_snark: NonStark,
    pending_coinbase_hash: Fp,
}

impl StagedLedgerHash {
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L290
    pub fn of_aux_ledger_and_coinbase_hash(
        aux_hash: AuxHash,
        ledger_hash: Fp,
        pending_coinbase: &mut PendingCoinbase,
    ) -> Self {
        Self {
            non_snark: NonStark {
                ledger_hash,
                aux_hash,
                pending_coinbase_aux: pending_coinbase.hash_extra(),
            },
            pending_coinbase_hash: pending_coinbase.merkle_root(),
        }
    }
}

impl ToInputs for StagedLedgerHash {
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        let Self {
            non_snark,
            pending_coinbase_hash,
        } = self;

        inputs.append(non_snark);
        inputs.append(pending_coinbase_hash);
    }
}
