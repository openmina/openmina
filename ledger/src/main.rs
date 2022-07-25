#![allow(dead_code)]

use mina_signer::CompressedPubKey;

type PlaceHolder = ();

// TODO: Not sure if it's the correct type
type Balance = u64;

// TODO: Not sure if it's the correct type
type Amount = u64;

// TODO: Not sure if it's the correct type
type TokenId = u64;

type Slot = u32;

// TODO: Those types are `Field.t` in ocaml
//       not sure how to represent them in Rust, they seem to be 256 bits
// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/receipt.mli#L67
type ReceiptChainHash = [u8; 32];
type VotingFor = [u8; 32];

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account_timing.ml#L31-L34
#[derive(Clone, Debug)]
enum Timing {
    Untimed,
    Timed {
        initial_minimum_balance: Balance,
        cliff_time: Slot,
        cliff_amount: Amount,
        vesting_period: Slot,
        vesting_increment: Amount,
    },
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account.ml#L93
type TokenSymbol = String;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_numbers/intf.ml#L155
type Nonce = u32;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/token_permissions.ml#L9
#[derive(Clone, Debug)]
enum TokenPermissions {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L10
#[derive(Clone, Debug)]
enum AuthRequired {
    None,
    Either,
    Proof,
    Signature,
    Impossible,
}

impl Default for AuthRequired {
    fn default() -> Self {
        Self::None
    }
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L49
#[derive(Clone, Debug)]
struct Permissions<Controller> {
    edit_state: Controller,
    send: Controller,
    receive: Controller,
    set_delegate: Controller,
    set_permissions: Controller,
    set_verification_key: Controller,
    set_zkapp_uri: Controller,
    edit_sequence_state: Controller,
    set_token_symbol: Controller,
    increment_nonce: Controller,
    set_voting_for: Controller,
}

impl Default for Permissions<AuthRequired> {
    fn default() -> Self {
        Self::user_default()
    }
}

impl Permissions<AuthRequired> {
    fn user_default() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: Signature,
            send: Signature,
            receive: None,
            set_delegate: Signature,
            set_permissions: Signature,
            set_verification_key: Signature,
            set_zkapp_uri: Signature,
            edit_sequence_state: Signature,
            set_token_symbol: Signature,
            increment_nonce: Signature,
            set_voting_for: Signature,
        }
    }

    fn empty() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: None,
            send: None,
            receive: None,
            set_delegate: None,
            set_permissions: None,
            set_verification_key: None,
            set_zkapp_uri: None,
            edit_sequence_state: None,
            set_token_symbol: None,
            increment_nonce: None,
            set_voting_for: None,
        }
    }
}

// TODO: Fill this struct
// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/zkapp_account.ml#L148-L170
#[derive(Clone, Debug)]
struct ZkAppAccount {
    app_state: Vec<u8>,
    verification: (),
    zkapp_version: u32,
    sequence_state: (),
    last_sequence_slot: Slot,
    proved_state: bool,
}

// https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368
#[derive(Clone, Debug)]
struct Account {
    pub public_key: CompressedPubKey,         // Public_key.Compressed.t
    pub token_id: TokenId,                    // Token_id.t
    pub token_permissions: TokenPermissions,  // Token_permissions.t
    pub token_symbol: TokenSymbol,            // Token_symbol.t
    pub balance: Balance,                     // Balance.t
    pub nonce: Nonce,                         // Nonce.t
    pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
    pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
    pub voting_for: VotingFor,                // State_hash.t
    pub timing: Timing,                       // Timing.t
    pub permissions: Permissions<AuthRequired>, // Permissions.t
    pub zkapp: Option<ZkAppAccount>,          // Zkapp_account.t
    pub zkapp_uri: String,                    // string
}

use mina_hasher::{create_legacy, Hashable, Hasher, ROInput};

impl Hashable for Account {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new();

        roi.append_field(self.public_key.x);
        roi.append_bool(self.public_key.is_odd);

        roi
    }

    fn domain_string(_: ()) -> Option<String> {
        Some("CodaAccount*********".to_string())
    }
}

// mina_hasher::poseidon::

impl Account {
    fn create() -> Self {
        Self {
            public_key: CompressedPubKey::from_address(
                "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
                // "B62qiTKpEPjGTSHZrtM8uXiKgn8So916pLmNJKDhKeyBQL9TDb3nvBG", // Public_key.Compressed.empty
            )
            .unwrap(),
            token_id: 0,
            token_permissions: TokenPermissions::NotOwned {
                account_disabled: false,
            },
            token_symbol: String::new(),
            balance: 0,
            nonce: 0,
            receipt_chain_hash: ReceiptChainHash::default(),
            delegate: None,
            voting_for: VotingFor::default(),
            timing: Timing::Untimed,
            permissions: Permissions::default(),
            zkapp: None,
            zkapp_uri: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct Hash {
    inner: Box<[u8; 32]>,
}

#[derive(Clone, Debug)]
enum NodeOrLeaf {
    Leaf(Leaf),
    Node(Node),
}

#[derive(Clone, Debug, Default)]
struct Node {
    left: Option<Box<NodeOrLeaf>>,
    left_hash: Option<Hash>,
    right: Option<Box<NodeOrLeaf>>,
    right_hash: Option<Hash>,
}

#[derive(Clone, Debug)]
struct Leaf {
    account: Box<Account>,
}

#[derive(Debug)]
struct Database {
    root: Option<NodeOrLeaf>,
    depth: u8,
    last_location: Option<Address>,
}

impl NodeOrLeaf {
    fn add_to_path(&mut self, account: Account, path_iter: &mut AddressIterator) {
        let direction = match path_iter.next() {
            Some(direction) => direction,
            None => {
                *self = NodeOrLeaf::Leaf(Leaf {
                    account: Box::new(account),
                });
                return;
            }
        };

        let node = match self {
            NodeOrLeaf::Node(node) => node,
            NodeOrLeaf::Leaf(_) => panic!("Expected node"),
        };

        let child = match direction {
            Direction::Left => &mut node.left,
            Direction::Right => &mut node.right,
        };

        let child = match child {
            Some(child) => child,
            None => {
                *child = Some(Box::new(NodeOrLeaf::Node(Node::default())));
                child.as_mut().unwrap()
            }
        };

        child.add_to_path(account, path_iter);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

#[derive(Clone, Eq)]
struct Address {
    inner: [u8; 32],
    length: usize,
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
    fn iter(&self) -> AddressIterator {
        AddressIterator {
            addr: self.clone(),
            length: self.length,
            iter_index: 0,
        }
    }

    fn into_iter(self) -> AddressIterator {
        AddressIterator {
            length: self.length,
            addr: self,
            iter_index: 0,
        }
    }

    fn first(length: usize) -> Self {
        Self {
            inner: [0; 32],
            length,
        }
    }

    fn last(length: usize) -> Self {
        Self {
            inner: [!0; 32],
            length,
        }
    }

    fn get(&self, index: usize) -> Direction {
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

    fn next(&self) -> Option<Address> {
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

    fn prev(&self) -> Option<Address> {
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

struct AddressIterator {
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

#[derive(Debug, PartialEq, Eq)]
enum DatabaseError {
    OutOfLeaves,
}

impl Database {
    fn create(depth: u8) -> Self {
        assert!((1..0xfe).contains(&depth));

        Self {
            depth,
            root: None,
            last_location: None,
        }
    }

    fn create_account(
        &mut self,
        _account_id: PlaceHolder,
        account: Account,
    ) -> Result<Address, DatabaseError> {
        if self.root.is_none() {
            self.root = Some(NodeOrLeaf::Node(Node::default()));
        }

        let location = match self.last_location.as_ref() {
            Some(last) => last.next().ok_or(DatabaseError::OutOfLeaves)?,
            None => Address::first(self.depth as usize),
        };

        let root = self.root.as_mut().unwrap();
        let mut path_iter = location.clone().into_iter();
        root.add_to_path(account, &mut path_iter);

        self.last_location = Some(location.clone());

        Ok(location)
    }

    fn naccounts(&self) -> usize {
        let mut naccounts = 0;

        if let Some(root) = self.root.as_ref() {
            self.naccounts_recursive(root, &mut naccounts)
        };

        naccounts
    }

    fn naccounts_recursive(&self, elem: &NodeOrLeaf, naccounts: &mut usize) {
        match elem {
            NodeOrLeaf::Leaf(_) => *naccounts += 1,
            NodeOrLeaf::Node(node) => {
                if let Some(left) = node.left.as_ref() {
                    self.naccounts_recursive(left, naccounts);
                };
                if let Some(right) = node.right.as_ref() {
                    self.naccounts_recursive(right, naccounts);
                };
            }
        }
    }
}

fn main() {
    let db = Database::create(3);

    println!("Hello, world! {:#?}", db);
}

#[cfg(test)]
mod tests {
    use mina_hasher::create_kimchi;

    use super::*;

    #[test]
    fn test_db() {
        let two: usize = 2;

        for depth in 2..17 {
            let mut db = Database::create(depth);

            for _ in 0..two.pow(depth as u32) {
                db.create_account((), Account::create()).unwrap();
            }

            let naccounts = db.naccounts();
            assert_eq!(naccounts, two.pow(depth as u32));

            assert_eq!(
                db.create_account((), Account::create()).unwrap_err(),
                DatabaseError::OutOfLeaves
            );

            println!("depth={:?} naccounts={:?}", depth, naccounts);
        }
    }

    #[test]
    fn test_address() {
        use Direction::*;

        let mut inner: [u8; 32] = Default::default();
        inner[0] = 0b10101010;

        let addr = Address { inner, length: 8 };
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Right, Left, Right, Left, Right, Left, Right, Left]
        );

        let mut inner: [u8; 32] = Default::default();
        inner[0] = 0b01010101;

        let addr = Address { inner, length: 8 };
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Left, Right, Left, Right, Left, Right, Left, Right]
        );

        let mut inner: [u8; 32] = Default::default();
        inner[0] = 0b01010101;

        let addr = Address { inner, length: 9 };
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Left, Right, Left, Right, Left, Right, Left, Right, Left]
        );

        let mut inner: [u8; 32] = Default::default();
        inner[0] = 0b01010101;
        inner[1] = 0b01000000;

        let addr = Address { inner, length: 10 };
        assert_eq!(
            addr.iter().collect::<Vec<_>>(),
            &[Left, Right, Left, Right, Left, Right, Left, Right, Left, Right]
        );
    }

    #[test]
    fn test_address_next() {
        let two: usize = 2;

        // prev
        for length in 5..25 {
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
        for length in 5..25 {
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
    fn test_hash_account() {
        let acc = Account::create();

        let mut hasher = create_kimchi::<Account>(());
        hasher.update(&acc);
        let out = hasher.digest();

        println!("kimchi={}", out.to_string());

        let mut hasher = create_legacy::<Account>(());
        hasher.update(&acc);
        let out = hasher.digest();

        println!("legacy={}", out.to_string());
    }
}
