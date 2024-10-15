use super::{
    context::{FuzzerCtx, PermissionModel},
    generator::{
        sign_account_updates, Generator, GeneratorFromAccount, GeneratorRange32, GeneratorRange64,
        GeneratorWrapper,
    },
};
use crate::transaction_fuzzer::generator::gen_curve_point;
use ark_ff::Zero;
use ledger::{
    generators::zkapp_command_builder::get_transaction_commitments,
    hash_with_kimchi,
    scan_state::{
        currency::{Amount, Balance, Fee, MinMax, Nonce, Signed, Slot},
        transaction_logic::{
            zkapp_command::{
                self, AccountPreconditions, AccountUpdate, Body, ClosedInterval, FeePayer,
                FeePayerBody, OrIgnore, Preconditions, SetOrKeep, Update, ZkAppCommand,
                ZkAppPreconditions,
            },
            Transaction, UserCommand,
        },
    },
    Account, AuthRequired, Permissions, Timing, TokenId, TokenSymbol, VerificationKey,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    array::ArrayN16,
    bigint::BigInt,
    pseq::PaddedSeq,
    v2::{
        PicklesProofProofsVerified2ReprStableV2, PicklesProofProofsVerified2ReprStableV2PrevEvals,
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals,
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
        PicklesWrapWireProofCommitmentsStableV1, PicklesWrapWireProofEvaluationsStableV1,
        PicklesWrapWireProofStableV1, PicklesWrapWireProofStableV1Bulletproof,
        TransactionSnarkProofStableV2, TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
        TransactionSnarkStableV2,
    },
};
use mina_signer::{CompressedPubKey, NetworkId, Signature, Signer};
use rand::{
    distributions::{Alphanumeric, DistString},
    seq::SliceRandom,
    Rng,
};

#[coverage(off)]
fn rand_elements(ctx: &mut FuzzerCtx, count: usize) -> Vec<usize> {
    let elements: Vec<usize> = (0..count).collect();
    // We give more weight to smaller amount of elements since in general we want to perform fewer mutations
    if let Ok(amount) = elements.choose_weighted(
        &mut ctx.gen.rng,
        #[coverage(off)]
        |x| elements.len() - x,
    ) {
        elements
            .choose_multiple(&mut ctx.gen.rng, *amount)
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}

pub trait Mutator<T> {
    fn mutate(&mut self, t: &mut T);
}

impl Mutator<BigInt> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut BigInt) {
        *t = self.gen();
    }
}

impl Mutator<(BigInt, BigInt)> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut (BigInt, BigInt)) {
        *t = gen_curve_point::<Fp>(self);
    }
}

impl Mutator<((BigInt, BigInt), (BigInt, BigInt))> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut ((BigInt, BigInt), (BigInt, BigInt))) {
        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.0 .0);
        }

        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.0 .1);
        }

        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.1 .0);
        }

        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.1 .1);
        }
    }
}

impl Mutator<(Vec<BigInt>, Vec<BigInt>)> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut (Vec<BigInt>, Vec<BigInt>)) {
        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.0);
        }

        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.1);
        }
    }
}

impl<T> Mutator<(ArrayN16<T>, ArrayN16<T>)> for FuzzerCtx
where
    FuzzerCtx: Mutator<ArrayN16<T>>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut (ArrayN16<T>, ArrayN16<T>)) {
        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.0);
        }

        if self.gen.rng.gen_bool(0.5) {
            self.mutate(&mut t.1);
        }
    }
}

impl<T> Mutator<Vec<T>> for FuzzerCtx
where
    FuzzerCtx: Mutator<T>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut Vec<T>) {
        if t.is_empty() {
            // TODO(binier): maybe gen
            // t.lr = self.gen();
            return;
        }
        for i in rand_elements(self, t.len()) {
            self.mutate(&mut t[i]);
        }
    }
}

impl<T, const N: usize> Mutator<PaddedSeq<T, N>> for FuzzerCtx
where
    FuzzerCtx: Mutator<T>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PaddedSeq<T, N>) {
        for i in rand_elements(self, N) {
            self.mutate(&mut t.0[i])
        }
    }
}

impl Mutator<FeePayerBody> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut FeePayerBody) {
        //let account = self.get_account(&t.public_key).unwrap();

        for option in rand_elements(self, 2) {
            match option {
                0 => t.fee = GeneratorRange64::<Fee>::gen_range(self, 0..=Fee::max().as_u64()),
                1 => {
                    t.valid_until = if self.gen.rng.gen_bool(0.5) {
                        Some(Slot::from_u32(
                            self.gen.rng.gen_range(0..=Slot::max().as_u32()),
                        ))
                    } else {
                        None
                    }
                }
                //2 => {
                //    t.nonce = GeneratorRange32::<Nonce>::gen_range(self, 0..=Nonce::max().as_u32())
                //}
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<FeePayer> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut FeePayer) {
        self.mutate(&mut t.body)
    }
}

impl Mutator<Fp> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut Fp) {
        *t = self.gen();
    }
}

impl<T: Clone> Mutator<SetOrKeep<T>> for FuzzerCtx
where
    FuzzerCtx: Mutator<T> + Generator<T>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut SetOrKeep<T>) {
        match t {
            SetOrKeep::Set(inner) => {
                if self.gen.rng.gen_bool(0.5) {
                    self.mutate(inner)
                } else {
                    *t = SetOrKeep::Keep;
                }
            }
            SetOrKeep::Keep => *t = SetOrKeep::Set(self.gen()),
        }
    }
}

impl<T, const N: usize> Mutator<[T; N]> for FuzzerCtx
where
    FuzzerCtx: Mutator<T>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut [T; N]) {
        for i in rand_elements(self, t.len()) {
            self.mutate(&mut t[i])
        }
    }
}

pub trait MutatorFromAccount<T> {
    fn mutate_from_account(&mut self, t: &mut T, account: &Account);
}

impl MutatorFromAccount<Permissions<AuthRequired>> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut Permissions<AuthRequired>, account: &Account) {
        let permission_model = self.find_permissions(&account.public_key).unwrap();

        match permission_model {
            PermissionModel::Any => {
                for option in rand_elements(self, 11) {
                    match option {
                        0 => t.edit_state = self.gen(),
                        1 => t.send = self.gen(),
                        2 => t.receive = self.gen(),
                        3 => t.set_delegate = self.gen(),
                        4 => t.set_permissions = self.gen(),
                        5 => t.set_verification_key = self.gen(),
                        6 => t.set_zkapp_uri = self.gen(),
                        7 => t.edit_action_state = self.gen(),
                        8 => t.set_token_symbol = self.gen(),
                        9 => t.increment_nonce = self.gen(),
                        10 => t.set_voting_for = self.gen(),
                        _ => unimplemented!(),
                    }
                }
            }
            // Don't mutate permissions in the rest of the models
            PermissionModel::Empty => (),
            PermissionModel::Initial => (),
            PermissionModel::Default => (),
            PermissionModel::TokenOwner => (),
        }
    }
}

impl<T: Clone> MutatorFromAccount<SetOrKeep<T>> for FuzzerCtx
where
    FuzzerCtx: MutatorFromAccount<T> + GeneratorFromAccount<T>,
{
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut SetOrKeep<T>, account: &Account) {
        match t {
            SetOrKeep::Set(inner) => {
                if self.gen.rng.gen_bool(0.5) {
                    self.mutate_from_account(inner, account)
                } else {
                    *t = SetOrKeep::Keep;
                }
            }
            SetOrKeep::Keep => *t = SetOrKeep::Set(self.gen_from_account(account)),
        }
    }
}

impl MutatorFromAccount<zkapp_command::Timing> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut zkapp_command::Timing, _account: &Account) {
        for option in rand_elements(self, 5) {
            match option {
                0 => t.initial_minimum_balance = self.gen(),
                1 => t.cliff_time = self.gen(),
                2 => t.cliff_amount = self.gen(),
                3 => t.vesting_period = self.gen(),
                4 => t.vesting_increment = self.gen(),
                _ => unimplemented!(),
            }
        }
    }
}

impl MutatorFromAccount<Timing> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut Timing, _account: &Account) {
        if let Timing::Timed {
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = t
        {
            for option in rand_elements(self, 5) {
                match option {
                    0 => *initial_minimum_balance = self.gen(),
                    1 => *cliff_time = self.gen(),
                    2 => *cliff_amount = self.gen(),
                    3 => *vesting_period = self.gen(),
                    4 => *vesting_increment = self.gen(),
                    _ => unimplemented!(),
                }
            }
        } else {
            if self.gen.rng.gen_bool(0.5) {
                *t = Timing::Timed {
                    initial_minimum_balance: self.gen(),
                    cliff_time: self.gen(),
                    cliff_amount: self.gen(),
                    vesting_period: self.gen(),
                    vesting_increment: self.gen(),
                }
            }
        }
    }
}

impl MutatorFromAccount<Update> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut Update, account: &Account) {
        for option in rand_elements(self, 8) {
            match option {
                0 => self.mutate(&mut t.app_state),
                1 => {
                    let keypair = if self.gen.rng.gen_bool(0.5) {
                        self.random_keypair()
                    } else {
                        self.gen()
                    };

                    t.delegate = SetOrKeep::Set(keypair.public.into_compressed())
                }
                2 => {
                    let data: VerificationKey = self.gen();
                    let hash = if self.gen.rng.gen_bool(0.5) {
                        data.digest()
                    } else {
                        self.gen()
                    };

                    t.verification_key = SetOrKeep::Set(zkapp_command::WithHash { data, hash });
                }
                3 => self.mutate_from_account(&mut t.permissions, account),
                4 => {
                    t.zkapp_uri = self.gen_wrap(
                        #[coverage(off)]
                        |x| x.gen(), // TODO
                    )
                }
                5 => {
                    let rnd_len = self.gen.rng.gen_range(1..=6);
                    // TODO: fix n random chars for n random bytes
                    t.token_symbol = SetOrKeep::Set(TokenSymbol(
                        Alphanumeric.sample_string(&mut self.gen.rng, rnd_len),
                    ));
                }
                6 => self.mutate_from_account(&mut t.timing, account),
                7 => t.voting_for = SetOrKeep::Set(self.gen()),
                _ => unimplemented!(),
            }
        }
    }
}

impl<T: Clone> Mutator<OrIgnore<T>> for FuzzerCtx
where
    FuzzerCtx: Mutator<T> + Generator<T>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut OrIgnore<T>) {
        match t {
            OrIgnore::Check(inner) => {
                if self.gen.rng.gen_bool(0.9) {
                    self.mutate(inner)
                } else {
                    *t = OrIgnore::Ignore;
                }
            }
            OrIgnore::Ignore => *t = OrIgnore::Check(self.gen()),
        }
    }
}

impl<T: Clone> MutatorFromAccount<OrIgnore<T>> for FuzzerCtx
where
    FuzzerCtx: MutatorFromAccount<T> + GeneratorFromAccount<T>,
{
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut OrIgnore<T>, account: &Account) {
        match t {
            OrIgnore::Check(inner) => {
                if self.gen.rng.gen_bool(0.9) {
                    self.mutate_from_account(inner, account)
                } else {
                    *t = OrIgnore::Ignore;
                }
            }
            OrIgnore::Ignore => *t = OrIgnore::Check(self.gen_from_account(account)),
        }
    }
}

impl Mutator<CompressedPubKey> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut CompressedPubKey) {
        *t = self.gen();
    }
}

impl Mutator<bool> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut bool) {
        *t = !*t;
    }
}

impl<T: Clone + MinMax> Mutator<ClosedInterval<T>> for FuzzerCtx
where
    FuzzerCtx: Mutator<T>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut ClosedInterval<T>) {
        for option in rand_elements(self, 8) {
            match option {
                0 => self.mutate(&mut t.lower),
                1 => self.mutate(&mut t.upper),
                _ => unimplemented!(),
            }
        }
    }
}

impl MutatorFromAccount<Balance> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut Balance, _account: &Account) {
        //*t = self.gen_from_account(account);
        *t = GeneratorRange64::<Balance>::gen_range(self, 0..=Balance::max().as_u64())
    }
}

impl MutatorFromAccount<Nonce> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut Nonce, _account: &Account) {
        //*t = self.gen_from_account(account);
        *t = GeneratorRange32::<Nonce>::gen_range(self, 0..=u32::MAX);
    }
}

impl<T: Clone + MinMax> MutatorFromAccount<ClosedInterval<T>> for FuzzerCtx
where
    FuzzerCtx: MutatorFromAccount<T>,
{
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut ClosedInterval<T>, account: &Account) {
        for option in rand_elements(self, 8) {
            match option {
                0 => self.mutate_from_account(&mut t.lower, account),
                1 => self.mutate_from_account(&mut t.upper, account),
                _ => unimplemented!(),
            }
        }
    }
}

impl MutatorFromAccount<zkapp_command::Account> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut zkapp_command::Account, account: &Account) {
        for option in rand_elements(self, 8) {
            match option {
                0 => self.mutate_from_account(&mut t.balance, account),
                1 => self.mutate_from_account(&mut t.nonce, account),
                2 => self.mutate(&mut t.receipt_chain_hash),
                3 => self.mutate(&mut t.delegate),
                4 => self.mutate(&mut t.state),
                5 => self.mutate(&mut t.action_state),
                6 => self.mutate(&mut t.proved_state),
                7 => self.mutate(&mut t.is_new),
                _ => unimplemented!(),
            }
        }
    }
}

impl MutatorFromAccount<AccountPreconditions> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut AccountPreconditions, account: &Account) {
        self.mutate_from_account(&mut t.0, account)
    }
}

impl Mutator<zkapp_command::EpochData> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut zkapp_command::EpochData) {
        for option in rand_elements(self, 5) {
            match option {
                0 => {
                    *t.ledger_mut() = zkapp_command::EpochLedger {
                        hash: OrIgnore::Check(self.gen()),
                        total_currency: OrIgnore::Check(self.gen_wrap(
                            #[coverage(off)]
                            |x| GeneratorRange64::<Amount>::gen_range(x, 0..=u64::MAX),
                        )),
                    }
                }
                1 => t.seed = OrIgnore::Check(self.gen()),
                2 => t.start_checkpoint = OrIgnore::Check(self.gen()),
                3 => t.lock_checkpoint = OrIgnore::Check(self.gen()),
                4 => {
                    t.epoch_length = OrIgnore::Check(self.gen_wrap(
                        #[coverage(off)]
                        |x| x.gen(),
                    ))
                }
                _ => unimplemented!(),
            }
        }
    }
}

impl MutatorFromAccount<ZkAppPreconditions> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut ZkAppPreconditions, _account: &Account) {
        for option in rand_elements(self, 7) {
            match option {
                0 => t.snarked_ledger_hash = OrIgnore::Check(self.gen()),
                1 => {
                    let blockchain_length = self.gen_wrap(
                        #[coverage(off)]
                        |x| x.gen(),
                    );
                    t.blockchain_length = OrIgnore::Check(blockchain_length);
                }
                2 => {
                    let min_window_density = self.gen_wrap(
                        #[coverage(off)]
                        |x| x.gen(),
                    );
                    t.min_window_density = OrIgnore::Check(min_window_density);
                }
                3 => {
                    let total_currency = self.gen_wrap(
                        #[coverage(off)]
                        |x| GeneratorRange64::<Amount>::gen_range(x, 0..=u64::MAX),
                    );
                    t.total_currency = OrIgnore::Check(total_currency);
                }
                4 => {
                    let global_slot_since_genesis = self.gen_wrap(
                        #[coverage(off)]
                        |x| Slot::from_u32(x.gen.rng.gen_range(0..Slot::max().as_u32())),
                    );
                    t.global_slot_since_genesis = OrIgnore::Check(global_slot_since_genesis);
                }
                5 => self.mutate(&mut t.staking_epoch_data),
                6 => self.mutate(&mut t.next_epoch_data),
                _ => unimplemented!(),
            }
        }
    }
}

impl MutatorFromAccount<Preconditions> for FuzzerCtx {
    #[coverage(off)]
    fn mutate_from_account(&mut self, t: &mut Preconditions, account: &Account) {
        for option in rand_elements(self, 2) {
            match option {
                0 => self.mutate_from_account(t.network_mut(), account),
                1 => self.mutate_from_account(&mut t.account, account),
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<Body> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut Body) {
        let account = self.get_account(&t.public_key).unwrap();

        for option in rand_elements(self, 11) {
            match option {
                0 => t.token_id = TokenId(self.gen()),
                1 => self.mutate_from_account(&mut t.update, &account),
                2 => {
                    t.balance_change = if self.gen.rng.gen_bool(0.5) {
                        let magnitude =
                            GeneratorRange64::<Amount>::gen_range(self, 0..=Amount::max().as_u64());
                        Signed::<Amount>::create(magnitude, self.gen())
                    } else {
                        Signed::<Amount>::zero()
                    }
                }
                3 => self.mutate(&mut t.increment_nonce),
                4 => t.events = self.gen(),
                5 => t.actions = self.gen(),
                6 => t.call_data = self.gen(),
                7 => self.mutate_from_account(&mut t.preconditions, &account),
                8 => self.mutate(&mut t.use_full_commitment),
                9 => (), // Can't mutate because it breaks binprot
                10 => {
                    let vk_hash = if self.gen.rng.gen_bool(0.5)
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
                    t.authorization_kind = options.choose(&mut self.gen.rng).unwrap().clone();
                }
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<AccountUpdate> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut AccountUpdate) {
        for option in rand_elements(self, 2) {
            match option {
                0 => self.mutate(&mut t.body),
                1 => {
                    if self.gen.rng.gen_bool(0.5) {
                        t.authorization = match t.body.authorization_kind {
                            zkapp_command::AuthorizationKind::NoneGiven => {
                                zkapp_command::Control::NoneGiven
                            }
                            zkapp_command::AuthorizationKind::Signature => {
                                zkapp_command::Control::Signature(Signature::dummy())
                            }
                            zkapp_command::AuthorizationKind::Proof(_) => {
                                zkapp_command::Control::Proof(self.gen())
                            }
                        };
                    } else {
                        t.authorization = match vec![0, 1, 2].choose(&mut self.gen.rng).unwrap() {
                            0 => zkapp_command::Control::NoneGiven,
                            1 => zkapp_command::Control::Signature(Signature::dummy()),
                            2 => zkapp_command::Control::Proof(self.gen()),
                            _ => unimplemented!(),
                        };
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<zkapp_command::CallForest<AccountUpdate>> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut zkapp_command::CallForest<AccountUpdate>) {
        for i in rand_elements(self, t.0.len()) {
            let tree_digest = {
                let tree = &mut t.0[i].elt;

                for option in rand_elements(self, 2) {
                    match option {
                        0 => {
                            self.mutate(&mut tree.account_update);
                            tree.account_update_digest = tree.account_update.digest();
                        }
                        1 => self.mutate(&mut tree.calls),
                        _ => unimplemented!(),
                    }
                }

                tree.digest()
            };

            let h_tl = if let Some(x) = t.0.get(i + 1) {
                x.stack_hash
            } else {
                Fp::zero()
            };

            t.0[i].stack_hash = hash_with_kimchi("MinaAcctUpdateCons", &[tree_digest, h_tl]);
        }
    }
}

#[coverage(off)]
pub fn fix_nonces(
    ctx: &mut FuzzerCtx,
    account_updates: &mut zkapp_command::CallForest<AccountUpdate>,
) {
    for acc_update in account_updates.0.iter_mut() {
        let account_update = &mut acc_update.elt.account_update;

        if let zkapp_command::Account {
            nonce: OrIgnore::Check(_),
            ..
        } = &account_update.body.preconditions.account.0
        {
            let account = ctx.get_account(&account_update.public_key()).unwrap();

            account_update.body.preconditions.account.0.nonce =
                OrIgnore::Check(ctx.gen_from_account(&account));
        }

        fix_nonces(ctx, &mut acc_update.elt.calls);
    }
}

impl Mutator<ZkAppCommand> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut ZkAppCommand) {
        for option in rand_elements(self, 3) {
            match option {
                0 => self.mutate(&mut t.fee_payer),
                1 => self.mutate(&mut t.account_updates),
                2 => t.memo = self.gen(),
                _ => unimplemented!(),
            }
        }

        // Fix fee_payer nonce.
        let public_key = t.fee_payer.body.public_key.clone();
        let account = self.get_account(&public_key).unwrap();
        t.fee_payer.body.nonce = self.gen_from_account(&account);

        // Fix account updates nonces.
        fix_nonces(self, &mut t.account_updates);

        let (txn_commitment, full_txn_commitment) = get_transaction_commitments(t);
        let mut signer = mina_signer::create_kimchi(NetworkId::TESTNET);

        if self.gen.rng.gen_bool(0.9) {
            let keypair = self.find_keypair(&t.fee_payer.body.public_key).unwrap();
            t.fee_payer.authorization = signer.sign(keypair, &full_txn_commitment);
        }

        if self.gen.rng.gen_bool(0.9) {
            sign_account_updates(
                self,
                &mut signer,
                &txn_commitment,
                &full_txn_commitment,
                &mut t.account_updates,
            );
        }
    }
}

impl Mutator<UserCommand> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut UserCommand) {
        match t {
            UserCommand::ZkAppCommand(zkapp_command) => self.mutate(zkapp_command.as_mut()),
            _ => unimplemented!(),
        }
    }
}

impl Mutator<Transaction> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut Transaction) {
        match t {
            Transaction::Command(user_command) => self.mutate(user_command),
            _ => unimplemented!(),
        }
    }
}

// impl Mutator<MinaStateSnarkedLedgerStateWithSokStableV2> for FuzzerCtx {
//     #[no_coverage]
//     fn mutate(&mut self, t: &mut MinaStateSnarkedLedgerStateWithSokStableV2) {
//         for option in rand_elements(self, 6) {
//             match option {
//                 // 0 => self.mutate(&mut t.source),
//                 // 1 => self.mutate(&mut t.target),
//                 // 2 => self.mutate(&mut t.connecting_ledger_left),
//                 // 3 => self.mutate(&mut t.connecting_ledger_right),
//                 // 4 => self.mutate(&mut t.supply_increase),
//                 // 5 => self.mutate(&mut t.fee_excess),
//                 // // 6 => self.mutate(&mut t.sok_digest),
//                 _ => unimplemented!(),
//             }
//         }
//     }
// }

impl Mutator<PicklesProofProofsVerified2ReprStableV2PrevEvals> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesProofProofsVerified2ReprStableV2PrevEvals) {
        for option in rand_elements(self, 2) {
            match option {
                // 0 => self.mutate(&mut t.statement),
                0 => self.mutate(&mut t.evals),
                1 => self.mutate(&mut t.ft_eval1),
                _ => unimplemented!(),
            }
        }
    }
}

impl<T> Mutator<ArrayN16<T>> for FuzzerCtx
where
    FuzzerCtx: Mutator<Vec<T>>,
{
    #[coverage(off)]
    fn mutate(&mut self, t: &mut ArrayN16<T>) {
        self.mutate(t.inner_mut());
    }
}

impl Mutator<PicklesWrapWireProofStableV1Bulletproof> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesWrapWireProofStableV1Bulletproof) {
        for option in rand_elements(self, 3) {
            match option {
                0 => self.mutate(&mut t.lr),
                1 => self.mutate(&mut t.z_1),
                2 => self.mutate(&mut t.z_2),
                3 => self.mutate(&mut t.delta),
                4 => self.mutate(&mut t.challenge_polynomial_commitment),
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals) {
        for option in rand_elements(self, 6) {
            match option {
                0 => self.mutate(&mut t.w),
                1 => self.mutate(&mut t.coefficients),
                2 => self.mutate(&mut t.z),
                3 => self.mutate(&mut t.s),
                4 => self.mutate(&mut t.generic_selector),
                5 => self.mutate(&mut t.poseidon_selector),
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals) {
        for option in rand_elements(self, 2) {
            match option {
                0 => self.mutate(&mut t.public_input),
                1 => self.mutate(&mut t.evals),
                _ => unimplemented!(),
            }
        }
    }
}

// impl Mutator<PicklesProofProofsVerified2ReprStableV2ProofOpenings> for FuzzerCtx {
//     #[coverage(off)]
//     fn mutate(&mut self, t: &mut PicklesProofProofsVerified2ReprStableV2ProofOpenings) {
//         for option in rand_elements(self, 3) {
//             match option {
//                 0 => self.mutate(&mut t.proof),
//                 1 => self.mutate(&mut t.evals),
//                 2 => self.mutate(&mut t.ft_eval1),
//                 _ => unimplemented!(),
//             }
//         }
//     }
// }

// impl Mutator<PicklesProofProofsVerified2ReprStableV2Proof> for FuzzerCtx {
//     #[coverage(off)]
//     fn mutate(&mut self, t: &mut PicklesProofProofsVerified2ReprStableV2Proof) {
//         for option in rand_elements(self, 2) {
//             match option {
//                 // 0 => self.mutate(&mut t.statement),
//                 0 => self.mutate(&mut t.messages),
//                 1 => self.mutate(&mut t.openings),
//                 _ => unimplemented!(),
//             }
//         }
//     }
// }

impl Mutator<PicklesWrapWireProofCommitmentsStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesWrapWireProofCommitmentsStableV1) {
        for option in rand_elements(self, 3) {
            match option {
                0 => self.mutate(&mut t.w_comm),
                1 => self.mutate(&mut t.z_comm),
                2 => self.mutate(&mut t.t_comm),
                _ => unreachable!(),
            }
        }
    }
}

impl Mutator<PicklesWrapWireProofEvaluationsStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesWrapWireProofEvaluationsStableV1) {
        for option in rand_elements(self, 10) {
            match option {
                0 => self.mutate(&mut t.w),
                1 => self.mutate(&mut t.coefficients),
                2 => self.mutate(&mut t.z),
                3 => self.mutate(&mut t.s),
                4 => self.mutate(&mut t.generic_selector),
                5 => self.mutate(&mut t.poseidon_selector),
                6 => self.mutate(&mut t.complete_add_selector),
                7 => self.mutate(&mut t.mul_selector),
                8 => self.mutate(&mut t.emul_selector),
                9 => self.mutate(&mut t.endomul_scalar_selector),
                _ => unreachable!(),
            }
        }
    }
}

impl Mutator<PicklesWrapWireProofStableV1> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesWrapWireProofStableV1) {
        for option in rand_elements(self, 4) {
            match option {
                0 => self.mutate(&mut t.commitments),
                1 => self.mutate(&mut t.evaluations),
                2 => self.mutate(&mut t.ft_eval1),
                3 => self.mutate(&mut t.bulletproof),
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<PicklesProofProofsVerified2ReprStableV2> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut PicklesProofProofsVerified2ReprStableV2) {
        for option in rand_elements(self, 2) {
            match option {
                // 0 => self.mutate(&mut t.statement),
                0 => self.mutate(&mut t.prev_evals),
                1 => self.mutate(&mut t.proof),
                _ => unimplemented!(),
            }
        }
    }
}

impl Mutator<TransactionSnarkProofStableV2> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut TransactionSnarkProofStableV2) {
        self.mutate(&mut t.0)
    }
}

impl Mutator<TransactionSnarkStableV2> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut TransactionSnarkStableV2) {
        self.mutate(&mut t.proof)
        // for option in rand_elements(self, 2) {
        //     match option {
        //         0 => self.mutate(&mut t.statement),
        //         1 => self.mutate(&mut t.proof),
        //         _ => unimplemented!(),
        //     }
        // }
    }
}

impl Mutator<TransactionSnarkScanStateLedgerProofWithSokMessageStableV2> for FuzzerCtx {
    #[coverage(off)]
    fn mutate(&mut self, t: &mut TransactionSnarkScanStateLedgerProofWithSokMessageStableV2) {
        self.mutate(&mut t.0 .0)
    }
}
