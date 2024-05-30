#[cfg(test)]
mod tests;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use generic_array::{typenum, GenericArray};
use inout::InOutBuf;

use zeroize::{Zeroize, ZeroizeOnDrop};

pub type XSalsa20 = XSalsa<10>;

#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct XSalsa<const R: usize> {
    core: XSalsaCore<R>,
    #[serde(serialize_with = "helpers::ser_bytes")]
    #[serde(deserialize_with = "helpers::de_bytes")]
    buffer: [u8; 64],
    pos: u8,
}

impl<const R: usize> XSalsa<R> {
    pub fn new(key: [u8; 32], iv: [u8; 24]) -> Self {
        XSalsa {
            core: XSalsaCore::new(key, iv),
            buffer: [0; 64],
            pos: 0,
        }
    }

    /// Return current cursor position.
    #[inline]
    pub fn get_pos(&self) -> usize {
        let pos = self.pos as usize;
        if pos >= 64 {
            debug_assert!(false);
            // SAFETY: `pos` is set only to values smaller than block size
            unsafe { core::hint::unreachable_unchecked() }
        }
        self.pos as usize
    }

    #[inline]
    pub fn set_pos_unchecked(&mut self, pos: usize) {
        debug_assert!(pos < 64);
        self.pos = pos as u8;
    }

    /// Return number of remaining bytes in the internal buffer.
    #[inline]
    pub fn remaining(&self) -> usize {
        64 - self.get_pos()
    }
    #[allow(clippy::result_unit_err)]
    pub fn check_remaining(&self, dlen: usize) -> Result<(), ()> {
        let rem_blocks = match self.core.remaining_blocks() {
            Some(v) => v,
            None => return Ok(()),
        };

        let bytes = if self.pos == 0 {
            dlen
        } else {
            let rem = self.remaining();
            if dlen > rem {
                dlen - rem
            } else {
                return Ok(());
            }
        };
        let bs = 64;
        let blocks = if bytes % bs == 0 {
            bytes / bs
        } else {
            bytes / bs + 1
        };
        if blocks > rem_blocks {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn apply_keystream(&mut self, buf: &mut [u8]) {
        let mut data = InOutBuf::from(buf);

        self.check_remaining(data.len()).unwrap();

        let pos = self.get_pos();
        if pos != 0 {
            let rem = &self.buffer[pos..];
            let n = data.len();
            if n < rem.len() {
                data.xor_in2out(&rem[..n]);
                self.set_pos_unchecked(pos + n);
                return;
            }
            let (mut left, right) = data.split_at(rem.len());
            data = right;
            left.xor_in2out(rem);
        }

        let (blocks, mut leftover) = data.into_chunks();
        self.core.apply_keystream_blocks_inout(blocks);

        let n = leftover.len();
        if n != 0 {
            self.core.write_keystream_block(&mut self.buffer);
            leftover.xor_in2out(&self.buffer[..n]);
        }
        self.set_pos_unchecked(n);
    }
}

#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct XSalsaCore<const R: usize>(SalsaCore<R>);

#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct SalsaCore<const R: usize> {
    state: [u32; 16],
}

const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

impl<const R: usize> XSalsaCore<R> {
    fn new(key: [u8; 32], iv: [u8; 24]) -> Self {
        let subkey = hsalsa::<R>(key, iv[..16].as_ref().try_into().unwrap());
        let mut padded_iv = [0; 8];
        padded_iv.copy_from_slice(&iv[16..]);
        Self(SalsaCore::new(subkey, padded_iv))
    }

    #[inline(always)]
    fn remaining_blocks(&self) -> Option<usize> {
        self.0.remaining_blocks()
    }

    #[inline(always)]
    fn gen_ks_block(&mut self, block: &mut [u8; 64]) {
        let res = run_rounds::<R>(&self.0.state);
        self.0.set_block_pos(self.0.get_block_pos() + 1);

        for (chunk, val) in block.chunks_exact_mut(4).zip(res.iter()) {
            chunk.copy_from_slice(&val.to_le_bytes());
        }
    }

    /// Write keystream block.
    ///
    /// WARNING: this method does not check number of remaining blocks!
    #[inline]
    fn write_keystream_block(&mut self, block: &mut [u8; 64]) {
        self.gen_ks_block(block);
    }

    /// Apply keystream blocks.
    ///
    /// WARNING: this method does not check number of remaining blocks!
    #[inline]
    fn apply_keystream_blocks_inout(
        &mut self,
        blocks: InOutBuf<'_, '_, GenericArray<u8, typenum::U64>>,
    ) {
        for mut block in blocks {
            let mut t = [0; 64];
            self.gen_ks_block(&mut t);
            block.xor_in2out(GenericArray::from_slice(&t));
        }
    }
}

/// The HSalsa20 function defined in the paper "Extending the Salsa20 nonce"
///
/// <https://cr.yp.to/snuffle/xsalsa-20110204.pdf>
///
/// HSalsa20 takes 512-bits of input:
///
/// - Constants (`u32` x 4)
/// - Key (`u32` x 8)
/// - Nonce (`u32` x 4)
///
/// It produces 256-bits of output suitable for use as a Salsa20 key
fn hsalsa<const R: usize>(key: [u8; 32], input: [u8; 16]) -> [u8; 32] {
    #[inline(always)]
    fn to_u32(chunk: &[u8]) -> u32 {
        u32::from_le_bytes(chunk.try_into().unwrap())
    }

    let mut state = [0u32; 16];
    state[0] = CONSTANTS[0];
    state[1..5]
        .iter_mut()
        .zip(key[0..16].chunks_exact(4))
        .for_each(|(v, chunk)| *v = to_u32(chunk));
    state[5] = CONSTANTS[1];
    state[6..10]
        .iter_mut()
        .zip(input.chunks_exact(4))
        .for_each(|(v, chunk)| *v = to_u32(chunk));
    state[10] = CONSTANTS[2];
    state[11..15]
        .iter_mut()
        .zip(key[16..].chunks_exact(4))
        .for_each(|(v, chunk)| *v = to_u32(chunk));
    state[15] = CONSTANTS[3];

    // 20 rounds consisting of 10 column rounds and 10 diagonal rounds
    for _ in 0..R {
        // column rounds
        quarter_round(0, 4, 8, 12, &mut state);
        quarter_round(5, 9, 13, 1, &mut state);
        quarter_round(10, 14, 2, 6, &mut state);
        quarter_round(15, 3, 7, 11, &mut state);

        // diagonal rounds
        quarter_round(0, 1, 2, 3, &mut state);
        quarter_round(5, 6, 7, 4, &mut state);
        quarter_round(10, 11, 8, 9, &mut state);
        quarter_round(15, 12, 13, 14, &mut state);
    }

    let mut output = [0; 32];
    let key_idx: [usize; 8] = [0, 5, 10, 15, 6, 7, 8, 9];

    for (i, chunk) in output.chunks_exact_mut(4).enumerate() {
        chunk.copy_from_slice(&state[key_idx[i]].to_le_bytes());
    }

    output
}

#[inline(always)]
fn run_rounds<const R: usize>(state: &[u32; 16]) -> [u32; 16] {
    let mut res = *state;

    for _ in 0..R {
        // column rounds
        quarter_round(0, 4, 8, 12, &mut res);
        quarter_round(5, 9, 13, 1, &mut res);
        quarter_round(10, 14, 2, 6, &mut res);
        quarter_round(15, 3, 7, 11, &mut res);

        // diagonal rounds
        quarter_round(0, 1, 2, 3, &mut res);
        quarter_round(5, 6, 7, 4, &mut res);
        quarter_round(10, 11, 8, 9, &mut res);
        quarter_round(15, 12, 13, 14, &mut res);
    }

    for (s1, s0) in res.iter_mut().zip(state.iter()) {
        *s1 = s1.wrapping_add(*s0);
    }
    res
}

#[inline]
#[allow(clippy::many_single_char_names)]
fn quarter_round(a: usize, b: usize, c: usize, d: usize, state: &mut [u32; 16]) {
    state[b] ^= state[a].wrapping_add(state[d]).rotate_left(7);
    state[c] ^= state[b].wrapping_add(state[a]).rotate_left(9);
    state[d] ^= state[c].wrapping_add(state[b]).rotate_left(13);
    state[a] ^= state[d].wrapping_add(state[c]).rotate_left(18);
}

impl<const R: usize> SalsaCore<R> {
    fn new(key: [u8; 32], iv: [u8; 8]) -> Self {
        let mut state = [0u32; 16];
        state[0] = CONSTANTS[0];

        for (i, chunk) in key[..16].chunks(4).enumerate() {
            state[1 + i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }

        state[5] = CONSTANTS[1];

        for (i, chunk) in iv.chunks(4).enumerate() {
            state[6 + i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }

        state[8] = 0;
        state[9] = 0;
        state[10] = CONSTANTS[2];

        for (i, chunk) in key[16..].chunks(4).enumerate() {
            state[11 + i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }

        state[15] = CONSTANTS[3];

        SalsaCore { state }
    }

    #[inline(always)]
    fn remaining_blocks(&self) -> Option<usize> {
        let rem = u64::MAX - self.get_block_pos();
        rem.try_into().ok()
    }

    #[inline(always)]
    fn get_block_pos(&self) -> u64 {
        (self.state[8] as u64) + ((self.state[9] as u64) << 32)
    }

    #[inline(always)]
    fn set_block_pos(&mut self, pos: u64) {
        self.state[8] = (pos & 0xffff_ffff) as u32;
        self.state[9] = ((pos >> 32) & 0xffff_ffff) as u32;
    }
}

#[cfg(feature = "serde")]
mod helpers {
    use std::fmt;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn ser_bytes<const N: usize, S>(v: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[cfg(feature = "hex")]
        if serializer.is_human_readable() {
            return serializer.serialize_str(&hex::encode(v));
        }

        serializer.serialize_bytes(v)
    }

    pub fn de_bytes<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        #[cfg(feature = "hex")]
        if deserializer.is_human_readable() {
            let str = String::deserialize(deserializer)?;
            let bytes = hex::decode(str).map_err(de::Error::custom)?;
            return bytes.as_slice().try_into().map_err(de::Error::custom);
        }

        struct V<const N: usize>;

        impl<'de, const N: usize> de::Visitor<'de> for V<N> {
            type Value = [u8; N];

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{N} bytes")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                v.try_into().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_bytes(V::<N>)
    }
}
