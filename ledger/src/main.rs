#![allow(dead_code)]

mod poseidon;

use std::{default, str::FromStr, borrow::Cow};

use ark_ff::{Zero, PrimeField};
use mina_signer::CompressedPubKey;
use o1_utils::field_helpers::FieldHelpers;

type PlaceHolder = ();

// TODO: Not sure if it's the correct type
type Balance = u64;

// TODO: Not sure if it's the correct type
type Amount = u64;

// TODO: Not sure if it's the correct type
// type TokenId = Fp;

// TODO: Not sure if it's the correct type
//       It seems that the token id is a simple number, but on ocaml when they
//       convert it to/from string (base58), they add/remove the byte 0x1C:
//       https://github.com/MinaProtocol/mina/blob/3a35532cb19d17583b63036bc50d8dde5460b791/src/lib/mina_base/account_id.ml#L30
//       need more research
#[derive(Clone, Debug)]
struct TokenId(u64);

impl Default for TokenId {
    fn default() -> Self {
        Self(1)
    }
}

type Slot = u32;

// TODO: Those types are `Field.t` in ocaml
//       not sure how to represent them in Rust, they seem to be 256 bits
// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/receipt.mli#L67
// type VotingFor = [u8; 32];

#[derive(Clone, Debug, Default)]
struct VotingFor(Fp);

impl VotingFor {
    fn dummy() -> Self {
        Self(Fp::zero())
    }
}

#[derive(Clone, Debug)]
struct ReceiptChainHash(Fp);

impl ReceiptChainHash {
    fn empty() -> Self {
        Self(empty_receipt_hash())
    }
}

fn empty_receipt_hash() -> Fp {
    // Value of `Receipt.Chain_hash.empty` in Ocaml (`develop` branch)
    // let empty_hex = "9be4b7c51ed9c2e4524727805fd36f5220fbfc70a749f62623b0ed2908433320";
    // Fp::from_hex(&empty_hex).unwrap()

    // Value of `Receipt.Chain_hash.empty` in Ocaml (`compatible` branch)
    Fp::from_hex("0b143c0645497a5987a7b88f66340e03db943f0a0df48b69a3a82921ce97b10a").unwrap()
}

impl Default for ReceiptChainHash {
    fn default() -> Self {
        Self::empty()
    }
}

// CodaReceiptEmpty

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

impl Default for TokenPermissions {
    fn default() -> Self {
        Self::NotOwned { account_disabled: false }
    }
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L10
#[derive(Copy, Clone, Debug)]
enum AuthRequired {
    None,
    Either,
    Proof,
    Signature,
    Impossible,
    Both, // Legacy only
}

impl Default for AuthRequired {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, Debug)]
struct AuthRequiredEncoded {
    constant: bool,
    signature_necessary: bool,
    signature_sufficient: bool,
}

impl AuthRequired {
    fn encode(self) -> AuthRequiredEncoded {
        let (constant, signature_necessary, signature_sufficient) = match self {
            AuthRequired::None => (true, false, true),
            AuthRequired::Either => (false, false, true),
            AuthRequired::Proof => (false, false, false),
            AuthRequired::Signature => (false, true, true),
            AuthRequired::Impossible => (true, true, false),
            AuthRequired::Both => (false, true, false),
        };

        AuthRequiredEncoded {
            constant,
            signature_necessary,
            signature_sufficient,
        }
    }
}

impl AuthRequiredEncoded {
    fn decode(self) -> AuthRequired {
        match (
            self.constant,
            self.signature_necessary,
            self.signature_sufficient,
        ) {
            (true, _, false) => AuthRequired::Impossible,
            (true, _, true) => AuthRequired::None,
            (false, false, false) => AuthRequired::Proof,
            (false, true, true) => AuthRequired::Signature,
            (false, false, true) => AuthRequired::Either,
            (false, true, false) => AuthRequired::Both,
        }
    }

    fn to_bits(self) -> [bool; 3] {
        [
            self.constant,
            self.signature_necessary,
            self.signature_sufficient,
        ]
    }
}

#[derive(Clone, Debug)]
struct PermissionsLegacy<Controller> {
    stake: bool,
    edit_state: Controller,
    send: Controller,
    receive: Controller,
    set_delegate: Controller,
    set_permissions: Controller,
    set_verification_key: Controller,
}

impl PermissionsLegacy<AuthRequired> {
    fn user_default() -> Self {
        use AuthRequired::*;

        Self {
            stake: true,
            edit_state: Signature,
            send: Signature,
            receive: None,
            set_delegate: Signature,
            set_permissions: Signature,
            set_verification_key: Signature,
        }
    }

    fn empty() -> Self {
        use AuthRequired::*;

        Self {
            stake: false,
            edit_state: None,
            send: None,
            receive: None,
            set_delegate: None,
            set_permissions: None,
            set_verification_key: None,
        }
    }
}

impl Default for PermissionsLegacy<AuthRequired> {
    fn default() -> Self {
        Self::user_default()
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

#[derive(Clone, Debug)]
struct SnappAccount {
    app_state: Vec<Fp>,
    verification_key: Option<Fp>,
}

impl Default for SnappAccount {
    fn default() -> Self {
        Self { app_state: vec![Fp::zero(); 8], verification_key: None }
    }
}

impl Hashable for SnappAccount {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new();

        if let Some(vk) = self.verification_key.as_ref() {
            roi.append_field(*vk);
        } else {
            roi.append_field(
                // Value of `dummy_vk_hash`:
                // https://github.com/MinaProtocol/mina/blob/4f765c866b81fa6fed66be52707fd91fd915041d/src/lib/mina_base/snapp_account.ml#L116
                Fp::from_hex("77a430a03efafd14d72e1a3c45a1fdca8267fcce9a729a1d25128bb5dec69d3f").unwrap()
            );
        }

        for field in &self.app_state {
            roi.append_field(*field);
        }

        // println!("ROInput={:?}", roi);

        roi
    }

    fn domain_string(_domain_param: Self::D) -> Option<String> {
        Some("CodaSnappAccount****".to_string())
    }
}

// https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368
#[derive(Clone, Debug)]
struct Account {
    pub public_key: CompressedPubKey,         // Public_key.Compressed.t
    pub token_id: TokenId,                    // Token_id.t
    pub token_permissions: TokenPermissions,  // Token_permissions.t
    pub balance: Balance,                     // Balance.t
    pub nonce: Nonce,                         // Nonce.t
    pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
    pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
    pub voting_for: VotingFor,                // State_hash.t
    pub timing: Timing,                       // Timing.t
    pub permissions: PermissionsLegacy<AuthRequired>, // Permissions.t
    pub snap: Option<SnappAccount>,

    // Below fields are for `develop` branch
    // pub token_symbol: TokenSymbol,            // Token_symbol.t
    // pub zkapp: Option<ZkAppAccount>,          // Zkapp_account.t
    // pub zkapp_uri: String,                    // string
}

use mina_hasher::{create_kimchi, create_legacy, Fp, Hashable, Hasher, ROInput};

impl Hashable for Account {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new();

        // Self::token_symbol

        // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
        // assert!(self.token_symbol.len() <= 6);

        // if !self.token_symbol.is_empty() {
        //     let mut s = <[u8; 6]>::default();
        //     let len = self.token_symbol.len();

        //     s[..len].copy_from_slice(&self.token_symbol.as_bytes());
        //     roi.append_bytes(self.token_symbol.as_bytes());
        // } else {
        //     roi.append_bytes(&[0; 6]);
        // }

        // Self::snapp
        let snapp_accout = match self.snap.as_ref() {
            Some(snapp) => Cow::Borrowed(snapp),
            None => Cow::Owned(SnappAccount::default()),
        };
        let mut hasher = create_legacy::<SnappAccount>(());
        hasher.update(snapp_accout.as_ref());
        let snapp_digest = hasher.digest();

        roi.append_field(snapp_digest);

        // println!("ROINPUT={:?}", roi);

        // Self::permissions
        for auth in [
            self.permissions.set_verification_key,
            self.permissions.set_permissions,
            self.permissions.set_delegate,
            self.permissions.receive,
            self.permissions.send,
            self.permissions.edit_state,
        ] {
            for bit in auth.encode().to_bits() {
                roi.append_bool(bit);
            }
        }
        roi.append_bool(self.permissions.stake);

        // Self::timing
        match self.timing {
            Timing::Untimed => {
                roi.append_bool(false);
                roi.append_u64(0); // initial_minimum_balance
                roi.append_u32(0); // cliff_time
                roi.append_u64(0); // cliff_amount
                roi.append_u32(1); // vesting_period
                roi.append_u64(0); // vesting_increment
            },
            Timing::Timed { initial_minimum_balance, cliff_time, cliff_amount, vesting_period, vesting_increment } => {
                roi.append_bool(true);
                roi.append_u64(initial_minimum_balance);
                roi.append_u32(cliff_time);
                roi.append_u64(cliff_amount);
                roi.append_u32(vesting_period);
                roi.append_u64(vesting_increment);
            },
        }

        // Self::voting_for
        roi.append_field(self.voting_for.0);

        // Self::delegate
        match self.delegate.as_ref() {
            Some(delegate) => {
                roi.append_field(delegate.x);
                roi.append_bool(delegate.is_odd);
            },
            None => {
                // Public_key.Compressed.empty
                roi.append_field(Fp::zero());
                roi.append_bool(false);
            },
        }

        // Self::receipt_chain_hash
        roi.append_field(self.receipt_chain_hash.0);

        // Self::nonce
        roi.append_u32(self.nonce);

        // Self::balance
        roi.append_u64(self.balance);

        // Self::token_permissions
        match self.token_permissions {
            TokenPermissions::TokenOwned { disable_new_accounts } => {
                roi.append_bool(true);
                roi.append_bool(disable_new_accounts);
            },
            TokenPermissions::NotOwned { account_disabled } => {
                roi.append_bool(false);
                roi.append_bool(account_disabled);
            },
        }

        // Self::token_id
        roi.append_u64(self.token_id.0);

        // Self::public_key
        roi.append_field(self.public_key.x);
        roi.append_bool(self.public_key.is_odd);

        roi
    }

    // fn to_roinput(&self) -> ROInput {
    //     let mut roi = ROInput::new();

    //     // Self::public_key
    //     roi.append_field(self.public_key.x);
    //     roi.append_bool(self.public_key.is_odd);

    //     // Self::token_id
    //     roi.append_u64(self.token_id.0);

    //     // Self::token_permissions
    //     match self.token_permissions {
    //         TokenPermissions::TokenOwned { disable_new_accounts } => {
    //             roi.append_bool(true);
    //             roi.append_bool(disable_new_accounts);
    //         },
    //         TokenPermissions::NotOwned { account_disabled } => {
    //             roi.append_bool(false);
    //             roi.append_bool(account_disabled);
    //         },
    //     }

    //     // Self::balance
    //     roi.append_u64(self.balance);

    //     // Self::token_symbol

    //     // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
    //     // assert!(self.token_symbol.len() <= 6);

    //     // if !self.token_symbol.is_empty() {
    //     //     let mut s = <[u8; 6]>::default();
    //     //     let len = self.token_symbol.len();

    //     //     s[..len].copy_from_slice(&self.token_symbol.as_bytes());
    //     //     roi.append_bytes(self.token_symbol.as_bytes());
    //     // } else {
    //     //     roi.append_bytes(&[0; 6]);
    //     // }

    //     // Self::nonce
    //     roi.append_u32(self.nonce);

    //     // Self::receipt_chain_hash
    //     roi.append_field(self.receipt_chain_hash.0);

    //     // Self::delegate
    //     match self.delegate.as_ref() {
    //         Some(delegate) => {
    //             roi.append_field(delegate.x);
    //             roi.append_bool(delegate.is_odd);
    //         },
    //         None => {
    //             // Public_key.Compressed.empty
    //             roi.append_field(Fp::zero());
    //             roi.append_bool(false);
    //         },
    //     }

    //     // Self::voting_for
    //     roi.append_field(self.voting_for.0);

    //     // Self::timing
    //     match self.timing {
    //         Timing::Untimed => {
    //             roi.append_bool(false);
    //             roi.append_u64(0); // initial_minimum_balance
    //             roi.append_u32(0); // cliff_time
    //             roi.append_u64(0); // cliff_amount
    //             roi.append_u32(1); // vesting_period
    //             roi.append_u64(0); // vesting_increment
    //         },
    //         Timing::Timed { initial_minimum_balance, cliff_time, cliff_amount, vesting_period, vesting_increment } => {
    //             roi.append_bool(true);
    //             roi.append_u64(initial_minimum_balance);
    //             roi.append_u32(cliff_time);
    //             roi.append_u64(cliff_amount);
    //             roi.append_u32(vesting_period);
    //             roi.append_u64(vesting_increment);
    //         },
    //     }

    //     // Self::permissions
    //     for auth in [
    //         self.permissions.set_verification_key,
    //         self.permissions.set_permissions,
    //         self.permissions.set_delegate,
    //         self.permissions.receive,
    //         self.permissions.send,
    //         self.permissions.edit_state,
    //     ] {
    //         for bit in auth.encode().to_bits() {
    //             roi.append_bool(bit);
    //         }
    //     }
    //     roi.append_bool(self.permissions.stake);

    //     // Self::snapp
    //     let snapp_accout = match self.snap.as_ref() {
    //         Some(snapp) => Cow::Borrowed(snapp),
    //         None => Cow::Owned(SnappAccount::default()),
    //     };
    //     let mut hasher = create_legacy::<SnappAccount>(());
    //     hasher.update(snapp_accout.as_ref());
    //     let snapp_digest = hasher.digest();

    //     roi.append_field(snapp_digest);

    //     println!("ROINPUT={:?}", roi);

    //     roi
    // }

    fn domain_string(_: ()) -> Option<String> {
        Some("CodaAccount*********".to_string())
    }
}

// mina_hasher::poseidon::

impl Account {
    fn create() -> Self {
        // use o1_utils::field_helpers::FieldHelpers;

        // let token_id = bs58::decode("wSHV2S4qX9jFsLjQo8r1BsMLH2ZRKsZx6EJd1sbozGPieEC4Jf").into_vec().unwrap();
        // let token_id = Fp::from_bytes(&token_id).unwrap();

        // println!("token_id={:?}", token_id.to_string());

        // let t = bs58::encode(token_id).into_string();
        // let t = bs58::encode(token_id.to_bytes()).into_string();
        // println!("token_id={:?}", t);

        let pubkey = CompressedPubKey::from_address(
            "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
            // "B62qiTKpEPjGTSHZrtM8uXiKgn8So916pLmNJKDhKeyBQL9TDb3nvBG", // Public_key.Compressed.empty
        )
        .unwrap();

        Self {
            public_key: pubkey.clone(),
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::NotOwned {
                account_disabled: false,
            },
            // token_symbol: "".to_string(),
            // token_symbol: String::new(),
            balance: 10101,
            nonce: 62772,
            receipt_chain_hash: ReceiptChainHash::default(),
            delegate: Some(pubkey),
            // delegate: None,
            voting_for: VotingFor::default(),
            timing: Timing::Untimed,
            permissions: PermissionsLegacy::user_default(),
            snap: None,
            // zkapp: None,
            // zkapp_uri: String::new(),
        }
    }

    fn empty() -> Self {
        Self {
            public_key: CompressedPubKey { x: Fp::zero().into(), is_odd: false },
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::default(),
            balance: 0,
            nonce: 0,
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: PermissionsLegacy::user_default(),
            snap: None,
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
    fn add_to_path(node_or_leaf: &mut Self, account: Account, path: AddressIterator) {
        let mut node_or_leaf = node_or_leaf;

        for direction in path {
            let node = match node_or_leaf {
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

            node_or_leaf = &mut *child;
        }

        *node_or_leaf = NodeOrLeaf::Leaf(Leaf {
            account: Box::new(account),
        });
    }

    fn hash(&self, depth: usize) -> Fp {
        let node = match self {
            NodeOrLeaf::Node(node) => node,
            NodeOrLeaf::Leaf(leaf) => {
                let account = &leaf.account;

                let mut hasher = create_legacy::<Account>(());
                hasher.update(account);
                let h = hasher.digest();
                println!("ACC={:?}", h.to_string());
                return h;
            },
        };

        let left_hash = match &node.left {
            Some(left) => {
                left.hash(depth - 1)
            },
            None => {
                empty_hash_at_depth(depth)
            },
        };

        let right_hash = match &node.right {
            Some(right) => {
                right.hash(depth - 1)
            },
            None => {
                empty_hash_at_depth(depth)
            },
        };

        let mut hasher = create_legacy::<TwoHashes>(depth as u32);
        hasher.update(&TwoHashes(left_hash, right_hash));
        let hash = hasher.digest();

        println!("depth={:?} HASH={:?}",
                 depth,
                 hash.to_string(),
        );

        // println!("depth={:?} HASH={:?} left={:?} right={:?}",
        //          depth,
        //          hash.to_string(),
        //          left_hash.to_string(),
        //          right_hash.to_string(),
        // );

        hash
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

impl<'a> TryFrom<&'a str> for Address {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if s.len() >= 256 {
            return Err(());
        }

        let mut addr = Address { inner: [0; 32], length: s.len()};
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

fn account_empty_hash() -> Fp {
    let mut hasher = create_legacy::<Account>(());
    hasher.update(&Account::empty());
    hasher.digest()
}

#[derive(Clone, Debug)]
struct TwoHashes(Fp, Fp);

impl Hashable for TwoHashes {
    type D = u32; // depth

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new();
        roi.append_field(self.0);
        roi.append_field(self.1);
        roi
    }

    fn domain_string(depth: Self::D) -> Option<String> {
        Some(format!("CodaMklTree{:03}", depth))
    }
}

fn empty_hash_at_depth(depth: usize) -> Fp {
    let mut hash = account_empty_hash();

    for depth in 0..depth {
        hash = {
            let mut hasher = create_legacy::<TwoHashes>(depth  as u32);
            hasher.update(&TwoHashes(hash, hash));
            hasher.digest()
        };
    }

    hash
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
        let path_iter = location.clone().into_iter();
        NodeOrLeaf::add_to_path(root, account, path_iter);

        self.last_location = Some(location.clone());

        Ok(location)
    }

    fn root_hash(&self) -> Option<Fp> {
        self.root.as_ref().map(|root| root.hash(self.depth as usize - 1))
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
    use ark_ff::Zero;
    use mina_hasher::create_kimchi;
    use mina_signer::BaseField;

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
    fn test_hash_empty() {
        let account_empty_hash = account_empty_hash();
        assert_eq!(account_empty_hash.to_hex(), "70ccdba14f829608e59a37ed98ffcaeef06dad928d568a9adbde13e3dd104a20");

        for (depth, s) in [
            (0, "70ccdba14f829608e59a37ed98ffcaeef06dad928d568a9adbde13e3dd104a20"),
            (5, "4590712e4bd873ba93d01b665940e0edc48db1a7c90859948b7799f45a443b15"),
            (10, "ba083b16b757794c81233d4ebf1ab000ba4a174a8174c1e8ee8bf0846ec2e10d"),
            (11, "5d65e7d5f4c5441ac614769b913400aa3201f3bf9c0f33441dbf0a33a1239822"),
            (100, "0e4ecb6104658cf8c06fca64f7f1cb3b0f1a830ab50c8c7ed9de544b8e6b2530"),
            (2000, "b05105f8281f75efaf3c6b324563685c8be3a01b1c7d3f314ae733d869d95209"),
        ] {
            let hash = empty_hash_at_depth(depth);
            assert_eq!(hash.to_hex(), s, "invalid hash at depth={:?}", depth);
        }
    }

    #[test]
    fn test_root_hash() {
        for depth in 0..5 {
            let hash = empty_hash_at_depth(depth);
            println!("depth={:?} HASH={:?}", depth, hash.to_string());
        }

        println!("DONE\n\n");

        let mut db = Database::create(4);

        for _ in 0..1 {
            db.create_account((), Account::empty()).unwrap();
        }

        // assert!(db.create_account((), Account::empty()).is_err());

        let hash = db.root_hash().unwrap();
        println!("ROOT_HASH_4={:?}", hash.to_string());

        // let mut db = Database::create(30);
        // db.create_account((), Account::empty()).unwrap();
        // let hash = db.root_hash().unwrap();
        // println!("ROOT_HASH_30={:?}", hash.to_string());
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

        // let fp = [true,true,true,false,true,true,true,false,false,false,true,false,false,true,false,true,false,false,false,false,true,true,false,false,false,false,false,false,false,true,false,true,false,true,true,true,true,true,false,false,false,true,false,true,true,true,true,true,true,false,true,true,true,true,true,true,false,false,true,false,true,false,false,false,true,true,true,false,true,false,true,true,false,true,true,true,false,true,false,false,false,true,false,true,true,false,false,false,false,false,true,true,true,true,false,false,true,false,true,false,false,false,true,false,true,false,false,false,false,true,false,true,true,false,true,true,true,true,true,true,false,true,false,true,false,false,true,true,false,true,false,false,false,false,false,true,true,true,true,false,false,true,true,false,false,false,true,true,true,true,true,true,false,true,true,true,false,false,true,true,false,true,false,true,true,false,false,true,false,true,false,false,true,true,true,false,false,true,false,true,true,false,false,true,true,false,true,true,true,false,false,false,true,false,true,false,false,true,false,false,false,true,false,false,true,false,false,false,true,true,false,true,false,false,false,true,true,false,true,false,true,true,false,true,false,true,true,true,true,false,true,true,false,true,true,false,false,false,true,true,true,false,true,true,true,false,false,true,true,true,true,true,true,true,false];

        // let fp = Fp::from_bits(&fp).unwrap();
        // println!("FP={:?}", fp.to_hex());

        // let fp = Fp::from_hex(hex)

        // let bytes: Vec<u8> = hex::decode("2033430829EDB02326F649A770FCFB20526FD35F80274752E4C2D91EC5B7E49B").unwrap();
        // println!("STR  ={:?}", "2033430829EDB02326F649A770FCFB20526FD35F80274752E4C2D91EC5B7E49B");
        // println!("LEN={:?} BYTES={:?}", bytes.len(), bytes);

        // let bs = bs58::decode("2n1hGCgg3jCKQJzVBgfujGqyV6D9riKgq27zhXqYgTRVZM5kqfkm").into_vec().unwrap();
        // let bsd = Fp::from_bytes(&bs);
        // println!("BYTES={:?} BS={:?}", bs, bsd);

        // let base = BaseField::from_bytes(&bytes);
        // println!("BASE={:?}", base);

        // use ark_serialize::CanonicalDeserialize;

        // // let fp = Fp::from_hex(&"2033430829EDB02326F649A770FCFB20526FD35F80274752E4C2D91EC5B7E49B".to_lowercase()).unwrap();
        // let fp = Fp::deserialize_uncompressed(&mut &*bytes);

        // println!("FP={:?}", fp);

        // let array = [true,true,false,true,true,false,false,true,false,false,true,false,false,true,true,true,true,true,true,false,true,true,false,true,true,false,true,false,false,false,true,true,false,true,true,true,true,false,false,false,true,false,false,true,true,false,true,true,false,true,false,false,false,false,true,true,false,false,true,false,false,true,true,true,false,true,false,false,true,false,true,false,true,true,true,false,false,false,true,false,true,true,true,false,false,true,false,false,false,false,false,false,false,false,false,true,true,true,true,true,true,false,true,false,true,true,false,false,true,false,true,true,true,true,true,true,false,true,true,false,false,true,false,false,true,false,true,false,false,false,false,false,false,true,false,false,true,true,false,true,true,true,true,true,false,false,true,true,true,true,true,true,false,false,false,false,true,true,true,false,true,true,true,false,false,true,false,true,true,false,false,true,false,false,true,false,false,true,true,false,true,true,true,true,false,true,true,false,false,true,false,false,true,true,false,false,false,true,false,false,false,false,false,false,true,true,false,true,true,false,true,true,false,true,true,true,true,false,false,true,false,true,false,false,false,false,false,true,false,false,false,false,true,true,false,false,false,false,true,false,true,true,false,false,true,true,false,false,false,false,false,false,false,true,false];

        // let bytes = array
        //     .iter()
        //     .enumerate()
        //     .fold(Fp::zero().to_bytes(), |mut bytes, (i, bit)| {
        //         bytes[i / 8] |= (*bit as u8) << (i % 8);
        //         bytes
        //     });

        // println!("LEN={:?} BYTES={:?}", bytes.len(), bytes);

        // let fp = Fp::from_bits(&array).unwrap();
        // println!("FP_BITS={:?}", fp);

        // let fp = Fp::from_bytes(&bytes).unwrap();
        // println!("FP_BYTES={:?}", fp);

        // let hex = hex::encode(&bytes);
        // println!("HEX={:?}", hex);
        // let fp = Fp::from_hex(&hex).unwrap();
        // println!("FP_HEX={:?}", fp);

        // let empty = "9be4b7c51ed9c2e4524727805fd36f5220fbfc70a749f62623b0ed2908433320";
        // let fp = Fp::from_hex(&hex).unwrap();
        // println!("FP_HEX={:?}", fp);

        // empty_receipt_hash();

        // let prefix = "CodaReceiptEmpty";
        // const MAX_DOMAIN_STRING_LEN: usize = 20;
        // assert!(prefix.len() <= MAX_DOMAIN_STRING_LEN);
        // let prefix = &prefix[..std::cmp::min(prefix.len(), MAX_DOMAIN_STRING_LEN)];
        // let bytes = format!("{:*<MAX_DOMAIN_STRING_LEN$}", prefix);
        // println!("LA={:?}", bytes)

        // /// Transform domain prefix string to field element
        // fn domain_prefix_to_field<F: PrimeField>(prefix: String) -> F {
        //     const MAX_DOMAIN_STRING_LEN: usize = 20;
        //     assert!(prefix.len() <= MAX_DOMAIN_STRING_LEN);
        //     let prefix = &prefix[..std::cmp::min(prefix.len(), MAX_DOMAIN_STRING_LEN)];
        //     let mut bytes = format!("{:*<MAX_DOMAIN_STRING_LEN$}", prefix)
        //         .as_bytes()
        //         .to_vec();
        //     bytes.resize(F::size_in_bytes(), 0);
        //     F::from_bytes(&bytes).expect("invalid domain bytes")
        // }
    }
}
