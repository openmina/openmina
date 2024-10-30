use ark_ec::AffineCurve;
use ark_ec::ProjectiveCurve;
use ark_ff::SquareRootField;
use ark_ff::{Field, UniformRand};
use ledger::generators::zkapp_command_builder::get_transaction_commitments;
use ledger::proofs::field::FieldWitness;
use ledger::proofs::transaction::InnerCurve;
use ledger::scan_state::currency::{Magnitude, SlotSpan, TxnVersion};
use ledger::{
    proofs::transaction::PlonkVerificationKeyEvals,
    scan_state::{
        currency::{Amount, Balance, BlockTime, Fee, Length, MinMax, Nonce, Sgn, Signed, Slot},
        transaction_logic::{
            signed_command::{
                self, PaymentPayload, SignedCommand, SignedCommandPayload, StakeDelegationPayload,
            },
            transaction_union_payload::TransactionUnionPayload,
            zkapp_command::{
                self, AccountPreconditions, AccountUpdate, ClosedInterval, FeePayer, FeePayerBody,
                MayUseToken, Numeric, OrIgnore, SetOrKeep, Update, ZkAppCommand,
            },
            zkapp_statement::TransactionCommitment,
            Memo, Transaction, UserCommand,
        },
    },
    Account, AuthRequired, Permissions, ProofVerified, TokenId, TokenSymbol, VerificationKey,
    VotingFor, ZkAppUri,
};
use ledger::{SetVerificationKey, TXN_VERSION_CURRENT};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::array::ArrayN;
use mina_p2p_messages::list::List;
use mina_p2p_messages::v2::{
    PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk,
    PicklesWrapWireProofCommitmentsStableV1, PicklesWrapWireProofEvaluationsStableV1,
    PicklesWrapWireProofStableV1, PicklesWrapWireProofStableV1Bulletproof,
};
use mina_p2p_messages::{
    number::Number,
    pseq::PaddedSeq,
    v2::{
        CompositionTypesBranchDataDomainLog2StableV1, CompositionTypesBranchDataStableV1,
        CompositionTypesDigestConstantStableV1, CurrencyAmountStableV1, CurrencyFeeStableV1,
        LedgerHash, LimbVectorConstantHex64StableV1, MinaBaseAccountIdDigestStableV1,
        MinaBaseCallStackDigestStableV1, MinaBaseFeeExcessStableV1, MinaBaseLedgerHash0StableV1,
        MinaBasePendingCoinbaseCoinbaseStackStableV1, MinaBasePendingCoinbaseStackHashStableV1,
        MinaBasePendingCoinbaseStackVersionedStableV1, MinaBasePendingCoinbaseStateStackStableV1,
        MinaBaseStackFrameStableV1, MinaBaseTransactionStatusFailureCollectionStableV1,
        MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
        PicklesBaseProofsVerifiedStableV1,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
        PicklesProofProofsVerified2ReprStableV2PrevEvals,
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals,
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
        PicklesProofProofsVerified2ReprStableV2Statement,
        PicklesProofProofsVerified2ReprStableV2StatementFp,
        PicklesProofProofsVerified2ReprStableV2StatementProofState,
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
        PicklesProofProofsVerifiedMaxStableV2,
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2,
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
        SgnStableV1, SignedAmount, TokenFeeExcess, TokenIdKeyHash, UnsignedExtendedUInt32StableV1,
        UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};
use mina_signer::{
    CompressedPubKey, CurvePoint, Keypair, NetworkId, ScalarField, SecKey, Signature, Signer,
};
use rand::distributions::DistString;
use rand::Rng;
use rand::{distributions::Alphanumeric, seq::SliceRandom};
use std::{array, iter, ops::RangeInclusive, sync::Arc};
use tuple_map::TupleMap2;

use super::context::{FuzzerCtx, PermissionModel};

macro_rules! impl_default_generator_for_wrapper_type {
    ($fuzz_ctx: ty, $wrapper: tt) => {
        impl Generator<$wrapper> for $fuzz_ctx {
            #[coverage(off)]
            fn gen(&mut self) -> $wrapper {
                $wrapper(self.gen())
            }
        }
    };
    ($fuzz_ctx: ty, $wrapper: tt, $inner: ty) => {
        impl Generator<$wrapper> for $fuzz_ctx {
            #[coverage(off)]
            fn gen(&mut self) -> $wrapper {
                let inner: $inner = self.gen();
                inner.into()
            }
        }
    };
}

pub trait Generator<T> {
    fn gen(&mut self) -> T;
}

impl Generator<bool> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> bool {
        self.gen.rng.gen_bool(0.5)
    }
}

/*
impl Generator<Fp> for FuzzerCtx {
    // rnd_base_field
    fn gen(&mut self) -> Fp {
        let mut bf = None;

        // TODO: optimize by masking out MSBs from bytes and remove loop
        while bf.is_none() {
            let bytes = self.gen.rng.gen::<[u8; 32]>();
            bf = Fp::from_random_bytes_with_flags::<ark_serialize::EmptyFlags>(&bytes);
        }

        bf.unwrap().0
    }
}
*/

impl Generator<Fp> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Fp {
        Fp::rand(&mut self.gen.rng)
    }
}

impl Generator<Slot> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Slot {
        if self.gen.rng.gen_bool(0.9) {
            self.txn_state_view.global_slot_since_genesis
        } else {
            Slot::from_u32(self.gen.rng.gen_range(0..Slot::max().as_u32()))
        }
    }
}

impl Generator<SlotSpan> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> SlotSpan {
        if self.gen.rng.gen_bool(0.9) {
            SlotSpan::from_u32(self.txn_state_view.global_slot_since_genesis.as_u32())
        } else {
            SlotSpan::from_u32(self.gen.rng.gen_range(0..SlotSpan::max().as_u32()))
        }
    }
}

impl Generator<SecKey> for FuzzerCtx {
    /*
        Reimplement random key generation w/o the restriction on CryptoRgn trait.
        Since we are only using this for fuzzing we want a faster (unsafe) Rng like SmallRng.
    */
    #[coverage(off)]
    fn gen(&mut self) -> SecKey {
        let secret: ScalarField = ScalarField::rand(&mut self.gen.rng);
        SecKey::new(secret)
    }
}

impl Generator<Keypair> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Keypair {
        let sec_key: SecKey = self.gen();
        let scalar = sec_key.into_scalar();
        let public = CurvePoint::prime_subgroup_generator()
            .mul(scalar)
            .into_affine();

        let keypair = Keypair::from_parts_unsafe(scalar, public);

        if !self.state.potential_senders.iter().any(
            #[coverage(off)]
            |(kp, _)| kp.public == keypair.public,
        ) {
            let permission_model = self.gen();
            self.state
                .potential_new_accounts
                .push((keypair.clone(), permission_model))
        }

        keypair
    }
}

impl Generator<CompressedPubKey> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> CompressedPubKey {
        let keypair = if self.gen.rng.gen_bool(0.9) {
            // use existing account
            self.random_keypair()
        } else {
            // create new account
            self.gen()
        };

        keypair.public.into_compressed()
    }
}

impl Generator<Memo> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Memo {
        Memo::with_number(self.gen.rng.gen())
    }
}

impl<F: Field + From<i32> + SquareRootField> Generator<(F, F)> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> (F, F) {
        /*
            WARNING: we need to generate valid curve points to avoid binprot deserializarion
            exceptions in the OCaml side. However this is an expensive task.

            TODO: a more efficient way of doing this?
        */
        let mut x = F::rand(&mut self.gen.rng);

        loop {
            let y_squared = x.square().mul(x).add(Into::<F>::into(5));

            if let Some(y) = y_squared.sqrt() {
                return (x, y);
            }

            x.add_assign(F::one());
        }
    }
}

impl<F: FieldWitness> Generator<InnerCurve<F>> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> InnerCurve<F> {
        let (x, y) = self.gen();
        InnerCurve::<F>::from((x, y))
    }
}

impl<F: FieldWitness> Generator<PlonkVerificationKeyEvals<F>> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PlonkVerificationKeyEvals<F> {
        PlonkVerificationKeyEvals {
            sigma: array::from_fn(
                #[coverage(off)]
                |_| self.gen(),
            ),
            coefficients: array::from_fn(
                #[coverage(off)]
                |_| self.gen(),
            ),
            generic: self.gen(),
            psm: self.gen(),
            complete_add: self.gen(),
            mul: self.gen(),
            emul: self.gen(),
            endomul_scalar: self.gen(),
        }
    }
}

impl Generator<VerificationKey> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> VerificationKey {
        VerificationKey {
            max_proofs_verified: vec![ProofVerified::N0, ProofVerified::N1, ProofVerified::N2]
                .choose(&mut self.gen.rng)
                .unwrap()
                .clone(),
            actual_wrap_domain_size: vec![ProofVerified::N0, ProofVerified::N1, ProofVerified::N2]
                .choose(&mut self.gen.rng)
                .unwrap()
                .clone(),
            wrap_index: Box::new(self.gen()),
            wrap_vk: None, // TODO
        }
    }
}

/*
impl<T: Hasher<T> + Hashable> Generator<zkapp_command::WithHash<T>> for FuzzerCtx
where
    FuzzerCtx: Generator<T>,
{
    fn gen(&mut self) -> zkapp_command::WithHash<T> {
        let data: T = self.gen();
        let hash = data.digest();
        zkapp_command::WithHash { data, hash }
    }
}
*/

impl Generator<zkapp_command::WithHash<VerificationKey>> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> zkapp_command::WithHash<VerificationKey> {
        let data: VerificationKey = self.gen();
        let hash = data.digest();
        zkapp_command::WithHash { data, hash }
    }
}

impl Generator<AuthRequired> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> AuthRequired {
        *vec![
            AuthRequired::None,
            AuthRequired::Either,
            AuthRequired::Proof,
            AuthRequired::Signature,
            AuthRequired::Impossible,
            //AuthRequired::Both,
        ]
        .choose(&mut self.gen.rng)
        .unwrap()
    }
}

impl Generator<PermissionModel> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PermissionModel {
        vec![
            PermissionModel::Any,
            PermissionModel::Empty,
            PermissionModel::Initial,
            PermissionModel::Default,
            PermissionModel::TokenOwner,
        ]
        .choose(&mut self.gen.rng)
        .unwrap()
        .clone()
    }
}

impl Generator<ZkAppUri> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> ZkAppUri {
        /*
            TODO: this needs to be fixed (assign a boundary) in the protocol since it is
            possible to set a zkApp URI of arbitrary size.

            Since the field is opaque to the Mina protocol logic, randomly generating
            URIs makes little sense and will consume a significant amount of ledger space.
        */
        ZkAppUri::new()
    }
}

impl Generator<TokenSymbol> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> TokenSymbol {
        /*
            TokenSymbol must be <= 6 **bytes**. This boundary doesn't exist at type-level,
            instead it is check by binprot after deserializing the *string* object:
            https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account.ml#L124

            We will let this function generate strings larger than 6 bytes with low probability,
            just to cover the error handling code, but must of the time we want to avoid failing
            this check.
        */
        if self.gen.rng.gen_bool(0.9) {
            TokenSymbol::default()
        } else {
            let rnd_len = self.gen.rng.gen_range(1..=6);
            // TODO: fix n random chars for n random bytes
            TokenSymbol(Alphanumeric.sample_string(&mut self.gen.rng, rnd_len))
        }
    }
}

impl Generator<VotingFor> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> VotingFor {
        VotingFor(self.gen())
    }
}

impl Generator<zkapp_command::Events> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> zkapp_command::Events {
        /*
           An Event is a list of arrays of Fp, there doesn't seem to be any limit
           neither in the size of the list or the array's size. The total size should
           be bounded by the transport protocol (currently libp2p, ~32MB).
        */

        if self.gen.rng.gen_bool(0.9) {
            zkapp_command::Events(Vec::new())
        } else {
            // Generate random Events in the same fashion as Mina's generator (up to 5x3 elements).
            let n = self.gen.rng.gen_range(0..=5);

            zkapp_command::Events(
                (0..=n)
                    .map(
                        #[coverage(off)]
                        |_| {
                            let n = self.gen.rng.gen_range(0..=3);
                            zkapp_command::Event(
                                (0..=n)
                                    .map(
                                        #[coverage(off)]
                                        |_| self.gen(),
                                    )
                                    .collect(),
                            )
                        },
                    )
                    .collect(),
            )
        }
    }
}

impl Generator<zkapp_command::Actions> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> zkapp_command::Actions {
        // See comment in generator above

        if self.gen.rng.gen_bool(0.9) {
            zkapp_command::Actions(Vec::new())
        } else {
            let n = self.gen.rng.gen_range(0..=5);

            zkapp_command::Actions(
                (0..=n)
                    .map(
                        #[coverage(off)]
                        |_| {
                            let n = self.gen.rng.gen_range(0..=3);
                            zkapp_command::Event(
                                (0..=n)
                                    .map(
                                        #[coverage(off)]
                                        |_| self.gen(),
                                    )
                                    .collect(),
                            )
                        },
                    )
                    .collect(),
            )
        }
    }
}

impl Generator<BlockTime> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> BlockTime {
        self.gen.rng.gen()
    }
}

impl Generator<Length> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Length {
        self.gen.rng.gen()
    }
}

pub trait GeneratorRange32<T> {
    fn gen_range(&mut self, range: RangeInclusive<u32>) -> T;
}

pub trait GeneratorRange64<T> {
    fn gen_range(&mut self, range: RangeInclusive<u64>) -> T;
}

impl GeneratorRange64<Balance> for FuzzerCtx {
    #[coverage(off)]
    fn gen_range(&mut self, range: RangeInclusive<u64>) -> Balance {
        Balance::from_u64(self.gen.rng.gen_range(range))
    }
}

impl GeneratorRange64<Fee> for FuzzerCtx {
    #[coverage(off)]
    fn gen_range(&mut self, range: RangeInclusive<u64>) -> Fee {
        Fee::from_u64(self.gen.rng.gen_range(range))
    }
}

impl GeneratorRange64<Amount> for FuzzerCtx {
    #[coverage(off)]
    fn gen_range(&mut self, range: RangeInclusive<u64>) -> Amount {
        Amount::from_u64(self.gen.rng.gen_range(range))
    }
}

impl GeneratorRange32<Nonce> for FuzzerCtx {
    #[coverage(off)]
    fn gen_range(&mut self, range: RangeInclusive<u32>) -> Nonce {
        Nonce::from_u32(self.gen.rng.gen_range(range))
    }
}

impl GeneratorRange32<Length> for FuzzerCtx {
    #[coverage(off)]
    fn gen_range(&mut self, range: RangeInclusive<u32>) -> Length {
        Length::from_u32(self.gen.rng.gen_range(range))
    }
}

pub trait GeneratorWrapper<W, T, F: FnMut(&mut Self) -> T> {
    fn gen_wrap(&mut self, f: F) -> W;
}

impl<T: Clone, F: FnMut(&mut Self) -> T> GeneratorWrapper<Option<T>, T, F> for FuzzerCtx {
    #[coverage(off)]
    fn gen_wrap(&mut self, mut f: F) -> Option<T> {
        if self.gen.rng.gen_bool(0.9) {
            None
        } else {
            Some(f(self))
        }
    }
}

impl<T: Clone, F: FnMut(&mut Self) -> T> GeneratorWrapper<OrIgnore<T>, T, F> for FuzzerCtx {
    #[coverage(off)]
    fn gen_wrap(&mut self, mut f: F) -> OrIgnore<T> {
        if self.gen.rng.gen_bool(0.5) {
            OrIgnore::Ignore
        } else {
            OrIgnore::Check(f(self))
        }
    }
}

impl<T: Clone, F: FnMut(&mut Self) -> T> GeneratorWrapper<SetOrKeep<T>, T, F> for FuzzerCtx {
    #[coverage(off)]
    fn gen_wrap(&mut self, mut f: F) -> SetOrKeep<T> {
        if self.gen.rng.gen_bool(0.5) {
            SetOrKeep::Keep
        } else {
            SetOrKeep::Set(f(self))
        }
    }
}

impl<T: Clone + MinMax, F: FnMut(&mut Self) -> T> GeneratorWrapper<ClosedInterval<T>, T, F>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen_wrap(&mut self, mut f: F) -> ClosedInterval<T> {
        if self.gen.rng.gen_bool(0.5) {
            // constant case
            let val = f(self);

            ClosedInterval {
                lower: val.clone(),
                upper: val,
            }
        } else {
            ClosedInterval {
                lower: f(self),
                upper: f(self),
            }
        }
    }
}

impl Generator<Sgn> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Sgn {
        if self.gen.rng.gen_bool(0.5) {
            Sgn::Pos
        } else {
            Sgn::Neg
        }
    }
}

pub trait GeneratorWrapperMany<W, T, F: FnMut(&mut Self) -> T> {
    fn gen_wrap_many(&mut self, f: F) -> W;
}

impl<T, F: FnMut(&mut Self) -> T, const N: u64> GeneratorWrapperMany<ArrayN<T, N>, T, F>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen_wrap_many(&mut self, mut f: F) -> ArrayN<T, N> {
        iter::repeat_with(
            #[coverage(off)]
            || f(self),
        )
        .take(N as usize)
        .collect()
    }
}

impl Generator<Numeric<Amount>> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Numeric<Amount> {
        self.gen_wrap(
            #[coverage(off)]
            |x| {
                x.gen_wrap(
                    #[coverage(off)]
                    |x| -> Amount { GeneratorRange64::gen_range(x, 0..=u64::MAX) },
                )
            },
        )
    }
}

impl Generator<Numeric<Length>> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Numeric<Length> {
        self.gen_wrap(
            #[coverage(off)]
            |x| {
                x.gen_wrap(
                    #[coverage(off)]
                    |x| -> Length { GeneratorRange32::gen_range(x, 0..=u32::MAX) },
                )
            },
        )
    }
}

impl Generator<FeePayer> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> FeePayer {
        let public_key = if self.gen.attempt_valid_zkapp || self.gen.rng.gen_bool(0.9) {
            self.random_keypair().public.into_compressed()
        } else {
            self.gen()
        };

        let account = self.get_account(&public_key);
        // FIXME: boundary at i64 MAX because OCaml side is encoding it as int
        let max_fee = match account.as_ref() {
            Some(account) if self.gen.attempt_valid_zkapp || self.gen.rng.gen_bool(0.9) => {
                self.gen.minimum_fee.max(account.balance.as_u64())
            }
            _ => self
                .gen
                .rng
                .gen_range(self.gen.minimum_fee + 1..=10_000_000),
        };

        let fee: Fee = GeneratorRange64::gen_range(self, self.gen.minimum_fee..=max_fee);

        let nonce = match self.gen.nonces.get(&public_key.into_address()) {
            Some(nonce) => *nonce,
            None => account
                .as_ref()
                .map(|account| account.nonce)
                .unwrap_or_else(|| {
                    if self.gen.rng.gen_bool(0.9) {
                        Nonce::from_u32(0)
                    } else {
                        GeneratorRange32::gen_range(self, 0..=u32::MAX)
                    }
                }),
        };

        self.gen
            .nonces
            .insert(public_key.into_address(), nonce.incr());

        FeePayer {
            body: FeePayerBody {
                public_key,
                fee,
                valid_until: self.gen_wrap(
                    #[coverage(off)]
                    |x| -> Slot { x.gen() },
                ),
                nonce,
            },
            // filled later when tx is complete
            authorization: Signature::dummy(),
        }
    }
}

impl Generator<zkapp_command::EpochData> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> zkapp_command::EpochData {
        zkapp_command::EpochData::new(
            zkapp_command::EpochLedger {
                hash: self.gen_wrap(
                    #[coverage(off)]
                    |x| x.gen(),
                ),
                total_currency: self.gen(),
            },
            self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            self.gen(),
        )
    }
}

impl Generator<LimbVectorConstantHex64StableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> LimbVectorConstantHex64StableV1 {
        LimbVectorConstantHex64StableV1(Number(self.gen.rng.gen()))
    }
}

impl
    Generator<
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
    > for FuzzerCtx
{
    #[coverage(off)]
    fn gen(
        &mut self,
    ) -> PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags
    {
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags {
            range_check0: self.gen.rng.gen_bool(0.5),
            range_check1: self.gen.rng.gen_bool(0.5),
            foreign_field_add: self.gen.rng.gen_bool(0.5),
            foreign_field_mul: self.gen.rng.gen_bool(0.5),
            xor: self.gen.rng.gen_bool(0.5),
            rot: self.gen.rng.gen_bool(0.5),
            lookup: self.gen.rng.gen_bool(0.5),
            runtime_tables: self.gen.rng.gen_bool(0.5),
        }
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen(
        &mut self,
    ) -> PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk {
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk {
            alpha: self.gen(),
            beta: self.gen(),
            gamma: self.gen(),
            zeta: self.gen(),
            joint_combiner: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            feature_flags: self.gen(),
        }
    }
}

impl Generator<mina_p2p_messages::bigint::BigInt> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> mina_p2p_messages::bigint::BigInt {
        mina_p2p_messages::bigint::BigInt::from(Generator::<Fp>::gen(self))
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2StatementFp> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2StatementFp {
        PicklesProofProofsVerified2ReprStableV2StatementFp::ShiftedValue(self.gen())
    }
}

impl Generator<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen(
        &mut self,
    ) -> PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
            prechallenge: self.gen(),
        }
    }
}

impl<T, const N: usize> Generator<PaddedSeq<T, N>> for FuzzerCtx
where
    FuzzerCtx: Generator<T>,
{
    #[coverage(off)]
    fn gen(&mut self) -> PaddedSeq<T, N> {
        PaddedSeq::<T, N>(array::from_fn(
            #[coverage(off)]
            |_| self.gen(),
        ))
    }
}

impl<T: Clone, F: FnMut(&mut Self) -> T, const N: usize> GeneratorWrapper<PaddedSeq<T, N>, T, F>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen_wrap(&mut self, mut f: F) -> PaddedSeq<T, N> {
        PaddedSeq::<T, N>(array::from_fn(
            #[coverage(off)]
            |_| f(self),
        ))
    }
}

impl
    Generator<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen(
        &mut self,
    ) -> PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
            inner: self.gen(),
        }
    }
}

impl Generator<CompositionTypesBranchDataDomainLog2StableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> CompositionTypesBranchDataDomainLog2StableV1 {
        CompositionTypesBranchDataDomainLog2StableV1(mina_p2p_messages::char::Char(
            self.gen.rng.gen(),
        ))
    }
}

impl Generator<CompositionTypesBranchDataStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> CompositionTypesBranchDataStableV1 {
        CompositionTypesBranchDataStableV1 {
            proofs_verified: vec![
                PicklesBaseProofsVerifiedStableV1::N0,
                PicklesBaseProofsVerifiedStableV1::N1,
                PicklesBaseProofsVerifiedStableV1::N2,
            ]
            .choose(&mut self.gen.rng)
            .unwrap()
            .clone(),
            domain_log2: self.gen(),
        }
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
            plonk: self.gen(),
            bulletproof_challenges: self.gen(),
            branch_data: self.gen(),
        }
    }
}

impl Generator<CompositionTypesDigestConstantStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> CompositionTypesDigestConstantStableV1 {
        CompositionTypesDigestConstantStableV1(self.gen())
    }
}

impl Generator<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2>
    for FuzzerCtx
{
    #[coverage(off)]
    fn gen(
        &mut self,
    ) -> PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2 {
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2(self.gen())
    }
}

#[coverage(off)]
pub fn gen_curve_point<T: Field + From<i32>>(
    ctx: &mut impl Generator<(T, T)>,
) -> (
    mina_p2p_messages::bigint::BigInt,
    mina_p2p_messages::bigint::BigInt,
)
where
    mina_p2p_messages::bigint::BigInt: From<T>,
{
    Generator::<(T, T)>::gen(ctx).map(mina_p2p_messages::bigint::BigInt::from)
}

#[coverage(off)]
pub fn gen_curve_point_many<T: Field + SquareRootField + From<i32>, const N: u64>(
    ctx: &mut FuzzerCtx,
) -> ArrayN<
    (
        mina_p2p_messages::bigint::BigInt,
        mina_p2p_messages::bigint::BigInt,
    ),
    { N },
>
where
    mina_p2p_messages::bigint::BigInt: From<T>,
{
    ctx.gen_wrap_many(
        #[coverage(off)]
        |x| gen_curve_point::<T>(x),
    )
}

#[coverage(off)]
pub fn gen_curve_point_many_unzip<T: Field + SquareRootField + From<i32>, const N: u64>(
    ctx: &mut FuzzerCtx,
) -> (
    ArrayN<mina_p2p_messages::bigint::BigInt, { N }>,
    ArrayN<mina_p2p_messages::bigint::BigInt, { N }>,
)
where
    mina_p2p_messages::bigint::BigInt: From<T> + Default,
{
    let (a, b): (Vec<_>, Vec<_>) = gen_curve_point_many::<T, N>(ctx).into_iter().unzip();

    (
        ArrayN::from_iter(a.into_iter()),
        ArrayN::from_iter(b.into_iter()),
    )
}

impl Generator<PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
            challenge_polynomial_commitment: gen_curve_point::<Fq>(self),
            old_bulletproof_challenges: self.gen(),
        }
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2StatementProofState> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2StatementProofState {
        PicklesProofProofsVerified2ReprStableV2StatementProofState {
            deferred_values: self.gen(),
            sponge_digest_before_evaluations: self.gen(),
            messages_for_next_wrap_proof: self.gen(),
        }
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
            app_state: (),
            challenge_polynomial_commitments: List::one(gen_curve_point::<Fp>(self)), // TODO: variable num of elements
            old_bulletproof_challenges: List::one(self.gen_wrap(
                // TODO: variable num of elements
                #[coverage(off)]
                |x| x.gen(),
            )),
        }
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2Statement> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2Statement {
        PicklesProofProofsVerified2ReprStableV2Statement {
            proof_state: self.gen(),
            messages_for_next_step_proof: self.gen(),
        }
    }
}

// impl Generator<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA> for FuzzerCtx {
//     #[coverage(off)]
//     fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA {
//         PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA {
//             sorted: self.gen_wrap_many(
//                 #[coverage(off)]
//                 |x| gen_curve_point_many_unzip::<Fp>(x, 1),
//                 1,
//             ),
//             aggreg: gen_curve_point_many_unzip::<Fp>(self, 1),
//             table: gen_curve_point_many_unzip::<Fp>(self, 1),
//             runtime: self.gen_wrap(
//                 #[coverage(off)]
//                 |x| gen_curve_point_many_unzip::<Fp>(x, 1),
//             ),
//         }
//     }
// }

impl Generator<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
            w: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            complete_add_selector: gen_curve_point_many_unzip::<Fp, 16>(self),
            mul_selector: gen_curve_point_many_unzip::<Fp, 16>(self),
            emul_selector: gen_curve_point_many_unzip::<Fp, 16>(self),
            endomul_scalar_selector: gen_curve_point_many_unzip::<Fp, 16>(self),
            range_check0_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            range_check1_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            foreign_field_add_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            foreign_field_mul_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            xor_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            rot_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            lookup_aggregation: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            lookup_table: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            lookup_sorted: self.gen_wrap(
                #[coverage(off)]
                |x| {
                    x.gen_wrap(
                        #[coverage(off)]
                        |x| gen_curve_point_many_unzip::<Fp, 16>(x),
                    )
                },
            ),
            runtime_lookup_table: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            runtime_lookup_table_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            xor_lookup_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            lookup_gate_lookup_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            range_check_lookup_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            foreign_field_mul_lookup_selector: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            coefficients: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            z: gen_curve_point_many_unzip::<Fp, 16>(self),
            s: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point_many_unzip::<Fp, 16>(x),
            ),
            generic_selector: gen_curve_point_many_unzip::<Fp, 16>(self),
            poseidon_selector: gen_curve_point_many_unzip::<Fp, 16>(self),
        }
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
            public_input: gen_curve_point::<Fp>(self),
            evals: self.gen(),
        }
    }
}

impl Generator<PicklesProofProofsVerified2ReprStableV2PrevEvals> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2PrevEvals {
        PicklesProofProofsVerified2ReprStableV2PrevEvals {
            evals: self.gen(),
            ft_eval1: mina_p2p_messages::bigint::BigInt::from(Generator::<Fp>::gen(self)),
        }
    }
}

// impl Generator<PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA> for FuzzerCtx {
//     #[coverage(off)]
//     fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA {
//         PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA {
//             sorted: self.gen_wrap_many(
//                 #[coverage(off)]
//                 |x| gen_curve_point_many::<Fp>(x, 1),
//                 1,
//             ),
//             aggreg: gen_curve_point_many::<Fp>(self, 1),
//             runtime: self.gen_wrap(
//                 #[coverage(off)]
//                 |x| gen_curve_point_many::<Fp>(x, 1),
//             ),
//         }
//     }
// }

// impl Generator<PicklesProofProofsVerified2ReprStableV2ProofMessages> for FuzzerCtx {
//     #[coverage(off)]
//     fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2ProofMessages {
//         PicklesProofProofsVerified2ReprStableV2ProofMessages {
//             w_comm: self.gen_wrap(
//                 #[coverage(off)]
//                 |x| gen_curve_point_many::<Fp>(x, 1),
//             ),
//             z_comm: gen_curve_point_many::<Fp>(self, 1),
//             t_comm: gen_curve_point_many::<Fp>(self, 1),
//             lookup: self.gen_wrap(
//                 #[coverage(off)]
//                 |x| x.gen(),
//             ),
//         }
//     }
// }

// impl Generator<PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof> for FuzzerCtx {
//     #[coverage(off)]
//     fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof {
//         PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof {
//             lr: self.gen_wrap_many(
//                 #[coverage(off)]
//                 |x| (gen_curve_point::<Fp>(x), gen_curve_point::<Fp>(x)),
//                 1,
//             ),
//             z_1: self.gen(),
//             z_2: self.gen(),
//             delta: gen_curve_point::<Fp>(self),
//             challenge_polynomial_commitment: gen_curve_point::<Fp>(self),
//         }
//     }
// }

// impl Generator<PicklesProofProofsVerified2ReprStableV2ProofOpenings> for FuzzerCtx {
//     #[coverage(off)]
//     fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2ProofOpenings {
//         PicklesProofProofsVerified2ReprStableV2ProofOpenings {
//             proof: self.gen(),
//             evals: self.gen(),
//             ft_eval1: self.gen(),
//         }
//     }
// }

// impl Generator<PicklesProofProofsVerified2ReprStableV2Proof> for FuzzerCtx {
//     #[coverage(off)]
//     fn gen(&mut self) -> PicklesProofProofsVerified2ReprStableV2Proof {
//         PicklesProofProofsVerified2ReprStableV2Proof {
//             messages: self.gen(),
//             openings: self.gen(),
//         }
//     }
// }

impl Generator<PicklesWrapWireProofCommitmentsStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesWrapWireProofCommitmentsStableV1 {
        PicklesWrapWireProofCommitmentsStableV1 {
            w_comm: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point::<Fp>(x),
            ),
            z_comm: gen_curve_point::<Fp>(self),
            t_comm: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point::<Fp>(x),
            ),
        }
    }
}

impl Generator<PicklesWrapWireProofStableV1Bulletproof> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesWrapWireProofStableV1Bulletproof {
        PicklesWrapWireProofStableV1Bulletproof {
            lr: self.gen_wrap_many(
                #[coverage(off)]
                |x| (gen_curve_point::<Fp>(x), gen_curve_point::<Fp>(x)),
            ),
            z_1: Generator::<Fp>::gen(self).into(),
            z_2: Generator::<Fp>::gen(self).into(),
            delta: gen_curve_point::<Fp>(self),
            challenge_polynomial_commitment: gen_curve_point::<Fp>(self),
        }
    }
}

impl Generator<PicklesWrapWireProofEvaluationsStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesWrapWireProofEvaluationsStableV1 {
        PicklesWrapWireProofEvaluationsStableV1 {
            w: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point::<Fp>(x),
            ),
            coefficients: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point::<Fp>(x),
            ),
            z: gen_curve_point::<Fp>(self),
            s: self.gen_wrap(
                #[coverage(off)]
                |x| gen_curve_point::<Fp>(x),
            ),
            generic_selector: gen_curve_point::<Fp>(self),
            poseidon_selector: gen_curve_point::<Fp>(self),
            complete_add_selector: gen_curve_point::<Fp>(self),
            mul_selector: gen_curve_point::<Fp>(self),
            emul_selector: gen_curve_point::<Fp>(self),
            endomul_scalar_selector: gen_curve_point::<Fp>(self),
        }
    }
}

impl Generator<PicklesWrapWireProofStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesWrapWireProofStableV1 {
        PicklesWrapWireProofStableV1 {
            bulletproof: self.gen(),
            evaluations: self.gen(),
            ft_eval1: self.gen(),
            commitments: self.gen(),
        }
    }
}

impl Generator<PicklesProofProofsVerifiedMaxStableV2> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> PicklesProofProofsVerifiedMaxStableV2 {
        PicklesProofProofsVerifiedMaxStableV2 {
            statement: self.gen(),
            prev_evals: self.gen(),
            proof: self.gen(),
        }
    }
}

impl Generator<zkapp_command::SideLoadedProof> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> zkapp_command::SideLoadedProof {
        Arc::new(self.gen())
    }
}

impl Generator<SetVerificationKey<AuthRequired>> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> SetVerificationKey<AuthRequired> {
        SetVerificationKey {
            auth: self.gen(),
            txn_version: if self.gen.rng.gen_bool(0.9) {
                TXN_VERSION_CURRENT
            } else {
                TxnVersion::from(self.gen.rng.gen())
            },
        }
    }
}

impl Generator<Balance> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Balance {
        GeneratorRange64::<Balance>::gen_range(self, 0..=Balance::max().as_u64())
    }
}

impl Generator<Amount> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Amount {
        GeneratorRange64::<Amount>::gen_range(self, 0..=Amount::max().as_u64())
    }
}

pub trait GeneratorFromAccount<T> {
    fn gen_from_account(&mut self, account: &Account) -> T;
}

impl GeneratorFromAccount<zkapp_command::AuthorizationKind> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> zkapp_command::AuthorizationKind {
        let vk_hash = if self.gen.rng.gen_bool(0.9)
            && account.zkapp.is_some()
            && account.zkapp.as_ref().unwrap().verification_key.is_some()
        {
            account
                .zkapp
                .as_ref()
                .unwrap()
                .verification_key
                .as_ref()
                .unwrap()
                .hash()
        } else {
            self.gen()
        };

        let options = vec![
            zkapp_command::AuthorizationKind::NoneGiven,
            zkapp_command::AuthorizationKind::Signature,
            zkapp_command::AuthorizationKind::Proof(vk_hash),
        ];

        options.choose(&mut self.gen.rng).unwrap().clone()
    }
}

impl GeneratorFromAccount<Permissions<AuthRequired>> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> Permissions<AuthRequired> {
        let permission_model = match self.find_permissions(&account.public_key) {
            Some(model) => model,
            None => [
                PermissionModel::Any,
                PermissionModel::Empty,
                PermissionModel::Initial,
                PermissionModel::TokenOwner,
            ]
            .choose(&mut self.gen.rng)
            .unwrap(),
        };

        match permission_model {
            PermissionModel::Any => Permissions::<AuthRequired> {
                edit_state: self.gen(),
                access: self.gen(),
                send: self.gen(),
                receive: self.gen(),
                set_delegate: self.gen(),
                set_permissions: self.gen(),
                set_verification_key: self.gen(),
                set_zkapp_uri: self.gen(),
                edit_action_state: self.gen(),
                set_token_symbol: self.gen(),
                increment_nonce: self.gen(),
                set_voting_for: self.gen(),
                set_timing: self.gen(),
            },
            PermissionModel::Empty => Permissions::<AuthRequired>::empty(),
            PermissionModel::Initial => Permissions::<AuthRequired>::user_default(),
            PermissionModel::Default => Permissions::<AuthRequired> {
                edit_state: AuthRequired::Proof,
                access: AuthRequired::None,
                send: AuthRequired::Signature,
                receive: AuthRequired::None,
                set_delegate: AuthRequired::Signature,
                set_permissions: AuthRequired::Signature,
                set_verification_key: SetVerificationKey::<AuthRequired> {
                    auth: AuthRequired::Signature,
                    txn_version: TXN_VERSION_CURRENT,
                },
                set_zkapp_uri: AuthRequired::Signature,
                edit_action_state: AuthRequired::Proof,
                set_token_symbol: AuthRequired::Signature,
                increment_nonce: AuthRequired::Signature,
                set_voting_for: AuthRequired::Signature,
                set_timing: AuthRequired::Proof,
            },
            PermissionModel::TokenOwner => Permissions::<AuthRequired> {
                edit_state: AuthRequired::Either, // Proof | Signature
                access: AuthRequired::Either,     // Proof | Signature
                send: AuthRequired::Signature,
                receive: AuthRequired::Proof,
                set_delegate: AuthRequired::Signature,
                set_permissions: AuthRequired::Signature,
                set_verification_key: SetVerificationKey::<AuthRequired> {
                    auth: AuthRequired::Signature,
                    txn_version: TXN_VERSION_CURRENT,
                },
                set_zkapp_uri: AuthRequired::Signature,
                edit_action_state: AuthRequired::Proof,
                set_token_symbol: AuthRequired::Signature,
                increment_nonce: AuthRequired::Signature,
                set_voting_for: AuthRequired::Signature,
                set_timing: AuthRequired::Proof,
            },
        }
    }
}

impl<T: Clone + MinMax> GeneratorFromAccount<ClosedInterval<T>> for FuzzerCtx
where
    FuzzerCtx: GeneratorFromAccount<T>,
{
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> ClosedInterval<T> {
        ClosedInterval {
            lower: self.gen_from_account(account),
            upper: self.gen_from_account(account),
        }
    }
}

impl GeneratorFromAccount<Fee> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> Fee {
        GeneratorRange64::<Fee>::gen_range(self, 0..=account.balance.as_u64() / 100)
    }
}

impl GeneratorFromAccount<Balance> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> Balance {
        GeneratorRange64::<Balance>::gen_range(self, 0..=(account.balance.as_u64() / 100))
    }
}

impl GeneratorFromAccount<Signed<Amount>> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> Signed<Amount> {
        if self.gen.rng.gen_bool(0.9) {
            Signed::<Amount>::zero()
        } else {
            if self.gen.token_id == TokenId::default() {
                if self.gen.excess_fee.is_zero() {
                    let magnitude = GeneratorRange64::<Amount>::gen_range(
                        self,
                        0..=(account.balance.as_u64().wrapping_div(10).saturating_mul(7)),
                    );
                    self.gen.excess_fee = Signed::<Amount>::create(magnitude, self.gen());
                    self.gen.excess_fee
                } else {
                    let ret = self.gen.excess_fee.negate();
                    self.gen.excess_fee = Signed::<Amount>::zero();
                    ret
                }
            } else {
                // Custom Tokens
                let magnitude = GeneratorRange64::<Amount>::gen_range(self, 0..=u64::MAX);
                Signed::<Amount>::create(magnitude, self.gen())
            }
        }
    }
}

impl GeneratorFromAccount<Amount> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> Amount {
        if self.gen.token_id == TokenId::default() && self.gen.rng.gen_bool(0.9) {
            GeneratorRange64::<Amount>::gen_range(self, 0..=account.balance.as_u64())
        } else {
            // Custom Tokens
            GeneratorRange64::<Amount>::gen_range(self, 0..=u64::MAX)
        }
    }
}

impl GeneratorFromAccount<zkapp_command::Timing> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> zkapp_command::Timing {
        if self.gen.rng.gen_bool(0.5) {
            zkapp_command::Timing {
                initial_minimum_balance: Balance::zero(),
                cliff_time: Slot::zero(),
                cliff_amount: Amount::zero(),
                vesting_period: SlotSpan::from_u32(1),
                vesting_increment: Amount::zero(),
            }
        } else {
            zkapp_command::Timing {
                initial_minimum_balance: self.gen_from_account(account),
                cliff_time: self.gen(),
                cliff_amount: self.gen_from_account(account),
                vesting_period: self.gen(),
                vesting_increment: self.gen_from_account(account),
            }
        }
    }
}

impl GeneratorFromAccount<Nonce> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> Nonce {
        let nonce = match self.gen.nonces.get(&account.public_key.into_address()) {
            Some(nonce) => *nonce,
            None => account.nonce,
        };
        // We assume successful aplication
        self.gen
            .nonces
            .insert(account.public_key.into_address(), nonce.incr());
        nonce
    }
}

impl GeneratorFromAccount<Update> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> Update {
        Update {
            app_state: if self.gen.rng.gen_bool(0.9) {
                array::from_fn(
                    #[coverage(off)]
                    |_| {
                        self.gen_wrap(
                            #[coverage(off)]
                            |x| x.gen(),
                        )
                    },
                )
            } else {
                // cover changing_entire_app_state
                array::from_fn(
                    #[coverage(off)]
                    |_| SetOrKeep::Set(self.gen()),
                )
            },
            delegate: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            verification_key: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            permissions: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen_from_account(account),
            ),
            zkapp_uri: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            token_symbol: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            timing: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen_from_account(account),
            ),
            voting_for: self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
        }
    }
}

impl GeneratorFromAccount<zkapp_command::ZkAppPreconditions> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> zkapp_command::ZkAppPreconditions {
        if self.gen.rng.gen_bool(0.9) {
            zkapp_command::ZkAppPreconditions::accept()
        } else {
            zkapp_command::ZkAppPreconditions {
                snarked_ledger_hash: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        if x.gen.rng.gen_bool(0.9) {
                            x.txn_state_view.snarked_ledger_hash
                        } else {
                            x.gen()
                        }
                    },
                ),
                blockchain_length: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        x.gen_wrap(
                            #[coverage(off)]
                            |x| {
                                if x.gen.rng.gen_bool(0.9) {
                                    x.txn_state_view.blockchain_length
                                } else {
                                    x.gen()
                                }
                            },
                        )
                    },
                ),
                min_window_density: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        x.gen_wrap(
                            #[coverage(off)]
                            |x| {
                                if x.gen.rng.gen_bool(0.9) {
                                    x.txn_state_view.min_window_density
                                } else {
                                    x.gen()
                                }
                            },
                        )
                    },
                ),
                total_currency: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        x.gen_wrap(
                            #[coverage(off)]
                            |x| {
                                if x.gen.rng.gen_bool(0.9) {
                                    x.txn_state_view.total_currency
                                } else {
                                    x.gen_from_account(account)
                                }
                            },
                        )
                    },
                ),
                global_slot_since_genesis: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        x.gen_wrap(
                            #[coverage(off)]
                            |x| x.gen(),
                        )
                    },
                ),

                staking_epoch_data: if self.gen.rng.gen_bool(0.9) {
                    let epoch_data = self.txn_state_view.staking_epoch_data.clone();

                    zkapp_command::EpochData::new(
                        zkapp_command::EpochLedger {
                            hash: OrIgnore::Check(epoch_data.ledger.hash),
                            total_currency: OrIgnore::Check(ClosedInterval::<Amount> {
                                lower: epoch_data.ledger.total_currency.clone(),
                                upper: epoch_data.ledger.total_currency,
                            }),
                        },
                        OrIgnore::Check(epoch_data.seed),
                        OrIgnore::Check(epoch_data.start_checkpoint),
                        OrIgnore::Check(epoch_data.lock_checkpoint),
                        OrIgnore::Check(ClosedInterval::<Length> {
                            lower: epoch_data.epoch_length.clone(),
                            upper: epoch_data.epoch_length,
                        }),
                    )
                } else {
                    self.gen()
                },
                next_epoch_data: if self.gen.rng.gen_bool(0.9) {
                    let epoch_data = self.txn_state_view.next_epoch_data.clone();
                    zkapp_command::EpochData::new(
                        zkapp_command::EpochLedger {
                            hash: OrIgnore::Check(epoch_data.ledger.hash),
                            total_currency: OrIgnore::Check(ClosedInterval::<Amount> {
                                lower: epoch_data.ledger.total_currency.clone(),
                                upper: epoch_data.ledger.total_currency,
                            }),
                        },
                        OrIgnore::Check(epoch_data.seed),
                        OrIgnore::Check(epoch_data.start_checkpoint),
                        OrIgnore::Check(epoch_data.lock_checkpoint),
                        OrIgnore::Check(ClosedInterval::<Length> {
                            lower: epoch_data.epoch_length.clone(),
                            upper: epoch_data.epoch_length,
                        }),
                    )
                } else {
                    self.gen()
                },
            }
        }
    }
}

impl GeneratorFromAccount<zkapp_command::Account> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> zkapp_command::Account {
        if self.gen.rng.gen_bool(0.9) {
            zkapp_command::Account::accept()
        } else {
            zkapp_command::Account {
                balance: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        x.gen_wrap(
                            #[coverage(off)]
                            |x| x.gen_from_account(account),
                        )
                    },
                ),
                nonce: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        x.gen_wrap(
                            #[coverage(off)]
                            |x| x.gen_from_account(account),
                        )
                    },
                ),
                receipt_chain_hash: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        if x.gen.rng.gen_bool(0.9) {
                            account.receipt_chain_hash.0
                        } else {
                            x.gen()
                        }
                    },
                ),
                delegate: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        let rnd_pk: CompressedPubKey = x.gen();
                        account
                            .delegate
                            .as_ref()
                            .map(
                                #[coverage(off)]
                                |pk| {
                                    if x.gen.rng.gen_bool(0.9) {
                                        pk.clone()
                                    } else {
                                        rnd_pk.clone()
                                    }
                                },
                            )
                            .unwrap_or(rnd_pk)
                    },
                ),
                state: {
                    let rnd_state = array::from_fn(
                        #[coverage(off)]
                        |_| {
                            self.gen_wrap(
                                #[coverage(off)]
                                |x| x.gen(),
                            )
                        },
                    );

                    account
                        .zkapp
                        .as_ref()
                        .map(
                            #[coverage(off)]
                            |zkapp| {
                                if self.gen.rng.gen_bool(0.9) {
                                    zkapp.app_state.map(OrIgnore::Check)
                                } else {
                                    rnd_state.clone()
                                }
                            },
                        )
                        .unwrap_or(rnd_state)
                },
                action_state: self.gen_wrap(
                    #[coverage(off)]
                    |x| x.gen(),
                ),
                proved_state: self.gen_wrap(
                    #[coverage(off)]
                    |x| {
                        let rnd_bool = x.gen.rng.gen_bool(0.5);
                        account
                            .zkapp
                            .as_ref()
                            .map(
                                #[coverage(off)]
                                |zkapp| {
                                    if x.gen.rng.gen_bool(0.9) {
                                        zkapp.proved_state
                                    } else {
                                        rnd_bool
                                    }
                                },
                            )
                            .unwrap_or(rnd_bool)
                    },
                ),
                is_new: self.gen_wrap(
                    #[coverage(off)]
                    |x| x.gen.rng.gen_bool(0.5),
                ),
            }
        }
    }
}

impl GeneratorFromAccount<AccountPreconditions> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> AccountPreconditions {
        AccountPreconditions(self.gen_from_account(account))
    }
}

impl GeneratorFromAccount<zkapp_command::Preconditions> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> zkapp_command::Preconditions {
        zkapp_command::Preconditions::new(
            self.gen_from_account(account),
            self.gen_from_account(account),
            self.gen_wrap(
                #[coverage(off)]
                |x| {
                    x.gen_wrap(
                        #[coverage(off)]
                        |x| x.gen(),
                    )
                },
            ),
        )
    }
}

impl Generator<TokenId> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> TokenId {
        if self.gen.rng.gen_bool(0.9) {
            TokenId::default()
        } else {
            TokenId(self.gen())
        }
    }
}

impl Generator<MayUseToken> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> MayUseToken {
        if self.gen.token_id.is_default() {
            MayUseToken::No
        } else {
            match vec![0, 1, 2].choose(&mut self.gen.rng).unwrap() {
                0 => MayUseToken::No,
                1 => MayUseToken::ParentsOwnToken,
                2 => MayUseToken::InheritFromParent,
                _ => unimplemented!(),
            }
        }
    }
}

impl GeneratorFromAccount<zkapp_command::Body> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> zkapp_command::Body {
        zkapp_command::Body {
            public_key: account.public_key.clone(),
            token_id: self.gen.token_id.clone(),
            update: self.gen_from_account(&account),
            balance_change: self.gen_from_account(&account),
            increment_nonce: self.gen.rng.gen_bool(0.5),
            events: self.gen(),
            actions: self.gen(),
            call_data: self.gen(),
            preconditions: self.gen_from_account(&account),
            use_full_commitment: self.gen.rng.gen_bool(0.9),
            implicit_account_creation_fee: self.gen.rng.gen_bool(0.1),
            may_use_token: self.gen(),
            authorization_kind: self.gen_from_account(&account),
        }
    }
}

impl Generator<zkapp_command::Body> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> zkapp_command::Body {
        let account = Account::empty();

        zkapp_command::Body {
            public_key: self.gen(),
            token_id: self.gen(),
            update: self.gen_from_account(&account),
            balance_change: self.gen_from_account(&account),
            increment_nonce: self.gen.rng.gen_bool(0.5),
            events: self.gen(),
            actions: self.gen(),
            call_data: self.gen(),
            preconditions: self.gen_from_account(&account),
            use_full_commitment: self.gen.rng.gen_bool(0.9),
            implicit_account_creation_fee: self.gen.rng.gen_bool(0.1),
            may_use_token: self.gen(),
            authorization_kind: self.gen_from_account(&account),
        }
    }
}

pub trait GeneratorFromToken<T> {
    fn gen_from_token(&mut self, token_id: TokenId, depth: usize) -> T;
}

impl GeneratorFromToken<AccountUpdate> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_token(&mut self, token_id: TokenId, _depth: usize) -> AccountUpdate {
        self.gen.token_id = token_id;

        let public_key = if self.gen.attempt_valid_zkapp || self.gen.rng.gen_bool(0.9) {
            self.random_keypair().public.into_compressed()
        } else {
            self.gen()
        };

        // let public_key = self.random_keypair().public.into_compressed();
        let account = self.get_account(&public_key);
        let body: zkapp_command::Body = if account.is_some() && self.gen.rng.gen_bool(0.9) {
            self.gen_from_account(account.as_ref().unwrap())
        } else {
            self.gen()
        };

        let authorization = match body.authorization_kind {
            zkapp_command::AuthorizationKind::NoneGiven => zkapp_command::Control::NoneGiven,
            zkapp_command::AuthorizationKind::Signature => {
                zkapp_command::Control::Signature(Signature::dummy())
            }
            zkapp_command::AuthorizationKind::Proof(_) => zkapp_command::Control::Proof(self.gen()),
        };

        AccountUpdate {
            body,
            authorization,
        }
    }
}

impl GeneratorFromToken<zkapp_command::CallForest<AccountUpdate>> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_token(
        &mut self,
        token_id: TokenId,
        depth: usize,
    ) -> zkapp_command::CallForest<AccountUpdate> {
        let mut forest = zkapp_command::CallForest::<AccountUpdate>::new();
        let max_count = 3usize.saturating_sub(depth);
        let count = self.gen.rng.gen_range(0..=max_count);

        for _ in 0..count {
            let account_update: AccountUpdate = self.gen_from_token(token_id.clone(), depth);
            let token_id = account_update.account_id().derive_token_id();
            let calls = if self.gen.rng.gen_bool(0.8) || depth >= 3 {
                None
            } else {
                // recursion
                Some(self.gen_from_token(token_id, depth + 1))
            };

            forest = forest.cons(calls, account_update);
        }

        forest
    }
}

#[coverage(off)]
pub fn sign_account_updates(
    ctx: &mut FuzzerCtx,
    signer: &mut impl Signer<TransactionCommitment>,
    txn_commitment: &TransactionCommitment,
    full_txn_commitment: &TransactionCommitment,
    account_updates: &mut zkapp_command::CallForest<AccountUpdate>,
) {
    for acc_update in account_updates.0.iter_mut() {
        let account_update = &mut acc_update.elt.account_update;

        let signature = match account_update.authorization {
            zkapp_command::Control::Signature(_) => {
                let kp = ctx
                    .find_keypair(&account_update.body.public_key)
                    .cloned()
                    .unwrap_or_else(|| ctx.gen());
                let input = match account_update.body.use_full_commitment {
                    true => full_txn_commitment,
                    false => txn_commitment,
                };
                Some(signer.sign(&kp, &input))
            }
            _ => None,
        };

        if let Some(signature) = signature {
            account_update.authorization = zkapp_command::Control::Signature(signature);
        }

        sign_account_updates(
            ctx,
            signer,
            txn_commitment,
            full_txn_commitment,
            &mut acc_update.elt.calls,
        );
    }
}

impl Generator<ZkAppCommand> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> ZkAppCommand {
        self.gen.attempt_valid_zkapp = self.gen.rng.gen_bool(0.9);

        let mut zkapp_command = ZkAppCommand {
            fee_payer: self.gen(),
            account_updates: self.gen_from_token(TokenId::default(), 0),
            memo: self.gen(),
        };
        let (txn_commitment, full_txn_commitment) = get_transaction_commitments(&zkapp_command);
        let mut signer = mina_signer::create_kimchi(NetworkId::TESTNET);

        let keypair = match self.find_keypair(&zkapp_command.fee_payer.body.public_key) {
            Some(keypair) => keypair.clone(),
            None => self.gen(),
        };
        zkapp_command.fee_payer.authorization = signer.sign(&keypair, &full_txn_commitment);

        sign_account_updates(
            self,
            &mut signer,
            &txn_commitment,
            &full_txn_commitment,
            &mut zkapp_command.account_updates,
        );
        zkapp_command
    }
}

impl GeneratorFromAccount<PaymentPayload> for FuzzerCtx {
    #[coverage(off)]
    fn gen_from_account(&mut self, account: &Account) -> PaymentPayload {
        let is_source_account = self.gen.rng.gen_bool(0.9);

        let source_pk = if is_source_account {
            account.public_key.clone()
        } else {
            self.gen()
        };

        let receiver_pk = if is_source_account && self.gen.rng.gen_bool(0.9) {
            // same source & receiver
            source_pk.clone()
        } else {
            self.gen()
        };

        PaymentPayload {
            receiver_pk,
            amount: self.gen_from_account(account),
        }
    }
}

impl Generator<StakeDelegationPayload> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> StakeDelegationPayload {
        StakeDelegationPayload::SetDelegate {
            new_delegate: self.gen(),
        }
    }
}

#[coverage(off)]
fn sign_payload(keypair: &Keypair, payload: &SignedCommandPayload) -> Signature {
    let tx = TransactionUnionPayload::of_user_command_payload(payload);
    let mut signer = mina_signer::create_legacy(NetworkId::TESTNET);
    signer.sign(keypair, &tx)
}

impl Generator<SignedCommandPayload> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> SignedCommandPayload {
        let fee_payer_pk = if self.gen.rng.gen_bool(0.8) {
            self.random_keypair().public.into_compressed()
        } else {
            self.gen()
        };

        let account = self.get_account(&fee_payer_pk);

        let body = if account.is_some() && self.gen.rng.gen_bool(0.8) {
            signed_command::Body::Payment(self.gen_from_account(account.as_ref().unwrap()))
        } else {
            signed_command::Body::StakeDelegation(self.gen())
        };

        let fee = match account.as_ref() {
            Some(account) => self.gen_from_account(account),
            None => GeneratorRange64::gen_range(self, 0u64..=10_000_000u64),
        };

        let nonce = match account.as_ref() {
            Some(account) => self.gen_from_account(account),
            None => GeneratorRange32::gen_range(self, 0u32..=10_000_000u32),
        };

        SignedCommandPayload::create(
            fee,
            fee_payer_pk,
            nonce,
            self.gen_wrap(
                #[coverage(off)]
                |x| x.gen(),
            ),
            self.gen(),
            body,
        )
    }
}

impl Generator<SignedCommand> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> SignedCommand {
        let payload: SignedCommandPayload = self.gen();
        let keypair = if self.gen.rng.gen_bool(0.9) {
            self.find_keypair(&payload.common.fee_payer_pk)
                .cloned()
                .unwrap_or_else(|| self.gen())
        } else {
            self.gen()
        };

        let signature = sign_payload(&keypair, &payload);

        SignedCommand {
            payload,
            signer: keypair.public.into_compressed(),
            signature,
        }
    }
}

impl Generator<UserCommand> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> UserCommand {
        match vec![0, 1].choose(&mut self.gen.rng).unwrap() {
            0 => UserCommand::SignedCommand(Box::new(self.gen())),
            1 => UserCommand::ZkAppCommand(Box::new(self.gen())),
            _ => unimplemented!(),
        }
    }
}

impl Generator<Transaction> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> Transaction {
        Transaction::Command(self.gen())
    }
}

impl Generator<Number<u64>> for FuzzerCtx {
    fn gen(&mut self) -> Number<u64> {
        Number(self.gen.rng.gen())
    }
}

impl Generator<Number<i64>> for FuzzerCtx {
    fn gen(&mut self) -> Number<i64> {
        Number(self.gen.rng.gen())
    }
}

impl Generator<Number<u32>> for FuzzerCtx {
    fn gen(&mut self) -> Number<u32> {
        Number(self.gen.rng.gen())
    }
}

impl Generator<Number<i32>> for FuzzerCtx {
    fn gen(&mut self) -> Number<i32> {
        Number(self.gen.rng.gen())
    }
}

impl_default_generator_for_wrapper_type!(FuzzerCtx, MinaBaseLedgerHash0StableV1);
impl_default_generator_for_wrapper_type!(FuzzerCtx, LedgerHash, MinaBaseLedgerHash0StableV1);

impl_default_generator_for_wrapper_type!(FuzzerCtx, MinaBaseAccountIdDigestStableV1);
impl_default_generator_for_wrapper_type!(
    FuzzerCtx,
    TokenIdKeyHash,
    MinaBaseAccountIdDigestStableV1
);

impl_default_generator_for_wrapper_type!(FuzzerCtx, MinaBasePendingCoinbaseStackHashStableV1);
impl_default_generator_for_wrapper_type!(FuzzerCtx, MinaBasePendingCoinbaseCoinbaseStackStableV1);
impl_default_generator_for_wrapper_type!(FuzzerCtx, MinaBaseStackFrameStableV1);
impl_default_generator_for_wrapper_type!(FuzzerCtx, MinaBaseCallStackDigestStableV1);

impl_default_generator_for_wrapper_type!(
    FuzzerCtx,
    UnsignedExtendedUInt64Int64ForVersionTagsStableV1
);
impl_default_generator_for_wrapper_type!(FuzzerCtx, CurrencyAmountStableV1);
impl_default_generator_for_wrapper_type!(FuzzerCtx, CurrencyFeeStableV1);

impl_default_generator_for_wrapper_type!(FuzzerCtx, UnsignedExtendedUInt32StableV1);

impl Generator<MinaBasePendingCoinbaseStateStackStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> MinaBasePendingCoinbaseStateStackStableV1 {
        let init: MinaBasePendingCoinbaseStackHashStableV1 = self.gen();
        let curr: MinaBasePendingCoinbaseStackHashStableV1 = self.gen();

        MinaBasePendingCoinbaseStateStackStableV1 {
            init: init.into(),
            curr: curr.into(),
        }
    }
}

impl Generator<MinaBasePendingCoinbaseStackVersionedStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> MinaBasePendingCoinbaseStackVersionedStableV1 {
        let data: MinaBasePendingCoinbaseCoinbaseStackStableV1 = self.gen();

        MinaBasePendingCoinbaseStackVersionedStableV1 {
            data: data.into(),
            state: self.gen(),
        }
    }
}

impl Generator<SgnStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> SgnStableV1 {
        match self.gen.rng.gen_bool(0.5) {
            true => SgnStableV1::Pos,
            false => SgnStableV1::Neg,
        }
    }
}

impl Generator<SignedAmount> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> SignedAmount {
        SignedAmount {
            magnitude: self.gen(),
            sgn: self.gen(),
        }
    }
}

impl Generator<MinaBaseFeeExcessStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> MinaBaseFeeExcessStableV1 {
        MinaBaseFeeExcessStableV1(
            TokenFeeExcess {
                token: self.gen(),
                amount: self.gen(),
            },
            TokenFeeExcess {
                token: self.gen(),
                amount: self.gen(),
            },
        )
    }
}

impl Generator<MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
            stack_frame: self.gen(),
            call_stack: self.gen(),
            transaction_commitment: self.gen(),
            full_transaction_commitment: self.gen(),
            excess: self.gen(),
            supply_increase: self.gen(),
            ledger: self.gen(),
            success: self.gen(),
            account_update_index: self.gen(),
            failure_status_tbl: MinaBaseTransactionStatusFailureCollectionStableV1(List::new()),
            will_succeed: self.gen(),
        }
    }
}

impl Generator<MinaStateBlockchainStateValueStableV2LedgerProofStatementSource> for FuzzerCtx {
    #[coverage(off)]
    fn gen(&mut self) -> MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
        MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
            first_pass_ledger: self.gen(),
            second_pass_ledger: self.gen(),
            pending_coinbase_stack: self.gen(),
            local_state: self.gen(),
        }
    }
}
