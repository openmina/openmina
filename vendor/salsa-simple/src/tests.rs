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

fn new_with(key: [u8; 32], nonce: [u8; 24]) -> (XSalsa20, XSalsa20Ref) {
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

// =============================================================================
// Regression tests with known test vectors
// =============================================================================

/// NaCl test vector for XSalsa20.
/// Source: https://cr.yp.to/snuffle/xsalsa-20110204.pdf
/// and NaCl crypto_stream/xsalsa20 test vectors
#[test]
fn regtest_nacl_test_vector() {
    // Test vector from NaCl
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // Expected first 64 bytes of keystream (encrypting zeros gives keystream)
    let expected_keystream: [u8; 64] = [
        0xee, 0xa6, 0xa7, 0x25, 0x1c, 0x1e, 0x72, 0x91, 0x6d, 0x11, 0xc2, 0xcb, 0x21, 0x4d, 0x3c,
        0x25, 0x25, 0x39, 0x12, 0x1d, 0x8e, 0x23, 0x4e, 0x65, 0x2d, 0x65, 0x1f, 0xa4, 0xc8, 0xcf,
        0xf8, 0x80, 0x30, 0x9e, 0x64, 0x5a, 0x74, 0xe9, 0xe0, 0xa6, 0x0d, 0x82, 0x43, 0xac, 0xd9,
        0x17, 0x7a, 0xb5, 0x1a, 0x1b, 0xeb, 0x8d, 0x5a, 0x2f, 0x5d, 0x70, 0x0c, 0x09, 0x3c, 0x5e,
        0x55, 0x85, 0x57, 0x96,
    ];

    let mut cipher = XSalsa20::new(key, nonce);
    let mut output = [0u8; 64];
    cipher.apply_keystream(&mut output);

    assert_eq!(output, expected_keystream);
}

/// Test with all-zero key and nonce to verify deterministic output.
/// Expected values verified against reference salsa20 crate.
#[test]
fn regtest_zero_key_nonce() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    // Get expected keystream from reference implementation
    let mut theirs = XSalsa20Ref::new(
        GenericArray::from_slice(&key),
        GenericArray::from_slice(&nonce),
    );
    let mut expected = [0u8; 64];
    theirs.apply_keystream(&mut expected);

    // Test our implementation
    let mut cipher = XSalsa20::new(key, nonce);
    let mut output = [0u8; 64];
    cipher.apply_keystream(&mut output);

    assert_eq!(output, expected);

    // Also verify the output is deterministic on repeated creation
    let mut cipher2 = XSalsa20::new(key, nonce);
    let mut output2 = [0u8; 64];
    cipher2.apply_keystream(&mut output2);
    assert_eq!(output, output2);
}

/// Test multiple blocks to verify counter increments correctly
#[test]
fn regtest_multi_block() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // Get expected second block from reference implementation
    let mut theirs = XSalsa20Ref::new(
        GenericArray::from_slice(&key),
        GenericArray::from_slice(&nonce),
    );
    let mut first_block_ref = [0u8; 64];
    theirs.apply_keystream(&mut first_block_ref);
    let mut expected_second_block = [0u8; 64];
    theirs.apply_keystream(&mut expected_second_block);

    // Test our implementation
    let mut cipher = XSalsa20::new(key, nonce);

    // Skip first block
    let mut first_block = [0u8; 64];
    cipher.apply_keystream(&mut first_block);

    // Get second block
    let mut second_block = [0u8; 64];
    cipher.apply_keystream(&mut second_block);

    assert_eq!(second_block, expected_second_block);
}

/// Test partial block handling with known values
#[test]
fn regtest_partial_block() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // First 32 bytes of keystream (from NaCl test vector)
    let expected_first_half: [u8; 32] = [
        0xee, 0xa6, 0xa7, 0x25, 0x1c, 0x1e, 0x72, 0x91, 0x6d, 0x11, 0xc2, 0xcb, 0x21, 0x4d, 0x3c,
        0x25, 0x25, 0x39, 0x12, 0x1d, 0x8e, 0x23, 0x4e, 0x65, 0x2d, 0x65, 0x1f, 0xa4, 0xc8, 0xcf,
        0xf8, 0x80,
    ];

    // Remaining 32 bytes of first block (from NaCl test vector)
    let expected_second_half: [u8; 32] = [
        0x30, 0x9e, 0x64, 0x5a, 0x74, 0xe9, 0xe0, 0xa6, 0x0d, 0x82, 0x43, 0xac, 0xd9, 0x17, 0x7a,
        0xb5, 0x1a, 0x1b, 0xeb, 0x8d, 0x5a, 0x2f, 0x5d, 0x70, 0x0c, 0x09, 0x3c, 0x5e, 0x55, 0x85,
        0x57, 0x96,
    ];

    let mut cipher = XSalsa20::new(key, nonce);

    // Get first 32 bytes
    let mut first_half = [0u8; 32];
    cipher.apply_keystream(&mut first_half);
    assert_eq!(first_half, expected_first_half);

    // Verify position is 32
    assert_eq!(cipher.get_pos(), 32);

    // Get remaining 32 bytes of the block
    let mut second_half = [0u8; 32];
    cipher.apply_keystream(&mut second_half);
    assert_eq!(second_half, expected_second_half);

    // Position should be 0 (wrapped)
    assert_eq!(cipher.get_pos(), 0);
}

/// Test encryption/decryption roundtrip with known plaintext
#[test]
fn regtest_encrypt_decrypt_roundtrip() {
    let key: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];
    let nonce: [u8; 24] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    ];

    let plaintext = b"The quick brown fox jumps over the lazy dog. 0123456789!";
    let mut ciphertext = *plaintext;

    // Encrypt
    let mut cipher = XSalsa20::new(key, nonce);
    cipher.apply_keystream(&mut ciphertext);

    // Verify ciphertext is different from plaintext
    assert_ne!(&ciphertext[..], &plaintext[..]);

    // Decrypt (XSalsa20 is symmetric - apply keystream again)
    let mut cipher = XSalsa20::new(key, nonce);
    let mut decrypted = ciphertext;
    cipher.apply_keystream(&mut decrypted);

    // Verify decryption matches original plaintext
    assert_eq!(&decrypted[..], &plaintext[..]);
}

/// Verify ciphertext matches expected value for known plaintext
#[test]
fn regtest_known_plaintext_ciphertext() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // "Hello, World!" as plaintext
    let plaintext: [u8; 13] = [
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21,
    ];

    // Keystream first 13 bytes: 0xee, 0xa6, 0xa7, 0x25, 0x1c, 0x1e, 0x72, 0x91, 0x6d, 0x11, 0xc2,
    // 0xcb, 0x21 Plaintext XOR keystream:
    // 0x48 ^ 0xee = 0xa6
    // 0x65 ^ 0xa6 = 0xc3
    // 0x6c ^ 0xa7 = 0xcb
    // 0x6c ^ 0x25 = 0x49
    // 0x6f ^ 0x1c = 0x73
    // 0x2c ^ 0x1e = 0x32
    // 0x20 ^ 0x72 = 0x52
    // 0x57 ^ 0x91 = 0xc6
    // 0x6f ^ 0x6d = 0x02
    // 0x72 ^ 0x11 = 0x63
    // 0x6c ^ 0xc2 = 0xae
    // 0x64 ^ 0xcb = 0xaf
    // 0x21 ^ 0x21 = 0x00
    let expected_ciphertext: [u8; 13] = [
        0xa6, 0xc3, 0xcb, 0x49, 0x73, 0x32, 0x52, 0xc6, 0x02, 0x63, 0xae, 0xaf, 0x00,
    ];

    let mut data = plaintext;
    let mut cipher = XSalsa20::new(key, nonce);
    cipher.apply_keystream(&mut data);

    assert_eq!(data, expected_ciphertext);
}

// =============================================================================
// Regression tests with different buffer sizes
// =============================================================================

/// Test apply_keystream with various buffer sizes to exercise different code paths
#[test]
fn regtest_buffer_sizes() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // Test sizes: 1 byte, 63 bytes (under block), 64 bytes (exact block),
    // 65 bytes (over block), 128 bytes (two blocks), 127 bytes, 129 bytes
    let sizes = [1, 63, 64, 65, 127, 128, 129, 256, 1000];

    for &size in &sizes {
        let (mut ours, mut theirs) = new_with(key, nonce);

        let mut ours_output = vec![0u8; size];
        let mut theirs_output = vec![0u8; size];

        ours.apply_keystream(&mut ours_output);
        theirs.apply_keystream(&mut theirs_output);

        assert_eq!(
            ours_output, theirs_output,
            "Mismatch for buffer size {}",
            size
        );
    }
}

/// Test that sequential small reads produce the same result as one large read
#[test]
fn regtest_sequential_vs_bulk() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // Get 256 bytes in one call
    let mut bulk_cipher = XSalsa20::new(key, nonce);
    let mut bulk_output = [0u8; 256];
    bulk_cipher.apply_keystream(&mut bulk_output);

    // Get 256 bytes in sequential 1-byte calls
    let mut seq_cipher = XSalsa20::new(key, nonce);
    let mut seq_output = [0u8; 256];
    for byte in seq_output.iter_mut() {
        let mut single = [0u8; 1];
        seq_cipher.apply_keystream(&mut single);
        *byte = single[0];
    }

    assert_eq!(bulk_output, seq_output);
}

/// Test that sequential reads of various sizes produce correct output
#[test]
fn regtest_mixed_sizes() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // Read in mixed sizes: 17, 33, 64, 7, 128, 1 (total = 250 bytes)
    let sizes = [17, 33, 64, 7, 128, 1];
    let total: usize = sizes.iter().sum();

    let (mut ours, mut theirs) = new_with(key, nonce);

    let mut ours_combined = Vec::new();
    let mut theirs_combined = Vec::new();

    for &size in &sizes {
        let mut ours_buf = vec![0u8; size];
        let mut theirs_buf = vec![0u8; size];

        ours.apply_keystream(&mut ours_buf);
        theirs.apply_keystream(&mut theirs_buf);

        ours_combined.extend_from_slice(&ours_buf);
        theirs_combined.extend_from_slice(&theirs_buf);
    }

    assert_eq!(ours_combined.len(), total);
    assert_eq!(ours_combined, theirs_combined);
}

/// Test position tracking across block boundaries
#[test]
fn regtest_position_tracking() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let mut cipher = XSalsa20::new(key, nonce);

    // Initial position should be 0
    assert_eq!(cipher.get_pos(), 0);

    // Read 10 bytes
    let mut buf = [0u8; 10];
    cipher.apply_keystream(&mut buf);
    assert_eq!(cipher.get_pos(), 10);

    // Read 54 more bytes (total 64 = one block)
    let mut buf = [0u8; 54];
    cipher.apply_keystream(&mut buf);
    assert_eq!(cipher.get_pos(), 0); // Should wrap to 0

    // Read 65 bytes (one block + 1)
    let mut buf = [0u8; 65];
    cipher.apply_keystream(&mut buf);
    assert_eq!(cipher.get_pos(), 1);

    // Read 63 bytes to complete another block
    let mut buf = [0u8; 63];
    cipher.apply_keystream(&mut buf);
    assert_eq!(cipher.get_pos(), 0);
}

/// Test remaining bytes calculation
#[test]
fn regtest_remaining_bytes() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let mut cipher = XSalsa20::new(key, nonce);

    // Initially 64 remaining in buffer (but buffer not filled yet, so position is 0)
    assert_eq!(cipher.remaining(), 64);

    // Read 10 bytes
    let mut buf = [0u8; 10];
    cipher.apply_keystream(&mut buf);
    assert_eq!(cipher.remaining(), 54);

    // Read 54 bytes (complete the block)
    let mut buf = [0u8; 54];
    cipher.apply_keystream(&mut buf);
    assert_eq!(cipher.remaining(), 64);
}

/// Test set_pos_unchecked method
#[test]
fn regtest_set_pos_unchecked() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    let mut cipher = XSalsa20::new(key, nonce);

    // Initial position is 0
    assert_eq!(cipher.get_pos(), 0);

    // Set position to 32
    cipher.set_pos_unchecked(32);
    assert_eq!(cipher.get_pos(), 32);
    assert_eq!(cipher.remaining(), 32);

    // Set position to 0
    cipher.set_pos_unchecked(0);
    assert_eq!(cipher.get_pos(), 0);
    assert_eq!(cipher.remaining(), 64);

    // Set position to 63 (max valid)
    cipher.set_pos_unchecked(63);
    assert_eq!(cipher.get_pos(), 63);
    assert_eq!(cipher.remaining(), 1);
}

/// Test check_remaining method
#[test]
fn regtest_check_remaining() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let cipher = XSalsa20::new(key, nonce);

    // Should succeed for any reasonable size since we have ~u64::MAX blocks available
    assert!(cipher.check_remaining(0).is_ok());
    assert!(cipher.check_remaining(64).is_ok());
    assert!(cipher.check_remaining(1000).is_ok());
    assert!(cipher.check_remaining(1_000_000).is_ok());
}

/// Test check_remaining with partial buffer consumption
#[test]
fn regtest_check_remaining_partial() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let mut cipher = XSalsa20::new(key, nonce);

    // Consume 32 bytes, leaving 32 in buffer
    let mut buf = [0u8; 32];
    cipher.apply_keystream(&mut buf);

    // Should succeed for sizes within and beyond remaining buffer
    assert!(cipher.check_remaining(0).is_ok());
    assert!(cipher.check_remaining(32).is_ok()); // Exactly remaining
    assert!(cipher.check_remaining(64).is_ok()); // Needs one more block
    assert!(cipher.check_remaining(100).is_ok()); // Needs more blocks
}

/// Test new() creates distinct cipher instances
#[test]
fn regtest_new_distinct_instances() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let mut cipher1 = XSalsa20::new(key, nonce);
    let mut cipher2 = XSalsa20::new(key, nonce);

    // Both should produce identical output for same input
    let mut output1 = [0u8; 64];
    let mut output2 = [0u8; 64];

    cipher1.apply_keystream(&mut output1);
    cipher2.apply_keystream(&mut output2);

    assert_eq!(output1, output2);

    // But now cipher1 and cipher2 are at different positions if we advance one
    let mut more1 = [0u8; 64];
    cipher1.apply_keystream(&mut more1);

    // cipher2 hasn't advanced, so its next output should match cipher1's first output
    let mut cipher3 = XSalsa20::new(key, nonce);
    let mut output3 = [0u8; 64];
    cipher3.apply_keystream(&mut output3);
    assert_eq!(output2, output3);
}

/// Test new() with different keys produces different output
#[test]
fn regtest_new_different_keys() {
    let key1 = [0u8; 32];
    let mut key2 = [0u8; 32];
    key2[0] = 1; // Only one bit different

    let nonce = [0u8; 24];

    let mut cipher1 = XSalsa20::new(key1, nonce);
    let mut cipher2 = XSalsa20::new(key2, nonce);

    let mut output1 = [0u8; 64];
    let mut output2 = [0u8; 64];

    cipher1.apply_keystream(&mut output1);
    cipher2.apply_keystream(&mut output2);

    // Outputs should be completely different
    assert_ne!(output1, output2);
}

/// Test new() with different nonces produces different output
#[test]
fn regtest_new_different_nonces() {
    let key = [0u8; 32];
    let nonce1 = [0u8; 24];
    let mut nonce2 = [0u8; 24];
    nonce2[0] = 1; // Only one bit different

    let mut cipher1 = XSalsa20::new(key, nonce1);
    let mut cipher2 = XSalsa20::new(key, nonce2);

    let mut output1 = [0u8; 64];
    let mut output2 = [0u8; 64];

    cipher1.apply_keystream(&mut output1);
    cipher2.apply_keystream(&mut output2);

    // Outputs should be completely different
    assert_ne!(output1, output2);
}

/// Test apply_keystream with empty buffer
#[test]
fn regtest_apply_keystream_empty() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let mut cipher = XSalsa20::new(key, nonce);

    // Empty buffer should not change position
    let mut empty: [u8; 0] = [];
    cipher.apply_keystream(&mut empty);
    assert_eq!(cipher.get_pos(), 0);

    // Read some bytes
    let mut buf = [0u8; 10];
    cipher.apply_keystream(&mut buf);
    assert_eq!(cipher.get_pos(), 10);

    // Empty buffer again should not change position
    cipher.apply_keystream(&mut empty);
    assert_eq!(cipher.get_pos(), 10);
}

/// Test apply_keystream produces XOR with keystream (encryption property)
#[test]
fn regtest_apply_keystream_xor_property() {
    let key: [u8; 32] = [
        0x1b, 0x27, 0x55, 0x64, 0x73, 0xe9, 0x85, 0xd4, 0x62, 0xcd, 0x51, 0x19, 0x7a, 0x9a, 0x46,
        0xc7, 0x60, 0x09, 0x54, 0x9e, 0xac, 0x64, 0x74, 0xf2, 0x06, 0xc4, 0xee, 0x08, 0x44, 0xf6,
        0x83, 0x89,
    ];
    let nonce: [u8; 24] = [
        0x69, 0x69, 0x6e, 0xe9, 0x55, 0xb6, 0x2b, 0x73, 0xcd, 0x62, 0xbd, 0xa8, 0x75, 0xfc, 0x73,
        0xd6, 0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37,
    ];

    // Get keystream by encrypting zeros
    let mut cipher1 = XSalsa20::new(key, nonce);
    let mut keystream = [0u8; 64];
    cipher1.apply_keystream(&mut keystream);

    // Encrypt some data
    let plaintext = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
    let mut ciphertext = plaintext;
    let mut cipher2 = XSalsa20::new(key, nonce);
    cipher2.apply_keystream(&mut ciphertext);

    // Verify: ciphertext = plaintext XOR keystream
    for i in 0..8 {
        assert_eq!(ciphertext[i], plaintext[i] ^ keystream[i]);
    }
}

/// Test get_pos boundary values
#[test]
fn regtest_get_pos_boundaries() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let mut cipher = XSalsa20::new(key, nonce);

    // Test position at block boundaries
    for expected_pos in [0, 1, 31, 32, 33, 62, 63] {
        cipher.set_pos_unchecked(expected_pos);
        assert_eq!(cipher.get_pos(), expected_pos);
    }
}

/// Test remaining at various positions
#[test]
fn regtest_remaining_at_positions() {
    let key = [0u8; 32];
    let nonce = [0u8; 24];

    let mut cipher = XSalsa20::new(key, nonce);

    // Test remaining at various positions
    let test_cases = [
        (0, 64),
        (1, 63),
        (31, 33),
        (32, 32),
        (33, 31),
        (62, 2),
        (63, 1),
    ];

    for (pos, expected_remaining) in test_cases {
        cipher.set_pos_unchecked(pos);
        assert_eq!(
            cipher.remaining(),
            expected_remaining,
            "At position {}, expected remaining {}",
            pos,
            expected_remaining
        );
    }
}
