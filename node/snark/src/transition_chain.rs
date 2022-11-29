use mina_hasher::Fp;

use crate::public_input::protocol_state;

/// https://github.com/MinaProtocol/mina/blob/aebd4e552b8b4bcd78d1e24523169e8778794857/src/lib/merkle_list_verifier/merkle_list_verifier.ml#L36
fn verify_impl<'a, T>(
    init_state_hash: Fp,
    target_hash: Fp,
    merkle_list_len: usize,
    merkle_list_iter: T,
) -> Option<Vec<Fp>>
where
    T: Iterator<Item = &'a Fp>,
{
    let mut hashes = Vec::with_capacity(merkle_list_len + 1);
    hashes.push(init_state_hash);

    for proof_elem in merkle_list_iter {
        let last = hashes.last().unwrap();
        hashes.push(protocol_state::hashes_abstract(*last, *proof_elem));
    }

    if hashes.last().unwrap() == &target_hash {
        hashes.reverse();
        Some(hashes)
    } else {
        None
    }
}

pub fn verify(target_hash: Fp, transition_chain_proof: (Fp, Vec<Fp>)) -> Option<Vec<Fp>> {
    let init_state_hash = transition_chain_proof.0;
    let merkle_list = transition_chain_proof.1;

    verify_impl(
        init_state_hash,
        target_hash,
        merkle_list.len(),
        merkle_list.iter(),
    )
}

pub fn verify_right(target_hash: Fp, transition_chain_proof: (Fp, Vec<Fp>)) -> Option<Vec<Fp>> {
    let init_state_hash = transition_chain_proof.0;
    let merkle_list = transition_chain_proof.1;

    verify_impl(
        init_state_hash,
        target_hash,
        merkle_list.len(),
        merkle_list.iter().rev(),
    )
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::utils::FpExt;

    use super::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn test_verify_empty_list() {
        let f = |s| Fp::from_str(s).unwrap();

        let target_hash =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let init_state =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let merkle_list = vec![];
        let transition_chain_proof = (init_state, merkle_list);

        let result = verify(target_hash, transition_chain_proof);

        let fields = result.unwrap();
        let fields_str = fields.iter().map(|f| f.to_decimal()).collect::<Vec<_>>();

        const OCAML_RESULT: &[&str] =
            &["13961539055339866639536775340930523333525643277685972296197275513682073214917"];

        assert_eq!(fields_str, OCAML_RESULT);
    }

    #[test]
    fn test_verify_fail() {
        let f = |s| Fp::from_str(s).unwrap();

        let target_hash =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let init_state =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let merkle_list = vec![
            f("13961539055339866639536775340930523333525643277685972296197275513682073214918"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214919"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214920"),
        ];
        let transition_chain_proof = (init_state, merkle_list);

        let result = verify(target_hash, transition_chain_proof);
        assert!(result.is_none());
    }

    #[test]
    fn test_verify_pass() {
        let f = |s| Fp::from_str(s).unwrap();

        let target_hash =
            f("800582919435166641451934253876519891456057321474706167207963223632227256531");
        let init_state =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let merkle_list = vec![
            f("13961539055339866639536775340930523333525643277685972296197275513682073214918"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214919"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214920"),
        ];
        let transition_chain_proof = (init_state, merkle_list);

        let result = verify(target_hash, transition_chain_proof);

        let fields = result.unwrap();
        let fields_str = fields.iter().map(|f| f.to_decimal()).collect::<Vec<_>>();

        const OCAML_RESULT: &[&str] = &[
            "800582919435166641451934253876519891456057321474706167207963223632227256531",
            "10598046929389065722730858200397844581724575694137702641700814979483247066836",
            "16081440258805529661738981264264797691385178952734852216885856078447011038998",
            "13961539055339866639536775340930523333525643277685972296197275513682073214917",
        ];

        assert_eq!(fields_str, OCAML_RESULT);
    }

    #[test]
    fn test_verify_right_empty_list() {
        let f = |s| Fp::from_str(s).unwrap();

        let target_hash =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let init_state =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let merkle_list = vec![];
        let transition_chain_proof = (init_state, merkle_list);

        let result = verify_right(target_hash, transition_chain_proof);

        let fields = result.unwrap();
        let fields_str = fields.iter().map(|f| f.to_decimal()).collect::<Vec<_>>();

        const OCAML_RESULT: &[&str] =
            &["13961539055339866639536775340930523333525643277685972296197275513682073214917"];

        assert_eq!(fields_str, OCAML_RESULT);
    }

    #[test]
    fn test_verify_right_fail() {
        let f = |s| Fp::from_str(s).unwrap();

        let target_hash =
            f("800582919435166641451934253876519891456057321474706167207963223632227256531");
        let init_state =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let merkle_list = vec![
            f("13961539055339866639536775340930523333525643277685972296197275513682073214918"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214919"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214920"),
        ];
        let transition_chain_proof = (init_state, merkle_list);

        let result = verify_right(target_hash, transition_chain_proof);
        assert!(result.is_none());
    }

    #[test]
    fn test_verify_right_pass() {
        let f = |s| Fp::from_str(s).unwrap();

        let target_hash =
            f("20637333162962631765110035680315905889290315150614944427035244260128010405493");
        let init_state =
            f("13961539055339866639536775340930523333525643277685972296197275513682073214917");
        let merkle_list = vec![
            f("13961539055339866639536775340930523333525643277685972296197275513682073214918"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214919"),
            f("13961539055339866639536775340930523333525643277685972296197275513682073214920"),
        ];
        let transition_chain_proof = (init_state, merkle_list);

        let result = verify_right(target_hash, transition_chain_proof);

        let fields = result.unwrap();
        let fields_str = fields.iter().map(|f| f.to_decimal()).collect::<Vec<_>>();

        const OCAML_RESULT: &[&str] = &[
            "20637333162962631765110035680315905889290315150614944427035244260128010405493",
            "8316842580468002558804570525036141813716506343242763169790486373222346781790",
            "15085839017587781134699581518615528888092289271567746845604070331351347632343",
            "13961539055339866639536775340930523333525643277685972296197275513682073214917",
        ];

        assert_eq!(fields_str, OCAML_RESULT);
    }
}
