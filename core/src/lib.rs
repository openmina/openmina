pub mod invariants;
pub mod log;
pub mod requests;

pub mod channels;

pub mod constants;
pub mod dummy;

pub mod block;
pub mod snark;

pub mod consensus;

/// Default chain id, as used by [berkeleynet](https://berkeley.minaexplorer.com/status).
pub const CHAIN_ID: &str =
    "fd7d111973bf5a9e3e87384f560fdead2f272589ca00b6d9e357fca9839631da";

pub fn preshared_key(chain_id: &str) -> [u8; 32] {
    use multihash::{Blake2b256, Hasher};
    let mut hasher = Blake2b256::default();
    hasher.update(b"/coda/0.0.1/");
    hasher.update(chain_id.as_bytes());
    let hash = hasher.finalize();
    let mut psk_fixed: [u8; 32] = Default::default();
    psk_fixed.copy_from_slice(hash.as_ref());
    psk_fixed
}
