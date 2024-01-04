use std::{fmt::Write, io::Cursor, str::FromStr};

use ark_ff::{BigInteger256, One, UniformRand, Zero};
use mina_hasher::Fp;
use mina_p2p_messages::binprot::{BinProtRead, BinProtWrite};
use mina_signer::CompressedPubKey;
use rand::{prelude::ThreadRng, seq::SliceRandom, Rng};

use crate::{
    gen_compressed,
    hash::{hash_noinputs, hash_with_kimchi, Inputs},
    proofs::{
        numbers::{
            currency::{CheckedBalance, CheckedCurrency},
            nat::CheckedSlot,
        },
        to_field_elements::ToFieldElements,
        witness::{
            make_group, transaction_snark::checked_min_balance_at_slot, Boolean, Check,
            FieldWitness, InnerCurve, PlonkVerificationKeyEvals, ToBoolean, Witness,
        },
    },
    scan_state::{
        currency::{Balance, Magnitude, Nonce, Slot},
        transaction_logic::account_min_balance_at_slot,
    },
    zkapps::snark::FlaggedOption,
    MerklePath, MyCow, ToInputs,
};

use super::common::*;

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

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L49
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permissions<Controller> {
    pub edit_state: Controller,
    pub access: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: Controller,
    pub set_zkapp_uri: Controller,
    pub edit_action_state: Controller,
    pub set_token_symbol: Controller,
    pub increment_nonce: Controller,
    pub set_voting_for: Controller,
    pub set_timing: Controller,
}

impl Permissions<AuthRequired> {
    pub fn iter_as_bits<F>(&self, mut fun: F)
    where
        F: FnMut(bool),
    {
        let Self {
            edit_state,
            access,
            send,
            receive,
            set_delegate,
            set_permissions,
            set_verification_key,
            set_zkapp_uri,
            edit_action_state,
            set_token_symbol,
            increment_nonce,
            set_voting_for,
            set_timing,
        } = self;

        for auth in [
            edit_state,
            access,
            send,
            receive,
            set_delegate,
            set_permissions,
            set_verification_key,
            set_zkapp_uri,
            edit_action_state,
            set_token_symbol,
            increment_nonce,
            set_voting_for,
            set_timing,
        ] {
            for bit in auth.encode().to_bits() {
                fun(bit);
            }
        }
    }
}

impl ToInputs for Permissions<AuthRequired> {
    fn to_inputs(&self, inputs: &mut Inputs) {
        self.iter_as_bits(|bit| {
            inputs.append_bool(bit);
        });
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
            set_verification_key: Signature,
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
            set_verification_key: None,
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
            set_verification_key: auth_required_gen(&mut rng),
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

        let max_proofs_verified = max_proofs_verified.to_int() as u64;
        Fp::from(max_proofs_verified).to_field_elements(fields);

        let actual_wrap_domain_size = actual_wrap_domain_size.to_int() as u64;
        Fp::from(actual_wrap_domain_size).to_field_elements(fields);

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
        use crate::proofs::witness::field;

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
    use crate::proofs::witness::field;

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
    use crate::proofs::witness::field;

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
        use crate::proofs::witness::transaction_snark::checked_hash;

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
                set_verification_key: gen_perm(rng),
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
    use crate::proofs::witness::transaction_snark::checked_hash;

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
        const SIZE: usize = 2528;

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
            "98cf7cf3a885d0523ac3ac51c3aca17ebb93ec94a15aed43787352cfe8e47204"
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
            "ef40252c54fa9e7539ae91db89c8104778ad19e1afab1b8df4a4dee51a270e1e"
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

    #[test]
    fn test_verify_merkle_path() {
        use mina_p2p_messages::v2::{MerkleTreeNode, MinaBaseAccountBinableArgStableV2};

        let account = "6c88f36f4a0fcbaaaf12e91f030ca5e33bd7b26a0b8e6c22bc2af33924f2ce3400010000000000000000000000000000000000000000000000000000000000000000fc00aaa0680b000000009be4b7c51ed9c2e4524727805fd36f5220fbfc70a749f62623b0ed2908433320016c88f36f4a0fcbaaaf12e91f030ca5e33bd7b26a0b8e6c22bc2af33924f2ce34000000000000000000000000000000000000000000000000000000000000000000000300030003030303030303030300";

        let merkle_path = "234f6812650aa27ce35d904b875809d380b2a7184dedd52d4c289274d6626e65ce5fff34354f681265d95c4d0208bcfd7d4ede27bbd1653d41ac8b0f37fe3fb6f39b5e8113df33f32279f722f9c7bbe7408ca42e90ef121191832b460ce9e990b3731abca9558f4df132614e294f68126502666e211f4d489e821916367014e5487bcbcaa582dc1154d8fdefd4b195ad1e79f722f9399038b193d310c012f421e9babd49367a32a3238eb02c584b936d5d07037b1f79f722f92bbd58fa3e868c956b31e5dfa31ad64f343694a46086659d9d63db0ddf70fb0d4f6812650c0b59c6d6ffab5339590603a2b00695d553784cc74e379cfa5c597266fbe0064f6812659c60712fd3e9663d535ade06b19c14a00d0d6214fc434bd374a34826dcfb7e1379f722f98422f50661c5e0c2b294bba3ebc22ff4f7f86f22d1611b308ea49e93e92d913b4f68126503518a63bb9daf70e3729f3922344dd470f721947cc07a4e4598ec871e4e64384f681265e16eed60ec1e56541360983741bde52a606f37da9495c6cd7244f9f30d9ac7154f681265ddaa309c792e62a1bbf6b4db04c323acf3a0fb702e1313c72755d7bbdb6c4f1e4f68126528405defcf11f365d0ccb31c9e68433441a8d0c77b3a798b7bb45d526715d43d79f722f96585a90bbfa518dafd94f5a2391a162299fd3c61c69b26be09be0c0905c4393d4f681265b2fca6df0ddacc2bb3561c695639837d39253baa3516f97c16556b1e7e6a7b3e4f681265ba91cd781a83e8f733213ef9817d2d958d26139adc4100c66150a169788cf0394f6812654e4fe5ed5ace8dc48426c601162e079b24b4adb72058d1211096ca709305f41f4f681265fb233966427765d8e0e0fb0116d5ee3bb10c5f41289193105c5b7c9c2a51c6094f681265ddf2b009d56e1f3bdfc22e9ef1850d097f6851458acf065816d443d2cd8894264f681265a9dc4535f5784e6148f2fdbcaa6e52d44999ce753cee4bad9de2df945129c1014f681265080aeeaab1058ef1663494607583ad838485b3abcfc5635b497f0c1aead8c2304f68126580be734b9057133b7d2c05187f18f2563dea8cf0bd238a17ee0242b60d98302c4f68126599ba4df1ad24dbc8090b66897d71f2a0cf21b1fb84d261b172e9333156358a3d4f681265f05f173a096c75c0f0148e426558139543535493a1933bd495d5a336e9eba1044f68126551c28fa437d4d89c1b839a1914529144cb3a3d9f8dc9cf4a95107e8cc9e5ee124f6812658ee873cbef184d38c2107cabd69ff87f710637ab9de8a1d7acb653949a72702c4f681265523e5324a58d7cca8ff8f40837656a7390e2515f265781aae422fff6a21b8b214f6812653555315baded133cd65e9d388fb7400f4323d5e79c44d7aee86a91712cdc30374f6812650bf6a75de59539f1be2a12bf307eaee979618e192c1e22d39fc53f98ab5375334f681265d199ee8af504dfc85afd7dd10da4e8872c096fc81e47dcfd2757ac6d9bd4312b4f6812659e2d0a145842af4119df8a7616e8a9687931e800cde90daaf3f7509aa081c10b4f68126593e74d2016c3711fb9486c5e4acb3435f5bf29ccfeefa37fde149bbee5b2430e4f6812651d7ba0bcbe637533740fcb73dfaaf254aea8830cc5555484479f80f2755f5b3d4f6812650146c059f09bc14cfadd69ebc5814dcf5a4301123a74bfa8f3514c5b161f81004f681265db914425a7d4c3bd6b9dc012a040cd94cb5857bb5051ccb6c61c90ada034f93d";

        let account = hex::decode(account).unwrap();
        let mut cursor = std::io::Cursor::new(account);
        let account = MinaBaseAccountBinableArgStableV2::binprot_read(&mut cursor).unwrap();
        let account: Account = (&account).into();

        let merkle_path = hex::decode(merkle_path).unwrap();
        let mut cursor = std::io::Cursor::new(merkle_path);
        let merkle_path = Vec::<MerkleTreeNode>::binprot_read(&mut cursor).unwrap();
        let merkle_path: Vec<_> = merkle_path
            .into_iter()
            .map(|node| match node {
                MerkleTreeNode::Left(f) => MerklePath::Left(f.to_field()),
                MerkleTreeNode::Right(f) => MerklePath::Right(f.to_field()),
            })
            .collect();

        let root_hash = verify_merkle_path(&account, merkle_path.as_slice());

        let expected_root_hash = Fp::from_str(
            "13294139316831045628856068053543468709149714488527059099223047292955286511556",
        )
        .unwrap();

        assert_eq!(root_hash, expected_root_hash);
    }
}
