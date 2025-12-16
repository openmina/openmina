use std::str::FromStr;

use o1_utils::FieldHelpers;

#[cfg(target_family = "wasm")]
use wasm_bindgen_test::wasm_bindgen_test as test;

use mina_curves::pasta::Fp;
use mina_signer::CompressedPubKey;
use mina_tree::{
    scan_state::{
        currency::{Amount, Fee, Nonce, Slot},
        transaction_logic::{
            cons_signed_command_payload,
            signed_command::{Body, Common, PaymentPayload, SignedCommandPayload},
            zkapp_command, Memo,
        },
    },
    ReceiptChainHash,
};

fn pub_key(address: &str) -> CompressedPubKey {
    mina_signer::PubKey::from_address(address)
        .unwrap()
        .into_compressed()
}

#[test]
fn test_hash_empty_event() {
    // Same value than OCaml
    const EXPECTED: &str =
        "6963060754718463299978089777716994949151371320681588566338620419071140958308";

    let event = zkapp_command::Event::empty();
    assert_eq!(event.hash(), Fp::from_str(EXPECTED).unwrap());
}

/// Test using same values as here:
/// <https://github.com/MinaProtocol/mina/blob/3a78f0e0c1343d14e2729c8b00205baa2ec70c93/src/lib/mina_base/receipt.ml#L136>
#[test]
fn test_cons_receipt_hash_ocaml() {
    let from = pub_key("B62qr71UxuyKpkSKYceCPsjw14nuaeLwWKZdMqaBMPber5AAF6nkowS");
    let to = pub_key("B62qnvGVnU7FXdy8GdkxL7yciZ8KattyCdq5J6mzo5NCxjgQPjL7BTH");

    let common = Common {
        fee: Fee::from_u64(9758327274353182341),
        fee_payer_pk: from,
        nonce: Nonce::from_u32(1609569868),
        valid_until: Slot::from_u32(2127252111),
        memo: Memo([
            1, 32, 101, 26, 225, 104, 115, 118, 55, 102, 76, 118, 108, 78, 114, 50, 0, 115, 110,
            108, 53, 75, 109, 112, 50, 110, 88, 97, 76, 66, 76, 81, 235, 79,
        ]),
    };

    let body = Body::Payment(PaymentPayload {
        receiver_pk: to,
        amount: Amount::from_u64(1155659205107036493),
    });

    let tx = SignedCommandPayload { common, body };

    let prev = "4918218371695029984164006552208340844155171097348169027410983585063546229555";
    let prev_receipt_chain_hash = ReceiptChainHash(Fp::from_str(prev).unwrap());

    let next = "19078048535981853335308913493724081578728104896524544653528728307378106007337";
    let next_receipt_chain_hash = ReceiptChainHash(Fp::from_str(next).unwrap());

    let result = cons_signed_command_payload(&tx, prev_receipt_chain_hash);
    assert_eq!(result, next_receipt_chain_hash);
}

#[test]
fn test_receipt_hash_update() {
    let from = pub_key("B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja");
    let to = pub_key("B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS");

    let common = Common {
        fee: Fee::from_u64(14500000),
        fee_payer_pk: from,
        nonce: Nonce::from_u32(15),
        valid_until: Slot::from_u32(-1i32 as u32),
        memo: Memo([
            1, 7, 84, 104, 101, 32, 49, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ]),
    };

    let body = Body::Payment(PaymentPayload {
        receiver_pk: to,
        amount: Amount::from_u64(2354000000),
    });

    let tx = SignedCommandPayload { common, body };

    let mut prev =
        hex::decode("09ac04c9965b885acfc9c54141dbecfc63b2394a4532ea2c598d086b894bfb14").unwrap();
    prev.reverse();
    let prev_receipt_chain_hash = ReceiptChainHash(Fp::from_bytes(&prev).unwrap());

    let mut next =
        hex::decode("3ecaa73739df77549a2f92f7decf822562d0593373cff1e480bb24b4c87dc8f0").unwrap();
    next.reverse();
    let next_receipt_chain_hash = ReceiptChainHash(Fp::from_bytes(&next).unwrap());

    let result = cons_signed_command_payload(&tx, prev_receipt_chain_hash);
    assert_eq!(result, next_receipt_chain_hash);
}
