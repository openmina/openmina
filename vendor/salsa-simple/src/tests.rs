use crate::XSalsa20;

use generic_array::GenericArray;
use rand::RngCore;
use salsa20::{
    cipher::{KeyIvInit, StreamCipher},
    XSalsa20 as XSalsa20Ref,
};

fn new() -> (XSalsa20, XSalsa20Ref) {
    let key = rand::random();
    let nonce = rand::random();

    (
        XSalsa20::new(key, nonce),
        XSalsa20Ref::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        ),
    )
}

fn try_bytes<const SIZE: usize>(ours: &mut XSalsa20, theirs: &mut XSalsa20Ref) {
    let mut bytes = [0; SIZE];
    rand::thread_rng().fill_bytes(&mut bytes);
    let original = bytes;

    ours.apply_keystream(&mut bytes);
    theirs.apply_keystream(&mut bytes);

    assert_eq!(bytes, original);
}

#[test]
fn single_block() {
    let (mut ours, mut theirs) = new();
    try_bytes::<64>(&mut ours, &mut theirs);
}

#[test]
fn five_blocks() {
    let (mut ours, mut theirs) = new();
    try_bytes::<{ 64 * 5 }>(&mut ours, &mut theirs);
}

#[test]
fn under_block() {
    let (mut ours, mut theirs) = new();
    try_bytes::<55>(&mut ours, &mut theirs);
}

#[test]
fn over_block() {
    let (mut ours, mut theirs) = new();
    try_bytes::<77>(&mut ours, &mut theirs);
}

#[test]
fn under_block_and_continue() {
    let (mut ours, mut theirs) = new();
    try_bytes::<55>(&mut ours, &mut theirs);
    try_bytes::<55>(&mut ours, &mut theirs);
    try_bytes::<18>(&mut ours, &mut theirs);
}

#[test]
fn over_block_and_continue() {
    let (mut ours, mut theirs) = new();
    try_bytes::<100>(&mut ours, &mut theirs);
    try_bytes::<100>(&mut ours, &mut theirs);
}

#[cfg(feature = "serde")]
#[test]
fn under_block_and_serde() {
    let (mut ours, mut theirs) = new();
    try_bytes::<55>(&mut ours, &mut theirs);
    let ours_str = dbg!(serde_json::to_string(&ours).unwrap());
    let ours_value = serde_json::to_value(&ours).unwrap();
    let pos = ours_value
        .get("pos")
        .unwrap()
        .as_number()
        .unwrap()
        .as_i64()
        .unwrap();
    assert_eq!(pos, 55);
    let mut ours_restored = serde_json::from_str::<XSalsa20>(&ours_str).unwrap();
    try_bytes::<55>(&mut ours_restored, &mut theirs);
    try_bytes::<18>(&mut ours_restored, &mut theirs);
}

#[cfg(feature = "serde")]
#[test]
fn over_block_and_serde() {
    let (mut ours, mut theirs) = new();
    try_bytes::<100>(&mut ours, &mut theirs);
    let ours_str = dbg!(serde_json::to_string(&ours).unwrap());
    let ours_value = serde_json::to_value(&ours).unwrap();
    let pos = ours_value
        .get("pos")
        .unwrap()
        .as_number()
        .unwrap()
        .as_i64()
        .unwrap();
    assert_eq!(pos, 100 % 64);
    let mut ours_restored = serde_json::from_str::<XSalsa20>(&ours_str).unwrap();
    try_bytes::<100>(&mut ours_restored, &mut theirs);
}
