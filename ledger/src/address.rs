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
            .finish()
    }
}

impl Address {
    pub fn iter(&self) -> AddressIterator {
        AddressIterator {
            addr: self.clone(),
            length: self.length,
            iter_index: 0,
        }
    }

    pub fn into_iter(self) -> AddressIterator {
        AddressIterator {
            length: self.length,
            addr: self,
            iter_index: 0,
        }
    }

    pub fn first(length: usize) -> Self {
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
}

pub struct AddressIterator {
    addr: Address,
    iter_index: usize,
    length: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(addr.iter().collect::<Vec<_>>().is_empty());

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
}
