use super::{
    protocol_state::{self, ProtocolStateView},
    zkapp_statement::TransactionCommitment,
    Memo, TransactionFailure, TransactionStatus, WithStatus,
};
use crate::{
    dummy, gen_compressed, gen_keypair,
    hash::AppendToInputs,
    proofs::{
        field::{Boolean, ToBoolean},
        to_field_elements::ToFieldElements,
        transaction::Check,
        witness::Witness,
    },
    scan_state::{
        currency::{
            Amount, Balance, Fee, Length, Magnitude, MinMax, Nonce, Sgn, Signed, Slot, SlotSpan,
        },
        fee_excess::FeeExcess,
        GenesisConstant, GENESIS_CONSTANT,
    },
    zkapps::checks::{ZkappCheck, ZkappCheckOps},
    AccountId, AuthRequired, ControlTag, MutableFp, MyCow, Permissions, SetVerificationKey,
    ToInputs, TokenId, TokenSymbol, VerificationKey, VerificationKeyWire, VotingFor, ZkAppAccount,
    ZkAppUri,
};
use ark_ff::{UniformRand, Zero};
use itertools::Itertools;
use mina_curves::pasta::Fp;
use mina_p2p_messages::v2::MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA;
use mina_signer::{CompressedPubKey, Signature};
use poseidon::hash::{
    hash_noinputs, hash_with_kimchi,
    params::{
        MINA_ACCOUNT_UPDATE_CONS, MINA_ACCOUNT_UPDATE_NODE, MINA_ZKAPP_EVENT, MINA_ZKAPP_EVENTS,
        MINA_ZKAPP_SEQ_EVENTS, NO_INPUT_MINA_ZKAPP_ACTIONS_EMPTY, NO_INPUT_MINA_ZKAPP_EVENTS_EMPTY,
    },
    Inputs,
};
use rand::{seq::SliceRandom, Rng};
use std::sync::Arc;

pub mod from_applied_sequence;
pub mod from_unapplied_sequence;
pub mod valid;
pub mod verifiable;
pub mod zkapp_weight;

#[derive(Debug, Clone, PartialEq)]
pub struct Event(pub Vec<Fp>);

impl Event {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn hash(&self) -> Fp {
        hash_with_kimchi(&MINA_ZKAPP_EVENT, &self.0[..])
    }

    pub fn len(&self) -> usize {
        let Self(list) = self;
        list.len()
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L834>
#[derive(Debug, Clone, PartialEq)]
pub struct Events(pub Vec<Event>);

/// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L155>
#[derive(Debug, Clone, PartialEq)]
pub struct Actions(pub Vec<Event>);

pub fn gen_events() -> Vec<Event> {
    let mut rng = rand::thread_rng();

    let n = rng.gen_range(0..=5);

    (0..=n)
        .map(|_| {
            let n = rng.gen_range(0..=3);
            let event = (0..=n).map(|_| Fp::rand(&mut rng)).collect();
            Event(event)
        })
        .collect()
}

use poseidon::hash::LazyParam;

/// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L23>
pub trait MakeEvents {
    const DERIVER_NAME: (); // Unused here for now

    fn get_salt_phrase() -> &'static LazyParam;

    fn get_hash_prefix() -> &'static LazyParam;

    fn events(&self) -> &[Event];

    fn empty_hash() -> Fp;
}

/// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L100>
impl MakeEvents for Events {
    const DERIVER_NAME: () = ();

    fn get_salt_phrase() -> &'static LazyParam {
        &NO_INPUT_MINA_ZKAPP_EVENTS_EMPTY
    }

    fn get_hash_prefix() -> &'static poseidon::hash::LazyParam {
        &MINA_ZKAPP_EVENTS
    }

    fn events(&self) -> &[Event] {
        self.0.as_slice()
    }

    fn empty_hash() -> Fp {
        cache_one!(Fp, events_to_field(&Events::empty()))
    }
}

/// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L156>
impl MakeEvents for Actions {
    const DERIVER_NAME: () = ();

    fn get_salt_phrase() -> &'static LazyParam {
        &NO_INPUT_MINA_ZKAPP_ACTIONS_EMPTY
    }

    fn get_hash_prefix() -> &'static poseidon::hash::LazyParam {
        &MINA_ZKAPP_SEQ_EVENTS
    }

    fn events(&self) -> &[Event] {
        self.0.as_slice()
    }

    fn empty_hash() -> Fp {
        cache_one!(Fp, events_to_field(&Actions::empty()))
    }
}

/// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L52>
pub fn events_to_field<E>(e: &E) -> Fp
where
    E: MakeEvents,
{
    let init = hash_noinputs(E::get_salt_phrase());

    e.events().iter().rfold(init, |accum, elem| {
        hash_with_kimchi(E::get_hash_prefix(), &[accum, elem.hash()])
    })
}

impl ToInputs for Events {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append(&events_to_field(self));
    }
}

impl ToInputs for Actions {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append(&events_to_field(self));
    }
}

impl ToFieldElements<Fp> for Events {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        events_to_field(self).to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for Actions {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        events_to_field(self).to_field_elements(fields);
    }
}

/// Note: It's a different one than in the normal `Account`
///
/// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L163>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timing {
    pub initial_minimum_balance: Balance,
    pub cliff_time: Slot,
    pub cliff_amount: Amount,
    pub vesting_period: SlotSpan,
    pub vesting_increment: Amount,
}

impl Timing {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L208>
    fn dummy() -> Self {
        Self {
            initial_minimum_balance: Balance::zero(),
            cliff_time: Slot::zero(),
            cliff_amount: Amount::zero(),
            vesting_period: SlotSpan::zero(),
            vesting_increment: Amount::zero(),
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/transaction_logic/mina_transaction_logic.ml#L1278>
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L228>
    pub fn of_account_timing(timing: crate::account::Timing) -> Option<Self> {
        match timing {
            crate::Timing::Untimed => None,
            crate::Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => Some(Self {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            }),
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L219>
    pub fn to_account_timing(self) -> crate::account::Timing {
        let Self {
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self;

        crate::account::Timing::Timed {
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        }
    }
}

impl ToFieldElements<Fp> for Timing {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self;

        initial_minimum_balance.to_field_elements(fields);
        cliff_time.to_field_elements(fields);
        cliff_amount.to_field_elements(fields);
        vesting_period.to_field_elements(fields);
        vesting_increment.to_field_elements(fields);
    }
}

impl Check<Fp> for Timing {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self;

        initial_minimum_balance.check(w);
        cliff_time.check(w);
        cliff_amount.check(w);
        vesting_period.check(w);
        vesting_increment.check(w);
    }
}

impl ToInputs for Timing {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L199>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Timing {
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self;

        inputs.append_u64(initial_minimum_balance.as_u64());
        inputs.append_u32(cliff_time.as_u32());
        inputs.append_u64(cliff_amount.as_u64());
        inputs.append_u32(vesting_period.as_u32());
        inputs.append_u64(vesting_increment.as_u64());
    }
}

impl Events {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push_event(acc: Fp, event: Event) -> Fp {
        hash_with_kimchi(Self::get_hash_prefix(), &[acc, event.hash()])
    }

    pub fn push_events(&self, acc: Fp) -> Fp {
        let hash = self
            .0
            .iter()
            .rfold(hash_noinputs(Self::get_salt_phrase()), |acc, e| {
                Self::push_event(acc, e.clone())
            });
        hash_with_kimchi(Self::get_hash_prefix(), &[acc, hash])
    }
}

impl Actions {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push_event(acc: Fp, event: Event) -> Fp {
        hash_with_kimchi(Self::get_hash_prefix(), &[acc, event.hash()])
    }

    pub fn push_events(&self, acc: Fp) -> Fp {
        let hash = self
            .0
            .iter()
            .rfold(hash_noinputs(Self::get_salt_phrase()), |acc, e| {
                Self::push_event(acc, e.clone())
            });
        hash_with_kimchi(Self::get_hash_prefix(), &[acc, hash])
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L100>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetOrKeep<T: Clone> {
    Set(T),
    Keep,
}

impl<T: Clone> SetOrKeep<T> {
    fn map<'a, F, U>(&'a self, fun: F) -> SetOrKeep<U>
    where
        F: FnOnce(&'a T) -> U,
        U: Clone,
    {
        match self {
            SetOrKeep::Set(v) => SetOrKeep::Set(fun(v)),
            SetOrKeep::Keep => SetOrKeep::Keep,
        }
    }

    pub fn into_map<F, U>(self, fun: F) -> SetOrKeep<U>
    where
        F: FnOnce(T) -> U,
        U: Clone,
    {
        match self {
            SetOrKeep::Set(v) => SetOrKeep::Set(fun(v)),
            SetOrKeep::Keep => SetOrKeep::Keep,
        }
    }

    pub fn set_or_keep(&self, x: T) -> T {
        match self {
            Self::Set(data) => data.clone(),
            Self::Keep => x,
        }
    }

    pub fn is_keep(&self) -> bool {
        match self {
            Self::Keep => true,
            Self::Set(_) => false,
        }
    }

    pub fn is_set(&self) -> bool {
        !self.is_keep()
    }

    pub fn gen<F>(mut fun: F) -> Self
    where
        F: FnMut() -> T,
    {
        let mut rng = rand::thread_rng();

        if rng.gen() {
            Self::Set(fun())
        } else {
            Self::Keep
        }
    }
}

impl<T, F> ToInputs for (&SetOrKeep<T>, F)
where
    T: ToInputs,
    T: Clone,
    F: Fn() -> T,
{
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_basic.ml#L223>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let (set_or_keep, default_fn) = self;

        match set_or_keep {
            SetOrKeep::Set(this) => {
                inputs.append_bool(true);
                this.to_inputs(inputs);
            }
            SetOrKeep::Keep => {
                inputs.append_bool(false);
                let default = default_fn();
                default.to_inputs(inputs);
            }
        }
    }
}

impl<T, F> ToFieldElements<Fp> for (&SetOrKeep<T>, F)
where
    T: ToFieldElements<Fp>,
    T: Clone,
    F: Fn() -> T,
{
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let (set_or_keep, default_fn) = self;

        match set_or_keep {
            SetOrKeep::Set(this) => {
                Boolean::True.to_field_elements(fields);
                this.to_field_elements(fields);
            }
            SetOrKeep::Keep => {
                Boolean::False.to_field_elements(fields);
                let default = default_fn();
                default.to_field_elements(fields);
            }
        }
    }
}

impl<T, F> Check<Fp> for (&SetOrKeep<T>, F)
where
    T: Check<Fp>,
    T: Clone,
    F: Fn() -> T,
{
    fn check(&self, w: &mut Witness<Fp>) {
        let (set_or_keep, default_fn) = self;
        let value = match set_or_keep {
            SetOrKeep::Set(this) => MyCow::Borrow(this),
            SetOrKeep::Keep => MyCow::Own(default_fn()),
        };
        value.check(w);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WithHash<T, H = Fp> {
    pub data: T,
    pub hash: H,
}

impl<T, H: Ord> Ord for WithHash<T, H> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl<T, H: PartialOrd> PartialOrd for WithHash<T, H> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl<T, H: Eq> Eq for WithHash<T, H> {}

impl<T, H: PartialEq> PartialEq for WithHash<T, H> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<T, Hash: std::hash::Hash> std::hash::Hash for WithHash<T, Hash> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self { data: _, hash } = self;
        hash.hash(state);
    }
}

impl<T> ToFieldElements<Fp> for WithHash<T> {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { data: _, hash } = self;
        hash.to_field_elements(fields);
    }
}

impl<T> std::ops::Deref for WithHash<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> WithHash<T> {
    pub fn of_data(data: T, hash_data: impl Fn(&T) -> Fp) -> Self {
        let hash = hash_data(&data);
        Self { data, hash }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L319>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub app_state: [SetOrKeep<Fp>; 8],
    pub delegate: SetOrKeep<CompressedPubKey>,
    pub verification_key: SetOrKeep<VerificationKeyWire>,
    pub permissions: SetOrKeep<Permissions<AuthRequired>>,
    pub zkapp_uri: SetOrKeep<ZkAppUri>,
    pub token_symbol: SetOrKeep<TokenSymbol>,
    pub timing: SetOrKeep<Timing>,
    pub voting_for: SetOrKeep<VotingFor>,
}

impl ToFieldElements<Fp> for Update {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            app_state,
            delegate,
            verification_key,
            permissions,
            zkapp_uri,
            token_symbol,
            timing,
            voting_for,
        } = self;

        for s in app_state {
            (s, Fp::zero).to_field_elements(fields);
        }
        (delegate, CompressedPubKey::empty).to_field_elements(fields);
        (&verification_key.map(|w| w.hash()), Fp::zero).to_field_elements(fields);
        (permissions, Permissions::empty).to_field_elements(fields);
        (&zkapp_uri.map(Some), || Option::<&ZkAppUri>::None).to_field_elements(fields);
        (token_symbol, TokenSymbol::default).to_field_elements(fields);
        (timing, Timing::dummy).to_field_elements(fields);
        (voting_for, VotingFor::dummy).to_field_elements(fields);
    }
}

impl Update {
    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L460>
    pub fn noop() -> Self {
        Self {
            app_state: std::array::from_fn(|_| SetOrKeep::Keep),
            delegate: SetOrKeep::Keep,
            verification_key: SetOrKeep::Keep,
            permissions: SetOrKeep::Keep,
            zkapp_uri: SetOrKeep::Keep,
            token_symbol: SetOrKeep::Keep,
            timing: SetOrKeep::Keep,
            voting_for: SetOrKeep::Keep,
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L472>
    pub fn dummy() -> Self {
        Self::noop()
    }

    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L338>
    pub fn gen(
        token_account: Option<bool>,
        zkapp_account: Option<bool>,
        vk: Option<&VerificationKeyWire>,
        permissions_auth: Option<crate::ControlTag>,
    ) -> Self {
        let mut rng = rand::thread_rng();

        let token_account = token_account.unwrap_or(false);
        let zkapp_account = zkapp_account.unwrap_or(false);

        let app_state: [_; 8] = std::array::from_fn(|_| SetOrKeep::gen(|| Fp::rand(&mut rng)));

        let delegate = if !token_account {
            SetOrKeep::gen(|| gen_keypair().public.into_compressed())
        } else {
            SetOrKeep::Keep
        };

        let verification_key = if zkapp_account {
            SetOrKeep::gen(|| match vk {
                None => VerificationKeyWire::dummy(),
                Some(vk) => vk.clone(),
            })
        } else {
            SetOrKeep::Keep
        };

        let permissions = match permissions_auth {
            None => SetOrKeep::Keep,
            Some(auth_tag) => SetOrKeep::Set(Permissions::gen(auth_tag)),
        };

        let zkapp_uri = SetOrKeep::gen(|| {
            ZkAppUri::from(
                [
                    "https://www.example.com",
                    "https://www.minaprotocol.com",
                    "https://www.gurgle.com",
                    "https://faceplant.com",
                ]
                .choose(&mut rng)
                .unwrap()
                .to_string()
                .into_bytes(),
            )
        });

        let token_symbol = SetOrKeep::gen(|| {
            TokenSymbol::from(
                ["MINA", "TOKEN1", "TOKEN2", "TOKEN3", "TOKEN4", "TOKEN5"]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string()
                    .into_bytes(),
            )
        });

        let voting_for = SetOrKeep::gen(|| VotingFor(Fp::rand(&mut rng)));

        let timing = SetOrKeep::Keep;

        Self {
            app_state,
            delegate,
            verification_key,
            permissions,
            zkapp_uri,
            token_symbol,
            timing,
            voting_for,
        }
    }
}

// TODO: This could be std::ops::Range ?
/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L23>
#[derive(Debug, Clone, PartialEq)]
pub struct ClosedInterval<T> {
    pub lower: T,
    pub upper: T,
}

impl<T> ClosedInterval<T>
where
    T: MinMax,
{
    pub fn min_max() -> Self {
        Self {
            lower: T::min(),
            upper: T::max(),
        }
    }
}

impl<T> ToInputs for ClosedInterval<T>
where
    T: ToInputs,
{
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L37>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let ClosedInterval { lower, upper } = self;

        lower.to_inputs(inputs);
        upper.to_inputs(inputs);
    }
}

impl<T> ToFieldElements<Fp> for ClosedInterval<T>
where
    T: ToFieldElements<Fp>,
{
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let ClosedInterval { lower, upper } = self;

        lower.to_field_elements(fields);
        upper.to_field_elements(fields);
    }
}

impl<T> Check<Fp> for ClosedInterval<T>
where
    T: Check<Fp>,
{
    fn check(&self, w: &mut Witness<Fp>) {
        let ClosedInterval { lower, upper } = self;
        lower.check(w);
        upper.check(w);
    }
}

impl<T> ClosedInterval<T>
where
    T: PartialOrd,
{
    pub fn is_constant(&self) -> bool {
        self.lower == self.upper
    }

    /// <https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_base/zkapp_precondition.ml#L30>
    pub fn gen<F>(mut fun: F) -> Self
    where
        F: FnMut() -> T,
    {
        let a1 = fun();
        let a2 = fun();

        if a1 <= a2 {
            Self {
                lower: a1,
                upper: a2,
            }
        } else {
            Self {
                lower: a2,
                upper: a1,
            }
        }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L232>
#[derive(Debug, Clone, PartialEq)]
pub enum OrIgnore<T> {
    Check(T),
    Ignore,
}

impl<T, F> ToInputs for (&OrIgnore<T>, F)
where
    T: ToInputs,
    F: Fn() -> T,
{
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L414>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let (or_ignore, default_fn) = self;

        match or_ignore {
            OrIgnore::Check(this) => {
                inputs.append_bool(true);
                this.to_inputs(inputs);
            }
            OrIgnore::Ignore => {
                inputs.append_bool(false);
                let default = default_fn();
                default.to_inputs(inputs);
            }
        }
    }
}

impl<T, F> ToFieldElements<Fp> for (&OrIgnore<T>, F)
where
    T: ToFieldElements<Fp>,
    F: Fn() -> T,
{
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let (or_ignore, default_fn) = self;

        match or_ignore {
            OrIgnore::Check(this) => {
                Boolean::True.to_field_elements(fields);
                this.to_field_elements(fields);
            }
            OrIgnore::Ignore => {
                Boolean::False.to_field_elements(fields);
                let default = default_fn();
                default.to_field_elements(fields);
            }
        };
    }
}

impl<T, F> Check<Fp> for (&OrIgnore<T>, F)
where
    T: Check<Fp>,
    F: Fn() -> T,
{
    fn check(&self, w: &mut Witness<Fp>) {
        let (or_ignore, default_fn) = self;
        let value = match or_ignore {
            OrIgnore::Check(this) => MyCow::Borrow(this),
            OrIgnore::Ignore => MyCow::Own(default_fn()),
        };
        value.check(w);
    }
}

impl<T> OrIgnore<T> {
    /// <https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_base/zkapp_basic.ml#L239>
    pub fn gen<F>(mut fun: F) -> Self
    where
        F: FnMut() -> T,
    {
        let mut rng = rand::thread_rng();

        if rng.gen() {
            Self::Check(fun())
        } else {
            Self::Ignore
        }
    }

    pub fn map<F, V>(&self, fun: F) -> OrIgnore<V>
    where
        F: Fn(&T) -> V,
    {
        match self {
            OrIgnore::Check(v) => OrIgnore::Check(fun(v)),
            OrIgnore::Ignore => OrIgnore::Ignore,
        }
    }
}

impl<T> OrIgnore<ClosedInterval<T>>
where
    T: PartialOrd,
{
    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_precondition.ml#L294>
    pub fn is_constant(&self) -> bool {
        match self {
            OrIgnore::Check(interval) => interval.lower == interval.upper,
            OrIgnore::Ignore => false,
        }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L439>
pub type Hash<T> = OrIgnore<T>;

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L298>
pub type EqData<T> = OrIgnore<T>;

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L178>
pub type Numeric<T> = OrIgnore<ClosedInterval<T>>;

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/epoch_ledger.ml#L9>
#[derive(Debug, Clone, PartialEq)]
pub struct EpochLedger {
    pub hash: Hash<Fp>,
    pub total_currency: Numeric<Amount>,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L797>
#[derive(Debug, Clone, PartialEq)]
pub struct EpochData {
    pub(crate) ledger: EpochLedger,
    pub seed: Hash<Fp>,
    pub start_checkpoint: Hash<Fp>,
    pub lock_checkpoint: Hash<Fp>,
    pub epoch_length: Numeric<Length>,
}

#[cfg(feature = "fuzzing")]
impl EpochData {
    pub fn new(
        ledger: EpochLedger,
        seed: Hash<Fp>,
        start_checkpoint: Hash<Fp>,
        lock_checkpoint: Hash<Fp>,
        epoch_length: Numeric<Length>,
    ) -> Self {
        EpochData {
            ledger,
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        }
    }

    pub fn ledger_mut(&mut self) -> &mut EpochLedger {
        &mut self.ledger
    }
}

impl ToInputs for EpochData {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L875>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let EpochData {
            ledger,
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;

        {
            let EpochLedger {
                hash,
                total_currency,
            } = ledger;

            inputs.append(&(hash, Fp::zero));
            inputs.append(&(total_currency, ClosedInterval::min_max));
        }

        inputs.append(&(seed, Fp::zero));
        inputs.append(&(start_checkpoint, Fp::zero));
        inputs.append(&(lock_checkpoint, Fp::zero));
        inputs.append(&(epoch_length, ClosedInterval::min_max));
    }
}

impl ToFieldElements<Fp> for EpochData {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let EpochData {
            ledger,
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;

        {
            let EpochLedger {
                hash,
                total_currency,
            } = ledger;

            (hash, Fp::zero).to_field_elements(fields);
            (total_currency, ClosedInterval::min_max).to_field_elements(fields);
        }

        (seed, Fp::zero).to_field_elements(fields);
        (start_checkpoint, Fp::zero).to_field_elements(fields);
        (lock_checkpoint, Fp::zero).to_field_elements(fields);
        (epoch_length, ClosedInterval::min_max).to_field_elements(fields);
    }
}

impl Check<Fp> for EpochData {
    fn check(&self, w: &mut Witness<Fp>) {
        let EpochData {
            ledger,
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;

        {
            let EpochLedger {
                hash,
                total_currency,
            } = ledger;

            (hash, Fp::zero).check(w);
            (total_currency, ClosedInterval::min_max).check(w);
        }

        (seed, Fp::zero).check(w);
        (start_checkpoint, Fp::zero).check(w);
        (lock_checkpoint, Fp::zero).check(w);
        (epoch_length, ClosedInterval::min_max).check(w);
    }
}

impl EpochData {
    pub fn gen() -> Self {
        let mut rng = rand::thread_rng();

        EpochData {
            ledger: EpochLedger {
                hash: OrIgnore::gen(|| Fp::rand(&mut rng)),
                total_currency: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
            },
            seed: OrIgnore::gen(|| Fp::rand(&mut rng)),
            start_checkpoint: OrIgnore::gen(|| Fp::rand(&mut rng)),
            lock_checkpoint: OrIgnore::gen(|| Fp::rand(&mut rng)),
            epoch_length: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
        }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L977>
#[derive(Debug, Clone, PartialEq)]
pub struct ZkAppPreconditions {
    pub snarked_ledger_hash: Hash<Fp>,
    pub blockchain_length: Numeric<Length>,
    pub min_window_density: Numeric<Length>,
    pub total_currency: Numeric<Amount>,
    pub global_slot_since_genesis: Numeric<Slot>,
    pub staking_epoch_data: EpochData,
    pub next_epoch_data: EpochData,
}

impl ZkAppPreconditions {
    pub fn zcheck<Ops: ZkappCheckOps>(
        &self,
        s: &ProtocolStateView,
        w: &mut Witness<Fp>,
    ) -> Boolean {
        let Self {
            snarked_ledger_hash,
            blockchain_length,
            min_window_density,
            total_currency,
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
        } = self;

        // NOTE: Here the 2nd element in the tuples is the default value of `OrIgnore`

        let epoch_data =
            |epoch_data: &EpochData, view: &protocol_state::EpochData<Fp>, w: &mut Witness<Fp>| {
                let EpochData {
                    ledger:
                        EpochLedger {
                            hash,
                            total_currency,
                        },
                    seed: _,
                    start_checkpoint,
                    lock_checkpoint,
                    epoch_length,
                } = epoch_data;
                // Reverse to match OCaml order of the list, while still executing `zcheck`
                // in correct order
                [
                    (epoch_length, ClosedInterval::min_max).zcheck::<Ops>(&view.epoch_length, w),
                    (lock_checkpoint, Fp::zero).zcheck::<Ops>(&view.lock_checkpoint, w),
                    (start_checkpoint, Fp::zero).zcheck::<Ops>(&view.start_checkpoint, w),
                    (total_currency, ClosedInterval::min_max)
                        .zcheck::<Ops>(&view.ledger.total_currency, w),
                    (hash, Fp::zero).zcheck::<Ops>(&view.ledger.hash, w),
                ]
            };

        let next_epoch_data = epoch_data(next_epoch_data, &s.next_epoch_data, w);
        let staking_epoch_data = epoch_data(staking_epoch_data, &s.staking_epoch_data, w);

        // Reverse to match OCaml order of the list, while still executing `zcheck`
        // in correct order
        let bools = [
            (global_slot_since_genesis, ClosedInterval::min_max)
                .zcheck::<Ops>(&s.global_slot_since_genesis, w),
            (total_currency, ClosedInterval::min_max).zcheck::<Ops>(&s.total_currency, w),
            (min_window_density, ClosedInterval::min_max).zcheck::<Ops>(&s.min_window_density, w),
            (blockchain_length, ClosedInterval::min_max).zcheck::<Ops>(&s.blockchain_length, w),
            (snarked_ledger_hash, Fp::zero).zcheck::<Ops>(&s.snarked_ledger_hash, w),
        ]
        .into_iter()
        .rev()
        .chain(staking_epoch_data.into_iter().rev())
        .chain(next_epoch_data.into_iter().rev());

        Ops::boolean_all(bools, w)
    }

    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_precondition.ml#L1303>
    pub fn accept() -> Self {
        let epoch_data = || EpochData {
            ledger: EpochLedger {
                hash: OrIgnore::Ignore,
                total_currency: OrIgnore::Ignore,
            },
            seed: OrIgnore::Ignore,
            start_checkpoint: OrIgnore::Ignore,
            lock_checkpoint: OrIgnore::Ignore,
            epoch_length: OrIgnore::Ignore,
        };

        Self {
            snarked_ledger_hash: OrIgnore::Ignore,
            blockchain_length: OrIgnore::Ignore,
            min_window_density: OrIgnore::Ignore,
            total_currency: OrIgnore::Ignore,
            global_slot_since_genesis: OrIgnore::Ignore,
            staking_epoch_data: epoch_data(),
            next_epoch_data: epoch_data(),
        }
    }
}

impl ToInputs for ZkAppPreconditions {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L1052>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let ZkAppPreconditions {
            snarked_ledger_hash,
            blockchain_length,
            min_window_density,
            total_currency,
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
        } = &self;

        inputs.append(&(snarked_ledger_hash, Fp::zero));
        inputs.append(&(blockchain_length, ClosedInterval::min_max));
        inputs.append(&(min_window_density, ClosedInterval::min_max));
        inputs.append(&(total_currency, ClosedInterval::min_max));
        inputs.append(&(global_slot_since_genesis, ClosedInterval::min_max));
        inputs.append(staking_epoch_data);
        inputs.append(next_epoch_data);
    }
}

impl ToFieldElements<Fp> for ZkAppPreconditions {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            snarked_ledger_hash,
            blockchain_length,
            min_window_density,
            total_currency,
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
        } = self;

        (snarked_ledger_hash, Fp::zero).to_field_elements(fields);
        (blockchain_length, ClosedInterval::min_max).to_field_elements(fields);
        (min_window_density, ClosedInterval::min_max).to_field_elements(fields);
        (total_currency, ClosedInterval::min_max).to_field_elements(fields);
        (global_slot_since_genesis, ClosedInterval::min_max).to_field_elements(fields);
        staking_epoch_data.to_field_elements(fields);
        next_epoch_data.to_field_elements(fields);
    }
}

impl Check<Fp> for ZkAppPreconditions {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            snarked_ledger_hash,
            blockchain_length,
            min_window_density,
            total_currency,
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
        } = self;

        (snarked_ledger_hash, Fp::zero).check(w);
        (blockchain_length, ClosedInterval::min_max).check(w);
        (min_window_density, ClosedInterval::min_max).check(w);
        (total_currency, ClosedInterval::min_max).check(w);
        (global_slot_since_genesis, ClosedInterval::min_max).check(w);
        staking_epoch_data.check(w);
        next_epoch_data.check(w);
    }
}

/// <https://github.com/MinaProtocol/mina/blob/da6ba9a52e71d03ec6b6803b01f6d249eebc1ccb/src/lib/mina_base/zkapp_basic.ml#L401>
fn invalid_public_key() -> CompressedPubKey {
    CompressedPubKey {
        x: Fp::zero(),
        is_odd: false,
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L478>
#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub balance: Numeric<Balance>,
    pub nonce: Numeric<Nonce>,
    pub receipt_chain_hash: Hash<Fp>, // TODO: Should be type `ReceiptChainHash`
    pub delegate: EqData<CompressedPubKey>,
    pub state: [EqData<Fp>; 8],
    pub action_state: EqData<Fp>,
    pub proved_state: EqData<bool>,
    pub is_new: EqData<bool>,
}

impl Account {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L525>
    pub fn accept() -> Self {
        Self {
            balance: Numeric::Ignore,
            nonce: Numeric::Ignore,
            receipt_chain_hash: Hash::Ignore,
            delegate: EqData::Ignore,
            state: std::array::from_fn(|_| EqData::Ignore),
            action_state: EqData::Ignore,
            proved_state: EqData::Ignore,
            is_new: EqData::Ignore,
        }
    }
}

impl Account {
    fn zchecks<Ops: ZkappCheckOps>(
        &self,
        account: &crate::Account,
        new_account: Boolean,
        w: &mut Witness<Fp>,
    ) -> Vec<(TransactionFailure, Boolean)> {
        use TransactionFailure::*;

        let Self {
            balance,
            nonce,
            receipt_chain_hash,
            delegate,
            state,
            action_state,
            proved_state,
            is_new,
        } = self;

        let zkapp_account = account.zkapp_or_empty();
        let is_new = is_new.map(ToBoolean::to_boolean);
        let proved_state = proved_state.map(ToBoolean::to_boolean);

        // NOTE: Here we need to execute all `zcheck` in the exact same order than OCaml
        // so we execute them in reverse order (compared to OCaml): OCaml evaluates from right
        // to left.
        // We then have to reverse the resulting vector, to match OCaml resulting list.

        // NOTE 2: Here the 2nd element in the tuples is the default value of `OrIgnore`
        let mut checks: Vec<(TransactionFailure, _)> = [
            (
                AccountIsNewPreconditionUnsatisfied,
                (&is_new, || Boolean::False).zcheck::<Ops>(&new_account, w),
            ),
            (
                AccountProvedStatePreconditionUnsatisfied,
                (&proved_state, || Boolean::False)
                    .zcheck::<Ops>(&zkapp_account.proved_state.to_boolean(), w),
            ),
        ]
        .into_iter()
        .chain({
            let bools = state
                .iter()
                .zip(&zkapp_account.app_state)
                .enumerate()
                // Reversed to enforce right-to-left order application of `f` like in OCaml
                .rev()
                .map(|(i, (s, account_s))| {
                    let b = (s, Fp::zero).zcheck::<Ops>(account_s, w);
                    (AccountAppStatePreconditionUnsatisfied(i as u64), b)
                })
                .collect::<Vec<_>>();
            // Not reversed again because we are constructing these results in
            // reverse order to match the OCaml evaluation order.
            bools.into_iter()
        })
        .chain([
            {
                let bools: Vec<_> = zkapp_account
                    .action_state
                    .iter()
                    // Reversed to enforce right-to-left order application of `f` like in OCaml
                    .rev()
                    .map(|account_s| {
                        (action_state, ZkAppAccount::empty_action_state).zcheck::<Ops>(account_s, w)
                    })
                    .collect();
                (
                    AccountActionStatePreconditionUnsatisfied,
                    Ops::boolean_any(bools, w),
                )
            },
            (
                AccountDelegatePreconditionUnsatisfied,
                (delegate, CompressedPubKey::empty).zcheck::<Ops>(&*account.delegate_or_empty(), w),
            ),
            (
                AccountReceiptChainHashPreconditionUnsatisfied,
                (receipt_chain_hash, Fp::zero).zcheck::<Ops>(&account.receipt_chain_hash.0, w),
            ),
            (
                AccountNoncePreconditionUnsatisfied,
                (nonce, ClosedInterval::min_max).zcheck::<Ops>(&account.nonce, w),
            ),
            (
                AccountBalancePreconditionUnsatisfied,
                (balance, ClosedInterval::min_max).zcheck::<Ops>(&account.balance, w),
            ),
        ])
        .collect::<Vec<_>>();

        checks.reverse();
        checks
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L613>
#[derive(Debug, Clone, PartialEq)]
pub struct AccountPreconditions(pub Account);

impl ToInputs for AccountPreconditions {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L635>
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L568>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Account {
            balance,
            nonce,
            receipt_chain_hash,
            delegate,
            state,
            action_state,
            proved_state,
            is_new,
        } = &self.0;

        inputs.append(&(balance, ClosedInterval::min_max));
        inputs.append(&(nonce, ClosedInterval::min_max));
        inputs.append(&(receipt_chain_hash, Fp::zero));
        inputs.append(&(delegate, CompressedPubKey::empty));
        for s in state.iter() {
            inputs.append(&(s, Fp::zero));
        }
        // <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L168>
        inputs.append(&(action_state, ZkAppAccount::empty_action_state));
        inputs.append(&(proved_state, || false));
        inputs.append(&(is_new, || false));
    }
}

impl ToFieldElements<Fp> for AccountPreconditions {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Account {
            balance,
            nonce,
            receipt_chain_hash,
            delegate,
            state,
            action_state,
            proved_state,
            is_new,
        } = &self.0;

        (balance, ClosedInterval::min_max).to_field_elements(fields);
        (nonce, ClosedInterval::min_max).to_field_elements(fields);
        (receipt_chain_hash, Fp::zero).to_field_elements(fields);
        (delegate, CompressedPubKey::empty).to_field_elements(fields);
        state.iter().for_each(|s| {
            (s, Fp::zero).to_field_elements(fields);
        });
        (action_state, ZkAppAccount::empty_action_state).to_field_elements(fields);
        (proved_state, || false).to_field_elements(fields);
        (is_new, || false).to_field_elements(fields);
    }
}

impl Check<Fp> for AccountPreconditions {
    fn check(&self, w: &mut Witness<Fp>) {
        let Account {
            balance,
            nonce,
            receipt_chain_hash,
            delegate,
            state,
            action_state,
            proved_state,
            is_new,
        } = &self.0;

        (balance, ClosedInterval::min_max).check(w);
        (nonce, ClosedInterval::min_max).check(w);
        (receipt_chain_hash, Fp::zero).check(w);
        (delegate, CompressedPubKey::empty).check(w);
        state.iter().for_each(|s| {
            (s, Fp::zero).check(w);
        });
        (action_state, ZkAppAccount::empty_action_state).check(w);
        (proved_state, || false).check(w);
        (is_new, || false).check(w);
    }
}

impl AccountPreconditions {
    pub fn with_nonce(nonce: Nonce) -> Self {
        use OrIgnore::{Check, Ignore};
        AccountPreconditions(Account {
            balance: Ignore,
            nonce: Check(ClosedInterval {
                lower: nonce,
                upper: nonce,
            }),
            receipt_chain_hash: Ignore,
            delegate: Ignore,
            state: std::array::from_fn(|_| EqData::Ignore),
            action_state: Ignore,
            proved_state: Ignore,
            is_new: Ignore,
        })
    }

    pub fn nonce(&self) -> Numeric<Nonce> {
        self.0.nonce.clone()
    }

    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L635>
    pub fn to_full(&self) -> MyCow<'_, Account> {
        MyCow::Borrow(&self.0)
    }

    pub fn zcheck<Ops, Fun>(
        &self,
        new_account: Boolean,
        account: &crate::Account,
        mut check: Fun,
        w: &mut Witness<Fp>,
    ) where
        Ops: ZkappCheckOps,
        Fun: FnMut(TransactionFailure, Boolean, &mut Witness<Fp>),
    {
        let this = self.to_full();
        for (failure, passed) in this.zchecks::<Ops>(account, new_account, w) {
            check(failure, passed, w);
        }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L758>
#[derive(Debug, Clone, PartialEq)]
pub struct Preconditions {
    pub network: ZkAppPreconditions,
    pub account: AccountPreconditions,
    pub valid_while: Numeric<Slot>,
}

#[cfg(feature = "fuzzing")]
impl Preconditions {
    pub fn new(
        network: ZkAppPreconditions,
        account: AccountPreconditions,
        valid_while: Numeric<Slot>,
    ) -> Self {
        Self {
            network,
            account,
            valid_while,
        }
    }

    pub fn network_mut(&mut self) -> &mut ZkAppPreconditions {
        &mut self.network
    }
}

impl ToFieldElements<Fp> for Preconditions {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            network,
            account,
            valid_while,
        } = self;

        network.to_field_elements(fields);
        account.to_field_elements(fields);
        (valid_while, ClosedInterval::min_max).to_field_elements(fields);
    }
}

impl Check<Fp> for Preconditions {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            network,
            account,
            valid_while,
        } = self;

        network.check(w);
        account.check(w);
        (valid_while, ClosedInterval::min_max).check(w);
    }
}

impl ToInputs for Preconditions {
    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1148>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            network,
            account,
            valid_while,
        } = self;

        inputs.append(network);
        inputs.append(account);
        inputs.append(&(valid_while, ClosedInterval::min_max));
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L27>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationKind {
    NoneGiven,
    Signature,
    Proof(Fp), // hash
}

impl AuthorizationKind {
    pub fn vk_hash(&self) -> Fp {
        match self {
            AuthorizationKind::NoneGiven | AuthorizationKind::Signature => {
                VerificationKey::dummy().hash()
            }
            AuthorizationKind::Proof(hash) => *hash,
        }
    }

    pub fn is_proved(&self) -> bool {
        match self {
            AuthorizationKind::Proof(_) => true,
            AuthorizationKind::NoneGiven => false,
            AuthorizationKind::Signature => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            AuthorizationKind::Proof(_) => false,
            AuthorizationKind::NoneGiven => false,
            AuthorizationKind::Signature => true,
        }
    }

    fn to_structured(&self) -> ([bool; 2], Fp) {
        // bits: [is_signed, is_proved]
        let bits = match self {
            AuthorizationKind::NoneGiven => [false, false],
            AuthorizationKind::Signature => [true, false],
            AuthorizationKind::Proof(_) => [false, true],
        };
        let field = self.vk_hash();
        (bits, field)
    }
}

impl ToInputs for AuthorizationKind {
    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L142>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let (bits, field) = self.to_structured();

        for bit in bits {
            inputs.append_bool(bit);
        }
        inputs.append_field(field);
    }
}

impl ToFieldElements<Fp> for AuthorizationKind {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        self.to_structured().to_field_elements(fields);
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1311>
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub public_key: CompressedPubKey,
    pub token_id: TokenId,
    pub update: Update,
    pub balance_change: Signed<Amount>,
    pub increment_nonce: bool,
    pub events: Events,
    pub actions: Actions,
    pub call_data: Fp,
    pub preconditions: Preconditions,
    pub use_full_commitment: bool,
    pub implicit_account_creation_fee: bool,
    pub may_use_token: MayUseToken,
    pub authorization_kind: AuthorizationKind,
}

impl ToInputs for Body {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L1297>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            public_key,
            token_id,
            update,
            balance_change,
            increment_nonce,
            events,
            actions,
            call_data,
            preconditions,
            use_full_commitment,
            implicit_account_creation_fee,
            may_use_token,
            authorization_kind,
        } = self;

        inputs.append(public_key);
        inputs.append(token_id);

        // `Body::update`
        {
            let Update {
                app_state,
                delegate,
                verification_key,
                permissions,
                zkapp_uri,
                token_symbol,
                timing,
                voting_for,
            } = update;

            for state in app_state {
                inputs.append(&(state, Fp::zero));
            }

            inputs.append(&(delegate, CompressedPubKey::empty));
            inputs.append(&(&verification_key.map(|w| w.hash()), Fp::zero));
            inputs.append(&(permissions, Permissions::empty));
            inputs.append(&(&zkapp_uri.map(Some), || Option::<&ZkAppUri>::None));
            inputs.append(&(token_symbol, TokenSymbol::default));
            inputs.append(&(timing, Timing::dummy));
            inputs.append(&(voting_for, VotingFor::dummy));
        }

        inputs.append(balance_change);
        inputs.append(increment_nonce);
        inputs.append(events);
        inputs.append(actions);
        inputs.append(call_data);
        inputs.append(preconditions);
        inputs.append(use_full_commitment);
        inputs.append(implicit_account_creation_fee);
        inputs.append(may_use_token);
        inputs.append(authorization_kind);
    }
}

impl ToFieldElements<Fp> for Body {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            public_key,
            token_id,
            update,
            balance_change,
            increment_nonce,
            events,
            actions,
            call_data,
            preconditions,
            use_full_commitment,
            implicit_account_creation_fee,
            may_use_token,
            authorization_kind,
        } = self;

        public_key.to_field_elements(fields);
        token_id.to_field_elements(fields);
        update.to_field_elements(fields);
        balance_change.to_field_elements(fields);
        increment_nonce.to_field_elements(fields);
        events.to_field_elements(fields);
        actions.to_field_elements(fields);
        call_data.to_field_elements(fields);
        preconditions.to_field_elements(fields);
        use_full_commitment.to_field_elements(fields);
        implicit_account_creation_fee.to_field_elements(fields);
        may_use_token.to_field_elements(fields);
        authorization_kind.to_field_elements(fields);
    }
}

impl Check<Fp> for Body {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            public_key: _,
            token_id: _,
            update:
                Update {
                    app_state: _,
                    delegate: _,
                    verification_key: _,
                    permissions,
                    zkapp_uri: _,
                    token_symbol,
                    timing,
                    voting_for: _,
                },
            balance_change,
            increment_nonce: _,
            events: _,
            actions: _,
            call_data: _,
            preconditions,
            use_full_commitment: _,
            implicit_account_creation_fee: _,
            may_use_token,
            authorization_kind: _,
        } = self;

        (permissions, Permissions::empty).check(w);
        (token_symbol, TokenSymbol::default).check(w);
        (timing, Timing::dummy).check(w);
        balance_change.check(w);

        preconditions.check(w);
        may_use_token.check(w);
    }
}

impl Body {
    pub fn account_id(&self) -> AccountId {
        let Self {
            public_key,
            token_id,
            ..
        } = self;
        AccountId::create(public_key.clone(), token_id.clone())
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1284>
#[derive(Debug, Clone, PartialEq)]
pub struct BodySimple {
    pub public_key: CompressedPubKey,
    pub token_id: TokenId,
    pub update: Update,
    pub balance_change: Signed<Amount>,
    pub increment_nonce: bool,
    pub events: Events,
    pub actions: Actions,
    pub call_data: Fp,
    pub call_depth: usize,
    pub preconditions: Preconditions,
    pub use_full_commitment: bool,
    pub implicit_account_creation_fee: bool,
    pub may_use_token: MayUseToken,
    pub authorization_kind: AuthorizationKind,
}

/// Notes:
/// The type in OCaml is this one:
/// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/pickles/proof.ml#L401>
///
/// For now we use the type from `mina_p2p_messages`, but we need to use our own.
/// Lots of inner types are (BigInt, Bigint) which should be replaced with `Pallas<_>` etc.
/// Also, in OCaml it has custom `{to/from}_binable` implementation.
///
/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/pickles/pickles_intf.ml#L316>
pub type SideLoadedProof = Arc<mina_p2p_messages::v2::PicklesProofProofsVerifiedMaxStableV2>;

/// Authorization methods for zkApp account updates.
///
/// Defines how an account update is authorized to modify an account's state.
///
/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/control.ml#L11>
#[derive(Clone, PartialEq)]
pub enum Control {
    /// Verified by a zero-knowledge proof against the account's verification
    /// key.
    Proof(SideLoadedProof),
    /// Signed by the account's private key.
    Signature(Signature),
    /// No authorization (only valid for certain operations).
    NoneGiven,
}

impl std::fmt::Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proof(_) => f.debug_tuple("Proof").field(&"_").finish(),
            Self::Signature(arg0) => f.debug_tuple("Signature").field(arg0).finish(),
            Self::NoneGiven => write!(f, "NoneGiven"),
        }
    }
}

impl Control {
    /// <https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_base/control.ml#L81>
    pub fn tag(&self) -> crate::ControlTag {
        match self {
            Control::Proof(_) => crate::ControlTag::Proof,
            Control::Signature(_) => crate::ControlTag::Signature,
            Control::NoneGiven => crate::ControlTag::NoneGiven,
        }
    }

    pub fn dummy_of_tag(tag: ControlTag) -> Self {
        match tag {
            ControlTag::Proof => Self::Proof(dummy::sideloaded_proof()),
            ControlTag::Signature => Self::Signature(Signature::dummy()),
            ControlTag::NoneGiven => Self::NoneGiven,
        }
    }

    pub fn dummy(&self) -> Self {
        Self::dummy_of_tag(self.tag())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MayUseToken {
    /// No permission to use any token other than the default Mina
    /// token
    No,
    /// Has permission to use the token owned by the direct parent of
    /// this account update, which may be inherited by child account
    /// updates.
    ParentsOwnToken,
    /// Inherit the token permission available to the parent.
    InheritFromParent,
}

impl MayUseToken {
    pub fn parents_own_token(&self) -> bool {
        matches!(self, Self::ParentsOwnToken)
    }

    pub fn inherit_from_parent(&self) -> bool {
        matches!(self, Self::InheritFromParent)
    }

    fn to_bits(&self) -> [bool; 2] {
        // [ parents_own_token; inherit_from_parent ]
        match self {
            MayUseToken::No => [false, false],
            MayUseToken::ParentsOwnToken => [true, false],
            MayUseToken::InheritFromParent => [false, true],
        }
    }
}

impl ToInputs for MayUseToken {
    fn to_inputs(&self, inputs: &mut Inputs) {
        for bit in self.to_bits() {
            inputs.append_bool(bit);
        }
    }
}

impl ToFieldElements<Fp> for MayUseToken {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        for bit in self.to_bits() {
            bit.to_field_elements(fields);
        }
    }
}

impl Check<Fp> for MayUseToken {
    fn check(&self, w: &mut Witness<Fp>) {
        use crate::proofs::field::field;

        let [parents_own_token, inherit_from_parent] = self.to_bits();
        let [parents_own_token, inherit_from_parent] = [
            parents_own_token.to_boolean(),
            inherit_from_parent.to_boolean(),
        ];

        let sum = parents_own_token.to_field::<Fp>() + inherit_from_parent.to_field::<Fp>();
        let _sum_squared = field::mul(sum, sum, w);
    }
}

pub struct CheckAuthorizationResult<Bool> {
    pub proof_verifies: Bool,
    pub signature_verifies: Bool,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1437>
pub type AccountUpdate = AccountUpdateSkeleton<Body>;

#[derive(Debug, Clone, PartialEq)]
pub struct AccountUpdateSkeleton<Body> {
    pub body: Body,
    pub authorization: Control,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1395>
#[derive(Debug, Clone, PartialEq)]
pub struct AccountUpdateSimple {
    pub body: BodySimple,
    pub authorization: Control,
}

impl ToInputs for AccountUpdate {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L1297>
    fn to_inputs(&self, inputs: &mut Inputs) {
        // Only the body is used
        let Self {
            body,
            authorization: _,
        } = self;

        inputs.append(body);
    }
}

impl AccountUpdate {
    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L1538>
    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1465>
    pub fn of_fee_payer(fee_payer: FeePayer) -> Self {
        let FeePayer {
            body:
                FeePayerBody {
                    public_key,
                    fee,
                    valid_until,
                    nonce,
                },
            authorization,
        } = fee_payer;

        Self {
            body: Body {
                public_key,
                token_id: TokenId::default(),
                update: Update::noop(),
                balance_change: Signed {
                    magnitude: Amount::of_fee(&fee),
                    sgn: Sgn::Neg,
                },
                increment_nonce: true,
                events: Events::empty(),
                actions: Actions::empty(),
                call_data: Fp::zero(),
                preconditions: Preconditions {
                    network: {
                        let mut network = ZkAppPreconditions::accept();

                        let valid_util = valid_until.unwrap_or_else(Slot::max);
                        network.global_slot_since_genesis = OrIgnore::Check(ClosedInterval {
                            lower: Slot::zero(),
                            upper: valid_util,
                        });

                        network
                    },
                    account: AccountPreconditions::with_nonce(nonce),
                    valid_while: Numeric::Ignore,
                },
                use_full_commitment: true,
                authorization_kind: AuthorizationKind::Signature,
                implicit_account_creation_fee: true,
                may_use_token: MayUseToken::No,
            },
            authorization: Control::Signature(authorization),
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/account_update.ml#L1535>
    pub fn account_id(&self) -> AccountId {
        AccountId::new(self.body.public_key.clone(), self.body.token_id.clone())
    }

    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L1327>
    pub fn digest(&self) -> Fp {
        self.hash_with_param(mina_core::NetworkConfig::global().account_update_hash_param)
    }

    pub fn timing(&self) -> SetOrKeep<Timing> {
        self.body.update.timing.clone()
    }

    pub fn may_use_parents_own_token(&self) -> bool {
        self.body.may_use_token.parents_own_token()
    }

    pub fn may_use_token_inherited_from_parent(&self) -> bool {
        self.body.may_use_token.inherit_from_parent()
    }

    pub fn public_key(&self) -> CompressedPubKey {
        self.body.public_key.clone()
    }

    pub fn token_id(&self) -> TokenId {
        self.body.token_id.clone()
    }

    pub fn increment_nonce(&self) -> bool {
        self.body.increment_nonce
    }

    pub fn implicit_account_creation_fee(&self) -> bool {
        self.body.implicit_account_creation_fee
    }

    // commitment and calls argument are ignored here, only used in the transaction snark
    pub fn check_authorization(
        &self,
        _will_succeed: bool,
        _commitment: Fp,
        _calls: CallForest<AccountUpdate>,
    ) -> CheckAuthorizationResult<bool> {
        match self.authorization {
            Control::Signature(_) => CheckAuthorizationResult {
                proof_verifies: false,
                signature_verifies: true,
            },
            Control::Proof(_) => CheckAuthorizationResult {
                proof_verifies: true,
                signature_verifies: false,
            },
            Control::NoneGiven => CheckAuthorizationResult {
                proof_verifies: false,
                signature_verifies: false,
            },
        }
    }

    pub fn permissions(&self) -> SetOrKeep<Permissions<AuthRequired>> {
        self.body.update.permissions.clone()
    }

    pub fn app_state(&self) -> [SetOrKeep<Fp>; 8] {
        self.body.update.app_state.clone()
    }

    pub fn zkapp_uri(&self) -> SetOrKeep<ZkAppUri> {
        self.body.update.zkapp_uri.clone()
    }

    /*
    pub fn token_symbol(&self) -> SetOrKeep<[u8; 6]> {
        self.body.update.token_symbol.clone()
    }
    */

    pub fn token_symbol(&self) -> SetOrKeep<TokenSymbol> {
        self.body.update.token_symbol.clone()
    }

    pub fn delegate(&self) -> SetOrKeep<CompressedPubKey> {
        self.body.update.delegate.clone()
    }

    pub fn voting_for(&self) -> SetOrKeep<VotingFor> {
        self.body.update.voting_for.clone()
    }

    pub fn verification_key(&self) -> SetOrKeep<VerificationKeyWire> {
        self.body.update.verification_key.clone()
    }

    pub fn valid_while_precondition(&self) -> OrIgnore<ClosedInterval<Slot>> {
        self.body.preconditions.valid_while.clone()
    }

    pub fn actions(&self) -> Actions {
        self.body.actions.clone()
    }

    pub fn balance_change(&self) -> Signed<Amount> {
        self.body.balance_change
    }
    pub fn use_full_commitment(&self) -> bool {
        self.body.use_full_commitment
    }

    pub fn protocol_state_precondition(&self) -> ZkAppPreconditions {
        self.body.preconditions.network.clone()
    }

    pub fn account_precondition(&self) -> AccountPreconditions {
        self.body.preconditions.account.clone()
    }

    pub fn is_proved(&self) -> bool {
        match &self.body.authorization_kind {
            AuthorizationKind::Proof(_) => true,
            AuthorizationKind::Signature | AuthorizationKind::NoneGiven => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match &self.body.authorization_kind {
            AuthorizationKind::Signature => true,
            AuthorizationKind::Proof(_) | AuthorizationKind::NoneGiven => false,
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction_logic/mina_transaction_logic.ml#L1708>
    pub fn verification_key_hash(&self) -> Option<Fp> {
        match &self.body.authorization_kind {
            AuthorizationKind::Proof(vk_hash) => Some(*vk_hash),
            _ => None,
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1333>
    pub fn of_simple(simple: &AccountUpdateSimple) -> Self {
        let AccountUpdateSimple {
            body:
                BodySimple {
                    public_key,
                    token_id,
                    update,
                    balance_change,
                    increment_nonce,
                    events,
                    actions,
                    call_data,
                    call_depth: _,
                    preconditions,
                    use_full_commitment,
                    implicit_account_creation_fee,
                    may_use_token,
                    authorization_kind,
                },
            authorization,
        } = simple.clone();

        Self {
            body: Body {
                public_key,
                token_id,
                update,
                balance_change,
                increment_nonce,
                events,
                actions,
                call_data,
                preconditions,
                use_full_commitment,
                implicit_account_creation_fee,
                may_use_token,
                authorization_kind,
            },
            authorization,
        }
    }

    /// Usage: Random `AccountUpdate` to compare hashes with OCaml
    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();
        let rng = &mut rng;

        Self {
            body: Body {
                public_key: gen_compressed(),
                token_id: TokenId(Fp::rand(rng)),
                update: Update {
                    app_state: std::array::from_fn(|_| SetOrKeep::gen(|| Fp::rand(rng))),
                    delegate: SetOrKeep::gen(gen_compressed),
                    verification_key: SetOrKeep::gen(VerificationKeyWire::gen),
                    permissions: SetOrKeep::gen(|| {
                        let auth_tag = [
                            ControlTag::NoneGiven,
                            ControlTag::Proof,
                            ControlTag::Signature,
                        ]
                        .choose(rng)
                        .unwrap();

                        Permissions::gen(*auth_tag)
                    }),
                    zkapp_uri: SetOrKeep::gen(ZkAppUri::gen),
                    token_symbol: SetOrKeep::gen(TokenSymbol::gen),
                    timing: SetOrKeep::gen(|| Timing {
                        initial_minimum_balance: rng.gen(),
                        cliff_time: rng.gen(),
                        cliff_amount: rng.gen(),
                        vesting_period: rng.gen(),
                        vesting_increment: rng.gen(),
                    }),
                    voting_for: SetOrKeep::gen(|| VotingFor(Fp::rand(rng))),
                },
                balance_change: Signed::gen(),
                increment_nonce: rng.gen(),
                events: Events(gen_events()),
                actions: Actions(gen_events()),
                call_data: Fp::rand(rng),
                preconditions: Preconditions {
                    network: ZkAppPreconditions {
                        snarked_ledger_hash: OrIgnore::gen(|| Fp::rand(rng)),
                        blockchain_length: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                        min_window_density: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                        total_currency: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                        global_slot_since_genesis: OrIgnore::gen(|| {
                            ClosedInterval::gen(|| rng.gen())
                        }),
                        staking_epoch_data: EpochData::gen(),
                        next_epoch_data: EpochData::gen(),
                    },
                    account: AccountPreconditions(Account {
                        balance: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                        nonce: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                        receipt_chain_hash: OrIgnore::gen(|| Fp::rand(rng)),
                        delegate: OrIgnore::gen(gen_compressed),
                        state: std::array::from_fn(|_| OrIgnore::gen(|| Fp::rand(rng))),
                        action_state: OrIgnore::gen(|| Fp::rand(rng)),
                        proved_state: OrIgnore::gen(|| rng.gen()),
                        is_new: OrIgnore::gen(|| rng.gen()),
                    }),
                    valid_while: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                },
                use_full_commitment: rng.gen(),
                implicit_account_creation_fee: rng.gen(),
                may_use_token: {
                    match MayUseToken::No {
                        MayUseToken::No => (),
                        MayUseToken::ParentsOwnToken => (),
                        MayUseToken::InheritFromParent => (),
                    };

                    [
                        MayUseToken::No,
                        MayUseToken::InheritFromParent,
                        MayUseToken::ParentsOwnToken,
                    ]
                    .choose(rng)
                    .cloned()
                    .unwrap()
                },
                authorization_kind: {
                    match AuthorizationKind::NoneGiven {
                        AuthorizationKind::NoneGiven => (),
                        AuthorizationKind::Signature => (),
                        AuthorizationKind::Proof(_) => (),
                    };

                    [
                        AuthorizationKind::NoneGiven,
                        AuthorizationKind::Signature,
                        AuthorizationKind::Proof(Fp::rand(rng)),
                    ]
                    .choose(rng)
                    .cloned()
                    .unwrap()
                },
            },
            authorization: {
                match Control::NoneGiven {
                    Control::Proof(_) => (),
                    Control::Signature(_) => (),
                    Control::NoneGiven => (),
                };

                match rng.gen_range(0..3) {
                    0 => Control::NoneGiven,
                    1 => Control::Signature(Signature::dummy()),
                    _ => Control::Proof(dummy::sideloaded_proof()),
                }
            },
        }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L49>
#[derive(Debug, Clone, PartialEq)]
pub struct Tree<AccUpdate: Clone + AccountUpdateRef> {
    pub account_update: AccUpdate,
    pub account_update_digest: MutableFp,
    pub calls: CallForest<AccUpdate>,
}

impl<AccUpdate: Clone + AccountUpdateRef> Tree<AccUpdate> {
    // TODO: Cache this result somewhere ?
    pub fn digest(&self) -> Fp {
        let stack_hash = match self.calls.0.first() {
            Some(e) => e.stack_hash.get().expect("Must call `ensure_hashed`"),
            None => Fp::zero(),
        };
        let account_update_digest = self.account_update_digest.get().unwrap();
        hash_with_kimchi(
            &MINA_ACCOUNT_UPDATE_NODE,
            &[account_update_digest, stack_hash],
        )
    }

    fn fold<F>(&self, init: Vec<AccountId>, f: &mut F) -> Vec<AccountId>
    where
        F: FnMut(Vec<AccountId>, &AccUpdate) -> Vec<AccountId>,
    {
        self.calls.fold(f(init, &self.account_update), f)
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/with_stack_hash.ml#L6>
#[derive(Debug, Clone)]
pub struct WithStackHash<AccUpdate: Clone + AccountUpdateRef> {
    pub elt: Tree<AccUpdate>,
    pub stack_hash: MutableFp,
}

impl<AccUpdate: Clone + AccountUpdateRef + PartialEq> PartialEq for WithStackHash<AccUpdate> {
    fn eq(&self, other: &Self) -> bool {
        self.elt == other.elt && self.stack_hash == other.stack_hash
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L345>
#[derive(Debug, Clone, PartialEq)]
pub struct CallForest<AccUpdate: Clone + AccountUpdateRef>(pub Vec<WithStackHash<AccUpdate>>);

impl<Data: Clone + AccountUpdateRef> Default for CallForest<Data> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct CallForestContext {
    caller: TokenId,
    this: TokenId,
}

pub trait AccountUpdateRef {
    fn account_update_ref(&self) -> &AccountUpdate;
}
impl AccountUpdateRef for AccountUpdate {
    fn account_update_ref(&self) -> &AccountUpdate {
        self
    }
}
impl<T> AccountUpdateRef for (AccountUpdate, T) {
    fn account_update_ref(&self) -> &AccountUpdate {
        let (this, _) = self;
        this
    }
}
impl AccountUpdateRef for AccountUpdateSimple {
    fn account_update_ref(&self) -> &AccountUpdate {
        // AccountUpdateSimple are first converted into `AccountUpdate`
        unreachable!()
    }
}

impl<AccUpdate: Clone + AccountUpdateRef> CallForest<AccUpdate> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn empty() -> Self {
        Self::new()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // In OCaml push/pop to the head is cheap because they work with lists.
    // In Rust we use vectors so we will push/pop to the tail.
    // To work with the elements as if they were in the original order we need to iterate backwards
    pub fn iter(&self) -> impl Iterator<Item = &WithStackHash<AccUpdate>> {
        self.0.iter() //.rev()
    }
    // Warning: Update this if we ever change the order
    pub fn first(&self) -> Option<&WithStackHash<AccUpdate>> {
        self.0.first()
    }
    // Warning: Update this if we ever change the order
    pub fn tail(&self) -> Option<&[WithStackHash<AccUpdate>]> {
        self.0.get(1..)
    }

    pub fn hash(&self) -> Fp {
        self.ensure_hashed();
        /*
        for x in self.0.iter() {
            println!("hash: {:?}", x.stack_hash);
        }
        */

        if let Some(x) = self.first() {
            x.stack_hash.get().unwrap() // Never fail, we called `ensure_hashed`
        } else {
            Fp::zero()
        }
    }

    fn cons_tree(&self, tree: Tree<AccUpdate>) -> Self {
        self.ensure_hashed();

        let hash = tree.digest();
        let h_tl = self.hash();

        let stack_hash = hash_with_kimchi(&MINA_ACCOUNT_UPDATE_CONS, &[hash, h_tl]);
        let node = WithStackHash::<AccUpdate> {
            elt: tree,
            stack_hash: MutableFp::new(stack_hash),
        };
        let mut forest = Vec::with_capacity(self.0.len() + 1);
        forest.push(node);
        forest.extend(self.0.iter().cloned());

        Self(forest)
    }

    pub fn pop_exn(&self) -> ((AccUpdate, CallForest<AccUpdate>), CallForest<AccUpdate>) {
        if self.0.is_empty() {
            panic!()
        }

        let Tree::<AccUpdate> {
            account_update,
            calls,
            ..
        } = self.0[0].elt.clone();
        (
            (account_update, calls),
            CallForest(Vec::from_iter(self.0[1..].iter().cloned())),
        )
    }

    /// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L68>
    fn fold_impl<'a, A, F>(&'a self, init: A, fun: &mut F) -> A
    where
        F: FnMut(A, &'a AccUpdate) -> A,
    {
        let mut accum = init;
        for elem in self.iter() {
            accum = fun(accum, &elem.elt.account_update);
            accum = elem.elt.calls.fold_impl(accum, fun);
        }
        accum
    }

    pub fn fold<'a, A, F>(&'a self, init: A, mut fun: F) -> A
    where
        F: FnMut(A, &'a AccUpdate) -> A,
    {
        self.fold_impl(init, &mut fun)
    }

    pub fn exists<'a, F>(&'a self, mut fun: F) -> bool
    where
        F: FnMut(&'a AccUpdate) -> bool,
    {
        self.fold(false, |acc, x| acc || fun(x))
    }

    fn map_to_impl<F, AnotherAccUpdate: Clone + AccountUpdateRef>(
        &self,
        fun: &F,
    ) -> CallForest<AnotherAccUpdate>
    where
        F: Fn(&AccUpdate) -> AnotherAccUpdate,
    {
        CallForest::<AnotherAccUpdate>(
            self.iter()
                .map(|item| WithStackHash::<AnotherAccUpdate> {
                    elt: Tree::<AnotherAccUpdate> {
                        account_update: fun(&item.elt.account_update),
                        account_update_digest: item.elt.account_update_digest.clone(),
                        calls: item.elt.calls.map_to_impl(fun),
                    },
                    stack_hash: item.stack_hash.clone(),
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn map_to<F, AnotherAccUpdate: Clone + AccountUpdateRef>(
        &self,
        fun: F,
    ) -> CallForest<AnotherAccUpdate>
    where
        F: Fn(&AccUpdate) -> AnotherAccUpdate,
    {
        self.map_to_impl(&fun)
    }

    fn map_with_trees_to_impl<F, AnotherAccUpdate: Clone + AccountUpdateRef>(
        &self,
        fun: &F,
    ) -> CallForest<AnotherAccUpdate>
    where
        F: Fn(&AccUpdate, &Tree<AccUpdate>) -> AnotherAccUpdate,
    {
        CallForest::<AnotherAccUpdate>(
            self.iter()
                .map(|item| {
                    let account_update = fun(&item.elt.account_update, &item.elt);

                    WithStackHash::<AnotherAccUpdate> {
                        elt: Tree::<AnotherAccUpdate> {
                            account_update,
                            account_update_digest: item.elt.account_update_digest.clone(),
                            calls: item.elt.calls.map_with_trees_to_impl(fun),
                        },
                        stack_hash: item.stack_hash.clone(),
                    }
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn map_with_trees_to<F, AnotherAccUpdate: Clone + AccountUpdateRef>(
        &self,
        fun: F,
    ) -> CallForest<AnotherAccUpdate>
    where
        F: Fn(&AccUpdate, &Tree<AccUpdate>) -> AnotherAccUpdate,
    {
        self.map_with_trees_to_impl(&fun)
    }

    fn try_map_to_impl<F, E, AnotherAccUpdate: Clone + AccountUpdateRef>(
        &self,
        fun: &mut F,
    ) -> Result<CallForest<AnotherAccUpdate>, E>
    where
        F: FnMut(&AccUpdate) -> Result<AnotherAccUpdate, E>,
    {
        Ok(CallForest::<AnotherAccUpdate>(
            self.iter()
                .map(|item| {
                    Ok(WithStackHash::<AnotherAccUpdate> {
                        elt: Tree::<AnotherAccUpdate> {
                            account_update: fun(&item.elt.account_update)?,
                            account_update_digest: item.elt.account_update_digest.clone(),
                            calls: item.elt.calls.try_map_to_impl(fun)?,
                        },
                        stack_hash: item.stack_hash.clone(),
                    })
                })
                .collect::<Result<_, E>>()?,
        ))
    }

    pub fn try_map_to<F, E, AnotherAccUpdate: Clone + AccountUpdateRef>(
        &self,
        mut fun: F,
    ) -> Result<CallForest<AnotherAccUpdate>, E>
    where
        F: FnMut(&AccUpdate) -> Result<AnotherAccUpdate, E>,
    {
        self.try_map_to_impl(&mut fun)
    }

    fn to_account_updates_impl(&self, accounts: &mut Vec<AccUpdate>) {
        // TODO: Check iteration order in OCaml
        for elem in self.iter() {
            accounts.push(elem.elt.account_update.clone());
            elem.elt.calls.to_account_updates_impl(accounts);
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L436>
    pub fn to_account_updates(&self) -> Vec<AccUpdate> {
        let mut accounts = Vec::with_capacity(128);
        self.to_account_updates_impl(&mut accounts);
        accounts
    }

    fn to_zkapp_command_with_hashes_list_impl(&self, output: &mut Vec<(AccUpdate, Fp)>) {
        self.iter().for_each(|item| {
            let WithStackHash { elt, stack_hash } = item;
            let Tree {
                account_update,
                account_update_digest: _,
                calls,
            } = elt;
            output.push((account_update.clone(), stack_hash.get().unwrap())); // Never fail, we called `ensure_hashed`
            calls.to_zkapp_command_with_hashes_list_impl(output);
        });
    }

    pub fn to_zkapp_command_with_hashes_list(&self) -> Vec<(AccUpdate, Fp)> {
        self.ensure_hashed();

        let mut output = Vec::with_capacity(128);
        self.to_zkapp_command_with_hashes_list_impl(&mut output);
        output
    }

    pub fn ensure_hashed(&self) {
        let Some(first) = self.first() else {
            return;
        };
        if first.stack_hash.get().is_none() {
            self.accumulate_hashes();
        }
    }
}

impl<AccUpdate: Clone + AccountUpdateRef> CallForest<AccUpdate> {
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L583>
    pub fn accumulate_hashes(&self) {
        /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L293>
        fn cons(hash: Fp, h_tl: Fp) -> Fp {
            hash_with_kimchi(&MINA_ACCOUNT_UPDATE_CONS, &[hash, h_tl])
        }

        /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L561>
        fn hash<AccUpdate: Clone + AccountUpdateRef>(
            elem: Option<&WithStackHash<AccUpdate>>,
        ) -> Fp {
            match elem {
                Some(next) => next.stack_hash.get().unwrap(), // Never fail, we hash them from reverse below
                None => Fp::zero(),
            }
        }

        // We traverse the list in reverse here (to get same behavior as OCaml recursivity)
        // Note that reverse here means 0 to last, see `CallForest::iter` for explaination
        //
        // We use indexes to make the borrow checker happy

        for index in (0..self.0.len()).rev() {
            let elem = &self.0[index];
            let WithStackHash {
                elt:
                    Tree::<AccUpdate> {
                        account_update,
                        account_update_digest,
                        calls,
                        ..
                    },
                ..
            } = elem;

            calls.accumulate_hashes();
            account_update_digest.set(account_update.account_update_ref().digest());

            let node_hash = elem.elt.digest();
            let hash = hash(self.0.get(index + 1));

            self.0[index].stack_hash.set(cons(node_hash, hash));
        }
    }
}

impl CallForest<AccountUpdate> {
    pub fn cons(
        &self,
        calls: Option<CallForest<AccountUpdate>>,
        account_update: AccountUpdate,
    ) -> Self {
        let account_update_digest = account_update.digest();

        let tree = Tree::<AccountUpdate> {
            account_update,
            account_update_digest: MutableFp::new(account_update_digest),
            calls: calls.unwrap_or_else(|| CallForest(Vec::new())),
        };
        self.cons_tree(tree)
    }

    pub fn accumulate_hashes_predicated(&mut self) {
        // Note: There seems to be no difference with `accumulate_hashes`
        self.accumulate_hashes();
    }

    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L830>
    pub fn of_wire(&mut self, _wired: &[MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA]) {
        self.accumulate_hashes();
    }

    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L840>
    pub fn to_wire(&self, _wired: &mut [MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA]) {
        // self.remove_callers(wired);
    }
}

impl CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)> {
    // Don't implement `{from,to}_wire` because the binprot types contain the hashes

    // /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L830>
    // pub fn of_wire(
    //     &mut self,
    //     _wired: &[v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesA],
    // ) {
    //     self.accumulate_hashes(&|(account_update, _vk_opt)| account_update.digest());
    // }

    // /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L840>
    // pub fn to_wire(
    //     &self,
    //     _wired: &mut [MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA],
    // ) {
    //     // self.remove_callers(wired);
    // }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1081>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeePayerBody {
    pub public_key: CompressedPubKey,
    pub fee: Fee,
    pub valid_until: Option<Slot>,
    pub nonce: Nonce,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1484>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeePayer {
    pub body: FeePayerBody,
    pub authorization: Signature,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L959>
#[derive(Debug, Clone, PartialEq)]
pub struct ZkAppCommand {
    pub fee_payer: FeePayer,
    pub account_updates: CallForest<AccountUpdate>,
    pub memo: Memo,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub enum AccessedOrNot {
    Accessed,
    NotAccessed,
}

impl ZkAppCommand {
    pub fn fee_payer(&self) -> AccountId {
        let public_key = self.fee_payer.body.public_key.clone();
        AccountId::new(public_key, self.fee_token())
    }

    pub fn fee_token(&self) -> TokenId {
        TokenId::default()
    }

    pub fn fee(&self) -> Fee {
        self.fee_payer.body.fee
    }

    pub fn fee_excess(&self) -> FeeExcess {
        FeeExcess::of_single((self.fee_token(), Signed::<Fee>::of_unsigned(self.fee())))
    }

    fn fee_payer_account_update(&self) -> &FeePayer {
        let Self { fee_payer, .. } = self;
        fee_payer
    }

    pub fn applicable_at_nonce(&self) -> Nonce {
        self.fee_payer_account_update().body.nonce
    }

    pub fn weight(&self) -> u64 {
        let Self {
            fee_payer,
            account_updates,
            memo,
        } = self;
        [
            zkapp_weight::fee_payer(fee_payer),
            zkapp_weight::account_updates(account_updates),
            zkapp_weight::memo(memo),
        ]
        .iter()
        .sum()
    }

    pub fn has_zero_vesting_period(&self) -> bool {
        self.account_updates
            .exists(|account_update| match &account_update.body.update.timing {
                SetOrKeep::Keep => false,
                SetOrKeep::Set(Timing { vesting_period, .. }) => vesting_period.is_zero(),
            })
    }

    pub fn is_incompatible_version(&self) -> bool {
        self.account_updates.exists(|account_update| {
            match &account_update.body.update.permissions {
                SetOrKeep::Keep => false,
                SetOrKeep::Set(Permissions {
                    set_verification_key,
                    ..
                }) => {
                    let SetVerificationKey {
                        auth: _,
                        txn_version,
                    } = set_verification_key;
                    *txn_version != crate::TXN_VERSION_CURRENT
                }
            }
        })
    }

    fn zkapp_cost(
        proof_segments: usize,
        signed_single_segments: usize,
        signed_pair_segments: usize,
    ) -> f64 {
        // (*10.26*np + 10.08*n2 + 9.14*n1 < 69.45*)
        let GenesisConstant {
            zkapp_proof_update_cost: proof_cost,
            zkapp_signed_pair_update_cost: signed_pair_cost,
            zkapp_signed_single_update_cost: signed_single_cost,
            ..
        } = GENESIS_CONSTANT;

        (proof_cost * (proof_segments as f64))
            + (signed_pair_cost * (signed_pair_segments as f64))
            + (signed_single_cost * (signed_single_segments as f64))
    }

    /// Zkapp_command transactions are filtered using this predicate
    /// - when adding to the transaction pool
    /// - in incoming blocks
    pub fn valid_size(&self) -> Result<(), String> {
        use crate::proofs::zkapp::group::{SegmentBasic, ZkappCommandIntermediateState};

        let Self {
            account_updates,
            fee_payer: _,
            memo: _,
        } = self;

        let events_elements = |events: &[Event]| -> usize { events.iter().map(Event::len).sum() };

        let mut n_account_updates = 0;
        let (mut num_event_elements, mut num_action_elements) = (0, 0);

        account_updates.fold((), |_, account_update| {
            num_event_elements += events_elements(account_update.body.events.events());
            num_action_elements += events_elements(account_update.body.actions.events());
            n_account_updates += 1;
        });

        let group = std::iter::repeat_n(((), (), ()), n_account_updates + 2) // + 2 to prepend two. See OCaml
            .collect::<Vec<_>>();

        let groups = crate::proofs::zkapp::group::group_by_zkapp_command_rev::<_, (), (), ()>(
            [self],
            vec![vec![((), (), ())], group],
        );

        let (mut proof_segments, mut signed_single_segments, mut signed_pair_segments) = (0, 0, 0);

        for ZkappCommandIntermediateState { spec, .. } in &groups {
            match spec {
                SegmentBasic::Proved => proof_segments += 1,
                SegmentBasic::OptSigned => signed_single_segments += 1,
                SegmentBasic::OptSignedOptSigned => signed_pair_segments += 1,
            }
        }

        let GenesisConstant {
            zkapp_transaction_cost_limit: cost_limit,
            max_event_elements,
            max_action_elements,
            ..
        } = GENESIS_CONSTANT;

        let zkapp_cost_within_limit =
            Self::zkapp_cost(proof_segments, signed_single_segments, signed_pair_segments)
                < cost_limit;
        let valid_event_elements = num_event_elements <= max_event_elements;
        let valid_action_elements = num_action_elements <= max_action_elements;

        if zkapp_cost_within_limit && valid_event_elements && valid_action_elements {
            return Ok(());
        }

        let err = [
            (zkapp_cost_within_limit, "zkapp transaction too expensive"),
            (valid_event_elements, "too many event elements"),
            (valid_action_elements, "too many action elements"),
        ]
        .iter()
        .filter(|(b, _s)| !b)
        .map(|(_b, s)| s)
        .join(";");

        Err(err)
    }

    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L997>
    pub fn account_access_statuses(
        &self,
        status: &TransactionStatus,
    ) -> Vec<(AccountId, AccessedOrNot)> {
        use AccessedOrNot::*;
        use TransactionStatus::*;

        // always `Accessed` for fee payer
        let init = vec![(self.fee_payer(), Accessed)];

        let status_sym = match status {
            Applied => Accessed,
            Failed(_) => NotAccessed,
        };

        let ids = self
            .account_updates
            .fold(init, |mut accum, account_update| {
                accum.push((account_update.account_id(), status_sym.clone()));
                accum
            });
        // WARNING: the code previous to merging latest changes wasn't doing the "rev()" call. Check this in case of errors.
        ids.iter()
            .unique() /*.rev()*/
            .cloned()
            .collect()
    }

    /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L1006>
    pub fn accounts_referenced(&self) -> Vec<AccountId> {
        self.account_access_statuses(&TransactionStatus::Applied)
            .into_iter()
            .map(|(id, _status)| id)
            .collect()
    }

    /// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L1346>
    pub fn of_verifiable(verifiable: verifiable::ZkAppCommand) -> Self {
        Self {
            fee_payer: verifiable.fee_payer,
            account_updates: verifiable.account_updates.map_to(|(acc, _)| acc.clone()),
            memo: verifiable.memo,
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1386>
    pub fn account_updates_hash(&self) -> Fp {
        self.account_updates.hash()
    }

    /// <https://github.com/MinaProtocol/mina/blob/02c9d453576fa47f78b2c388fb2e0025c47d991c/src/lib/mina_base/zkapp_command.ml#L989>
    pub fn extract_vks(&self) -> Vec<(AccountId, VerificationKeyWire)> {
        self.account_updates
            .fold(Vec::with_capacity(256), |mut acc, p| {
                if let SetOrKeep::Set(vk) = &p.body.update.verification_key {
                    acc.push((p.account_id(), vk.clone()));
                };
                acc
            })
    }

    pub fn all_account_updates(&self) -> CallForest<AccountUpdate> {
        let p = &self.fee_payer;

        let mut fee_payer = AccountUpdate::of_fee_payer(p.clone());
        fee_payer.authorization = Control::Signature(p.authorization.clone());

        self.account_updates.cons(None, fee_payer)
    }

    pub fn all_account_updates_list(&self) -> Vec<AccountUpdate> {
        let mut account_updates = Vec::with_capacity(16);
        account_updates.push(AccountUpdate::of_fee_payer(self.fee_payer.clone()));

        self.account_updates.fold(account_updates, |mut acc, u| {
            acc.push(u.clone());
            acc
        })
    }

    pub fn commitment(&self) -> TransactionCommitment {
        let account_updates_hash = self.account_updates_hash();
        TransactionCommitment::create(account_updates_hash)
    }
}

pub struct MaybeWithStatus<T> {
    pub cmd: T,
    pub status: Option<TransactionStatus>,
}

impl<T> From<WithStatus<T>> for MaybeWithStatus<T> {
    fn from(value: WithStatus<T>) -> Self {
        let WithStatus { data, status } = value;
        Self {
            cmd: data,
            status: Some(status),
        }
    }
}

impl<T> From<MaybeWithStatus<T>> for WithStatus<T> {
    fn from(value: MaybeWithStatus<T>) -> Self {
        let MaybeWithStatus { cmd, status } = value;
        Self {
            data: cmd,
            status: status.unwrap(),
        }
    }
}

impl<T> MaybeWithStatus<T> {
    pub fn cmd(&self) -> &T {
        &self.cmd
    }
    pub fn is_failed(&self) -> bool {
        self.status
            .as_ref()
            .map(TransactionStatus::is_failed)
            .unwrap_or(false)
    }
    pub fn map<V, F>(self, fun: F) -> MaybeWithStatus<V>
    where
        F: FnOnce(T) -> V,
    {
        MaybeWithStatus {
            cmd: fun(self.cmd),
            status: self.status,
        }
    }
}

pub trait ToVerifiableCache {
    fn find(&self, account_id: &AccountId, vk_hash: &Fp) -> Option<&VerificationKeyWire>;
    fn add(&mut self, account_id: AccountId, vk: VerificationKeyWire);
}

pub trait ToVerifiableStrategy {
    type Cache: ToVerifiableCache;

    fn create_all(
        cmd: &ZkAppCommand,
        is_failed: bool,
        cache: &mut Self::Cache,
    ) -> Result<verifiable::ZkAppCommand, String> {
        let verified_cmd = verifiable::create(cmd, is_failed, |vk_hash, account_id| {
            cache
                .find(account_id, &vk_hash)
                .cloned()
                .or_else(|| {
                    cmd.extract_vks()
                        .iter()
                        .find(|(id, _)| account_id == id)
                        .map(|(_, key)| key.clone())
                })
                .ok_or_else(|| format!("verification key not found in cache: {:?}", vk_hash))
        })?;
        if !is_failed {
            for (account_id, vk) in cmd.extract_vks() {
                cache.add(account_id, vk);
            }
        }
        Ok(verified_cmd)
    }
}
