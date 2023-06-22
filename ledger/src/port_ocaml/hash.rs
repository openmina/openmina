//! Implementation of janestreet hash:
//! https://github.com/janestreet/base/blob/v0.14/hash_types/src/internalhash_stubs.c
//!
//! IMPORTANT:
//!   This needs to be tested/fuzzed and compared with OCaml.

use std::hash::Hasher;

#[derive(Debug)]
pub(super) struct JaneStreetHasher {
    h: u32,
}

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
    d *= 0xcc9e2d51;
    d = rotl32(d, 15);
    d *= 0x1b873593;
    h ^= d;
    h = rotl32(h, 13);
    h = h * 5 + 0xe6546b64;
    h
}

/// https://github.com/janestreet/base/blob/v0.14/hash_types/src/internalhash_stubs.c#L35
fn final_mix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    h *= 0xc2b2ae35;
    h ^= h >> 16;
    h
}

impl Hasher for JaneStreetHasher {
    fn finish(&self) -> u64 {
        let h = final_mix(self.h);
        let h: u32 = h & 0x3FFF_FFFF; // 30 bits
        h as u64
    }

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
}
