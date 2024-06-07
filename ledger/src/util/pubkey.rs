use mina_signer::{pubkey::*, BaseField};
use o1_utils::field_helpers::FieldHelpers;
use sha2::{Digest, Sha256};
const MINA_ADDRESS_RAW_LEN: usize = 40;

// Needed because of two invalid keys in mina's mainnet genesis ledger that we
// must parse even if invalid to be able to reconstruct the genesis ledger.
pub fn compressed_pubkey_from_address_maybe_with_error(address: &str) -> Result<CompressedPubKey> {
    if address.len() != MINA_ADDRESS_LEN {
        return Err(PubKeyError::AddressLength);
    }

    let bytes = bs58::decode(address)
        .into_vec()
        .map_err(|_| PubKeyError::AddressBase58)?;

    if bytes.len() != MINA_ADDRESS_RAW_LEN {
        return Err(PubKeyError::AddressRawByteLength);
    }

    let (raw, checksum) = (&bytes[..bytes.len() - 4], &bytes[bytes.len() - 4..]);
    let hash = Sha256::digest(&Sha256::digest(raw)[..]);
    if checksum != &hash[..4] {
        return Err(PubKeyError::AddressChecksum);
    }

    let (version, x_bytes, y_parity) = (
        &raw[..3],
        &raw[3..bytes.len() - 5],
        raw[bytes.len() - 5] == 0x01,
    );
    if version != [0xcb, 0x01, 0x01] {
        return Err(PubKeyError::AddressVersion);
    }

    let x = BaseField::from_bytes(x_bytes).map_err(|_| PubKeyError::XCoordinateBytes)?;

    Ok(CompressedPubKey {
        x,
        is_odd: y_parity,
    })
}

#[test]
fn compressed_from_invalid_address() {
    let address = "B62qpyhbvLobnd4Mb52vP7LPFAasb2S6Qphq8h5VV8Sq1m7VNK1VZcW";
    let pk =
        compressed_pubkey_from_address_maybe_with_error(address).expect("failed to create pubkey");
    assert_eq!(pk.into_address(), address);

    let address = "B62qqdcf6K9HyBSaxqH5JVFJkc1SUEe1VzDc5kYZFQZXWSQyGHoino1";
    let pk =
        compressed_pubkey_from_address_maybe_with_error(address).expect("failed to create pubkey");
    assert_eq!(pk.into_address(), address);
}
