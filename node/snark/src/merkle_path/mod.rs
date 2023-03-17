use std::fmt::Write;

use mina_p2p_messages::{bigint::BigInt, v2::MerkleTreeNode};

use crate::public_input::protocol_state::MinaHash;

mod account;

/// Computes the root hash of the merkle tree with an account and its merkle path
///
/// - The output of this method should be compared with the expected root hash
/// - Caller must ensure that the length of `merkle_path` is equal to the depth of the tree
pub fn calc_merkle_root_hash(
    account: &mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2,
    merkle_path: &[MerkleTreeNode],
) -> BigInt {
    let account_hash = account.hash();
    let mut param = String::with_capacity(16);

    merkle_path
        .iter()
        .enumerate()
        .fold(account_hash, |child, (depth, path)| {
            // TODO(binier): panics!
            let hashes = match path {
                MerkleTreeNode::Left(right) => [child, right.to_fp().unwrap()],
                MerkleTreeNode::Right(left) => [left.to_fp().unwrap(), child],
            };

            param.clear();
            write!(&mut param, "MinaMklTree{:03}", depth).unwrap();

            crate::hash::hash_with_kimchi(param.as_str(), &hashes)
        })
        .into()
}

#[cfg(test)]
mod tests {
    use binprot::BinProtRead;

    use mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    #[test]
    fn test_verify_merkle_path() {
        #![allow(const_item_mutation)]

        let account = "6c88f36f4a0fcbaaaf12e91f030ca5e33bd7b26a0b8e6c22bc2af33924f2ce3400010000000000000000000000000000000000000000000000000000000000000000fc00aaa0680b000000009be4b7c51ed9c2e4524727805fd36f5220fbfc70a749f62623b0ed2908433320016c88f36f4a0fcbaaaf12e91f030ca5e33bd7b26a0b8e6c22bc2af33924f2ce34000000000000000000000000000000000000000000000000000000000000000000000300030003030303030303030300";

        let merkle_path = "234f6812650aa27ce35d904b875809d380b2a7184dedd52d4c289274d6626e65ce5fff34354f681265d95c4d0208bcfd7d4ede27bbd1653d41ac8b0f37fe3fb6f39b5e8113df33f32279f722f9c7bbe7408ca42e90ef121191832b460ce9e990b3731abca9558f4df132614e294f68126502666e211f4d489e821916367014e5487bcbcaa582dc1154d8fdefd4b195ad1e79f722f9399038b193d310c012f421e9babd49367a32a3238eb02c584b936d5d07037b1f79f722f92bbd58fa3e868c956b31e5dfa31ad64f343694a46086659d9d63db0ddf70fb0d4f6812650c0b59c6d6ffab5339590603a2b00695d553784cc74e379cfa5c597266fbe0064f6812659c60712fd3e9663d535ade06b19c14a00d0d6214fc434bd374a34826dcfb7e1379f722f98422f50661c5e0c2b294bba3ebc22ff4f7f86f22d1611b308ea49e93e92d913b4f68126503518a63bb9daf70e3729f3922344dd470f721947cc07a4e4598ec871e4e64384f681265e16eed60ec1e56541360983741bde52a606f37da9495c6cd7244f9f30d9ac7154f681265ddaa309c792e62a1bbf6b4db04c323acf3a0fb702e1313c72755d7bbdb6c4f1e4f68126528405defcf11f365d0ccb31c9e68433441a8d0c77b3a798b7bb45d526715d43d79f722f96585a90bbfa518dafd94f5a2391a162299fd3c61c69b26be09be0c0905c4393d4f681265b2fca6df0ddacc2bb3561c695639837d39253baa3516f97c16556b1e7e6a7b3e4f681265ba91cd781a83e8f733213ef9817d2d958d26139adc4100c66150a169788cf0394f6812654e4fe5ed5ace8dc48426c601162e079b24b4adb72058d1211096ca709305f41f4f681265fb233966427765d8e0e0fb0116d5ee3bb10c5f41289193105c5b7c9c2a51c6094f681265ddf2b009d56e1f3bdfc22e9ef1850d097f6851458acf065816d443d2cd8894264f681265a9dc4535f5784e6148f2fdbcaa6e52d44999ce753cee4bad9de2df945129c1014f681265080aeeaab1058ef1663494607583ad838485b3abcfc5635b497f0c1aead8c2304f68126580be734b9057133b7d2c05187f18f2563dea8cf0bd238a17ee0242b60d98302c4f68126599ba4df1ad24dbc8090b66897d71f2a0cf21b1fb84d261b172e9333156358a3d4f681265f05f173a096c75c0f0148e426558139543535493a1933bd495d5a336e9eba1044f68126551c28fa437d4d89c1b839a1914529144cb3a3d9f8dc9cf4a95107e8cc9e5ee124f6812658ee873cbef184d38c2107cabd69ff87f710637ab9de8a1d7acb653949a72702c4f681265523e5324a58d7cca8ff8f40837656a7390e2515f265781aae422fff6a21b8b214f6812653555315baded133cd65e9d388fb7400f4323d5e79c44d7aee86a91712cdc30374f6812650bf6a75de59539f1be2a12bf307eaee979618e192c1e22d39fc53f98ab5375334f681265d199ee8af504dfc85afd7dd10da4e8872c096fc81e47dcfd2757ac6d9bd4312b4f6812659e2d0a145842af4119df8a7616e8a9687931e800cde90daaf3f7509aa081c10b4f68126593e74d2016c3711fb9486c5e4acb3435f5bf29ccfeefa37fde149bbee5b2430e4f6812651d7ba0bcbe637533740fcb73dfaaf254aea8830cc5555484479f80f2755f5b3d4f6812650146c059f09bc14cfadd69ebc5814dcf5a4301123a74bfa8f3514c5b161f81004f681265db914425a7d4c3bd6b9dc012a040cd94cb5857bb5051ccb6c61c90ada034f93d";

        let account = hex::decode(account).unwrap();
        let mut cursor = std::io::Cursor::new(account);
        let account = MinaBaseAccountBinableArgStableV2::binprot_read(&mut cursor).unwrap();

        let merkle_path = hex::decode(merkle_path).unwrap();
        let mut cursor = std::io::Cursor::new(merkle_path);
        let merkle_path = Vec::<MerkleTreeNode>::binprot_read(&mut cursor).unwrap();

        let root_hash = calc_merkle_root_hash(&account, &merkle_path[..]);

        let expected_root_hash =
            hex::decode("bd5ab37bd7df1f0330b015c9501ac2b279270ca19a083e69f41e80f65804723d")
                .unwrap();

        assert_eq!(root_hash.as_ref(), expected_root_hash);

        // /// Empty account with:
        // /// - token_id: 202
        // /// - token_symbol: "token"
        // const ACCOUNT_BYTES: &[u8] = &[
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 202, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 5, 116, 111, 107, 101, 110, 0, 0, 155, 228, 183, 197, 30,
        //     217, 194, 228, 82, 71, 39, 128, 95, 211, 111, 82, 32, 251, 252, 112, 167, 73, 246, 38,
        //     35, 176, 237, 41, 8, 67, 51, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0,
        // ];

        // let f = |s: &str| Fp::from_str(s).unwrap().into();

        // let merkle_path = [
        //     MerkleTreeNode::Right(f(
        //         "18227536250766436420332506719307927763848621557295827586492752720030361639151",
        //     )),
        //     MerkleTreeNode::Left(f(
        //         "19058089777055582893709373756417201639841391101434051152781561397928725072682",
        //     )),
        //     MerkleTreeNode::Left(f(
        //         "14567363183521815157220528341758405505341431484217567976728226651987143469014",
        //     )),
        //     MerkleTreeNode::Left(f(
        //         "24964477018986196734411365850696768955131362119835160146013237098764675419844",
        //     )),
        // ];

        // let account = MinaBaseAccountBinableArgStableV2::binprot_read(&mut ACCOUNT_BYTES).unwrap();

        // let root_hash = calc_merkle_root_hash(&account, &merkle_path[..]);
        // let expected_root_hash =
        //     f("15669071938119177277046978685444858793777121704378331620682194305905804366005");

        // assert_eq!(root_hash, expected_root_hash);
    }
}
