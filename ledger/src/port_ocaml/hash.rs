//! Implementation of janestreet hash:
//! https://github.com/janestreet/base/blob/v0.14/hash_types/src/internalhash_stubs.c

use std::hash::Hasher;

use ark_ff::{BigInteger, BigInteger256};
use mina_hasher::Fp;

use crate::AccountId;

#[derive(Debug)]
pub(super) struct JaneStreetHasher {
    h: u32,
}

#[allow(clippy::derivable_impls)]
impl Default for JaneStreetHasher {
    fn default() -> Self {
        // the seed seems to be zero
        // https://github.com/janestreet/base/blob/v0.14/src/hash.ml#L165
        Self { h: 0 }
    }
}

fn rotl32(x: u32, n: u32) -> u32 {
    (x) << n | (x) >> (32 - n)
}

/// https://github.com/janestreet/base/blob/v0.14/hash_types/src/internalhash_stubs.c#L52
fn mix(mut h: u32, mut d: u32) -> u32 {
    d = d.wrapping_mul(0xcc9e2d51);
    d = rotl32(d, 15);
    d = d.wrapping_mul(0x1b873593);
    h ^= d;
    h = rotl32(h, 13);
    h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    h
}

/// https://github.com/janestreet/base/blob/v0.14/hash_types/src/internalhash_stubs.c#L35
fn final_mix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

impl Hasher for JaneStreetHasher {
    /// https://github.com/janestreet/base/blob/v0.14/hash_types/src/internalhash_stubs.c#L42C1-L47C2
    fn finish(&self) -> u64 {
        let h = final_mix(self.h);
        let h: u32 = h & 0x3FFF_FFFF; // 30 bits
        h as u64
    }

    /// https://github.com/janestreet/base/blob/v0.14/hash_types/src/internalhash_stubs.c#L60C1-L90C2
    fn write(&mut self, s: &[u8]) {
        // Little endian implementation only
        for (chunk, chunk_len) in s.chunks(4).map(|chunk| (chunk, chunk.len())) {
            let w: u32 = if chunk_len == 4 {
                u32::from_le_bytes(chunk.try_into().unwrap())
            } else {
                // Finish with up to 3 bytes
                let mut w = [0u8; 4];
                w[..chunk_len].copy_from_slice(chunk);
                u32::from_le_bytes(w)
            };
            self.h = mix(self.h, w);
        }

        // Finally, mix in the length. Ignore the upper 32 bits, generally 0.
        self.h ^= (s.len() & 0xFFFF_FFFF) as u32;
    }

    fn write_u32(&mut self, i: u32) {
        self.h = mix(self.h, i);
    }

    /// This is used for `bool` in OCaml
    fn write_i64(&mut self, d: i64) {
        let n = (d >> 32) ^ (d >> 63) ^ d;
        self.h = mix(self.h, n as u32);
    }
}

/// https://github.com/ocaml/Zarith/blob/6f840fb026ab6920104ea7b43140fdcc3e936914/caml_z.c#L3333-L3358
fn hash_field(f: &Fp) -> u32 {
    let mut acc = 0;

    let bigint: BigInteger256 = (*f).into();
    let nignore: usize = bigint.0.iter().rev().take_while(|&b| *b == 0).count();

    for bigint in bigint.0.iter().take(BigInteger256::NUM_LIMBS - nignore) {
        acc = mix(acc, (bigint & 0xFFFF_FFFF) as u32);
        acc = mix(acc, (bigint >> 32) as u32);
    }

    if bigint.0.last().unwrap() & 0x8000_0000_0000_0000 != 0 {
        // TODO: Not sure if that condition is correct
        acc += 1;
    }

    acc
}

pub fn account_id_ocaml_hash(account_id: &AccountId) -> u32 {
    let mut hasher = JaneStreetHasher::default();
    hasher.write_u32(hash_field(&account_id.public_key.x));
    hasher.write_u32(account_id.public_key.is_odd as u32);
    hasher.write_u32(hash_field(&account_id.token_id.0));
    hasher.finish() as u32
}

pub trait OCamlHash {
    fn ocaml_hash(&self) -> u32;
}

impl OCamlHash for AccountId {
    fn ocaml_hash(&self) -> u32 {
        account_id_ocaml_hash(self)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mina_signer::CompressedPubKey;

    use crate::AccountId;

    use super::*;

    #[test]
    fn test_hash_fp() {
        const TOKEN_ID: &str =
            "4967745125912355413023996042397960619663294160624364664207862415774117920201";
        const PK: &str =
            "9706454913247140231138457869942480497494390376582974522927223806983428482248";

        let fp = Fp::from_str(TOKEN_ID).unwrap();
        let acc = hash_field(&fp);
        dbg!(acc as i32);
        assert_eq!(acc, (-1384641025i32) as u32);

        let fp = Fp::from_str(PK).unwrap();
        let acc = hash_field(&fp);
        dbg!(acc as i32);
        assert_eq!(acc, (-2044176935i32) as u32);

        let fp = Fp::from_str(TOKEN_ID).unwrap();
        let mut hasher = JaneStreetHasher { h: 1178103103 };
        hasher.write_u32(hash_field(&fp));
        assert_eq!(hasher.h, 327258694);

        let fp = Fp::from_str(TOKEN_ID).unwrap();
        let hash = {
            let mut hasher = JaneStreetHasher::default();
            hasher.write_u32(hash_field(&fp));
            hasher.finish()
        };
        assert_eq!(hash, 712332047);

        let pk = CompressedPubKey {
            x: Fp::from_str(PK).unwrap(),
            is_odd: true,
        };
        let hash = {
            let mut hasher = JaneStreetHasher::default();
            hasher.write_u32(hash_field(&pk.x));
            hasher.write_u32(pk.is_odd as u32);
            hasher.finish()
        };
        assert_eq!(hash, 284693557);

        let account_id = AccountId {
            public_key: pk,
            token_id: crate::TokenId(Fp::from_str(TOKEN_ID).unwrap()),
        };
        let hash = {
            let mut hasher = JaneStreetHasher::default();
            hasher.write_u32(hash_field(&account_id.public_key.x));
            hasher.write_u32(account_id.public_key.is_odd as u32);
            hasher.write_u32(hash_field(&account_id.token_id.0));
            hasher.finish()
        };
        assert_eq!(hash, 386994658);
    }
}
