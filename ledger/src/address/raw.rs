use mina_p2p_messages::v2::MerkleAddressBinableArgStableV1;

use crate::base::AccountIndex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Clone, Eq, PartialOrd, Ord)]
pub struct Address<const NBYTES: usize> {
    pub(super) inner: [u8; NBYTES],
    pub(super) length: usize,
}

impl<'a, const NBYTES: usize> TryFrom<&'a str> for Address<NBYTES> {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if s.len() >= (NBYTES * 8) {
            return Err(());
        }

        let mut addr = Address {
            inner: [0; NBYTES],
            length: s.len(),
        };
        for (index, c) in s.chars().enumerate() {
            if c == '1' {
                addr.set(index);
            } else if c != '0' {
                return Err(());
            }
        }
        Ok(addr)
    }
}

impl<const NBYTES: usize> PartialEq for Address<NBYTES> {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }

        let nused_bytes = self.nused_bytes();

        if self.inner[0..nused_bytes - 1] != other.inner[0..nused_bytes - 1] {
            return false;
        }

        const MASK: [u8; 8] = [
            0b11111111, 0b10000000, 0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11111100,
            0b11111110,
        ];

        let bit_index = self.length % 8;
        let mask = MASK[bit_index];

        self.inner[nused_bytes - 1] & mask == other.inner[nused_bytes - 1] & mask
    }
}

impl<const NBYTES: usize> std::fmt::Debug for Address<NBYTES> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::with_capacity(NBYTES * 8);

        for index in 0..self.length {
            if index != 0 && index % 8 == 0 {
                s.push('_');
            }
            match self.get(index) {
                Direction::Left => s.push('0'),
                Direction::Right => s.push('1'),
            }
        }

        f.debug_struct("Address")
            .field("inner", &s)
            .field("length", &self.length)
            .field("index", &self.to_index())
            .finish()
    }
}

mod serde_address_impl {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct LedgerAddress {
        index: u64,
        length: usize,
    }

    impl<const NBYTES: usize> Serialize for Address<NBYTES> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let addr = LedgerAddress {
                index: self.to_index().0,
                length: self.length(),
            };
            addr.serialize(serializer)
        }
    }

    impl<'de, const NBYTES: usize> Deserialize<'de> for Address<NBYTES> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let addr = LedgerAddress::deserialize(deserializer)?;
            Ok(Address::from_index(AccountIndex(addr.index), addr.length))
        }
    }
}

impl<const NBYTES: usize> IntoIterator for Address<NBYTES> {
    type Item = Direction;

    type IntoIter = AddressIterator<NBYTES>;

    fn into_iter(self) -> Self::IntoIter {
        let length = self.length;
        AddressIterator {
            length,
            addr: self,
            iter_index: 0,
            iter_back_index: length,
        }
    }
}

impl<const NBYTES: usize> From<Address<NBYTES>> for MerkleAddressBinableArgStableV1 {
    fn from(value: Address<NBYTES>) -> Self {
        Self((value.length() as u64).into(), value.used_bytes().into())
    }
}

impl<const NBYTES: usize> Address<NBYTES> {
    pub fn to_linear_index(&self) -> usize {
        let index = self.to_index();

        2usize.pow(self.length as u32) + index.0 as usize - 1
    }

    pub fn iter(&self) -> AddressIterator<NBYTES> {
        AddressIterator {
            addr: self.clone(),
            length: self.length,
            iter_index: 0,
            iter_back_index: self.length,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn root() -> Self {
        Self {
            inner: [0; NBYTES],
            length: 0,
        }
    }

    pub const fn first(length: usize) -> Self {
        Self {
            inner: [0; NBYTES],
            length,
        }
    }

    pub fn last(length: usize) -> Self {
        Self {
            inner: [!0; NBYTES],
            length,
        }
    }

    pub fn child_left(&self) -> Self {
        Self {
            inner: self.inner,
            length: self.length + 1,
        }
    }

    pub fn child_right(&self) -> Self {
        let mut child = self.child_left();
        child.set(child.length() - 1);
        child
    }

    pub fn parent(&self) -> Option<Self> {
        if self.length == 0 {
            None
        } else {
            Some(Self {
                inner: self.inner,
                length: self.length - 1,
            })
        }
    }

    pub fn is_root(&self) -> bool {
        self.length == 0
    }

    pub fn get(&self, index: usize) -> Direction {
        let byte_index = index / 8;
        let bit_index = index % 8;

        if self.inner[byte_index] & (1 << (7 - bit_index)) != 0 {
            Direction::Right
        } else {
            Direction::Left
        }
    }

    fn set(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        self.inner[byte_index] |= 1 << (7 - bit_index);
    }

    fn unset(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        self.inner[byte_index] &= !(1 << (7 - bit_index));
    }

    pub fn nused_bytes(&self) -> usize {
        self.length.saturating_sub(1) / 8 + 1

        // let length_div = self.length / 8;
        // let length_mod = self.length % 8;

        // if length_mod == 0 {
        //     length_div
        // } else {
        //     length_div + 1
        // }
    }

    pub fn used_bytes(&self) -> &[u8] {
        &self.inner[..self.nused_bytes()]
    }

    pub(super) fn clear_after(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        const MASK: [u8; 8] = [
            0b10000000, 0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11111100, 0b11111110,
            0b11111111,
        ];

        self.inner[byte_index] &= MASK[bit_index];

        for byte_index in byte_index + 1..self.nused_bytes() {
            self.inner[byte_index] = 0;
        }
    }

    fn set_after(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        const MASK: [u8; 8] = [
            0b01111111, 0b00111111, 0b00011111, 0b00001111, 0b00000111, 0b00000011, 0b00000001,
            0b00000000,
        ];

        self.inner[byte_index] |= MASK[bit_index];

        for byte_index in byte_index + 1..self.nused_bytes() {
            self.inner[byte_index] = !0;
        }
    }

    pub fn next(&self) -> Option<Self> {
        let length = self.length;
        let mut next = self.clone();

        let nused_bytes = self.nused_bytes();

        const MASK: [u8; 8] = [
            0b00000000, 0b01111111, 0b00111111, 0b00011111, 0b00001111, 0b00000111, 0b00000011,
            0b00000001,
        ];

        next.inner[nused_bytes - 1] |= MASK[length % 8];

        let rightmost_clear_index = next.inner[0..nused_bytes]
            .iter()
            .rev()
            .enumerate()
            .find_map(|(index, byte)| match byte.trailing_ones() as usize {
                x if x == 8 => None,
                x => Some((nused_bytes - index) * 8 - x - 1),
            })?;

        next.set(rightmost_clear_index);
        next.clear_after(rightmost_clear_index);

        assert_ne!(self, &next);

        Some(next)
    }

    pub fn prev(&self) -> Option<Self> {
        let length = self.length;
        let mut prev = self.clone();
        let nused_bytes = self.nused_bytes();

        const MASK: [u8; 8] = [
            0b11111111, 0b10000000, 0b11000000, 0b11100000, 0b11110000, 0b11111000, 0b11111100,
            0b11111110,
        ];

        prev.inner[nused_bytes - 1] &= MASK[length % 8];

        let nused_bytes = self.nused_bytes();

        let rightmost_one_index = prev.inner[0..nused_bytes]
            .iter()
            .rev()
            .enumerate()
            .find_map(|(index, byte)| match byte.trailing_zeros() as usize {
                x if x == 8 => None,
                x => Some((nused_bytes - index) * 8 - x - 1),
            })?;

        prev.unset(rightmost_one_index);
        prev.set_after(rightmost_one_index);

        assert_ne!(self, &prev);

        Some(prev)
    }

    /// Returns first address in the next depth.
    pub fn next_depth(&self) -> Self {
        Self::first(self.length.saturating_add(1))
    }

    /// Returns next address on the same depth or
    /// the first address in the next depth.
    pub fn next_or_next_depth(&self) -> Self {
        self.next().unwrap_or_else(|| self.next_depth())
    }

    pub fn to_index(&self) -> AccountIndex {
        if self.length == 0 {
            return AccountIndex(0);
        }

        let mut account_index: u64 = 0;
        let nused_bytes = self.nused_bytes();
        let mut shift = 0;

        self.inner[0..nused_bytes]
            .iter()
            .rev()
            .enumerate()
            .for_each(|(index, byte)| {
                let byte = *byte as u64;

                if index == 0 && self.length % 8 != 0 {
                    let nunused = self.length % 8;
                    account_index |= byte >> (8 - nunused);
                    shift += nunused;
                } else {
                    account_index |= byte << shift;
                    shift += 8;
                }
            });

        AccountIndex(account_index)
    }

    pub fn from_index(index: AccountIndex, length: usize) -> Self {
        let account_index = index.0;
        let mut addr = Address::first(length);

        for (index, bit_index) in (0..length).rev().enumerate() {
            if account_index & (1 << bit_index) != 0 {
                addr.set(index);
            }
        }

        addr
    }

    pub fn iter_children(&self, length: usize) -> AddressChildrenIterator<NBYTES> {
        assert!(self.length <= length);

        let root_length = self.length;
        let mut current = self.clone();

        let mut until = current.next().map(|mut until| {
            until.length = length;
            until.clear_after(root_length);
            until
        });

        current.length = length;
        current.clear_after(root_length);

        let current = Some(current);
        if until == current {
            until = None;
        }

        AddressChildrenIterator {
            current,
            until,
            nchildren: 2u64.pow(length as u32 - root_length as u32),
        }
    }

    pub fn is_before(&self, other: &Self) -> bool {
        assert!(self.length <= other.length);

        let mut other = other.clone();
        other.length = self.length;

        self.to_index() <= other.to_index()

        // self == &other
    }

    pub fn is_parent_of(&self, other: &Self) -> bool {
        if self.length == 0 {
            return true;
        }

        assert!(self.length <= other.length);

        let mut other = other.clone();
        other.length = self.length;

        self == &other
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let mut s = String::with_capacity(self.length());

        for index in 0..self.length {
            match self.get(index) {
                Direction::Left => s.push('0'),
                Direction::Right => s.push('1'),
            }
        }

        s
    }

    #[cfg(test)]
    pub fn rand_nonleaf(max_depth: usize) -> Self {
        use rand::{Rng, RngCore};

        let mut rng = rand::thread_rng();
        let length = rng.gen_range(0..max_depth);

        let mut inner = [0; NBYTES];
        rng.fill_bytes(&mut inner[0..(length / 8) + 1]);

        Self { inner, length }
    }
}

pub struct AddressIterator<const NBYTES: usize> {
    addr: Address<NBYTES>,
    iter_index: usize,
    iter_back_index: usize,
    length: usize,
}

impl<const NBYTES: usize> DoubleEndedIterator for AddressIterator<NBYTES> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let prev = self.iter_back_index.checked_sub(1)?;
        self.iter_back_index = prev;
        Some(self.addr.get(prev))
    }
}

impl<const NBYTES: usize> Iterator for AddressIterator<NBYTES> {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        let iter_index = self.iter_index;

        if iter_index >= self.length {
            return None;
        }
        self.iter_index += 1;

        Some(self.addr.get(iter_index))
    }
}

#[derive(Debug)]
pub struct AddressChildrenIterator<const NBYTES: usize> {
    current: Option<Address<NBYTES>>,
    until: Option<Address<NBYTES>>,
    nchildren: u64,
}

impl<const NBYTES: usize> AddressChildrenIterator<NBYTES> {
    pub fn len(&self) -> usize {
        self.nchildren as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<const NBYTES: usize> Iterator for AddressChildrenIterator<NBYTES> {
    type Item = Address<NBYTES>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.until {
            return None;
        }
        let current = self.current.clone()?;
        self.current = current.next();

        Some(current)
    }
}
