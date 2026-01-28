//! # Merkle Path Verification Utilities
//!
//! This module provides utilities for computing and verifying Merkle tree paths,
//! which are essential for account existence proofs in the Mina ledger.
//!
//! [`calc_merkle_root_hash`] computes the root hash from an account and its
//! Merkle path, allowing verification that the account is part of a specific
//! ledger state.
//! This is commonly used in transaction verification (ensuring account exists).
//! It uses the Poseidon hash function, as specified in the Mina protocol.

use mina_hasher::{Hashable, Hasher, ROInput};
use mina_p2p_messages::{
    bigint::{BigInt, InvalidBigInt},
    v2::MerkleTreeNode,
};

// Wrapper for array of fields with domain height
#[derive(Clone)]
struct FieldsHashable(Vec<mina_curves::pasta::Fp>);
impl Hashable for FieldsHashable {
    type D = u32;
    fn to_roinput(&self) -> ROInput {
        let mut inputs = ROInput::new();
        for field in &self.0 {
            inputs = inputs.append_field(*field);
        }
        inputs
    }
    fn domain_string(height: u32) -> Option<String> {
        Some(format!("MinaMklTree{:03}", height))
    }
}

/// Computes the root hash of the merkle tree with an account and its merkle path
///
/// - The output of this method should be compared with the expected root hash
/// - Caller must ensure that the length of `merkle_path` is equal to the depth of the tree
pub fn calc_merkle_root_hash(
    account: &mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2,
    merkle_path: &[MerkleTreeNode],
) -> Result<BigInt, InvalidBigInt> {
    let account: ledger::Account = account.try_into()?;
    let mut child_hash = account.hash();

    for (height, path) in merkle_path.iter().enumerate() {
        let hashes = match path {
            MerkleTreeNode::Left(right) => [child_hash, right.to_field()?],
            MerkleTreeNode::Right(left) => [left.to_field()?, child_hash],
        };

        let mut hasher = mina_hasher::create_kimchi::<FieldsHashable>(height as u32);
        hasher.update(&FieldsHashable(hashes.to_vec()));
        child_hash = hasher.digest();
    }

    Ok(child_hash.into())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mina_p2p_messages::binprot::BinProtRead;

    use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    #[test]
    fn test_verify_merkle_path() {
        #![allow(const_item_mutation)]

        // let account = "...";
        // let merkle_path = "...";
        // let expected_root_hash = "jxTAZfKKDxoX4vtt68pQCWooXoVLjnfBpusaMwewrcZxsL3uWp6";

        // let account = hex::decode(account).unwrap();
        // let mut cursor = std::io::Cursor::new(account);
        // let account = MinaBaseAccountBinableArgStableV2::binprot_read(&mut cursor).unwrap();

        // let merkle_path = hex::decode(merkle_path).unwrap();
        // let mut cursor = std::io::Cursor::new(merkle_path);
        // let merkle_path = Vec::<MerkleTreeNode>::binprot_read(&mut cursor).unwrap();

        // let root_hash = calc_merkle_root_hash(&account, &merkle_path[..]).unwrap();

        // let expected_root_hash = LedgerHash::from_str(expected_root_hash).unwrap().0.clone();

        // assert_eq!(root_hash, expected_root_hash);
    }
}
