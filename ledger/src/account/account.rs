use std::{fmt::Write, io::Cursor, str::FromStr};

use ark_ff::{BigInteger256, One, UniformRand, Zero};
use mina_hasher::Fp;
use mina_p2p_messages::binprot::{BinProtRead, BinProtWrite};
use mina_signer::CompressedPubKey;
use rand::{prelude::ThreadRng, seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    gen_compressed,
    hash::{hash_noinputs, hash_with_kimchi, Inputs},
    proofs::{
        field::{Boolean, FieldWitness, ToBoolean},
        numbers::{
            currency::{CheckedBalance, CheckedCurrency},
            nat::CheckedSlot,
        },
        to_field_elements::ToFieldElements,
        transaction::{
            make_group, transaction_snark::checked_min_balance_at_slot, Check, InnerCurve,
            PlonkVerificationKeyEvals,
        },
        witness::Witness,
    },
    scan_state::{
        currency::{Balance, Magnitude, Nonce, Slot, TxnVersion},
        transaction_logic::account_min_balance_at_slot,
    },
    zkapps::snark::FlaggedOption,
    MerklePath, MyCow, ToInputs,
};

use super::common::*;

/// Mina_numbers.Txn_version.current
pub const TXN_VERSION_CURRENT: TxnVersion = TxnVersion::from_u32(3);

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenId(pub Fp);

impl std::fmt::Debug for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::FpExt;
        f.write_fmt(format_args!("TokenId({})", self.0.to_decimal()))
    }
}

impl Default for TokenId {
    fn default() -> Self {
        Self(Fp::one())
    }
}

impl From<u64> for TokenId {
    fn from(num: u64) -> Self {
        TokenId(Fp::from(num))
    }
}

impl TokenId {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account.ml#L93
#[derive(Clone, Debug, PartialEq, Eq, derive_more::Deref, derive_more::From)]
pub struct TokenSymbol(pub String);

impl TokenSymbol {
    pub fn gen() -> Self {
        let mut rng = rand::thread_rng();

        let sym: u32 = rng.gen();
        let mut sym = sym.to_string();
        sym.truncate(6);

        Self(sym)
    }

    pub fn to_bytes(&self, bytes: &mut [u8]) {
        if self.is_empty() {
            return;
        }
        let len = self.len();
        let s = self.as_bytes();
        bytes[..len].copy_from_slice(&s[..len.min(6)]);
    }

    pub fn to_field<F: FieldWitness>(&self) -> F {
        use ark_ff::FromBytes;

        let mut s = <[u8; 32]>::default();
        self.to_bytes(&mut s);

        let bigint = BigInteger256::read(&s[..]).unwrap();
        F::from(bigint)
    }
}

#[allow(clippy::derivable_impls)]
impl Default for TokenSymbol {
    fn default() -> Self {
        // empty string
        // https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account.ml#L133
        Self(String::new())
    }
}

impl TryFrom<&mina_p2p_messages::string::ByteString> for TokenSymbol {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: &mina_p2p_messages::string::ByteString) -> Result<Self, Self::Error> {
        Ok(Self(value.clone().try_into()?))
    }
}

impl From<&TokenSymbol> for mina_p2p_messages::string::ByteString {
    fn from(value: &TokenSymbol) -> Self {
        value.0.as_bytes().into()
    }
}

impl ToInputs for TokenSymbol {
    fn to_inputs(&self, inputs: &mut Inputs) {
        // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
        //assert!(self.len() <= 6);

        let mut s = <[u8; 6]>::default();
        self.to_bytes(&mut s);
        inputs.append_u48(s);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetVerificationKey<Controller> {
    pub auth: Controller,
    pub txn_version: TxnVersion,
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L49
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permissions<Controller> {
    pub edit_state: Controller,
    pub access: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: SetVerificationKey<Controller>,
    pub set_zkapp_uri: Controller,
    pub edit_action_state: Controller,
    pub set_token_symbol: Controller,
    pub increment_nonce: Controller,
    pub set_voting_for: Controller,
    pub set_timing: Controller,
}

pub enum AuthOrVersion<'a, T> {
    Auth(&'a T),
    Version(TxnVersion),
}

impl Permissions<AuthRequired> {
    pub fn iter_as_bits<F>(&self, mut fun: F)
    where
        F: FnMut(AuthOrVersion<'_, bool>),
    {
        let Self {
            edit_state,
            access,
            send,
            receive,
            set_delegate,
            set_permissions,
            set_verification_key:
                SetVerificationKey {
                    auth: set_verification_key_auth,
                    txn_version,
                },
            set_zkapp_uri,
            edit_action_state,
            set_token_symbol,
            increment_nonce,
            set_voting_for,
            set_timing,
        } = self;

        for auth in [
            AuthOrVersion::Auth(edit_state),
            AuthOrVersion::Auth(access),
            AuthOrVersion::Auth(send),
            AuthOrVersion::Auth(receive),
            AuthOrVersion::Auth(set_delegate),
            AuthOrVersion::Auth(set_permissions),
            AuthOrVersion::Auth(set_verification_key_auth),
            AuthOrVersion::Version(*txn_version),
            AuthOrVersion::Auth(set_zkapp_uri),
            AuthOrVersion::Auth(edit_action_state),
            AuthOrVersion::Auth(set_token_symbol),
            AuthOrVersion::Auth(increment_nonce),
            AuthOrVersion::Auth(set_voting_for),
            AuthOrVersion::Auth(set_timing),
        ] {
            match auth {
                AuthOrVersion::Auth(auth) => {
                    for bit in auth.encode().to_bits() {
                        fun(AuthOrVersion::Auth(&bit));
                    }
                }
                AuthOrVersion::Version(version) => {
                    fun(AuthOrVersion::Version(version));
                }
            }
        }
    }
}

impl ToInputs for Permissions<AuthRequired> {
    fn to_inputs(&self, inputs: &mut Inputs) {
        self.iter_as_bits(|bit| match bit {
            AuthOrVersion::Auth(bit) => inputs.append_bool(*bit),
            AuthOrVersion::Version(version) => inputs.append(&version),
        });
    }
}

impl<F: FieldWitness> Check<F> for Permissions<AuthRequired> {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
            edit_state: _,
            access: _,
            send: _,
            receive: _,
            set_delegate: _,
            set_permissions: _,
            set_verification_key:
                SetVerificationKey {
                    auth: _,
                    txn_version,
                },
            set_zkapp_uri: _,
            edit_action_state: _,
            set_token_symbol: _,
            increment_nonce: _,
            set_voting_for: _,
            set_timing: _,
        } = self;

        txn_version.check(w);
    }
}

impl Default for Permissions<AuthRequired> {
    fn default() -> Self {
        Self::user_default()
    }
}

impl Permissions<AuthRequired> {
    pub fn user_default() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: Signature,
            send: Signature,
            receive: None,
            set_delegate: Signature,
            set_permissions: Signature,
            set_verification_key: SetVerificationKey {
                auth: Signature,
                txn_version: TXN_VERSION_CURRENT,
            },
            set_zkapp_uri: Signature,
            edit_action_state: Signature,
            set_token_symbol: Signature,
            increment_nonce: Signature,
            set_voting_for: Signature,
            set_timing: Signature,
            access: None,
        }
    }

    pub fn empty() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: None,
            send: None,
            receive: None,
            access: None,
            set_delegate: None,
            set_permissions: None,
            set_verification_key: SetVerificationKey {
                auth: None,
                txn_version: TXN_VERSION_CURRENT,
            },
            set_zkapp_uri: None,
            edit_action_state: None,
            set_token_symbol: None,
            increment_nonce: None,
            set_voting_for: None,
            set_timing: None,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/permissions.ml#L385
    pub fn gen(auth_tag: ControlTag) -> Self {
        let mut rng = rand::thread_rng();

        let auth_required_gen = match auth_tag {
            ControlTag::Proof => AuthRequired::gen_for_proof_authorization,
            ControlTag::Signature => AuthRequired::gen_for_signature_authorization,
            ControlTag::NoneGiven => AuthRequired::gen_for_none_given_authorization,
        };

        Self {
            edit_state: auth_required_gen(&mut rng),
            send: auth_required_gen(&mut rng),
            receive: auth_required_gen(&mut rng),
            set_delegate: auth_required_gen(&mut rng),
            set_permissions: auth_required_gen(&mut rng),
            set_verification_key: SetVerificationKey {
                auth: auth_required_gen(&mut rng),
                txn_version: TXN_VERSION_CURRENT,
            },
            set_zkapp_uri: auth_required_gen(&mut rng),
            edit_action_state: auth_required_gen(&mut rng),
            set_token_symbol: auth_required_gen(&mut rng),
            increment_nonce: auth_required_gen(&mut rng),
            set_voting_for: auth_required_gen(&mut rng),
            set_timing: auth_required_gen(&mut rng),
            access: {
                // Access permission is significantly more restrictive, do not arbitrarily
                // set it when tests may not be intending to exercise it.
                AuthRequired::gen_for_none_given_authorization(&mut rng)
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofVerified {
    N0,
    N1,
    N2,
}

impl ProofVerified {
    /// https://github.com/MinaProtocol/mina/blob/47a269c2e917775b34a83775b8a55fcc44830831/src/lib/pickles_base/proofs_verified.ml#L17
    pub fn to_int(&self) -> usize {
        match self {
            ProofVerified::N0 => 0,
            ProofVerified::N1 => 1,
            ProofVerified::N2 => 2,
        }
    }
}

impl ToInputs for ProofVerified {
    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/pickles_base/proofs_verified.ml#L125
    fn to_inputs(&self, inputs: &mut Inputs) {
        let bits = match self {
            ProofVerified::N0 => [true, false, false],
            ProofVerified::N1 => [false, true, false],
            ProofVerified::N2 => [false, false, true],
        };

        for bit in bits {
            inputs.append_bool(bit);
        }
    }
}

// One_hot
impl ToFieldElements<Fp> for ProofVerified {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        use Boolean::{False, True};

        let bits = match self {
            ProofVerified::N0 => [True, False, False],
            ProofVerified::N1 => [False, True, False],
            ProofVerified::N2 => [False, False, True],
        };

        bits.to_field_elements(fields);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationKey {
    pub max_proofs_verified: ProofVerified,
    pub actual_wrap_domain_size: ProofVerified,
    pub wrap_index: PlonkVerificationKeyEvals<Fp>,
    // `wrap_vk` is not used for hash inputs
    pub wrap_vk: Option<()>,
}

impl Check<Fp> for VerificationKey {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            max_proofs_verified: _,
            actual_wrap_domain_size: _,
            wrap_index,
            wrap_vk: _,
        } = self;

        wrap_index.check(w);
    }
}

impl ToFieldElements<Fp> for VerificationKey {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            max_proofs_verified,
            actual_wrap_domain_size,
            wrap_index:
                PlonkVerificationKeyEvals {
                    sigma,
                    coefficients,
                    generic,
                    psm,
                    complete_add,
                    mul,
                    emul,
                    endomul_scalar,
                },
            wrap_vk: _,
        } = self;

        max_proofs_verified.to_field_elements(fields);
        actual_wrap_domain_size.to_field_elements(fields);

        sigma.to_field_elements(fields);
        coefficients.to_field_elements(fields);
        generic.to_field_elements(fields);
        psm.to_field_elements(fields);
        complete_add.to_field_elements(fields);
        mul.to_field_elements(fields);
        emul.to_field_elements(fields);
        endomul_scalar.to_field_elements(fields);
    }
}

impl ToInputs for VerificationKey {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            max_proofs_verified,
            actual_wrap_domain_size,
            wrap_index:
                PlonkVerificationKeyEvals {
                    sigma,
                    coefficients,
                    generic,
                    psm,
                    complete_add,
                    mul,
                    emul,
                    endomul_scalar,
                },
            wrap_vk: _,
        } = self;

        inputs.append(max_proofs_verified);
        inputs.append(actual_wrap_domain_size);

        for sigma in sigma {
            inputs.append(sigma);
        }
        for coefficients in coefficients {
            inputs.append(coefficients);
        }
        inputs.append(generic);
        inputs.append(psm);
        inputs.append(complete_add);
        inputs.append(mul);
        inputs.append(emul);
        inputs.append(endomul_scalar);
    }
}

impl VerificationKey {
    pub const HASH_PARAM: &'static str = "MinaSideLoadedVk";

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/pickles/side_loaded_verification_key.ml#L310
    pub fn dummy() -> Self {
        let g = InnerCurve::of_affine(make_group(
            Fp::one(),
            Fp::from_str(
                "12418654782883325593414442427049395787963493412651469444558597405572177144507",
            )
            .unwrap(),
        ));
        Self {
            max_proofs_verified: ProofVerified::N2,
            actual_wrap_domain_size: ProofVerified::N2,
            wrap_index: PlonkVerificationKeyEvals {
                sigma: std::array::from_fn(|_| g.clone()),
                coefficients: std::array::from_fn(|_| g.clone()),
                generic: g.clone(),
                psm: g.clone(),
                complete_add: g.clone(),
                mul: g.clone(),
                emul: g.clone(),
                endomul_scalar: g,
            },
            wrap_vk: None,
        }
    }

    pub fn digest(&self) -> Fp {
        self.hash()
    }

    pub fn hash(&self) -> Fp {
        self.hash_with_param(Self::HASH_PARAM)
    }

    pub fn gen() -> Self {
        let mut rng = rand::thread_rng();

        VerificationKey {
            max_proofs_verified: {
                let n: u64 = rng.gen();

                if n % 3 == 0 {
                    ProofVerified::N2
                } else if n % 2 == 0 {
                    ProofVerified::N1
                } else {
                    ProofVerified::N0
                }
            },
            wrap_index: PlonkVerificationKeyEvals::rand(),
            wrap_vk: None,
            actual_wrap_domain_size: {
                let n: u64 = rng.gen();

                if n % 3 == 0 {
                    ProofVerified::N2
                } else if n % 2 == 0 {
                    ProofVerified::N1
                } else {
                    ProofVerified::N0
                }
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub struct ZkAppUri(String);

impl ZkAppUri {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn gen() -> Self {
        let mut rng = rand::thread_rng();

        let zkapp_uri: u64 = rng.gen();
        let zkapp_uri = zkapp_uri.to_string();

        Self(zkapp_uri)
    }

    fn opt_to_field(opt: Option<&ZkAppUri>) -> Fp {
        let mut inputs = Inputs::new();

        match opt {
            Some(zkapp_uri) => {
                for c in zkapp_uri.0.as_bytes() {
                    for j in 0..8 {
                        inputs.append_bool((c & (1 << j)) != 0);
                    }
                }
                inputs.append_bool(true);
            }
            None => {
                inputs.append_field(Fp::zero());
                inputs.append_field(Fp::zero());
            }
        }

        hash_with_kimchi("MinaZkappUri", &inputs.to_fields())
    }
}

impl ToFieldElements<Fp> for Option<&ZkAppUri> {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let field_zkapp_uri = ZkAppUri::opt_to_field(*self);
        field_zkapp_uri.to_field_elements(fields);
    }
}

impl ToInputs for Option<&ZkAppUri> {
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L313
    fn to_inputs(&self, inputs: &mut Inputs) {
        let field_zkapp_uri = ZkAppUri::opt_to_field(*self);
        inputs.append(&field_zkapp_uri);
    }
}

impl std::ops::Deref for ZkAppUri {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&mina_p2p_messages::string::ByteString> for ZkAppUri {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: &mina_p2p_messages::string::ByteString) -> Result<Self, Self::Error> {
        Ok(Self(value.clone().try_into()?))
    }
}

impl From<&ZkAppUri> for mina_p2p_messages::string::ByteString {
    fn from(value: &ZkAppUri) -> Self {
        Self::from(value.0.as_bytes())
    }
}

impl From<&str> for ZkAppUri {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/zkapp_account.ml#L148-L170
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZkAppAccount {
    pub app_state: [Fp; 8],
    pub verification_key: Option<VerificationKey>,
    // pub verification_key: Option<WithHash<VerificationKey>>, // TODO
    pub zkapp_version: u32,
    pub action_state: [Fp; 5],
    pub last_action_slot: Slot,
    pub proved_state: bool,
    pub zkapp_uri: ZkAppUri,
}

impl ToInputs for ZkAppAccount {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            app_state,
            verification_key,
            zkapp_version,
            action_state,
            last_action_slot,
            proved_state,
            zkapp_uri,
        } = self;

        // Self::zkapp_uri
        inputs.append(&Some(zkapp_uri));

        inputs.append_bool(*proved_state);
        inputs.append_u32(last_action_slot.as_u32());
        for fp in action_state {
            inputs.append_field(*fp);
        }
        inputs.append_u32(*zkapp_version);
        let vk_hash = MyCow::borrow_or_else(verification_key, VerificationKey::dummy).hash();
        inputs.append_field(vk_hash);
        for fp in app_state {
            inputs.append_field(*fp);
        }
    }
}

impl ToFieldElements<Fp> for ZkAppAccount {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            app_state,
            verification_key,
            zkapp_version,
            action_state,
            last_action_slot,
            proved_state,
            zkapp_uri,
        } = self;

        app_state.to_field_elements(fields);
        (
            FlaggedOption::from(
                verification_key
                    .as_ref()
                    .map(VerificationKey::hash)
                    .as_ref(),
            ),
            || VerificationKey::dummy().hash(),
        )
            .to_field_elements(fields);
        Fp::from(*zkapp_version).to_field_elements(fields);
        action_state.to_field_elements(fields);
        last_action_slot.to_field_elements(fields);
        proved_state.to_field_elements(fields);
        Some(zkapp_uri).to_field_elements(fields);
    }
}

impl Check<Fp> for ZkAppAccount {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            app_state: _,
            verification_key: _,
            zkapp_version,
            action_state: _,
            last_action_slot,
            proved_state: _,
            zkapp_uri: _,
        } = self;

        zkapp_version.check(w);
        last_action_slot.check(w);
    }
}

impl Default for ZkAppAccount {
    fn default() -> Self {
        Self {
            app_state: [Fp::zero(); 8],
            verification_key: None,
            zkapp_version: 0,
            action_state: {
                let empty = Self::empty_action_state();
                [empty, empty, empty, empty, empty]
            },
            last_action_slot: Slot::zero(),
            proved_state: false,
            zkapp_uri: ZkAppUri::new(),
        }
    }
}

impl ZkAppAccount {
    pub const HASH_PARAM: &'static str = "MinaZkappAccount";

    pub fn hash(&self) -> Fp {
        self.hash_with_param(Self::HASH_PARAM)
    }

    /// empty_state_element
    pub fn empty_action_state() -> Fp {
        cache_one!(Fp, { hash_noinputs("MinaZkappActionStateEmptyElt") })
    }
}

#[derive(Clone, Eq)]
pub struct AccountId {
    pub public_key: CompressedPubKey,
    pub token_id: TokenId,
}

impl Ord for AccountId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for AccountId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_pk: BigInteger256 = self.public_key.x.into();
        let other_pk: BigInteger256 = other.public_key.x.into();
        match self_pk.partial_cmp(&other_pk) {
            Some(core::cmp::Ordering::Equal) | None => {}
            ord => return ord,
        }

        match self.public_key.is_odd.partial_cmp(&other.public_key.is_odd) {
            Some(core::cmp::Ordering::Equal) | None => {}
            ord => return ord,
        }

        let self_token_id: BigInteger256 = self.token_id.0.into();
        let other_token_id: BigInteger256 = other.token_id.0.into();

        self_token_id.partial_cmp(&other_token_id)
    }
}

impl ToInputs for AccountId {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            public_key,
            token_id,
        } = self;
        inputs.append(public_key);
        inputs.append(token_id);
    }
}

impl AccountId {
    pub const DERIVE_TOKEN_ID_HASH_PARAM: &'static str = "MinaDeriveTokenId";

    pub fn empty() -> Self {
        Self {
            public_key: CompressedPubKey::empty(),
            token_id: TokenId::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::empty()
    }

    pub fn derive_token_id(&self) -> TokenId {
        // TODO: Use `ToInputs`
        let is_odd_field = match self.public_key.is_odd {
            true => Fp::one(),
            false => Fp::zero(),
        };

        TokenId(hash_with_kimchi(
            Self::DERIVE_TOKEN_ID_HASH_PARAM,
            &[self.public_key.x, self.token_id.0, is_odd_field],
        ))
    }

    pub fn new(public_key: CompressedPubKey, token_id: TokenId) -> Self {
        Self {
            public_key,
            token_id,
        }
    }

    pub fn create(public_key: CompressedPubKey, token_id: TokenId) -> Self {
        Self::new(public_key, token_id)
    }

    pub fn ocaml_hash(&self) -> u32 {
        crate::port_ocaml::account_id_ocaml_hash(self)
    }

    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            public_key: gen_compressed(),
            token_id: TokenId(Fp::rand(&mut rng)),
        }
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);
        AccountId::binprot_read(&mut cursor).unwrap()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10000);
        self.binprot_write(&mut bytes).unwrap();
        bytes
    }

    pub fn checked_equal(&self, other: &Self, w: &mut Witness<Fp>) -> Boolean {
        use crate::proofs::field::field;

        // public_key
        let pk_equal = checked_equal_compressed_key(&self.public_key, &other.public_key, w);

        // token_id
        let tid_equal = field::equal(self.token_id.0, other.token_id.0, w);

        // both
        pk_equal.and(&tid_equal, w)
    }
}

pub fn checked_equal_compressed_key(
    a: &CompressedPubKey,
    b: &CompressedPubKey,
    w: &mut Witness<Fp>,
) -> Boolean {
    use crate::proofs::field::field;

    let x_eq = field::equal(a.x, b.x, w);
    let odd_eq = Boolean::equal(&a.is_odd.to_boolean(), &b.is_odd.to_boolean(), w);
    x_eq.and(&odd_eq, w)
}

// TODO: Dedup with above
pub fn checked_equal_compressed_key_const_and(
    a: &CompressedPubKey,
    b: &CompressedPubKey,
    w: &mut Witness<Fp>,
) -> Boolean {
    use crate::proofs::field::field;

    if b == &CompressedPubKey::empty() {
        let x_eq = field::equal(a.x, b.x, w);
        let odd_eq = Boolean::const_equal(&a.is_odd.to_boolean(), &b.is_odd.to_boolean());
        x_eq.and(&odd_eq, w)
    } else {
        let x_eq = field::equal(a.x, b.x, w);
        let odd_eq = Boolean::equal(&a.is_odd.to_boolean(), &b.is_odd.to_boolean(), w);
        x_eq.const_and(&odd_eq)
    }
}

impl std::fmt::Debug for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountId")
            .field("public_key", &self.public_key)
            .field("token_id", &self.token_id)
            .finish()
    }
}

impl std::hash::Hash for AccountId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.public_key.x.hash(state);
        self.public_key.is_odd.hash(state);
        self.token_id.hash(state);
    }
}

impl PartialEq for AccountId {
    fn eq(&self, other: &Self) -> bool {
        self.public_key.x == other.public_key.x
            && self.public_key.is_odd == other.public_key.is_odd
            && self.token_id.0 == other.token_id.0
    }
}

#[derive(Debug)]
pub enum PermissionTo {
    Access,
    Send,
    Receive,
    SetDelegate,
    IncrementNonce,
}

#[derive(Copy, Clone, Debug)]
pub enum ControlTag {
    Proof,
    Signature,
    NoneGiven,
}

impl ControlTag {
    pub fn gen(rng: &mut ThreadRng) -> Self {
        // Match will fail when a variant added
        match Self::NoneGiven {
            ControlTag::Proof => {}
            ControlTag::Signature => {}
            ControlTag::NoneGiven => {}
        };

        [Self::Proof, Self::Signature, Self::NoneGiven]
            .choose(rng)
            .copied()
            .unwrap()
    }
}

pub fn check_permission(auth: AuthRequired, tag: ControlTag) -> bool {
    use AuthRequired::*;
    use ControlTag as Tag;

    match (auth, tag) {
        (Impossible, _) => false,
        (None, _) => true,
        (Proof, Tag::Proof) => true,
        (Signature, Tag::Signature) => true,
        // The signatures and proofs have already been checked by this point.
        (Either, Tag::Proof | Tag::Signature) => true,
        (Signature, Tag::Proof) => false,
        (Proof, Tag::Signature) => false,
        (Proof | Signature | Either, Tag::NoneGiven) => false,
        (Both, _) => unimplemented!("check_permission with `Both` Not implemented in OCaml"),
    }
}

// TODO: Dedup with the one in `snark.rs`
pub fn eval_no_proof<F: FieldWitness>(
    auth: AuthRequired,
    signature_verifies: Boolean,
    is_and_const: bool,
    is_or_const: bool,
    w: &mut Witness<F>,
) -> Boolean {
    // TODO: Remove this hack with `is_const`

    let AuthRequiredEncoded {
        constant,
        signature_necessary: _,
        signature_sufficient,
    } = auth.encode();

    let constant = constant.to_boolean();
    let signature_sufficient = signature_sufficient.to_boolean();

    let a = if is_and_const {
        constant.neg().const_and(&signature_verifies)
    } else {
        constant.neg().and(&signature_verifies, w)
    };
    let b = if is_or_const {
        constant.const_or(&a)
    } else {
        constant.or(&a, w)
    };
    signature_sufficient.and(&b, w)
}

pub struct PermsConst {
    pub and_const: bool,
    pub or_const: bool,
}

// https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub public_key: CompressedPubKey, // Public_key.Compressed.t
    pub token_id: TokenId,            // Token_id.t
    /// the `token_symbol` describes a token id owned by the account id
    /// from this account, not the token id used by this account
    pub token_symbol: TokenSymbol, // Token_symbol.t
    pub balance: Balance,             // Balance.t
    pub nonce: Nonce,                 // Nonce.t
    pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
    pub delegate: Option<CompressedPubKey>, // Public_key.Compressed.t option
    pub voting_for: VotingFor,        // State_hash.t
    pub timing: Timing,               // Timing.t
    pub permissions: Permissions<AuthRequired>, // Permissions.t
    pub zkapp: Option<ZkAppAccount>,  // Zkapp_account.t
}

impl Account {
    #[cfg(test)]
    pub fn create() -> Self {
        let pubkey = CompressedPubKey::from_address(
            "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
        )
        .unwrap();

        Self {
            public_key: pubkey.clone(),
            token_id: TokenId::default(),
            token_symbol: TokenSymbol::default(),
            balance: Balance::from_u64(10101),
            nonce: Nonce::zero(),
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: Some(pubkey),
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
        }
    }

    pub fn create_with(account_id: AccountId, balance: Balance) -> Self {
        let delegate = if account_id.token_id.is_default() {
            // Only allow delegation if this account is for the default token.
            Some(account_id.public_key.clone())
        } else {
            None
        };

        Self {
            public_key: account_id.public_key,
            token_id: account_id.token_id,
            token_symbol: TokenSymbol::default(),
            balance,
            nonce: Nonce::zero(),
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
        }
    }

    pub fn delegate_or_empty(&self) -> MyCow<CompressedPubKey> {
        MyCow::borrow_or_else(&self.delegate, CompressedPubKey::empty)
    }

    pub fn zkapp_or_empty(&self) -> MyCow<ZkAppAccount> {
        MyCow::borrow_or_else(&self.zkapp, ZkAppAccount::default)
    }

    pub fn initialize(account_id: &AccountId) -> Self {
        Self::create_with(account_id.clone(), Balance::zero())
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);
        Account::binprot_read(&mut cursor).unwrap()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10000);
        self.binprot_write(&mut bytes).unwrap();
        bytes
    }

    pub fn empty() -> Self {
        Self {
            public_key: CompressedPubKey {
                x: Fp::zero(),
                is_odd: false,
            },
            token_id: TokenId::default(),
            token_symbol: TokenSymbol::default(),
            balance: Balance::zero(),
            nonce: Nonce::zero(),
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
        }
    }

    pub fn id(&self) -> AccountId {
        AccountId {
            public_key: self.public_key.clone(),
            token_id: self.token_id.clone(),
        }
    }

    pub fn has_locked_tokens(&self, global_slot: Slot) -> bool {
        match self.timing {
            Timing::Untimed => false,
            Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => {
                let curr_min_balance = account_min_balance_at_slot(
                    global_slot,
                    cliff_time,
                    cliff_amount,
                    vesting_period,
                    vesting_increment,
                    initial_minimum_balance,
                );

                !curr_min_balance.is_zero()
            }
        }
    }

    pub fn has_locked_tokens_checked(
        &self,
        global_slot: &CheckedSlot<Fp>,
        w: &mut Witness<Fp>,
    ) -> Boolean {
        let TimingAsRecordChecked {
            is_timed: _,
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self.timing.to_record_checked::<Fp>();

        let cur_min_balance = checked_min_balance_at_slot(
            global_slot,
            &cliff_time,
            &cliff_amount,
            &vesting_period,
            &vesting_increment,
            &initial_minimum_balance,
            w,
        );

        let zero_min_balance = CheckedBalance::zero().equal(&cur_min_balance, w);
        zero_min_balance.neg()
    }

    /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account.ml#L794
    pub fn has_permission_to(&self, control: ControlTag, to: PermissionTo) -> bool {
        match to {
            PermissionTo::Access => check_permission(self.permissions.access, control),
            PermissionTo::Send => check_permission(self.permissions.send, control),
            PermissionTo::Receive => check_permission(self.permissions.receive, control),
            PermissionTo::SetDelegate => check_permission(self.permissions.set_delegate, control),
            PermissionTo::IncrementNonce => {
                check_permission(self.permissions.increment_nonce, control)
            }
        }
    }

    pub fn checked_has_permission_to<F: FieldWitness>(
        &self,
        consts: PermsConst,
        signature_verifies: Option<Boolean>,
        to: PermissionTo,
        w: &mut Witness<F>,
    ) -> Boolean {
        let signature_verifies = match signature_verifies {
            Some(signature_verifies) => signature_verifies,
            None => match to {
                PermissionTo::Send => Boolean::True,
                PermissionTo::Receive => Boolean::False,
                PermissionTo::SetDelegate => Boolean::True,
                PermissionTo::IncrementNonce => Boolean::True,
                PermissionTo::Access => {
                    panic!("signature_verifies argument must be given for access permission")
                }
            },
        };

        let auth = match to {
            PermissionTo::Send => self.permissions.send,
            PermissionTo::Receive => self.permissions.receive,
            PermissionTo::SetDelegate => self.permissions.set_delegate,
            PermissionTo::IncrementNonce => self.permissions.increment_nonce,
            PermissionTo::Access => self.permissions.access,
        };

        eval_no_proof(
            auth,
            signature_verifies,
            consts.and_const,
            consts.or_const,
            w,
        )
    }

    /// [true] iff account has permissions set that enable them to transfer Mina (assuming the command is signed)
    pub fn has_permission_to_send(&self) -> bool {
        self.has_permission_to(ControlTag::Signature, PermissionTo::Access)
            && self.has_permission_to(ControlTag::Signature, PermissionTo::Send)
    }

    /// [true] iff account has permissions set that enable them to receive Mina
    pub fn has_permission_to_receive(&self) -> bool {
        self.has_permission_to(ControlTag::NoneGiven, PermissionTo::Access)
            && self.has_permission_to(ControlTag::NoneGiven, PermissionTo::Receive)
    }

    /// [true] iff account has permissions set that enable them to set their delegate (assuming the command is signed)
    pub fn has_permission_to_set_delegate(&self) -> bool {
        self.has_permission_to(ControlTag::Signature, PermissionTo::Access)
            && self.has_permission_to(ControlTag::Signature, PermissionTo::SetDelegate)
    }

    /// [true] iff account has permissions set that enable them to increment their nonce (assuming the command is signed)
    pub fn has_permission_to_increment_nonce(&self) -> bool {
        self.has_permission_to(ControlTag::Signature, PermissionTo::Access)
            && self.has_permission_to(ControlTag::Signature, PermissionTo::IncrementNonce)
    }

    pub fn hash(&self) -> Fp {
        let inputs = self.to_inputs_owned();
        hash_with_kimchi("MinaAccount", &inputs.to_fields())
    }

    pub fn checked_hash(&self, w: &mut Witness<Fp>) -> Fp {
        use crate::proofs::transaction::transaction_snark::checked_hash;

        let inputs = self.to_inputs_owned();

        checked_hash("MinaAccount", &inputs.to_fields(), w)
    }

    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();
        let rng = &mut rng;

        let symbol: u64 = rng.gen();
        let mut symbol = symbol.to_string();
        symbol.truncate(6);

        let zkapp_uri: u64 = rng.gen();
        let mut zkapp_uri = zkapp_uri.to_string();
        zkapp_uri.truncate(6);

        let gen_perm = |rng: &mut ThreadRng| {
            let n: u64 = rng.gen();
            if n % 5 == 0 {
                AuthRequired::Either
            } else if n % 4 == 0 {
                AuthRequired::Impossible
            } else if n % 3 == 0 {
                AuthRequired::None
            } else if n % 2 == 0 {
                AuthRequired::Proof
            } else {
                AuthRequired::Signature
            }
        };

        Self {
            public_key: gen_compressed(),
            token_id: TokenId(Fp::rand(rng)),
            token_symbol: TokenSymbol(symbol),
            balance: rng.gen(),
            nonce: rng.gen(),
            receipt_chain_hash: ReceiptChainHash(Fp::rand(rng)),
            delegate: if rng.gen() {
                Some(gen_compressed())
            } else {
                None
            },
            voting_for: VotingFor(Fp::rand(rng)),
            timing: if rng.gen() {
                Timing::Untimed
            } else {
                Timing::Timed {
                    initial_minimum_balance: rng.gen(),
                    cliff_time: rng.gen(),
                    cliff_amount: rng.gen(),
                    vesting_period: rng.gen(),
                    vesting_increment: rng.gen(),
                }
            },
            permissions: Permissions {
                edit_state: gen_perm(rng),
                send: gen_perm(rng),
                receive: gen_perm(rng),
                set_delegate: gen_perm(rng),
                set_permissions: gen_perm(rng),
                set_verification_key: SetVerificationKey {
                    auth: gen_perm(rng),
                    txn_version: TXN_VERSION_CURRENT, // TODO: Make the version random ?
                },
                set_zkapp_uri: gen_perm(rng),
                edit_action_state: gen_perm(rng),
                set_token_symbol: gen_perm(rng),
                increment_nonce: gen_perm(rng),
                set_voting_for: gen_perm(rng),
                access: gen_perm(rng),
                set_timing: gen_perm(rng),
            },
            zkapp: if rng.gen() {
                Some(ZkAppAccount {
                    app_state: [
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                    ],
                    verification_key: if rng.gen() {
                        Some(VerificationKey::gen())
                    } else {
                        None
                    },
                    zkapp_version: rng.gen(),
                    action_state: [
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                        Fp::rand(rng),
                    ],
                    last_action_slot: rng.gen(),
                    proved_state: rng.gen(),
                    zkapp_uri: ZkAppUri(zkapp_uri),
                })
            } else {
                None
            },
        }
    }
}

impl ToInputs for Account {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            public_key,
            token_id,
            token_symbol,
            balance,
            nonce,
            receipt_chain_hash,
            delegate,
            voting_for,
            timing,
            permissions,
            zkapp,
        } = self;

        // Self::zkapp
        let field_zkapp = {
            let zkapp = MyCow::borrow_or_default(zkapp);
            zkapp.hash()
        };
        inputs.append_field(field_zkapp);
        inputs.append(permissions);

        // Self::timing
        let TimingAsRecord {
            is_timed,
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = timing.to_record();
        inputs.append_bool(is_timed);
        inputs.append_u64(initial_minimum_balance.as_u64());
        inputs.append_u32(cliff_time.as_u32());
        inputs.append_u64(cliff_amount.as_u64());
        inputs.append_u32(vesting_period.as_u32());
        inputs.append_u64(vesting_increment.as_u64());

        // Self::voting_for
        inputs.append_field(voting_for.0);
        // Self::delegate
        let delegate = MyCow::borrow_or_else(delegate, CompressedPubKey::empty);
        inputs.append(delegate.as_ref());
        // Self::receipt_chain_hash
        inputs.append_field(receipt_chain_hash.0);
        // Self::nonce
        inputs.append_u32(nonce.as_u32());
        // Self::balance
        inputs.append_u64(balance.as_u64());
        // Self::token_symbol
        // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
        assert!(token_symbol.len() <= 6);
        inputs.append(token_symbol);
        // Self::token_id
        inputs.append_field(token_id.0);
        // Self::public_key
        inputs.append(public_key);
    }
}

fn verify_merkle_path(account: &Account, merkle_path: &[MerklePath]) -> Fp {
    let account_hash = account.hash();
    let mut param = String::with_capacity(16);

    merkle_path
        .iter()
        .enumerate()
        .fold(account_hash, |accum, (depth, path)| {
            let hashes = match path {
                MerklePath::Left(right) => [accum, *right],
                MerklePath::Right(left) => [*left, accum],
            };

            param.clear();
            write!(&mut param, "MinaMklTree{:03}", depth).unwrap();

            crate::hash::hash_with_kimchi(param.as_str(), &hashes)
        })
}

/// `implied_root` in OCaml
pub fn checked_verify_merkle_path(
    account: &Account,
    merkle_path: &[MerklePath],
    w: &mut Witness<Fp>,
) -> Fp {
    use crate::proofs::transaction::transaction_snark::checked_hash;

    let account_hash = account.checked_hash(w);
    let mut param = String::with_capacity(16);

    merkle_path
        .iter()
        .enumerate()
        .fold(account_hash, |accum, (depth, path)| {
            let hashes = match path {
                MerklePath::Left(right) => [accum, *right],
                MerklePath::Right(left) => [*left, accum],
            };

            param.clear();
            write!(&mut param, "MinaMklTree{:03}", depth).unwrap();

            w.exists(hashes);
            checked_hash(param.as_str(), &hashes, w)
        })
}

#[cfg(test)]
mod tests {
    use o1_utils::FieldHelpers;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[cfg(not(target_family = "wasm"))]
    use crate::{base::BaseLedger, database::Database, tree_version::V2};

    use super::*;

    #[test]
    fn test_size_account() {
        #[cfg(not(target_family = "wasm"))]
        const SIZE: usize = 3424;

        #[cfg(target_family = "wasm")]
        const SIZE: usize = 2496;

        assert_eq!(std::mem::size_of::<Account>(), SIZE);
    }

    #[test]
    fn test_hash_account() {
        let acc = Account::create();
        let hash = acc.hash();

        elog!("account_hash={}", hash);
        elog!("account_hash={}", hash.to_hex());

        assert_eq!(
            hash.to_hex(),
            "d17c17038db495e03eb95af0e4e79248b9ad1363862f4b194644d46932a62c1c"
        );

        let acc = Account {
            public_key: CompressedPubKey::from_address(
                "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
            )
            .unwrap(),
            token_id: TokenId::default(),
            token_symbol: TokenSymbol::from("seb".to_string()),
            balance: Balance::from_u64(10101),
            nonce: Nonce::from_u32(62772),
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
        };

        assert_eq!(
            acc.hash().to_hex(),
            "d39fb6f37dd1d7fb3928c8f493bbeade214fdeae89d3703192e2b4f1373e421c"
        );
    }

    #[test]
    fn test_dummy_sideloaded_verification_key() {
        assert_eq!(
            VerificationKey::dummy().hash().to_hex(),
            "d6da18e4091fbcd86843604fb8ff2d9613e76fa16c49b0263a1566a8e7188007"
        );
    }

    #[test]
    fn test_rand() {
        for _ in 0..1000 {
            let rand = Account::rand();
            let hash = rand.hash();

            let bytes = Account::serialize(&rand);
            let rand2: Account = Account::deserialize(&bytes);

            assert_eq!(hash, rand2.hash());
        }
    }

    #[cfg(not(target_family = "wasm"))] // Use multiple threads
    #[test]
    fn test_rand_tree() {
        use rayon::prelude::*;

        let mut db = Database::<V2>::create(20);
        let mut accounts = Vec::with_capacity(1000);

        const NACCOUNTS: usize = 1000;

        for _ in 0..NACCOUNTS {
            let rand = Account::rand();
            accounts.push(rand);
        }

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(16)
            .build()
            .unwrap();

        let now = std::time::Instant::now();
        let hashes = pool.install(|| {
            accounts
                .par_iter()
                .map(|acc| acc.hash())
                .collect::<Vec<_>>()
        });

        assert_eq!(hashes.len(), NACCOUNTS);
        elog!(
            "elapsed to hash accounts in 16 threads: {:?}",
            now.elapsed(),
        );

        let mut hashes = Vec::with_capacity(accounts.len());
        let now = std::time::Instant::now();
        for account in accounts.iter() {
            hashes.push(account.hash());
        }
        assert_eq!(hashes.len(), NACCOUNTS);
        elog!("elapsed to hash accounts in 1 thread: {:?}", now.elapsed(),);

        let now = std::time::Instant::now();
        for account in accounts.into_iter() {
            let id = account.id();
            db.get_or_create_account(id, account).unwrap();
        }
        assert_eq!(db.naccounts(), NACCOUNTS);
        elog!("elapsed to insert in tree: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        db.root_hash();
        elog!("root hash computed in {:?}", now.elapsed());
    }
}
