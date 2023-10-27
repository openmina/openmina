use ark_serialize::EmptyFlags;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::number::Number;
use mina_p2p_messages::pseq::PaddedSeq;
use mina_p2p_messages::v2::{
    CompositionTypesBranchDataDomainLog2StableV1, CompositionTypesBranchDataStableV1,
    CompositionTypesDigestConstantStableV1, LedgerHash, LimbVectorConstantHex64StableV1,
    PicklesBaseProofsVerifiedStableV1,
    PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
    PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
    PicklesProofProofsVerified2ReprStableV2PrevEvals,
    PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals,
    PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
    PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA,
    PicklesProofProofsVerified2ReprStableV2Proof,
    PicklesProofProofsVerified2ReprStableV2ProofMessages,
    PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA,
    PicklesProofProofsVerified2ReprStableV2ProofOpenings,
    PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof,
    PicklesProofProofsVerified2ReprStableV2Statement,
    PicklesProofProofsVerified2ReprStableV2StatementFp,
    PicklesProofProofsVerified2ReprStableV2StatementPlonk,
    PicklesProofProofsVerified2ReprStableV2StatementProofState,
    PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    PicklesProofProofsVerifiedMaxStableV2,
    PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2,
    PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
    PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
};
use mina_signer::{
    CompressedPubKey, CurvePoint, Keypair, NetworkId, ScalarField, SecKey, Signature, Signer,
};
use ocaml_interop::{ocaml_export, OCaml, OCamlBytes, OCamlRef, ToOCaml};
use rand::distributions::{Alphanumeric, DistString};
use rand::seq::SliceRandom;
use std::rc::Rc;
use std::str::FromStr;
use std::{array, iter};
use tuple_map::*;

use crate::ffi::util::{deserialize, serialize};
use crate::scan_state::currency::{
    Amount, Balance, BlockTime, Fee, Length, Magnitude, MinMax, Nonce, Sgn, Signed, Slot,
};
use crate::scan_state::scan_state::ConstraintConstants;
use crate::scan_state::transaction_logic::protocol_state::{
    EpochData, EpochLedger, ProtocolStateView,
};
use crate::scan_state::transaction_logic::signed_command::{
    PaymentPayload, SignedCommand, SignedCommandPayload,
};
use crate::scan_state::transaction_logic::transaction_applied::{
    signed_command_applied, CommandApplied, TransactionApplied, Varying,
};
use crate::scan_state::transaction_logic::transaction_union_payload::TransactionUnionPayload;
use crate::scan_state::transaction_logic::zkapp_command::{
    self, AccountPreconditions, AccountUpdate, ClosedInterval, FeePayer, FeePayerBody, Numeric,
    OrIgnore, SetOrKeep, Update, WithHash, ZkAppCommand,
};
use crate::scan_state::transaction_logic::{
    apply_transaction, signed_command, Memo, Transaction, UserCommand,
};
use crate::staged_ledger::sparse_ledger::LedgerIntf;
use crate::{
    Account, AccountId, AuthRequired, CurveAffine, Mask, Permissions, PlonkVerificationKeyEvals,
    ProofVerified, Timing, TokenId, TokenSymbol, VerificationKey, VotingFor, ZkAppUri,
};

use ark_ec::{AffineCurve, ProjectiveCurve};
use ark_ff::UniformRand;
use ark_ff::{Field, SquareRootField, Zero};
use rand::rngs::SmallRng;
use rand::{self, Rng, SeedableRng};

fn sign_payload(keypair: &Keypair, payload: &SignedCommandPayload) -> Signature {
    let tx = TransactionUnionPayload::of_user_command_payload(payload);
    let mut signer = mina_signer::create_legacy(NetworkId::TESTNET);
    signer.sign(keypair, &tx)
}

fn new_signed_command(
    keypair: &Keypair,
    fee: Fee,
    fee_payer_pk: CompressedPubKey,
    nonce: Nonce,
    valid_until: Option<Slot>,
    memo: Memo,
    body: signed_command::Body,
) -> SignedCommand {
    let payload = SignedCommandPayload::create(fee, fee_payer_pk, nonce, valid_until, memo, body);
    let signature = sign_payload(keypair, &payload);

    SignedCommand {
        payload,
        signer: keypair.public.into_compressed(),
        signature,
    }
}

fn new_payment(
    source_pk: CompressedPubKey,
    receiver_pk: CompressedPubKey,
    amount: Amount,
) -> signed_command::Body {
    let payload = PaymentPayload {
        source_pk,
        receiver_pk,
        amount,
    };
    signed_command::Body::Payment(payload)
}

fn new_payment_tx(
    keypair: &Keypair,
    fee: Fee,
    fee_payer_pk: CompressedPubKey,
    nonce: Nonce,
    valid_until: Option<Slot>,
    memo: Memo,
    receiver_pk: CompressedPubKey,
    amount: Amount,
) -> SignedCommand {
    let body = new_payment(keypair.public.into_compressed(), receiver_pk, amount);
    new_signed_command(keypair, fee, fee_payer_pk, nonce, valid_until, memo, body)
}

/*
    Reimplement random key generation w/o the restriction on CryptoRgn trait.
    Since we are only using this for fuzzing we want a faster (unsafe) Rng like SmallRng.
*/
fn gen_sk(rng: &mut SmallRng) -> SecKey {
    let secret: ScalarField = ScalarField::rand(rng);
    SecKey::new(secret)
}

fn gen_keypair(rng: &mut SmallRng) -> Keypair {
    let sec_key = gen_sk(rng);
    let scalar = sec_key.into_scalar();
    let public = CurvePoint::prime_subgroup_generator()
        .mul(scalar)
        .into_affine();

    Keypair::from_parts_unsafe(scalar, public)
}

// Taken from ocaml_tests
/// Same values when we run `dune runtest src/lib/staged_ledger -f`
fn dummy_state_view(global_slot_since_genesis: Option<Slot>) -> ProtocolStateView {
    // TODO: Use OCaml implementation, not hardcoded value
    let f = |s: &str| Fp::from_str(s).unwrap();

    ProtocolStateView {
        snarked_ledger_hash: f(
            "19095410909873291354237217869735884756874834695933531743203428046904386166496",
        ),
        timestamp: BlockTime::from_u64(1600251300000),
        blockchain_length: Length::from_u32(1),
        min_window_density: Length::from_u32(77),
        last_vrf_output: (),
        total_currency: Amount::from_u64(10016100000000000),
        global_slot_since_hard_fork: Slot::from_u32(0),
        global_slot_since_genesis: global_slot_since_genesis.unwrap_or_else(Slot::zero),
        staking_epoch_data: EpochData {
            ledger: EpochLedger {
                hash: f(
                    "19095410909873291354237217869735884756874834695933531743203428046904386166496",
                ),
                total_currency: Amount::from_u64(10016100000000000),
            },
            seed: Fp::zero(),
            start_checkpoint: Fp::zero(),
            lock_checkpoint: Fp::zero(),
            epoch_length: Length::from_u32(1),
        },
        next_epoch_data: EpochData {
            ledger: EpochLedger {
                hash: f(
                    "19095410909873291354237217869735884756874834695933531743203428046904386166496",
                ),
                total_currency: Amount::from_u64(10016100000000000),
            },
            seed: f(
                "18512313064034685696641580142878809378857342939026666126913761777372978255172",
            ),
            start_checkpoint: Fp::zero(),
            lock_checkpoint: f(
                "9196091926153144288494889289330016873963015481670968646275122329689722912273",
            ),
            epoch_length: Length::from_u32(2),
        },
    }
}

/// Same values when we run `dune runtest src/lib/staged_ledger -f`
const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
    sub_windows_per_window: 11,
    ledger_depth: 35,
    work_delay: 2,
    block_window_duration_ms: 180000,
    transaction_capacity_log_2: 7,
    pending_coinbase_depth: 5,
    coinbase_amount: Amount::from_u64(720000000000),
    supercharged_coinbase_factor: 2,
    account_creation_fee: Fee::from_u64(1000000000),
    fork: None,
};

struct FuzzerCtx {
    constraint_constants: ConstraintConstants,
    txn_state_view: ProtocolStateView,
    ledger: Mask,
    rng: SmallRng,
    potential_senders: Vec<Keypair>,
    potential_new_accounts: Vec<Keypair>,
}

impl FuzzerCtx {
    fn new(seed: u64, constraint_constants: ConstraintConstants) -> Self {
        let depth = constraint_constants.ledger_depth as usize;
        Self {
            constraint_constants,
            txn_state_view: dummy_state_view(None),
            ledger: {
                let root = Mask::new_root(crate::Database::create(depth.try_into().unwrap()));
                root.make_child()
            },
            rng: SmallRng::seed_from_u64(seed),
            potential_senders: Vec::new(),
            potential_new_accounts: Vec::new(),
        }
    }

    fn create_inital_accounts(&mut self, n: usize) {
        for _ in 0..n {
            loop {
                let keypair = gen_keypair(&mut self.rng);

                if !self
                    .potential_senders
                    .iter()
                    .any(|x| x.public == keypair.public)
                {
                    let pk_compressed = keypair.public.into_compressed();
                    let account_id = AccountId::new(pk_compressed, TokenId::default());
                    let mut account = Account::initialize(&account_id);

                    account.balance =
                        Balance::from_u64(self.rng.gen_range(1_000_000_000..u64::MAX));
                    account.nonce = Nonce::from_u32(self.rng.gen_range(0..1000));
                    account.timing = Timing::Untimed;

                    self.potential_senders.push(keypair);
                    self.ledger.create_new_account(account_id, account).unwrap();
                    break;
                }
            }
        }
    }

    fn rnd_base_field(&mut self) -> Fp {
        let mut bf = None;

        // TODO: optimize by masking out MSBs from bytes and remove loop
        while bf.is_none() {
            let bytes = self.rng.gen::<[u8; 32]>();
            bf = Fp::from_random_bytes_with_flags::<EmptyFlags>(&bytes);
        }

        bf.unwrap().0
    }

    fn rnd_option<F, T>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        if self.rng.gen_bool(0.9) {
            None
        } else {
            Some(f(self))
        }
    }

    fn rnd_or_ignore<F, T>(&mut self, mut f: F) -> OrIgnore<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        if self.rng.gen_bool(0.9) {
            OrIgnore::Ignore
        } else {
            OrIgnore::Check(f(self))
        }
    }

    fn rnd_set_or_keep<F, T: Clone>(&mut self, mut f: F) -> SetOrKeep<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        if self.rng.gen_bool(0.9) {
            SetOrKeep::Keep
        } else {
            SetOrKeep::Set(f(self))
        }
    }

    fn rnd_closed_interval<F, T: MinMax>(&mut self, mut f: F) -> ClosedInterval<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        ClosedInterval {
            lower: f(self),
            upper: f(self),
        }
    }

    fn rnd_numeric<F, T: MinMax>(&mut self, mut f: F) -> Numeric<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        self.rnd_or_ignore(|x| x.rnd_closed_interval(|x| f(x)))
    }

    fn rnd_signed<F, T: Magnitude + Ord>(&mut self, mut f: F) -> Signed<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        let sgn = if self.rng.gen_bool(0.5) {
            Sgn::Pos
        } else {
            Sgn::Neg
        };

        Signed::create(f(self), sgn)
    }

    fn rnd_slot(&mut self) -> Slot {
        Slot::from_u32(self.rng.gen_range(
            self.txn_state_view.global_slot_since_genesis.as_u32()..Slot::max().as_u32(),
        ))
    }

    fn rnd_pubkey(&mut self) -> CompressedPubKey {
        let keypair = self
            .potential_senders
            .choose(&mut self.rng)
            .unwrap()
            .clone();
        keypair.public.into_compressed()
    }

    fn find_keypair(&mut self, pkey: &CompressedPubKey) -> Option<&Keypair> {
        self.potential_senders
            .iter()
            .find(|x| x.public.into_compressed() == *pkey)
    }

    fn rnd_pubkey_new(&mut self) -> CompressedPubKey {
        let keypair = gen_keypair(&mut self.rng);
        let pk = keypair.public.into_compressed();

        if !self
            .potential_senders
            .iter()
            .any(|x| x.public == keypair.public)
        {
            self.potential_new_accounts.push(keypair)
        }

        pk
    }

    fn rnd_memo(&mut self) -> Memo {
        Memo::with_number(self.rng.gen())
    }

    fn account_from_pubkey(&mut self, pkey: &CompressedPubKey) -> Account {
        let account_location = self
            .ledger
            .location_of_account(&AccountId::new(pkey.clone(), TokenId::default()));

        let location = account_location.unwrap();
        self.ledger.get(&location).unwrap()
    }

    fn rnd_balance_u64(&mut self, account: &Account) -> u64 {
        let balance = account.balance.as_u64();

        if balance > 1 {
            self.rng.gen_range(0..balance)
        } else {
            0
        }
    }

    fn rnd_fee(&mut self, account: &Account) -> Fee {
        Fee::from_u64(self.rnd_balance_u64(account))
    }

    fn rnd_balance(&mut self, account: &Account) -> Balance {
        Balance::from_u64(self.rnd_balance_u64(account))
    }

    fn rnd_amount(&mut self, account: &Account, fee: Fee) -> Amount {
        let balance = self.rnd_balance_u64(account);
        Amount::from_u64(balance.saturating_sub(fee.as_u64()))
    }

    fn rnd_fee_payer(&mut self) -> FeePayer {
        let public_key = self.rnd_pubkey();
        let account = self.account_from_pubkey(&public_key);
        FeePayer {
            body: FeePayerBody {
                public_key,
                fee: self.rnd_fee(&account),
                valid_until: self.rnd_option(Self::rnd_slot),
                nonce: account.nonce,
            },
            // filled later when tx is complete
            authorization: Signature::dummy(),
        }
    }

    fn rnd_fp(&mut self) -> Fp {
        Fp::rand(&mut self.rng)
    }

    fn rnd_curve_point<F: Field + SquareRootField + From<i32>>(&mut self) -> (F, F) {
        /*
            WARNING: we need to generate valid curve points to avoid binprot deserializarion
            exceptions in the OCaml side. However this is an expensive task.

            TODO: a more efficient way of doing this?
        */
        let mut x = F::rand(&mut self.rng);

        loop {
            let y_squared = x.square().mul(x).add(Into::<F>::into(5));

            if let Some(y) = y_squared.sqrt() {
                return (x, y);
            }

            x.add_assign(F::one());
        }
    }

    fn rnd_curve_affine(&mut self) -> CurveAffine<Fp> {
        let (x, y) = self.rnd_curve_point();
        CurveAffine::<Fp>(x, y)
    }

    fn rnd_plonk_verification_key_evals(&mut self) -> PlonkVerificationKeyEvals {
        PlonkVerificationKeyEvals {
            sigma: array::from_fn(|_| self.rnd_curve_affine()),
            coefficients: array::from_fn(|_| self.rnd_curve_affine()),
            generic: self.rnd_curve_affine(),
            psm: self.rnd_curve_affine(),
            complete_add: self.rnd_curve_affine(),
            mul: self.rnd_curve_affine(),
            emul: self.rnd_curve_affine(),
            endomul_scalar: self.rnd_curve_affine(),
        }
    }

    fn rnd_verification_key(&mut self) -> WithHash<VerificationKey> {
        let data = VerificationKey {
            max_proofs_verified: vec![ProofVerified::N0, ProofVerified::N1, ProofVerified::N2]
                .choose(&mut self.rng)
                .unwrap()
                .clone(),
            wrap_index: self.rnd_plonk_verification_key_evals(),
            wrap_vk: None, // TODO
        };
        let hash = data.digest();
        WithHash { data, hash }
    }

    fn rnd_auth_required(&mut self) -> AuthRequired {
        *vec![
            AuthRequired::None,
            AuthRequired::Either,
            AuthRequired::Proof,
            AuthRequired::Signature,
            AuthRequired::Impossible,
            //AuthRequired::Both,
        ]
        .choose(&mut self.rng)
        .unwrap()
    }

    fn rnd_permissions(&mut self) -> Permissions<AuthRequired> {
        Permissions::<AuthRequired> {
            edit_state: self.rnd_auth_required(),
            send: self.rnd_auth_required(),
            receive: self.rnd_auth_required(),
            set_delegate: self.rnd_auth_required(),
            set_permissions: self.rnd_auth_required(),
            set_verification_key: self.rnd_auth_required(),
            set_zkapp_uri: self.rnd_auth_required(),
            edit_sequence_state: self.rnd_auth_required(),
            set_token_symbol: self.rnd_auth_required(),
            increment_nonce: self.rnd_auth_required(),
            set_voting_for: self.rnd_auth_required(),
        }
    }

    fn rnd_zkapp_uri(&mut self) -> ZkAppUri {
        /*
            TODO: this needs to be fixed (assign a boundary) in the protocol since it is
            possible to set a zkApp URI of arbitrary size.

            Since the field is opaque to the Mina protocol logic, randomly generating
            URIs makes little sense and will consume a significant amount of ledger space.
        */
        ZkAppUri::new()
    }

    fn rnd_token_symbol(&mut self) -> TokenSymbol {
        /*
            TokenSymbol must be <= 6 **bytes**. This boundary doesn't exist at type-level,
            instead it is check by binprot after deserializing the *string* object:
            https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account.ml#L124

            We will let this function generate strings larger than 6 bytes with low probability,
            just to cover the error handling code, but must of the time we want to avoid failing
            this check.
        */
        if self.rng.gen_bool(0.9) {
            TokenSymbol::default()
        } else {
            let rnd_len = self.rng.gen_range(1..=6);
            // TODO: fix n random chars for n random bytes
            TokenSymbol(Alphanumeric.sample_string(&mut self.rng, rnd_len))
        }
    }

    fn rnd_timing(&mut self, account: &Account) -> zkapp_command::Timing {
        let fee = self.rnd_fee(account);
        let fee2 = self.rnd_fee(account);

        zkapp_command::Timing {
            initial_minimum_balance: self.rnd_balance(account),
            cliff_time: self.rnd_slot(),
            cliff_amount: self.rnd_amount(account, fee),
            vesting_period: self.rnd_slot(),
            vesting_increment: self.rnd_amount(account, fee2),
        }
    }

    fn rnd_voting_for(&mut self) -> VotingFor {
        VotingFor(self.rnd_fp())
    }

    fn rnd_update(&mut self, account: &Account) -> Update {
        Update {
            app_state: array::from_fn(|_| self.rnd_set_or_keep(Self::rnd_fp)),
            delegate: self.rnd_set_or_keep(Self::rnd_pubkey),
            verification_key: self.rnd_set_or_keep(Self::rnd_verification_key),
            permissions: self.rnd_set_or_keep(Self::rnd_permissions),
            zkapp_uri: self.rnd_set_or_keep(Self::rnd_zkapp_uri),
            token_symbol: self.rnd_set_or_keep(Self::rnd_token_symbol),
            timing: self.rnd_set_or_keep(|x| Self::rnd_timing(x, account)),
            voting_for: self.rnd_set_or_keep(Self::rnd_voting_for),
        }
    }

    fn rnd_events(&mut self) -> zkapp_command::Events {
        /*
           An Event is a list of arrays of Fp, there doesn't seem to be any limit
           neither in the size of the list or the array's size. The total size should
           be bounded by the transport protocol (currently libp2p, ~32MB).

           Since this field is ignored by nodes (except maybe for archive nodes), we
           we will generate empty events (at least for the moment).
        */
        zkapp_command::Events(Vec::new())
    }

    fn rnd_sequence_events(&mut self) -> zkapp_command::Actions {
        // See comment above in rnd_events
        zkapp_command::Actions(Vec::new())
    }

    fn rnd_block_time(&mut self) -> BlockTime {
        self.rng.gen()
    }

    fn rnd_length(&mut self) -> Length {
        self.rng.gen()
    }

    fn rnd_nonce(&mut self, account: &Account) -> Nonce {
        if self.rng.gen_bool(0.9) {
            account.nonce
        } else {
            self.rng.gen()
        }
    }

    fn rnd_epoch_data(&mut self) -> zkapp_command::EpochData {
        zkapp_command::EpochData {
            ledger: zkapp_command::EpochLedger {
                hash: self.rnd_or_ignore(Self::rnd_fp),
                total_currency: self.rnd_numeric(|x| x.rng.gen()),
            },
            seed: self.rnd_or_ignore(Self::rnd_fp),
            start_checkpoint: self.rnd_or_ignore(Self::rnd_fp),
            lock_checkpoint: self.rnd_or_ignore(Self::rnd_fp),
            epoch_length: self.rnd_numeric(Self::rnd_length),
        }
    }

    fn rnd_zkapp_preconditions(&mut self, account: &Account) -> zkapp_command::ZkAppPreconditions {
        zkapp_command::ZkAppPreconditions {
            snarked_ledger_hash: self.rnd_or_ignore(Self::rnd_fp),
            timestamp: self.rnd_numeric(Self::rnd_block_time),
            blockchain_length: self.rnd_numeric(Self::rnd_length),
            min_window_density: self.rnd_numeric(Self::rnd_length),
            last_vrf_output: (),
            total_currency: self.rnd_numeric(|x| Self::rnd_amount(x, account, Fee::from_u64(0))),
            global_slot_since_hard_fork: self.rnd_numeric(Self::rnd_slot),
            global_slot_since_genesis: self.rnd_numeric(Self::rnd_slot),
            staking_epoch_data: self.rnd_epoch_data(),
            next_epoch_data: self.rnd_epoch_data(),
        }
    }

    fn rnd_account(&mut self, account: &Account) -> zkapp_command::Account {
        zkapp_command::Account {
            balance: self.rnd_numeric(|x| Self::rnd_balance(x, account)),
            nonce: self.rnd_numeric(|x| Self::rnd_nonce(x, account)),
            receipt_chain_hash: self.rnd_or_ignore(Self::rnd_fp),
            delegate: self.rnd_or_ignore(Self::rnd_pubkey),
            state: array::from_fn(|_| self.rnd_or_ignore(Self::rnd_fp)),
            sequence_state: self.rnd_or_ignore(Self::rnd_fp),
            proved_state: self.rnd_or_ignore(|x| x.rng.gen_bool(0.1)),
            is_new: self.rnd_or_ignore(|x| x.rng.gen_bool(0.1)),
        }
    }

    fn rnd_account_preconditions(&mut self, account: &Account) -> AccountPreconditions {
        match vec![0, 1, 2].choose(&mut self.rng).unwrap() {
            0 => AccountPreconditions::Accept,
            1 => AccountPreconditions::Nonce(self.rnd_nonce(account)),
            _ => AccountPreconditions::Full(Box::new(self.rnd_account(account))),
        }
    }

    fn rnd_preconditions(&mut self, account: &Account) -> zkapp_command::Preconditions {
        zkapp_command::Preconditions {
            network: self.rnd_zkapp_preconditions(account),
            account: self.rnd_account_preconditions(account),
        }
    }

    fn rnd_authorization(&mut self) -> zkapp_command::AuthorizationKind {
        vec![
            zkapp_command::AuthorizationKind::NoneGiven,
            zkapp_command::AuthorizationKind::Signature,
            zkapp_command::AuthorizationKind::Proof,
        ]
        .choose(&mut self.rng)
        .unwrap()
        .clone()
    }

    fn rnd_wrap_challenges_vector(
        &mut self,
    ) -> PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
            inner: PaddedSeq(array::from_fn(|_| {
                LimbVectorConstantHex64StableV1(Number(self.rng.gen()))
            })),
        }
    }

    fn rnd_proof_state(&mut self) -> PicklesProofProofsVerified2ReprStableV2StatementProofState {
        PicklesProofProofsVerified2ReprStableV2StatementProofState {
            deferred_values:
                PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
                    plonk: PicklesProofProofsVerified2ReprStableV2StatementPlonk {
                        alpha: self.rnd_wrap_challenges_vector(),
                        beta: PaddedSeq(array::from_fn(|_| {
                            LimbVectorConstantHex64StableV1(Number(self.rng.gen()))
                        })),
                        gamma: PaddedSeq(array::from_fn(|_| {
                            LimbVectorConstantHex64StableV1(Number(self.rng.gen()))
                        })),
                        zeta: self.rnd_wrap_challenges_vector(),
                        joint_combiner: self.rnd_option(|x| Self::rnd_wrap_challenges_vector(x)),
                    },
                    combined_inner_product:
                        PicklesProofProofsVerified2ReprStableV2StatementFp::ShiftedValue(
                            mina_p2p_messages::bigint::BigInt::from(self.rnd_fp()),
                        ),
                    b: PicklesProofProofsVerified2ReprStableV2StatementFp::ShiftedValue(
                        mina_p2p_messages::bigint::BigInt::from(self.rnd_fp()),
                    ),
                    xi: self.rnd_wrap_challenges_vector(),
                    bulletproof_challenges: PaddedSeq(array::from_fn(|_| {
                        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
                            prechallenge: self.rnd_wrap_challenges_vector()
                        }
                    })),
                    branch_data: CompositionTypesBranchDataStableV1 {
                        proofs_verified: (vec![
                            PicklesBaseProofsVerifiedStableV1::N0,
                            PicklesBaseProofsVerifiedStableV1::N1,
                            PicklesBaseProofsVerifiedStableV1::N2,
                        ]
                        .choose(&mut self.rng)
                        .unwrap()
                        .clone(),),
                        domain_log2: CompositionTypesBranchDataDomainLog2StableV1(
                            mina_p2p_messages::char::Char(self.rng.gen()),
                        ),
                    },
                },
            sponge_digest_before_evaluations: CompositionTypesDigestConstantStableV1(PaddedSeq(
                array::from_fn(|_| LimbVectorConstantHex64StableV1(Number(self.rng.gen()))),
            )),
            messages_for_next_wrap_proof:
                PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
                    challenge_polynomial_commitment: self
                        .rnd_curve_point::<Fq>()
                        .map(mina_p2p_messages::bigint::BigInt::from),
                    old_bulletproof_challenges: PaddedSeq(array::from_fn(|_| {
                        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2(
                            PaddedSeq(array::from_fn(|_| {
                                PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
                                        prechallenge: self.rnd_wrap_challenges_vector()
                                    }
                            })),
                        )
                    })),
                },
        }
    }

    fn rnd_vec<F, T>(&mut self, mut f: F, size: usize) -> Vec<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        //let size = self.rng.gen_range(0..=max_size);
        iter::repeat_with(|| f(self)).take(size).collect()
    }

    fn rnd_proof(&mut self) -> zkapp_command::SideLoadedProof {
        let proof = PicklesProofProofsVerifiedMaxStableV2 {
            statement: PicklesProofProofsVerified2ReprStableV2Statement {
                proof_state: self.rnd_proof_state(),
                messages_for_next_step_proof: PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
                    app_state: (),
                    challenge_polynomial_commitments: self.rnd_vec(
                        |x| {
                            x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from)
                        },
                        1
                    ),
                    old_bulletproof_challenges: self.rnd_vec(
                        |x| {
                            PaddedSeq(array::from_fn(|_| {
                                PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
                                        prechallenge: x.rnd_wrap_challenges_vector()
                                    }
                            }))
                        },
                        1
                    )
                }
            },
            prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals {
                evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
                    public_input: self.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                    evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
                        w: PaddedSeq(
                            array::from_fn(|_| self.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ).into_iter().unzip()
                        )),
                        z: self.rnd_vec(
                            |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                            1
                        ).into_iter().unzip(),
                        s: PaddedSeq(
                            array::from_fn(|_| self.rnd_vec(
                                |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                1
                            ).into_iter().unzip()
                        )),
                        generic_selector: self.rnd_vec(
                            |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                            1
                        ).into_iter().unzip(),
                        poseidon_selector: self.rnd_vec(
                            |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                            1
                        ).into_iter().unzip(),
                        lookup: self.rnd_option(|x| {
                            PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA {
                                sorted: x.rnd_vec(|x| x.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                   1
                                ).into_iter().unzip(),
                                    1
                                ),
                                aggreg: x.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ).into_iter().unzip(),
                                table: x.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ).into_iter().unzip(),
                                runtime: x.rnd_option(|x| x.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ).into_iter().unzip())
                            }
                        })
                    }
                },
                ft_eval1:  mina_p2p_messages::bigint::BigInt::from(self.rnd_fp())
            },
            proof: PicklesProofProofsVerified2ReprStableV2Proof {
                messages: PicklesProofProofsVerified2ReprStableV2ProofMessages {
                    w_comm: PaddedSeq(
                        array::from_fn(|_| {
                            self.rnd_vec(
                                |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                1
                            )
                        }
                        )
                    ),
                    z_comm: self.rnd_vec(
                        |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                        1
                    ),
                    t_comm: self.rnd_vec(
                        |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                        1
                    ),
                    lookup: self.rnd_option(|x| {
                        PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA {
                            sorted: x.rnd_vec(
                                |x| x.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ),
                                1
                            ),
                            aggreg: x.rnd_vec(
                                    |x|x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ),
                            runtime: x.rnd_option(
                                |x| x.rnd_vec(
                                    |x|x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                )
                            )
                        }
                    })
                },
                openings: PicklesProofProofsVerified2ReprStableV2ProofOpenings {
                    proof: PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof {
                        lr: self.rnd_vec(
                            |x| {
                                (
                                    x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from)
                                )
                            },
                            1
                        ),
                        z_1: mina_p2p_messages::bigint::BigInt::from(self.rnd_fp()),
                        z_2: mina_p2p_messages::bigint::BigInt::from(self.rnd_fp()),
                        delta: self.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                        challenge_polynomial_commitment: self.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from)
                    },
                    evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
                        w: PaddedSeq(
                            array::from_fn(|_| self.rnd_vec(
                                |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                1
                            ).into_iter().unzip()
                        )),
                        z: self.rnd_vec(
                            |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                            1
                        ).into_iter().unzip(),
                        s: PaddedSeq(
                            array::from_fn(|_| self.rnd_vec(
                                |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                1
                            ).into_iter().unzip()
                        )),
                        generic_selector: self.rnd_vec(
                            |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                            1
                        ).into_iter().unzip(),
                        poseidon_selector: self.rnd_vec(
                            |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                            1
                        ).into_iter().unzip(),
                        lookup: self.rnd_option(|x| {
                            PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA {
                                sorted: x.rnd_vec(
                                    |x| x.rnd_vec(
                                        |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                        1
                                    ).into_iter().unzip(),
                                    1
                                ),
                                aggreg: x.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ).into_iter().unzip(),
                                table: x.rnd_vec(
                                    |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                    1
                                ).into_iter().unzip(),
                                runtime: x.rnd_option(
                                    |x| x.rnd_vec(
                                        |x| x.rnd_curve_point::<Fp>().map(mina_p2p_messages::bigint::BigInt::from),
                                        1
                                    ).into_iter().unzip()
                                )
                            }
                        })
                    },
                    ft_eval1: mina_p2p_messages::bigint::BigInt::from(self.rnd_fp())
                }
            },
        };
        Rc::new(proof)
    }

    fn rnd_control(&mut self) -> zkapp_command::Control {
        match vec![0, 1, 2].choose(&mut self.rng).unwrap() {
            0 => zkapp_command::Control::NoneGiven,
            // TODO: calculate signature after building the transaction
            1 => zkapp_command::Control::Signature(Signature::dummy()),
            _ => zkapp_command::Control::Proof(self.rnd_proof()),
        }
    }

    fn rnd_account_update(&mut self) -> AccountUpdate {
        let public_key = self.rnd_pubkey();
        let account = self.account_from_pubkey(&public_key);

        AccountUpdate {
            body: zkapp_command::Body {
                public_key,
                token_id: TokenId::default(), // TODO: randomize
                update: self.rnd_update(&account),
                balance_change: self
                    .rnd_signed(|x| Self::rnd_amount(x, &account, Fee::from_u64(0))),
                increment_nonce: self.rng.gen_bool(0.9),
                events: self.rnd_events(),
                sequence_events: self.rnd_sequence_events(),
                call_data: self.rnd_fp(),
                preconditions: self.rnd_preconditions(&account),
                use_full_commitment: self.rng.gen_bool(0.5),
                caller: TokenId::default(), // TODO: randomize (MinaBaseAccountUpdateCallTypeStableV1)
                authorization_kind: self.rnd_authorization(),
            },
            authorization: self.rnd_control(),
        }
    }

    fn rnd_forest(&mut self) -> zkapp_command::CallForest<AccountUpdate> {
        let mut forest = zkapp_command::CallForest::<AccountUpdate>::new();
        //let count = self.rng.gen_range(0..10);
        let count = 1;

        for _ in 0..count {
            let calls = /*if self.rng.gen_bool(0.8*) {*/
                None
            /*} else {
                Some(self.rnd_forest())
            }*/;

            forest = forest.cons(calls, self.rnd_account_update());
        }

        //println!("rnd_forest len {}", forest.0.len());
        forest
    }

    fn rnd_zkapp_command(&mut self) -> ZkAppCommand {
        let fee_payer = self.rnd_fee_payer();
        let account_updates = self.rnd_forest();
        let memo = self.rnd_memo();

        ZkAppCommand {
            fee_payer,
            account_updates,
            memo,
        }
    }

    fn rnd_payment_tx(&mut self) -> SignedCommand {
        let fee_payer_pk = self.rnd_pubkey();
        let account = self.account_from_pubkey(&fee_payer_pk);
        let fee = self.rnd_fee(&account);
        let valid_until = self.rnd_option(Self::rnd_slot);
        let memo = self.rnd_memo();
        let receiver_pk = if self.rng.gen_bool(0.8) {
            // use existing account
            self.rnd_pubkey()
        } else {
            // create new account
            self.rnd_pubkey_new()
        };
        let amount = self.rnd_amount(&account, fee);

        new_payment_tx(
            self.find_keypair(&fee_payer_pk).unwrap(),
            fee,
            fee_payer_pk,
            account.nonce,
            valid_until,
            memo,
            receiver_pk,
            amount,
        )
    }

    fn apply_transaction(&mut self, tx: &Transaction) -> Result<TransactionApplied, String> {
        apply_transaction(
            &self.constraint_constants,
            &self.txn_state_view,
            &mut self.ledger,
            &tx,
        )
    }

    fn rnd_transaction(&mut self) -> Transaction {
        let zkapp_command = self.rnd_zkapp_command();
        Transaction::Command(UserCommand::ZkAppCommand(Box::new(zkapp_command)))

        //let signed_command = self.rnd_payment_tx();
        //Transaction::Command(UserCommand::SignedCommand(Box::new(signed_command)))
    }

    fn get_ledger_root(&mut self) -> Fp {
        self.ledger.merkle_root()
    }

    fn get_ledger_accounts(&self) -> Vec<Account> {
        let locations = self.ledger.account_locations();
        locations
            .iter()
            .map(|x| self.ledger.get(x).unwrap())
            .collect()
    }
}

ocaml_export! {
    fn rust_transaction_fuzzer(
        rt,
        set_initial_accounts_method: OCamlRef<fn(OCamlBytes) -> OCamlBytes>,
        apply_transaction_method: OCamlRef<fn(OCamlBytes) -> OCamlBytes>,
    ) {
        println!("Rust called");
        let mut ctx = FuzzerCtx::new(0, CONSTRAINT_CONSTANTS);

        println!("New context");
        ctx.create_inital_accounts(10);

        println!("Initial accounts");
        let initial_accounts = serialize(&ctx.get_ledger_accounts());
        let ocaml_method = set_initial_accounts_method.to_boxroot(rt);
        let rust_ledger_root_hash = ctx.get_ledger_root();

        println!("calling set_initial_accounts (OCaml)");
        // Duplicate initial accounts in the OCaml side
        let ocaml_ledger_root_hash: OCaml<OCamlBytes> = ocaml_method.try_call(rt, &initial_accounts).unwrap();
        let x: Vec<u8> = ocaml_ledger_root_hash.to_rust();
        let mut ocaml_ledger_root_hash = Fp::from(deserialize::<LedgerHash>(x.as_slice()).0.clone());

        println!("Initial ledger hash =>\n  Rust: {:?}\n  OCaml: {:?}", rust_ledger_root_hash, ocaml_ledger_root_hash);
        assert!(ocaml_ledger_root_hash == rust_ledger_root_hash);
        let mut iter_num = 0;

        loop {
            println!("Iteration {}", iter_num);
            iter_num += 1;
            let tx = ctx.rnd_transaction();

            /*
            if let Transaction::Command(UserCommand::ZkAppCommand(x)) = &tx {

                for tmp in x.account_updates.0.iter() {
                    let tmp2 = tmp.elt.account_update.clone();
                    println!("acc update: {:?}", tmp2.protocol_state_precondition());

                    for call in tmp.elt.calls.0.iter() {
                        let tmp2 = call.elt.account_update.clone();
                        println!("call acc update: {:?}", tmp2.protocol_state_precondition());
                    }

                }
            }
            */

            //println!("tx {:?}", tx);

            /*
                We don't have generated types for Transaction, but we have one
                for UserCommand (MinaBaseUserCommandStableV2). Extract and
                serialize the inner UserCommand and let a OCaml wrapper build
                the transaction.
            */
            let user_command = match &tx {
                Transaction::Command(user_command) => serialize(user_command),
                _ => unimplemented!()
            };

            let ocaml_method = apply_transaction_method.to_boxroot(rt);
            match ocaml_method.try_call(rt, &user_command) {
                Ok(ledger_root_hash) => {
                    let x: Vec<u8> = ledger_root_hash.to_rust();
                    let root_hash_deserialized = deserialize::<LedgerHash>(x.as_slice()).0.clone();
                    ocaml_ledger_root_hash = Fp::from(root_hash_deserialized);

                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    panic!()
                }
            }


            let applied = ctx.apply_transaction(&tx);
            println!("tx: {:?} applied: {:?}", tx, applied);

            // Add new accounts created by the transaction to the potential senders list

            if applied.is_ok() {
                let new_accounts = match applied.unwrap().varying {
                    Varying::Command(command) => {
                        match command {
                            CommandApplied::SignedCommand(cmd) => {
                                match cmd.body {
                                    signed_command_applied::Body::Payments { new_accounts } => Some(new_accounts),
                                    _ => None
                                }
                            },
                            CommandApplied::ZkappCommand(cmd) => {
                                Some(cmd.new_accounts)
                            }
                        }
                    },
                    _ => unimplemented!()
                };

                if let Some(new_accounts) = new_accounts {
                    let new_accounts = ctx.potential_new_accounts
                    .iter()
                    .filter(|kp| new_accounts.iter().any(|acc| acc.public_key == kp.public.into_compressed()));

                    for acc in new_accounts {
                        if !ctx
                        .potential_senders
                        .iter()
                        .any(|x| x.public == acc.public)
                        {
                          ctx.potential_senders.push(acc.clone())
                        }
                    }

                    ctx.potential_new_accounts.clear();
                }
            }

            let rust_ledger_root_hash = ctx.get_ledger_root();

            println!("ledger hash =>\n  Rust: {:?}\n  OCaml: {:?}", rust_ledger_root_hash, ocaml_ledger_root_hash);
            assert!(ocaml_ledger_root_hash == rust_ledger_root_hash);

        }

        OCaml::unit()
    }

}
