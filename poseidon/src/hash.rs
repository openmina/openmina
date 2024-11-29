use ark_ff::{BigInteger256, Field, FromBytes as _};
use mina_curves::pasta::Fp;

use crate::{PlonkSpongeConstantsKimchi, Sponge, SpongeParamsForField};

enum Item {
    Bool(bool),
    U2(u8),
    U8(u8),
    U32(u32),
    U48([u8; 6]),
    U64(u64),
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(arg0) => f.write_fmt(format_args!("{}_bool", i32::from(*arg0))),
            Self::U2(arg0) => f.write_fmt(format_args!("{}_u2", arg0)),
            Self::U8(arg0) => f.write_fmt(format_args!("{}_u8", arg0)),
            Self::U32(arg0) => f.write_fmt(format_args!("{}_u32", arg0)),
            Self::U48(arg0) => f.write_fmt(format_args!("{:?}_u48", arg0)),
            Self::U64(arg0) => f.write_fmt(format_args!("{}_u64", arg0)),
        }
    }
}

impl Item {
    fn nbits(&self) -> u32 {
        match self {
            Item::Bool(_) => 1,
            Item::U2(_) => 2,
            Item::U8(_) => 8,
            Item::U32(_) => 32,
            Item::U48(_) => 48,
            Item::U64(_) => 64,
        }
    }

    fn as_bigint(&self) -> u64 {
        match self {
            Item::Bool(v) => *v as u64,
            Item::U2(v) => *v as u64,
            Item::U8(v) => *v as u64,
            Item::U32(v) => *v as u64,
            Item::U48(v) => {
                let mut bytes = <[u8; 32]>::default();
                bytes[..6].copy_from_slice(&v[..]);
                BigInteger256::read(&bytes[..]).unwrap().to_64x4()[0] // Never fail with only 6 bytes
            }
            Item::U64(v) => *v,
        }
    }
}

pub struct Inputs {
    fields: Vec<Fp>,
    packeds: Vec<Item>,
}

impl Default for Inputs {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Inputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inputs")
            .field(&format!("fields[{:?}]", self.fields.len()), &self.fields)
            .field(&format!("packeds[{:?}]", self.packeds.len()), &self.packeds)
            .finish()
    }
}

#[allow(clippy::needless_range_loop)]
fn shl(bigint: &mut [u64; 4], mut n: u32) {
    if n >= 64 * 4 {
        *bigint = [0, 0, 0, 0];
        return;
    }
    while n >= 64 {
        let mut t = 0;
        for i in 0..4 {
            core::mem::swap(&mut t, &mut bigint[i]);
        }
        n -= 64;
    }
    if n > 0 {
        let mut t = 0;
        for i in 0..4 {
            let a = &mut bigint[i];
            let t2 = *a >> (64 - n);
            *a <<= n;
            *a |= t;
            t = t2;
        }
    }
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            fields: Vec::with_capacity(256),
            packeds: Vec::with_capacity(256),
        }
    }

    pub fn append_bool(&mut self, value: bool) {
        self.packeds.push(Item::Bool(value));
    }

    pub fn append_u2(&mut self, value: u8) {
        self.packeds.push(Item::U2(value));
    }

    pub fn append_u8(&mut self, value: u8) {
        self.packeds.push(Item::U8(value));
    }

    pub fn append_u32(&mut self, value: u32) {
        self.packeds.push(Item::U32(value));
    }

    pub fn append_u64(&mut self, value: u64) {
        self.packeds.push(Item::U64(value));
    }

    pub fn append_u48(&mut self, value: [u8; 6]) {
        self.packeds.push(Item::U48(value));
    }

    pub fn append_field(&mut self, value: Fp) {
        self.fields.push(value);
    }

    pub fn append_bytes(&mut self, value: &[u8]) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

        self.packeds.reserve(value.len() * 8);

        for byte in value {
            for bit in BITS {
                self.append_bool(byte & bit != 0);
            }
        }
    }

    // pub fn append<T>(&mut self, value: &T)
    // where
    //     T: ToInputs,
    // {
    //     value.to_inputs(self);
    // }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_fields(mut self) -> Vec<Fp> {
        let mut nbits = 0;
        let mut current: [u64; 4] = [0, 0, 0, 0];

        for (item, item_nbits) in self.packeds.iter().map(|i| (i.as_bigint(), i.nbits())) {
            nbits += item_nbits;

            if nbits < 255 {
                shl(&mut current, item_nbits);

                // Addition, but we use 'bitwise or' because we know bits of
                // `current` are zero (we just shift-left them)
                current[0] |= item;
            } else {
                self.fields
                    .push(BigInteger256::from_64x4(current).try_into().unwrap()); // Never fail
                current = [item, 0, 0, 0];
                nbits = item_nbits;
            }
        }

        if nbits > 0 {
            self.fields
                .push(BigInteger256::from_64x4(current).try_into().unwrap()); // Never fail
        }

        self.fields
    }
}

fn param_to_field_impl(param: &str, default: &[u8; 32]) -> Fp {
    let param_bytes = param.as_bytes();
    let len = param_bytes.len();

    let mut fp = *default;
    fp[..len].copy_from_slice(param_bytes);

    Fp::read(&fp[..]).expect("fp read failed")
}

pub fn param_to_field(param: &str) -> Fp {
    const DEFAULT: &[u8; 32] = b"********************\0\0\0\0\0\0\0\0\0\0\0\0";

    if param.len() > 20 {
        panic!("must be 20 byte maximum");
    }

    param_to_field_impl(param, DEFAULT)
}

fn param_to_field_noinputs(param: &str) -> Fp {
    const DEFAULT: &[u8; 32] = &[0; 32];

    if param.len() > 32 {
        panic!("must be 32 byte maximum");
    }

    param_to_field_impl(param, DEFAULT)
}

pub fn hash_with_kimchi(param: &LazyParam, fields: &[Fp]) -> Fp {
    let LazyParam {
        sponge_state,
        state,
        ..
    } = param;

    let mut sponge = Sponge {
        sponge_state: sponge_state.clone(),
        state: *state,
        ..Sponge::<Fp, PlonkSpongeConstantsKimchi>::default()
    };

    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_fields<F: Field + SpongeParamsForField<F>>(fields: &[F]) -> F {
    let mut sponge = Sponge::<F>::default();

    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_noinputs(param: &LazyParam) -> Fp {
    let LazyParam { last_squeezed, .. } = param;

    *last_squeezed
}

#[derive(Debug)]
#[allow(dead_code)] // `string` is never read
pub struct LazyParam {
    sponge_state: crate::SpongeState,
    state: [Fp; 3],
    last_squeezed: Fp,
    string: &'static str,
}

impl LazyParam {
    pub fn state(&self) -> [Fp; 3] {
        self.state
    }
}

pub mod params {
    use once_cell::sync::Lazy;

    use super::*;

    macro_rules! impl_params {
        ($({$name:tt, $string:tt}),*) => ($(
            pub static $name: Lazy<Box<LazyParam>> = Lazy::new(|| {
                let mut sponge = Sponge::<Fp>::default();
                sponge.absorb(&[param_to_field($string)]);
                let last_squeezed = sponge.squeeze();
                Box::new(LazyParam {
                    sponge_state: sponge.sponge_state,
                    state: sponge.state,
                    last_squeezed,
                    string: $string,
                })
            });
        )*)
    }

    impl_params!(
        {MINA_ACCOUNT, "MinaAccount"},
        {MINA_PROTO_STATE, "MinaProtoState"},
        {MINA_PROTO_STATE_BODY, "MinaProtoStateBody"},
        {MINA_DERIVE_TOKEN_ID, "MinaDeriveTokenId"},
        {MINA_EPOCH_SEED, "MinaEpochSeed"},
        {MINA_SIDELOADED_VK, "MinaSideLoadedVk"},
        {MINA_VRF_MESSAGE, "MinaVrfMessage"},
        {MINA_VRF_OUTPUT, "MinaVrfOutput"},

        {CODA_RECEIPT_UC, "CodaReceiptUC"},
        {COINBASE_STACK, "CoinbaseStack"},

        {MINA_ACCOUNT_UPDATE_CONS, "MinaAcctUpdateCons"},
        {MINA_ACCOUNT_UPDATE_NODE, "MinaAcctUpdateNode"},
        {MINA_ACCOUNT_UPDATE_STACK_FRAME, "MinaAcctUpdStckFrm"},
        {MINA_ACCOUNT_UPDATE_STACK_FRAME_CONS, "MinaActUpStckFrmCons"},

        {MINA_ZKAPP_ACCOUNT, "MinaZkappAccount"},
        {MINA_ZKAPP_MEMO, "MinaZkappMemo"},
        {MINA_ZKAPP_URI, "MinaZkappUri"},
        {MINA_ZKAPP_EVENT, "MinaZkappEvent"},
        {MINA_ZKAPP_EVENTS, "MinaZkappEvents"},
        {MINA_ZKAPP_SEQ_EVENTS, "MinaZkappSeqEvents"},

        // devnet
        {CODA_SIGNATURE, "CodaSignature"},
        {TESTNET_ZKAPP_BODY, "TestnetZkappBody"},
        // mainnet
        {MINA_SIGNATURE_MAINNET, "MinaSignatureMainnet"},
        {MAINNET_ZKAPP_BODY, "MainnetZkappBody"},

        {MINA_MERKLE_TREE_0, "MinaMklTree000"},
        {MINA_MERKLE_TREE_1, "MinaMklTree001"},
        {MINA_MERKLE_TREE_2, "MinaMklTree002"},
        {MINA_MERKLE_TREE_3, "MinaMklTree003"},
        {MINA_MERKLE_TREE_4, "MinaMklTree004"},
        {MINA_MERKLE_TREE_5, "MinaMklTree005"},
        {MINA_MERKLE_TREE_6, "MinaMklTree006"},
        {MINA_MERKLE_TREE_7, "MinaMklTree007"},
        {MINA_MERKLE_TREE_8, "MinaMklTree008"},
        {MINA_MERKLE_TREE_9, "MinaMklTree009"},
        {MINA_MERKLE_TREE_10, "MinaMklTree010"},
        {MINA_MERKLE_TREE_11, "MinaMklTree011"},
        {MINA_MERKLE_TREE_12, "MinaMklTree012"},
        {MINA_MERKLE_TREE_13, "MinaMklTree013"},
        {MINA_MERKLE_TREE_14, "MinaMklTree014"},
        {MINA_MERKLE_TREE_15, "MinaMklTree015"},
        {MINA_MERKLE_TREE_16, "MinaMklTree016"},
        {MINA_MERKLE_TREE_17, "MinaMklTree017"},
        {MINA_MERKLE_TREE_18, "MinaMklTree018"},
        {MINA_MERKLE_TREE_19, "MinaMklTree019"},
        {MINA_MERKLE_TREE_20, "MinaMklTree020"},
        {MINA_MERKLE_TREE_21, "MinaMklTree021"},
        {MINA_MERKLE_TREE_22, "MinaMklTree022"},
        {MINA_MERKLE_TREE_23, "MinaMklTree023"},
        {MINA_MERKLE_TREE_24, "MinaMklTree024"},
        {MINA_MERKLE_TREE_25, "MinaMklTree025"},
        {MINA_MERKLE_TREE_26, "MinaMklTree026"},
        {MINA_MERKLE_TREE_27, "MinaMklTree027"},
        {MINA_MERKLE_TREE_28, "MinaMklTree028"},
        {MINA_MERKLE_TREE_29, "MinaMklTree029"},
        {MINA_MERKLE_TREE_30, "MinaMklTree030"},
        {MINA_MERKLE_TREE_31, "MinaMklTree031"},
        {MINA_MERKLE_TREE_32, "MinaMklTree032"},
        {MINA_MERKLE_TREE_33, "MinaMklTree033"},
        {MINA_MERKLE_TREE_34, "MinaMklTree034"},
        {MINA_MERKLE_TREE_35, "MinaMklTree035"},

        {MINA_CB_MERKLE_TREE_0, "MinaCbMklTree000"},
        {MINA_CB_MERKLE_TREE_1, "MinaCbMklTree001"},
        {MINA_CB_MERKLE_TREE_2, "MinaCbMklTree002"},
        {MINA_CB_MERKLE_TREE_3, "MinaCbMklTree003"},
        {MINA_CB_MERKLE_TREE_4, "MinaCbMklTree004"},
        {MINA_CB_MERKLE_TREE_5, "MinaCbMklTree005"}
    );

    pub fn get_coinbase_param_for_height(height: usize) -> &'static LazyParam {
        static ARRAY: [&Lazy<Box<LazyParam>>; 6] = [
            &MINA_CB_MERKLE_TREE_0,
            &MINA_CB_MERKLE_TREE_1,
            &MINA_CB_MERKLE_TREE_2,
            &MINA_CB_MERKLE_TREE_3,
            &MINA_CB_MERKLE_TREE_4,
            &MINA_CB_MERKLE_TREE_5,
        ];

        ARRAY[height]
    }

    pub fn get_merkle_param_for_height(height: usize) -> &'static LazyParam {
        static ARRAY: [&Lazy<Box<LazyParam>>; 36] = [
            &MINA_MERKLE_TREE_0,
            &MINA_MERKLE_TREE_1,
            &MINA_MERKLE_TREE_2,
            &MINA_MERKLE_TREE_3,
            &MINA_MERKLE_TREE_4,
            &MINA_MERKLE_TREE_5,
            &MINA_MERKLE_TREE_6,
            &MINA_MERKLE_TREE_7,
            &MINA_MERKLE_TREE_8,
            &MINA_MERKLE_TREE_9,
            &MINA_MERKLE_TREE_10,
            &MINA_MERKLE_TREE_11,
            &MINA_MERKLE_TREE_12,
            &MINA_MERKLE_TREE_13,
            &MINA_MERKLE_TREE_14,
            &MINA_MERKLE_TREE_15,
            &MINA_MERKLE_TREE_16,
            &MINA_MERKLE_TREE_17,
            &MINA_MERKLE_TREE_18,
            &MINA_MERKLE_TREE_19,
            &MINA_MERKLE_TREE_20,
            &MINA_MERKLE_TREE_21,
            &MINA_MERKLE_TREE_22,
            &MINA_MERKLE_TREE_23,
            &MINA_MERKLE_TREE_24,
            &MINA_MERKLE_TREE_25,
            &MINA_MERKLE_TREE_26,
            &MINA_MERKLE_TREE_27,
            &MINA_MERKLE_TREE_28,
            &MINA_MERKLE_TREE_29,
            &MINA_MERKLE_TREE_30,
            &MINA_MERKLE_TREE_31,
            &MINA_MERKLE_TREE_32,
            &MINA_MERKLE_TREE_33,
            &MINA_MERKLE_TREE_34,
            &MINA_MERKLE_TREE_35,
        ];

        ARRAY[height]
    }

    macro_rules! impl_params_noinput {
        ($({$name:tt, $string:tt}),*) => ($(
            pub static $name: Lazy<Box<LazyParam>> = Lazy::new(|| {
                let mut sponge = Sponge::<Fp>::default();
                sponge.absorb(&[param_to_field_noinputs($string)]);
                let last_squeezed = sponge.squeeze();
                Box::new(LazyParam {
                    sponge_state: sponge.sponge_state,
                    state: sponge.state,
                    last_squeezed,
                    string: $string,
                })
            });
        )*)
    }

    impl_params_noinput!(
        {NO_INPUT_ZKAPP_ACTION_STATE_EMPTY_ELT, "MinaZkappActionStateEmptyElt"},
        {NO_INPUT_COINBASE_STACK, "CoinbaseStack"},
        {NO_INPUT_MINA_ZKAPP_EVENTS_EMPTY, "MinaZkappEventsEmpty"},
        {NO_INPUT_MINA_ZKAPP_ACTIONS_EMPTY, "MinaZkappActionsEmpty"}
    );
}

pub mod legacy {
    use ark_ff::fields::arithmetic::InvalidBigInt;

    use super::*;

    #[derive(Clone, Debug)]
    pub struct Inputs<F: Field> {
        fields: Vec<F>,
        bits: Vec<bool>,
    }

    impl<F: Field> Default for Inputs<F> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<F: Field> Inputs<F> {
        pub fn new() -> Self {
            Self {
                fields: Vec::with_capacity(256),
                bits: Vec::with_capacity(512),
            }
        }

        pub fn append_bit(&mut self, bit: bool) {
            self.bits.push(bit);
        }

        pub fn append_bool(&mut self, value: bool) {
            self.append_bit(value);
        }

        pub fn append_bits(&mut self, bits: &[bool]) {
            self.bits.extend(bits);
        }

        pub fn append_bytes(&mut self, bytes: &[u8]) {
            const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

            self.bits.reserve(bytes.len() * 8);

            for byte in bytes {
                for bit in BITS {
                    self.append_bit(byte & bit != 0);
                }
            }
        }

        pub fn append_u64(&mut self, value: u64) {
            self.append_bytes(&value.to_le_bytes());
        }

        pub fn append_u32(&mut self, value: u32) {
            self.append_bytes(&value.to_le_bytes());
        }

        pub fn append_field(&mut self, field: F) {
            self.fields.push(field);
        }
    }

    impl<F: Field + TryFrom<BigInteger256, Error = InvalidBigInt>> Inputs<F> {
        pub fn to_fields(mut self) -> Vec<F> {
            const NBITS: usize = 255 - 1;

            self.fields.reserve(self.bits.len() / NBITS);
            self.fields.extend(self.bits.chunks(NBITS).map(|bits| {
                let mut field = [0u64; 4];
                for (index, bit) in bits.iter().enumerate() {
                    let limb_index = index / 64;
                    let bit_index = index % 64;
                    field[limb_index] |= (*bit as u64) << bit_index;
                }
                F::try_from(BigInteger256::from_64x4(field)).unwrap() // Never fail
            }));
            self.fields
        }
    }

    pub fn hash_with_kimchi(param: &LazyParam, fields: &[Fp]) -> Fp {
        let LazyParam {
            sponge_state,
            state,
            ..
        } = param;

        let mut sponge = Sponge {
            sponge_state: sponge_state.clone(),
            state: *state,
            ..Sponge::new_legacy()
        };

        sponge.absorb(fields);
        sponge.squeeze()
    }

    pub mod params {
        use once_cell::sync::Lazy;

        use super::*;

        macro_rules! impl_params {
            ($({$name:tt, $string:tt}),*) => ($(
                pub static $name: Lazy<Box<LazyParam>> = Lazy::new(|| {
                    let mut sponge = Sponge::new_legacy();
                    sponge.absorb(&[param_to_field($string)]);
                    let last_squeezed = sponge.squeeze();
                    Box::new(LazyParam {
                        sponge_state: sponge.sponge_state,
                        state: sponge.state,
                        last_squeezed,
                        string: $string,
                    })
                });
            )*)
        }

        impl_params!(
            {CODA_RECEIPT_UC, "CodaReceiptUC"},

            // devnet
            {CODA_SIGNATURE, "CodaSignature"},
            {TESTNET_ZKAPP_BODY, "TestnetZkappBody"},
            // mainnet
            {MINA_SIGNATURE_MAINNET, "MinaSignatureMainnet"},
            {MAINNET_ZKAPP_BODY, "MainnetZkappBody"}
        );
    }
}
