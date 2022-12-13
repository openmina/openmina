use std::fmt::Write;

use mina_hasher::Fp;

use crate::public_input::protocol_state::MinaHash;

mod account;

pub enum MerklePath {
    Left(Fp),
    Right(Fp),
}

/// Computes the root hash of the merkle tree with an account and its merkle path
///
/// - The output of this method should be compared with the expected root hash
/// - Caller must ensure that the length of `merkle_path` is equal to the depth of the tree
pub fn verify_merkle_path(
    account: &mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2,
    merkle_path: &[MerklePath],
) -> Fp {
    let account_hash = account.hash();
    let mut param = String::with_capacity(16);

    merkle_path
        .iter()
        .enumerate()
        .fold(account_hash, |child, (depth, path)| {
            let hashes = match path {
                MerklePath::Left(right) => [child, *right],
                MerklePath::Right(left) => [*left, child],
            };

            param.clear();
            write!(&mut param, "MinaMklTree{:03}", depth).unwrap();

            crate::hash::hash_with_kimchi(param.as_str(), &hashes)
        })
}

#[cfg(test)]
mod tests {
    use binprot::BinProtRead;
    use std::str::FromStr;

    use mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2;
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    #[test]
    fn test_verify_merkle_path() {
        #![allow(const_item_mutation)]

        /// Empty account with:
        /// - token_id: 202
        /// - token_symbol: "token"
        const ACCOUNT_BYTES: &[u8] = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 202, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 5, 116, 111, 107, 101, 110, 0, 0, 155, 228, 183, 197, 30,
            217, 194, 228, 82, 71, 39, 128, 95, 211, 111, 82, 32, 251, 252, 112, 167, 73, 246, 38,
            35, 176, 237, 41, 8, 67, 51, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0,
        ];

        let f = |s: &str| Fp::from_str(s).unwrap();

        let merkle_path = [
            MerklePath::Right(f(
                "18227536250766436420332506719307927763848621557295827586492752720030361639151",
            )),
            MerklePath::Left(f(
                "19058089777055582893709373756417201639841391101434051152781561397928725072682",
            )),
            MerklePath::Left(f(
                "14567363183521815157220528341758405505341431484217567976728226651987143469014",
            )),
            MerklePath::Left(f(
                "24964477018986196734411365850696768955131362119835160146013237098764675419844",
            )),
        ];

        let account = MinaBaseAccountBinableArgStableV2::binprot_read(&mut ACCOUNT_BYTES).unwrap();

        let root_hash = verify_merkle_path(&account, &merkle_path[..]);
        let expected_root_hash =
            f("15669071938119177277046978685444858793777121704378331620682194305905804366005");

        assert_eq!(root_hash, expected_root_hash);
    }
}
