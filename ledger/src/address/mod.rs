pub mod raw;

const fn compute_nbytes(nbits: usize) -> usize {
    if nbits % 8 == 0 {
        nbits / 8
    } else {
        (nbits / 8) + 1
    }
}

/// Berkeleynet uses trees of depth 35, which requires addresses of 35 bits
const NBITS: usize = 35;
const NBYTES: usize = compute_nbytes(NBITS);

pub use raw::Direction;
pub type Address = raw::Address<NBYTES>;
pub type AddressIterator = raw::AddressIterator<NBYTES>;

#[cfg(test)]
mod tests {
    use crate::AccountIndex;

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
        assert!(Address::try_from("0".repeat((NBYTES * 8) - 1).as_str()).is_ok());
        assert!(Address::try_from("0".repeat(NBYTES * 8).as_str()).is_err());
    }

    #[test]
    fn test_address_next() {
        let two: usize = 2;

        // prev
        for length in 5..23 {
            let mut addr = Address::last(length);

            elog!("length={length} until={:?}", two.pow(length as u32));
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

            elog!("length={length} until={:?}", two.pow(length as u32));
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
        let mut inner: [u8; NBYTES] = Default::default();
        inner[0] = 0b11111111;
        inner[1] = 0b11111111;

        let mut addr = Address { inner, length: 12 };
        elog!("ADDR={:?}", addr);
        addr.clear_after(6);
        elog!("ADDR={:?}", addr);
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
        elog!("LA {:?}", addr);
        let sec = addr.next().unwrap();
        elog!("LA {:?}", sec);
        elog!("LA {:?}", sec.child_left());
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
    fn test_address_linear() {
        for (index, s) in [
            "", "0", "1", "00", "01", "10", "11", "000", "001", "010", "011", "100", "101", "110",
            "111",
        ]
        .iter()
        .enumerate()
        {
            let addr = Address::try_from(*s).unwrap();
            assert_eq!(index, addr.to_linear_index());
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

    #[test]
    fn test_address_children_parent_root_eq() {
        let left = Address::first(1);
        let right = left.next().unwrap();
        assert_eq!(left.parent().unwrap(), Address::root());
        assert_eq!(right.parent().unwrap(), Address::root());
        assert_eq!(left.parent(), right.parent());
    }
}
