use std::{env, fs};

use mina_node_account::AccountSecretKey;

#[test]
fn test_account_secret_key_bs58check_decode() {
    let parsed: AccountSecretKey = "EKFWgzXsoMYcP1Hnj7dBhsefxNucZ6wyz676Qg5uMFNzytXAi2Ww"
        .parse()
        .unwrap();
    // Test by comparing the public key
    assert_eq!(
        parsed.public_key().to_string(),
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS"
    );
}

#[test]
fn test_account_secret_key_display() {
    let parsed: AccountSecretKey = "EKFWgzXsoMYcP1Hnj7dBhsefxNucZ6wyz676Qg5uMFNzytXAi2Ww"
        .parse()
        .unwrap();
    assert_eq!(
        &parsed.to_string(),
        "EKFWgzXsoMYcP1Hnj7dBhsefxNucZ6wyz676Qg5uMFNzytXAi2Ww"
    );
}

#[test]
fn test_encrypt_decrypt() {
    let password = "not-very-secure-pass";

    let new_key = AccountSecretKey::rand();
    let tmp_dir = env::temp_dir();
    let tmp_path = format!("{}/{}-key", tmp_dir.display(), new_key.public_key());

    // dump encrypted file
    new_key
        .to_encrypted_file(&tmp_path, password)
        .expect("Failed to encrypt secret key");

    // load and decrypt
    let decrypted = AccountSecretKey::from_encrypted_file(&tmp_path, password)
        .unwrap_or_else(|_| panic!("Failed to decrypt secret key file: {}", tmp_path));

    assert_eq!(
        new_key.public_key(),
        decrypted.public_key(),
        "Encrypted and decrypted public keys do not match"
    );
}

#[test]
fn test_block_producer_key_decrypt() {
    // Get the workspace root directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = format!("{}/../../..", manifest_dir);
    let key_path = format!(
        "{}/tests/files/accounts/test-block-producer",
        workspace_root
    );
    let pubkey_path = format!(
        "{}/tests/files/accounts/test-block-producer.pub",
        workspace_root
    );
    let password = "test-password";

    // Load and decrypt the key
    let secret_key =
        AccountSecretKey::from_encrypted_file(&key_path, password).unwrap_or_else(|e| {
            panic!(
                "Failed to decrypt secret key file: {} - Error: {}",
                key_path, e
            )
        });

    // Get the public key from the decrypted secret key
    let public_key_from_secret = secret_key.public_key();

    // Load the expected public key from file
    let expected_public_key = fs::read_to_string(&pubkey_path)
        .unwrap_or_else(|_| panic!("Failed to read public key file: {}", pubkey_path))
        .trim()
        .to_string();

    // Verify they match
    assert_eq!(
        public_key_from_secret.to_string(),
        expected_public_key,
        "Public key from decrypted secret key does not match expected public key"
    );
}
