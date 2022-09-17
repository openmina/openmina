use crate::base::AccountIndex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Clone, Eq)]
pub struct Address {
    inner: [u8; 32],
    length: usize,
}

impl<'a> TryFrom<&'a str> for Address {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if s.len() >= 256 {
            return Err(());
        }

        let mut addr = Address {
            inner: [0; 32],
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

impl PartialEq for Address {
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

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::with_capacity(256);

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

impl Address {
    pub fn to_linear_index(&self) -> usize {
        let index = self.to_index();

        2usize.pow(self.length as u32) + index.0 as usize
    }

    pub fn iter(&self) -> AddressIterator {
        AddressIterator {
            addr: self.clone(),
            length: self.length,
            iter_index: 0,
            iter_back_index: self.length,
        }
    }

    pub fn into_iter(self) -> AddressIterator {
        let length = self.length;
        AddressIterator {
            length,
            addr: self,
            iter_index: 0,
            iter_back_index: length,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn root() -> Self {
        Self {
            inner: [0; 32],
            length: 0,
        }
    }

    pub const fn first(length: usize) -> Self {
        Self {
            inner: [0; 32],
            length,
        }
    }

    pub fn last(length: usize) -> Self {
        Self {
            inner: [!0; 32],
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

    fn nused_bytes(&self) -> usize {
        self.length.saturating_sub(1) / 8 + 1

        // let length_div = self.length / 8;
        // let length_mod = self.length % 8;

        // if length_mod == 0 {
        //     length_div
        // } else {
        //     length_div + 1
        // }
    }

    fn clear_after(&mut self, index: usize) {
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

    pub fn next(&self) -> Option<Address> {
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

    pub fn prev(&self) -> Option<Address> {
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

    pub fn iter_children(&self, length: usize) -> AddressChildrenIterator {
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

    pub fn is_before(&self, other: &Address) -> bool {
        assert!(self.length <= other.length);

        let mut other = other.clone();
        other.length = self.length;

        self.to_index() <= other.to_index()

        // self == &other
    }

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

        let mut inner = [0; 32];
        rng.fill_bytes(&mut inner[0..(length / 8) + 1]);

        Self { inner, length }
    }
}

pub struct AddressIterator {
    addr: Address,
    iter_index: usize,
    iter_back_index: usize,
    length: usize,
}

impl DoubleEndedIterator for AddressIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        let prev = self.iter_back_index.checked_sub(1)?;
        self.iter_back_index = prev;
        Some(self.addr.get(prev))
    }
}

impl Iterator for AddressIterator {
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
pub struct AddressChildrenIterator {
    current: Option<Address>,
    until: Option<Address>,
    nchildren: u64,
}

impl AddressChildrenIterator {
    pub fn len(&self) -> usize {
        self.nchildren as usize
    }
}

impl Iterator for AddressChildrenIterator {
    type Item = Address;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.until {
            return None;
        }
        let current = self.current.clone()?;
        self.current = current.next();

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn test_address_iter() {
        use Direction::*;

        let addr = Address::try_from("10101010").unwrap();
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Right, Left, Right, Left, Right, Left, Right, Left]
        );

        let addr = Address::try_from("01010101").unwrap();
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Left, Right, Left, Right, Left, Right, Left, Right]
        );

        let addr = Address::try_from("010101010").unwrap();
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Left, Right, Left, Right, Left, Right, Left, Right, Left]
        );

        let addr = Address::try_from("0101010101").unwrap();
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Left, Right, Left, Right, Left, Right, Left, Right, Left, Right]
        );

        let addr = Address::try_from("").unwrap();
        assert!(addr.iter().next().is_none());

        assert!(Address::try_from("0101010101a").is_err());
        assert!(Address::try_from("0".repeat(255).as_str()).is_ok());
        assert!(Address::try_from("0".repeat(256).as_str()).is_err());
    }

    #[test]
    fn test_address_next() {
        let two: usize = 2;

        // prev
        for length in 5..23 {
            let mut addr = Address::last(length);

            println!("length={length} until={:?}", two.pow(length as u32));
            for _ in 0..two.pow(length as u32) - 1 {
                let prev = addr.prev().unwrap();
                assert_eq!(prev.next().unwrap(), addr);
                addr = prev;
            }

            assert!(addr.prev().is_none());
        }

        // next
        for length in 5..23 {
            let mut addr = Address::first(length);

            println!("length={length} until={:?}", two.pow(length as u32));
            for _ in 0..two.pow(length as u32) - 1 {
                let next = addr.next().unwrap();
                assert_eq!(next.prev().unwrap(), addr);
                addr = next;
            }

            assert!(addr.next().is_none());
        }
    }

    #[test]
    fn test_address_clear() {
        let mut inner: [u8; 32] = Default::default();
        inner[0] = 0b11111111;
        inner[1] = 0b11111111;

        let mut addr = Address { inner, length: 12 };
        println!("ADDR={:?}", addr);
        addr.clear_after(6);
        println!("ADDR={:?}", addr);
    }

    #[test]
    fn test_address_can_reach() {
        let addr = Address::try_from("00").unwrap();
        assert!(addr.is_before(&Address::try_from("00").unwrap()));
        assert!(addr.is_before(&Address::try_from("01").unwrap()));
        assert!(addr.is_before(&Address::try_from("000").unwrap()));
        assert!(addr.is_before(&Address::try_from("001").unwrap()));
        assert!(addr.is_before(&Address::try_from("010").unwrap()));
        assert!(addr.is_before(&Address::try_from("100").unwrap()));

        let addr = Address::try_from("101").unwrap();
        assert!(addr.is_before(&Address::try_from("10100").unwrap()));
        assert!(addr.is_before(&Address::try_from("10111").unwrap()));
        assert!(!addr.is_before(&Address::try_from("10011").unwrap()));
    }

    #[test]
    fn test_address_show() {
        let addr = Address::first(2);
        println!("LA {:?}", addr);
        let sec = addr.next().unwrap();
        println!("LA {:?}", sec);
        println!("LA {:?}", sec.child_left());
    }

    #[test]
    fn test_address_index() {
        for length in 1..20 {
            let mut addr = Address::first(length);

            for index in 0..2u64.pow(length as u32) - 1 {
                let to_index = addr.to_index();

                assert_eq!(to_index, AccountIndex(index));
                assert_eq!(addr, Address::from_index(to_index, length));

                addr = addr.next().unwrap();
            }
        }
    }

    #[test]
    fn test_address_children() {
        let root = Address::try_from("00").unwrap();
        let iter_children = root.iter_children(4);
        assert_eq!(iter_children.len(), 4);
        assert_eq!(
            iter_children.collect::<Vec<_>>(),
            &[
                Address::try_from("0000").unwrap(),
                Address::try_from("0001").unwrap(),
                Address::try_from("0010").unwrap(),
                Address::try_from("0011").unwrap(),
            ]
        );

        let root = Address::try_from("01").unwrap();
        let iter_children = root.iter_children(4);
        assert_eq!(iter_children.len(), 4);
        assert_eq!(
            iter_children.collect::<Vec<_>>(),
            &[
                Address::try_from("0100").unwrap(),
                Address::try_from("0101").unwrap(),
                Address::try_from("0110").unwrap(),
                Address::try_from("0111").unwrap(),
            ]
        );

        let root = Address::try_from("10").unwrap();
        let iter_children = root.iter_children(4);
        assert_eq!(iter_children.len(), 4);
        assert_eq!(
            iter_children.collect::<Vec<_>>(),
            &[
                Address::try_from("1000").unwrap(),
                Address::try_from("1001").unwrap(),
                Address::try_from("1010").unwrap(),
                Address::try_from("1011").unwrap(),
            ]
        );

        let root = Address::try_from("11").unwrap();
        let iter_children = root.iter_children(4);
        assert_eq!(iter_children.len(), 4);
        assert_eq!(
            iter_children.collect::<Vec<_>>(),
            &[
                Address::try_from("1100").unwrap(),
                Address::try_from("1101").unwrap(),
                Address::try_from("1110").unwrap(),
                Address::try_from("1111").unwrap(),
            ]
        );

        let root = Address::try_from("00").unwrap();
        let iter_children = root.iter_children(6);
        assert_eq!(iter_children.len(), 16);
    }
}
