use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap, HashSet,
    },
    marker::PhantomData,
};

use ark_ff::{UniformRand, Zero};
use mina_hasher::Fp;
use mina_signer::{CompressedPubKey, Keypair};
use rand::{
    rngs::ThreadRng,
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::{
    gen_compressed, gen_keypair,
    scan_state::{
        currency::{
            Amount, Balance, BlockTime, BlockTimeSpan, Fee, Length, Magnitude, Nonce, Sgn, Signed,
            Slot,
        },
        transaction_logic::{
            protocol_state::{self, ProtocolStateView},
            zkapp_command::{
                self, AccountPreconditions, AccountUpdateSimple, AuthorizationKind, ClosedInterval,
                Control, FeePayer, FeePayerBody, Numeric, OrIgnore, Preconditions, SetOrKeep,
                Update, WithHash, WithStackHash, ZkAppCommand, ZkAppPreconditions,
            },
            Memo, Signature,
        },
        zkapp_logic,
    },
    staged_ledger::pre_diff_info::HashableCompressedPubKey,
    Account, AccountId, AuthRequired, BaseLedger, ControlTag, Mask, MyCowMut, Permissions,
    ReceiptChainHash, TokenId, VerificationKey, VotingFor, ZkAppAccount,
};

use mina_p2p_messages::v2::MinaBaseAccountUpdateCallTypeStableV1 as CallType;

/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L20
#[derive(Clone, Debug)]
pub enum Role {
    FeePayer,
    NewAccount,
    OrdinaryParticipant,
    NewTokenAccount,
}

/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L7
#[derive(Clone, Debug)]
pub enum NotPermitedOf {
    Delegate,
    AppState,
    VotingFor,
    VerificationKey,
    ZkappUri,
    TokenSymbol,
    Send,
    Receive,
}

/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L7
#[derive(Clone, Debug)]
pub enum Failure {
    InvalidAccountPrecondition,
    InvalidProtocolStatePrecondition,
    UpdateNotPermitted(NotPermitedOf),
}

/// keep max_account_updates small, so zkApp integration tests don't need lots
/// of block producers
/// because the other zkapp_command are split into a permissions-setter
/// and another account_update, the actual number of other zkapp_command is
/// twice this value, plus one, for the "balancing" account_update
/// when we have separate transaction accounts in integration tests
/// this number can be increased
///
/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L1111
const MAX_ACCOUNT_UPDATES: usize = 2;

/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L1113
const MAX_TOKEN_UPDATES: usize = 2;

/// Value when we run `dune runtest src/lib/staged_ledger -f`
const ACCOUNT_CREATION_FEE: Fee = Fee::from_u64(1000000000);

/// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_generators/zkapp_command_generators.ml#L443
fn gen_invalid_protocol_state_precondition(psv: &ProtocolStateView) -> ZkAppPreconditions {
    enum Tamperable {
        Timestamp,
        BlockchainLength,
        MinWindowDensity,
        TotalCurrency,
        GlobalSlotSinceHardFork,
        GlobalSlotSinceGenesis,
    }

    let mut rng = rand::thread_rng();

    let mut protocol_state_precondition = ZkAppPreconditions::accept();
    let lower = rng.gen::<bool>();

    match [
        Tamperable::Timestamp,
        Tamperable::BlockchainLength,
        Tamperable::MinWindowDensity,
        Tamperable::TotalCurrency,
        Tamperable::GlobalSlotSinceHardFork,
        Tamperable::GlobalSlotSinceGenesis,
    ]
    .choose(&mut rng)
    .unwrap()
    {
        Tamperable::Timestamp => {
            let timestamp: ClosedInterval<BlockTime> = {
                // TODO: Ocaml uses 1_000_000L 60_000_000L, not sure what are those `L`
                let epsilon = rng.gen_range(1_000_000..60_000_000);
                let epsilon = BlockTimeSpan::of_ms(epsilon);

                if lower || psv.timestamp > (BlockTime::zero().add(epsilon)) {
                    ClosedInterval {
                        lower: BlockTime::zero(),
                        upper: psv.timestamp.sub(epsilon),
                    }
                } else {
                    ClosedInterval {
                        lower: psv.timestamp.add(epsilon),
                        upper: BlockTime::max(),
                    }
                }
            };
            protocol_state_precondition.timestamp = OrIgnore::Check(timestamp);
        }
        Tamperable::BlockchainLength => {
            let blockchain_length = {
                let epsilon = Length::from_u32(rng.gen_range(1..10));

                if lower || psv.blockchain_length > epsilon {
                    ClosedInterval {
                        lower: Length::zero(),
                        upper: psv
                            .blockchain_length
                            .checked_sub(&epsilon)
                            .unwrap_or_else(Length::zero),
                    }
                } else {
                    ClosedInterval {
                        lower: psv.blockchain_length.checked_add(&epsilon).unwrap(),
                        upper: Length::max(),
                    }
                }
            };

            protocol_state_precondition.blockchain_length = OrIgnore::Check(blockchain_length);
        }
        Tamperable::MinWindowDensity => {
            let min_window_density = {
                let epsilon = Length::from_u32(rng.gen_range(1..10));

                if lower || psv.min_window_density > epsilon {
                    ClosedInterval {
                        lower: Length::zero(),
                        upper: psv
                            .min_window_density
                            .checked_sub(&epsilon)
                            .unwrap_or_else(Length::zero),
                    }
                } else {
                    // TODO: This should be `psv.min_window_density` here
                    //       Should open PR on mina repo
                    ClosedInterval {
                        lower: psv.blockchain_length.checked_add(&epsilon).unwrap(),
                        upper: Length::max(),
                    }
                }
            };

            protocol_state_precondition.min_window_density = OrIgnore::Check(min_window_density);
        }
        Tamperable::TotalCurrency => {
            let total_currency = {
                let epsilon = Amount::from_u64(rng.gen_range(1_000..1_000_000_000));

                if lower || psv.total_currency > epsilon {
                    ClosedInterval {
                        lower: Amount::zero(),
                        upper: psv
                            .total_currency
                            .checked_sub(&epsilon)
                            .unwrap_or_else(Amount::zero),
                    }
                } else {
                    ClosedInterval {
                        lower: psv.total_currency.checked_add(&epsilon).unwrap(),
                        upper: Amount::max(),
                    }
                }
            };

            protocol_state_precondition.total_currency = OrIgnore::Check(total_currency);
        }
        Tamperable::GlobalSlotSinceHardFork => {
            let global_slot_since_hard_fork = {
                let epsilon = Slot::from_u32(rng.gen_range(1..10));

                if lower || psv.global_slot_since_hard_fork > epsilon {
                    ClosedInterval {
                        lower: Slot::zero(),
                        upper: psv
                            .global_slot_since_hard_fork
                            .checked_sub(&epsilon)
                            .unwrap_or_else(Slot::zero),
                    }
                } else {
                    ClosedInterval {
                        lower: psv
                            .global_slot_since_hard_fork
                            .checked_add(&epsilon)
                            .unwrap(),
                        upper: Slot::max(),
                    }
                }
            };

            protocol_state_precondition.global_slot_since_hard_fork =
                OrIgnore::Check(global_slot_since_hard_fork);
        }
        Tamperable::GlobalSlotSinceGenesis => {
            let global_slot_since_genesis = {
                let epsilon = Slot::from_u32(rng.gen_range(1..10));

                if lower || psv.global_slot_since_genesis > epsilon {
                    ClosedInterval {
                        lower: Slot::zero(),
                        upper: psv
                            .global_slot_since_genesis
                            .checked_sub(&epsilon)
                            .unwrap_or_else(Slot::zero),
                    }
                } else {
                    ClosedInterval {
                        lower: psv.global_slot_since_genesis.checked_add(&epsilon).unwrap(),
                        upper: Slot::max(),
                    }
                }
            };

            protocol_state_precondition.global_slot_since_genesis =
                OrIgnore::Check(global_slot_since_genesis);
        }
    }

    protocol_state_precondition
}

fn closed_interval_exact<T: Copy>(value: T) -> ClosedInterval<T> {
    ClosedInterval {
        lower: value,
        upper: value,
    }
}

/// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_generators/zkapp_command_generators.ml#L319
fn gen_epoch_data_predicate(epoch_data: &protocol_state::EpochData) -> zkapp_command::EpochData {
    let mut rng = rand::thread_rng();

    let ledger = {
        let hash = OrIgnore::gen(|| epoch_data.ledger.hash);

        let total_currency =
            OrIgnore::gen(|| closed_interval_exact(epoch_data.ledger.total_currency));

        zkapp_command::EpochLedger {
            hash,
            total_currency,
        }
    };

    let seed = OrIgnore::gen(|| epoch_data.seed);
    let start_checkpoint = OrIgnore::gen(|| epoch_data.start_checkpoint);
    let lock_checkpoint = OrIgnore::gen(|| epoch_data.lock_checkpoint);

    let epoch_length = OrIgnore::gen(|| {
        let mut gen = || Length::from_u32(rng.gen_range(0..10));

        ClosedInterval {
            lower: epoch_data
                .epoch_length
                .checked_sub(&gen())
                .unwrap_or_else(Length::zero),
            upper: epoch_data.epoch_length.checked_add(&gen()).unwrap(),
        }
    });

    zkapp_command::EpochData {
        ledger,
        seed,
        start_checkpoint,
        lock_checkpoint,
        epoch_length,
    }
}

/// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_generators/zkapp_command_generators.ml#L367
fn gen_protocol_state_precondition(psv: &ProtocolStateView) -> ZkAppPreconditions {
    let mut rng = rand::thread_rng();

    let snarked_ledger_hash = OrIgnore::gen(|| psv.snarked_ledger_hash);

    let timestamp = OrIgnore::gen(|| {
        let mut gen = || rng.gen_range(0..60_000_000);

        ClosedInterval {
            lower: psv.timestamp.sub(BlockTimeSpan::of_ms(gen())),
            upper: psv.timestamp.add(BlockTimeSpan::of_ms(gen())),
        }
    });

    let blockchain_length = OrIgnore::gen(|| {
        let mut gen = || Length::from_u32(rng.gen_range(0..10));

        ClosedInterval {
            lower: psv
                .blockchain_length
                .checked_sub(&gen())
                .unwrap_or_else(Length::zero),
            upper: psv.blockchain_length.checked_add(&gen()).unwrap(),
        }
    });

    let min_window_density = OrIgnore::gen(|| {
        let mut gen = || Length::from_u32(rng.gen_range(0..10));

        ClosedInterval {
            lower: psv
                .min_window_density
                .checked_sub(&gen())
                .unwrap_or_else(Length::zero),
            upper: psv.min_window_density.checked_add(&gen()).unwrap(),
        }
    });

    let total_currency = OrIgnore::gen(|| {
        let mut gen = || Amount::from_u64(rng.gen_range(0..1_000_000_000));

        ClosedInterval {
            lower: psv
                .total_currency
                .checked_sub(&gen())
                .unwrap_or_else(Amount::zero),
            upper: psv
                .total_currency
                .checked_add(&gen())
                .unwrap_or(psv.total_currency),
        }
    });

    let global_slot_since_hard_fork = OrIgnore::gen(|| {
        let mut gen = || Slot::from_u32(rng.gen_range(0..10));

        ClosedInterval {
            lower: psv
                .global_slot_since_hard_fork
                .checked_sub(&gen())
                .unwrap_or_else(Slot::zero),
            upper: psv.global_slot_since_hard_fork.checked_add(&gen()).unwrap(),
        }
    });

    let global_slot_since_genesis = OrIgnore::gen(|| {
        let mut gen = || Slot::from_u32(rng.gen_range(0..10));

        ClosedInterval {
            lower: psv
                .global_slot_since_genesis
                .checked_sub(&gen())
                .unwrap_or_else(Slot::zero),
            upper: psv.global_slot_since_genesis.checked_add(&gen()).unwrap(),
        }
    });

    let staking_epoch_data = gen_epoch_data_predicate(&psv.staking_epoch_data);
    let next_epoch_data = gen_epoch_data_predicate(&psv.next_epoch_data);

    ZkAppPreconditions {
        snarked_ledger_hash,
        timestamp,
        blockchain_length,
        min_window_density,
        last_vrf_output: (),
        total_currency,
        global_slot_since_hard_fork,
        global_slot_since_genesis,
        staking_epoch_data,
        next_epoch_data,
    }
}

fn gen_account_precondition_from_account(
    failure: Option<Failure>,
    first_use_of_account: bool,
    account: &Account,
) -> AccountPreconditions {
    let mut rng = rand::thread_rng();

    let Account {
        balance,
        nonce,
        receipt_chain_hash,
        delegate,
        zkapp,
        ..
    } = account;

    // choose constructor
    if rng.gen() {
        // Full

        let balance = OrIgnore::gen(|| {
            let balance_change_int = rng.gen_range(1..10_000_000);
            let balance_change = Balance::from_u64(balance_change_int);

            let lower = balance
                .checked_sub(&balance_change)
                .unwrap_or_else(Balance::zero);
            let upper = balance
                .checked_add(&balance_change)
                .unwrap_or_else(Balance::max);

            ClosedInterval { lower, upper }
        });

        let nonce = OrIgnore::gen(|| {
            let nonce_change_int = rng.gen_range(1..10);
            let nonce_change = Nonce::from_u32(nonce_change_int);

            let lower = nonce.checked_sub(&nonce_change).unwrap_or_else(Nonce::zero);
            let upper = nonce.checked_add(&nonce_change).unwrap_or_else(Nonce::max);

            ClosedInterval { lower, upper }
        });

        let receipt_chain_hash = if first_use_of_account {
            OrIgnore::Check(receipt_chain_hash.clone())
        } else {
            OrIgnore::Ignore
        };

        let delegate = match delegate {
            Some(delegate) => OrIgnore::gen(|| delegate.clone()),
            None => OrIgnore::Ignore,
        };

        let (state, sequence_state, proved_state, is_new) = match zkapp {
            None => {
                // let len = Pickles_types.Nat.to_int Zkapp_state.Max_state_size.n

                let state = std::array::from_fn(|_| OrIgnore::Ignore);
                let sequence_state = OrIgnore::Ignore;
                let proved_state = OrIgnore::Ignore;
                let is_new = OrIgnore::Ignore;

                (state, sequence_state, proved_state, is_new)
            }
            Some(ZkAppAccount {
                app_state,
                sequence_state,
                proved_state,
                ..
            }) => {
                let state = std::array::from_fn(|i| OrIgnore::gen(|| app_state[i]));

                let sequence_state = {
                    // choose a value from account sequence state
                    OrIgnore::Check(sequence_state.choose(&mut rng).copied().unwrap())
                };

                let proved_state = OrIgnore::Check(*proved_state);

                // when we apply the generated Zkapp_command.t, the account
                // is always in the ledger
                let is_new = OrIgnore::Check(false);

                (state, sequence_state, proved_state, is_new)
            }
        };

        let mut predicate_account = zkapp_command::Account {
            balance,
            nonce,
            receipt_chain_hash: receipt_chain_hash.map(|a| a.0),
            delegate,
            state,
            sequence_state,
            proved_state,
            is_new,
        };

        let Account { balance, nonce, .. } = account;

        if let Some(Failure::InvalidAccountPrecondition) = failure {
            #[derive(Clone, Copy)]
            enum Tamperable {
                Balance,
                Nonce,
                ReceiptChainHash,
                Delegate,
                State,
                SequenceState,
                ProvedState,
            }

            // tamper with account using randomly chosen item
            match [
                Tamperable::Balance,
                Tamperable::Nonce,
                Tamperable::ReceiptChainHash,
                Tamperable::Delegate,
                Tamperable::State,
                Tamperable::SequenceState,
                Tamperable::ProvedState,
            ]
            .choose(&mut rng)
            .copied()
            .unwrap()
            {
                Tamperable::Balance => {
                    let new_balance = if balance.is_zero() {
                        Balance::max()
                    } else {
                        Balance::zero()
                    };

                    let balance = OrIgnore::Check(ClosedInterval {
                        lower: new_balance,
                        upper: new_balance,
                    });

                    predicate_account.balance = balance;
                }
                Tamperable::Nonce => {
                    let new_nonce = if nonce.is_zero() {
                        Nonce::max()
                    } else {
                        Nonce::zero()
                    };

                    let nonce = Numeric::gen(|| ClosedInterval::gen(|| new_nonce));

                    predicate_account.nonce = nonce;
                }
                Tamperable::ReceiptChainHash => {
                    let receipt_chain_hash = OrIgnore::gen(ReceiptChainHash::gen);

                    predicate_account.receipt_chain_hash = receipt_chain_hash.map(|v| v.0);
                }
                Tamperable::Delegate => {
                    let delegate = OrIgnore::gen(|| gen_keypair().public.into_compressed());

                    predicate_account.delegate = delegate;
                }
                Tamperable::State => {
                    let field = predicate_account.state.choose_mut(&mut rng).unwrap();
                    *field = OrIgnore::Check(Fp::rand(&mut rng));
                }
                Tamperable::SequenceState => {
                    predicate_account.sequence_state = OrIgnore::Check(Fp::rand(&mut rng));
                }
                Tamperable::ProvedState => {
                    let proved_state = match predicate_account.proved_state {
                        OrIgnore::Check(b) => OrIgnore::Check(!b),
                        OrIgnore::Ignore => OrIgnore::Check(true),
                    };

                    predicate_account.proved_state = proved_state;
                }
            };

            AccountPreconditions::Full(Box::new(predicate_account))
        } else {
            AccountPreconditions::Full(Box::new(predicate_account))
        }
    } else {
        // Nonce
        let Account { nonce, .. } = account;

        match failure {
            Some(Failure::InvalidAccountPrecondition) => AccountPreconditions::Nonce(nonce.succ()),
            _ => AccountPreconditions::Nonce(*nonce),
        }
    }
}

struct AccountUpdateBodyComponents<A, B, C, D> {
    public_key: CompressedPubKey,
    update: Update,
    token_id: C,
    balance_change: A,
    increment_nonce: bool,
    events: zkapp_command::Events,
    sequence_events: zkapp_command::SequenceEvents,
    call_data: Fp,
    call_depth: usize,
    protocol_state_precondition: ZkAppPreconditions,
    account_precondition: D,
    use_full_commitment: B,
    caller: CallType,
    authorization_kind: AuthorizationKind,
}

impl<B, C> AccountUpdateBodyComponents<Fee, B, C, Nonce> {
    /// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_generators/zkapp_command_generators.ml#L576
    fn to_fee_payer(&self) -> FeePayerBody {
        FeePayerBody {
            public_key: self.public_key.clone(),
            fee: self.balance_change,
            valid_until: match self.protocol_state_precondition.global_slot_since_genesis {
                OrIgnore::Ignore => None,
                OrIgnore::Check(ClosedInterval { lower: _, upper }) => Some(upper),
            },
            nonce: self.account_precondition,
        }
    }
}

impl AccountUpdateBodyComponents<Signed<Amount>, bool, TokenId, AccountPreconditions> {
    /// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_generators/zkapp_command_generators.ml#L592
    fn to_typical_account_update(self) -> zkapp_command::BodySimple {
        zkapp_command::BodySimple {
            public_key: self.public_key,
            token_id: self.token_id,
            update: self.update,
            balance_change: self.balance_change,
            increment_nonce: self.increment_nonce,
            events: self.events,
            sequence_events: self.sequence_events,
            call_data: self.call_data,
            call_depth: self.call_depth,
            preconditions: {
                Preconditions {
                    network: self.protocol_state_precondition,
                    account: self.account_precondition,
                }
            },
            use_full_commitment: self.use_full_commitment,
            caller: self.caller,
            authorization_kind: self.authorization_kind,
        }
    }
}

struct BodyComponentsParams<'a, A, B, C, D> {
    update: Option<Update>,
    account_id: Option<AccountId>,
    token_id: Option<TokenId>,
    caller: Option<CallType>,
    account_ids_seen: Option<&'a mut HashSet<AccountId>>,
    account_state_tbl: &'a mut HashMap<AccountId, (Account, Role)>,
    vk: Option<&'a WithHash<VerificationKey>>,
    failure: Option<&'a Failure>,
    new_account: Option<bool>,
    zkapp_account: Option<bool>,
    is_fee_payer: Option<bool>,
    available_public_keys: Option<&'a mut HashSet<HashableCompressedPubKey>>,
    permissions_auth: Option<ControlTag>,
    required_balance_change: Option<A>,
    protocol_state_view: Option<&'a ProtocolStateView>,
    zkapp_account_ids: &'a [AccountId],
    increment_nonce: (B, bool),
    authorization_tag: ControlTag,
    _phantom: PhantomData<(C, D)>,
}

/// The type `a` is associated with the `delta` field, which is an unsigned fee
/// for the fee payer, and a signed amount for other zkapp_command.
/// The type `b` is associated with the `use_full_commitment` field, which is
/// `unit` for the fee payer, and `bool` for other zkapp_command.
/// The type `c` is associated with the `token_id` field, which is `unit` for the
/// fee payer, and `Token_id.t` for other zkapp_command.
/// The type `d` is associated with the `account_precondition` field, which is
/// a nonce for the fee payer, and `Account_precondition.t` for other zkapp_command
fn gen_account_update_body_components<A, B, C, D>(
    params: BodyComponentsParams<A, B, C, D>,
    gen_balance_change: impl Fn(&Account) -> A,
    gen_use_full_commitment: impl Fn(&AccountPreconditions) -> B,
    f_balance_change: impl Fn(&A) -> Signed<Amount>,
    f_token_id: impl Fn(&TokenId) -> C,
    f_account_precondition: impl Fn(bool, &Account) -> D,
    f_account_update_account_precondition: impl Fn(&D) -> AccountPreconditions,
) -> AccountUpdateBodyComponents<A, B, C, D> {
    let BodyComponentsParams {
        update,
        account_id,
        token_id,
        caller,
        account_ids_seen,
        account_state_tbl,
        vk,
        failure,
        new_account,
        zkapp_account,
        is_fee_payer,
        available_public_keys,
        permissions_auth,
        required_balance_change,
        protocol_state_view,
        zkapp_account_ids,
        increment_nonce,
        authorization_tag,
        _phantom,
    } = params;

    let mut rng = rand::thread_rng();

    let new_account = new_account.unwrap_or(false);
    let zkapp_account = zkapp_account.unwrap_or(false);
    let is_fee_payer = is_fee_payer.unwrap_or(false);

    // fee payers have to be in the ledger
    assert!(!(is_fee_payer && new_account));

    let token_account = token_id.is_some();

    let mut update = match update {
        None => Update::gen(
            Some(token_account),
            Some(zkapp_account),
            vk,
            permissions_auth,
        ),
        Some(update) => update,
    };

    // account_update_increment_nonce for fee payer is unit and increment_nonce is true
    let (account_update_increment_nonce, increment_nonce) = increment_nonce;

    let verification_key = match vk {
        Some(vk) => vk.clone(),
        None => {
            let dummy = VerificationKey::dummy();
            let hash = dummy.digest();
            WithHash { data: dummy, hash }
        }
    };

    let mut account = if new_account {
        assert!(
            account_id.is_none(),
            "gen_account_update_body: new account_update is true, but an account \
             id, presumably from an existing account, was supplied"
        );
        let available_pks = match available_public_keys {
            None => panic!(
                "gen_account_update_body: new_account is true, but \
                 available_public_keys not provided"
            ),
            Some(available_pks) => available_pks,
        };

        let available_pk = available_pks
            .iter()
            .choose(&mut rng)
            .cloned()
            .expect("gen_account_update_body: no available public keys");

        // available public key no longer available
        available_pks.remove(&available_pk);

        let account_id = match token_id {
            Some(custom_token_id) => AccountId::create(available_pk.0, custom_token_id),
            None => AccountId::create(available_pk.0, TokenId::default()),
        };

        let mut account_with_pk = Account::create_with(account_id, Balance::from_u64(0));

        if zkapp_account {
            account_with_pk.zkapp = Some(ZkAppAccount {
                verification_key: Some(verification_key.data.clone()),
                ..ZkAppAccount::default()
            });
        }

        account_with_pk
    } else {
        match account_id {
            None => {
                if zkapp_account {
                    let zkapp_account_id = zkapp_account_ids.choose(&mut rng).cloned().unwrap();
                    match account_state_tbl.get(&zkapp_account_id) {
                        None => panic!("gen_account_update_body: fail to find zkapp account"),
                        Some((_, Role::FeePayer | Role::NewAccount | Role::NewTokenAccount)) => {
                            panic!(
                                "gen_account_update_body: all zkapp accounts were new \
                             accounts or used as fee_payer accounts"
                            )
                        }
                        Some((account, Role::OrdinaryParticipant)) => account.clone(),
                    }
                } else {
                    account_state_tbl
                        .values()
                        .filter(|(_, role)| {
                            match (&authorization_tag, role) {
                                (_, Role::FeePayer) => false,
                                (ControlTag::Proof, Role::NewAccount) => false,
                                (_, Role::NewTokenAccount) => false,
                                (_, Role::NewAccount) => {
                                    // `required_balance_change` is only for balancing account_update.
                                    // Newly created account should not be used in balancing account_update
                                    required_balance_change.is_none()
                                }
                                (_, Role::OrdinaryParticipant) => true,
                            }
                        })
                        .choose(&mut rng)
                        .cloned()
                        .unwrap()
                        .0
                }
            }
            Some(account_id) => {
                // get the latest state of the account
                let (account, _) = account_state_tbl.get(&account_id).unwrap();

                if zkapp_account && account.zkapp.is_none() {
                    panic!("gen_account_update_body: provided account has no zkapp field");
                }

                account.clone()
            }
        }
    };

    let public_key = account.public_key.clone();
    let token_id = account.token_id.clone();
    let balance_change = match required_balance_change {
        Some(bal_change) => bal_change,
        None => gen_balance_change(&account),
    };

    let mut field_array_list_gen = |max_array_len: usize, max_list_len: usize| {
        let array_gen = |rng: &mut ThreadRng| -> zkapp_command::Event {
            let array_len = rng.gen_range(0..max_array_len);
            zkapp_command::Event((0..array_len).map(|_| Fp::rand(rng)).collect())
        };
        let list_len = rng.gen_range(0..max_list_len);
        (0..list_len)
            .map(|_| array_gen(&mut rng))
            .collect::<Vec<_>>()
    };

    let events = zkapp_command::Events(field_array_list_gen(2, 1));
    let sequence_events = zkapp_command::SequenceEvents(field_array_list_gen(2, 1));

    let call_data = Fp::rand(&mut rng);

    let first_use_of_account = {
        let account_id = AccountId::create(public_key.clone(), token_id.clone());
        match account_ids_seen {
            None => {
                // fee payer
                true
            }
            Some(hash_set) => {
                // other account_updates
                !hash_set.contains(&account_id)
            }
        }
    };

    let account_precondition = f_account_precondition(first_use_of_account, &account);

    // update the depth when generating `account_updates` in Zkapp_command.t
    let call_depth: usize = 0;

    let use_full_commitment = {
        let full_account_precondition =
            f_account_update_account_precondition(&account_precondition);
        gen_use_full_commitment(&full_account_precondition)
    };

    let protocol_state_precondition = match protocol_state_view {
        Some(psv) => match failure {
            Some(Failure::InvalidProtocolStatePrecondition) => {
                gen_invalid_protocol_state_precondition(psv)
            }
            _ => gen_protocol_state_precondition(psv),
        },
        None => ZkAppPreconditions::accept(),
    };

    let caller = match caller {
        None => {
            // This match is just to make compilation fail if `CallType`
            // change (new variant)
            match CallType::Call {
                CallType::Call => {}
                CallType::DelegateCall => {}
            };
            [CallType::Call, CallType::DelegateCall]
                .choose(&mut rng)
                .cloned()
                .unwrap()
        }
        Some(caller) => caller,
    };

    let token_id = f_token_id(&token_id);

    let authorization_kind = match authorization_tag {
        ControlTag::NoneGiven => AuthorizationKind::NoneGiven,
        ControlTag::Signature => AuthorizationKind::Signature,
        ControlTag::Proof => AuthorizationKind::Proof,
    };

    // update account state table with all the changes
    let add_balance_and_balance_change =
        |balance: Balance, balance_change: Signed<Amount>| match balance_change.sgn {
            Sgn::Pos => balance
                .add_amount(balance_change.magnitude)
                .expect("add_balance_and_balance_change: overflow for sum"),
            Sgn::Neg => balance
                .sub_amount(balance_change.magnitude)
                .expect("add_balance_and_balance_change: underflow for difference"),
        };

    let balance_change_original = balance_change;
    let balance_change = f_balance_change(&balance_change_original);
    let nonce_incr = |n: Nonce| if increment_nonce { n.succ() } else { n };

    fn value_to_be_updated<T: Clone>(c: &SetOrKeep<T>, default: &T) -> T {
        match c {
            SetOrKeep::Set(x) => x.clone(),
            SetOrKeep::Keep => default.clone(),
        }
    }

    let delegate = |account: &Account| {
        if is_fee_payer {
            account.delegate.clone()
        } else {
            account
                .delegate
                .as_ref()
                .map(|delegate| value_to_be_updated(&update.delegate, delegate))
        }
    };

    let zkapp = |account: &Account| {
        if is_fee_payer {
            return account.zkapp.clone();
        }

        let zk = match account.zkapp.as_ref() {
            None => return None,
            Some(zkapp) => zkapp,
        };

        let app_state: [Fp; 8] = {
            let account_app_state = &zk.app_state;

            update
                .app_state
                .iter()
                .zip(account_app_state)
                .map(|(to_be_updated, current)| value_to_be_updated(to_be_updated, current))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        };

        let sequence_state = {
            let last_sequence_slot = zk.last_sequence_slot;
            let txn_global_slot = match protocol_state_view {
                None => last_sequence_slot,
                Some(ps) => ps.global_slot_since_genesis,
            };

            let (sequence_state, _last_sequence_slot) = zkapp_logic::update_sequence_state(
                zk.sequence_state,
                sequence_events.clone(),
                txn_global_slot,
                last_sequence_slot,
            );

            sequence_state
        };

        let proved_state = {
            let keeping_app_state = update.app_state.iter().all(|v| v.is_keep());
            let changing_entire_app_state = update.app_state.iter().all(|v| v.is_set());

            let proof_verifies = matches!(authorization_tag, ControlTag::Proof);

            if keeping_app_state {
                zk.proved_state
            } else if proof_verifies {
                if changing_entire_app_state {
                    true
                } else {
                    zk.proved_state
                }
            } else {
                false
            }
        };

        Some(ZkAppAccount {
            app_state,
            sequence_state,
            proved_state,
            ..zk.clone()
        })
    };

    match account_state_tbl.entry(account.id()) {
        Vacant(entry) => {
            // new entry in table
            account.balance = add_balance_and_balance_change(account.balance, balance_change);
            account.nonce = nonce_incr(account.nonce);
            account.delegate = delegate(&account);
            account.zkapp = zkapp(&account);

            let role = if token_account {
                Role::NewTokenAccount
            } else {
                Role::NewAccount
            };

            entry.insert((account, role));
        }
        Occupied(mut entry) => {
            std::mem::drop(account); // just making sure we work on `updated_account`

            // update entry in table
            let (updated_account, _role) = entry.get_mut();

            updated_account.balance =
                add_balance_and_balance_change(updated_account.balance, balance_change);
            updated_account.nonce = nonce_incr(updated_account.nonce);
            updated_account.delegate = delegate(updated_account);
            updated_account.zkapp = zkapp(updated_account);
        }
    }

    AccountUpdateBodyComponents {
        public_key,
        update: if new_account {
            update.verification_key = SetOrKeep::Set(verification_key);
            update
        } else {
            update
        },
        token_id,
        balance_change: balance_change_original,
        increment_nonce,
        events,
        sequence_events,
        call_data,
        call_depth,
        protocol_state_precondition,
        account_precondition,
        use_full_commitment,
        caller,
        authorization_kind,
    }
}

// struct BodyComponentsParams<'a, A, B, C, D> {
//     update: Option<Update>,
//     account_id: Option<AccountId>,
//     token_id: Option<TokenId>,
//     caller: Option<CallType>,
//     account_ids_seen: Option<HashSet<AccountId>>,
//     account_state_tbl: &'a mut HashMap<AccountId, (Account, Role)>,
//     vk: Option<WithHash<VerificationKey>>,
//     failure: Option<Failure>,
//     new_account: Option<bool>,
//     zkapp_account: Option<bool>,
//     is_fee_payer: Option<bool>,
//     available_public_keys: Option<HashSet<HashableCompressedPubKey>>,
//     permissions_auth: Option<ControlTag>,
//     required_balance_change: Option<A>,
//     protocol_state_view: Option<&'a ProtocolStateView>,
//     zkapp_account_ids: Vec<AccountId>,
//     increment_nonce: (B, bool),
//     authorization_tag: ControlTag,
//     _phantom: PhantomData<(C, D)>,
// }

fn gen_balance_change(
    permissions_auth: Option<ControlTag>,
    account: &Account,
    failure: Option<&Failure>,
    new_account: bool,
) -> Signed<Amount> {
    let mut rng = rand::thread_rng();

    let sgn = if new_account {
        Sgn::Pos
    } else {
        match (failure, permissions_auth) {
            (Some(Failure::UpdateNotPermitted(NotPermitedOf::Send)), _) => Sgn::Neg,
            (Some(Failure::UpdateNotPermitted(NotPermitedOf::Receive)), _) => Sgn::Pos,
            (_, Some(auth)) => match auth {
                ControlTag::NoneGiven => Sgn::Pos,
                _ => [Sgn::Pos, Sgn::Neg].choose(&mut rng).copied().unwrap(),
            },
            (_, None) => [Sgn::Pos, Sgn::Neg].choose(&mut rng).copied().unwrap(),
        }
    };
    // if negative, magnitude constrained to balance in account
    // the effective balance is what's in the account state table

    let effective_balance = account.balance;
    let small_balance_change = {
        // make small transfers to allow generating large number of zkapp_command
        // without an overflow
        if effective_balance < Balance::of_formatted_string("1.0") && !new_account {
            panic!("account has low balance");
        }

        Balance::of_formatted_string("0.000001")
    };

    let magnitude = if new_account {
        let min = Amount::of_formatted_string("50.0");
        let max = Amount::of_formatted_string("100.0");
        Amount::from_u64(rng.gen_range(min.as_u64()..max.as_u64()))
    } else {
        Amount::from_u64(rng.gen_range(0..small_balance_change.as_u64()))
    };

    Signed::<Amount> { magnitude, sgn }
}

fn gen_use_full_commitment(
    increment_nonce: bool,
    account_precondition: &AccountPreconditions,
    authorization: &zkapp_command::Control,
) -> bool {
    // check conditions to avoid replays
    let incr_nonce_and_constrains_nonce =
        increment_nonce && account_precondition.to_full().nonce.is_constant();

    let does_not_use_a_signature = !matches!(authorization.tag(), ControlTag::Signature);

    if incr_nonce_and_constrains_nonce || does_not_use_a_signature {
        rand::thread_rng().gen()
    } else {
        true
    }
}

struct AccountUpdateParams<'a> {
    update: Option<Update>,
    failure: Option<&'a Failure>,
    new_account: Option<bool>,
    zkapp_account: Option<bool>,
    account_id: Option<AccountId>,
    token_id: Option<TokenId>,
    caller: Option<CallType>,
    permissions_auth: Option<ControlTag>,
    required_balance_change: Option<Signed<Amount>>,
    zkapp_account_ids: &'a [AccountId],
    authorization: zkapp_command::Control,
    account_ids_seen: &'a mut HashSet<AccountId>,
    available_public_keys: &'a mut HashSet<HashableCompressedPubKey>,
    account_state_tbl: &'a mut HashMap<AccountId, (Account, Role)>,
    protocol_state_view: Option<&'a ProtocolStateView>,
    vk: Option<&'a WithHash<VerificationKey>>,
    // is_fee_payer: Option<bool>,
    // increment_nonce: (B, bool),
    // authorization_tag: ControlTag,
    // _phantom: PhantomData<(C, D)>,
}

fn gen_account_update_from(params: AccountUpdateParams) -> AccountUpdateSimple {
    let AccountUpdateParams {
        update,
        failure,
        new_account,
        zkapp_account,
        account_id,
        token_id,
        caller,
        permissions_auth,
        required_balance_change,
        zkapp_account_ids,
        authorization,
        account_ids_seen,
        available_public_keys,
        account_state_tbl,
        protocol_state_view,
        vk,
    } = params;

    // permissions_auth is used to generate updated permissions consistent with a
    // contemplated authorization;
    // allow incrementing the nonce only if we know the authorization will be Signature
    let increment_nonce = match params.permissions_auth {
        Some(tag) => match tag {
            ControlTag::Signature => true,
            ControlTag::Proof | ControlTag::NoneGiven => false,
        },
        None => false,
    };

    let new_account = new_account.unwrap_or(false);
    let zkapp_account = zkapp_account.unwrap_or(false);

    let params = BodyComponentsParams {
        update,
        account_id,
        token_id,
        caller,
        account_ids_seen: Some(account_ids_seen),
        account_state_tbl,
        vk,
        failure: failure.clone(),
        new_account: Some(new_account),
        zkapp_account: Some(zkapp_account),
        is_fee_payer: None,
        available_public_keys: Some(available_public_keys),
        permissions_auth,
        required_balance_change,
        protocol_state_view,
        zkapp_account_ids,
        increment_nonce: (increment_nonce, increment_nonce),
        authorization_tag: authorization.tag(),
        _phantom: PhantomData,
    };

    let body_components = gen_account_update_body_components(
        params,
        // gen_balance_change,
        |account| gen_balance_change(permissions_auth, account, failure.clone(), new_account),
        // gen_use_full_commitment,
        |account_precondition| {
            gen_use_full_commitment(increment_nonce, account_precondition, &authorization)
        },
        // f_balance_change,
        |balance| *balance,
        // f_token_id,
        |token_id| token_id.clone(),
        // f_account_precondition,
        |first_use_of_account, account| {
            gen_account_precondition_from_account(None, first_use_of_account, account)
        },
        // f_account_update_account_precondition
        |a| a.clone(),
    );

    let body = body_components.to_typical_account_update();
    let account_id = AccountId::create(body.public_key.clone(), body.token_id.clone());
    account_ids_seen.insert(account_id);

    AccountUpdateSimple {
        body,
        authorization,
    }
}

/// Value of `Mina_compile_config.minimum_user_command_fee` when we run `dune runtest src/lib/staged_ledger -f`
const MINIMUM_USER_COMMAND_FEE: Fee = Fee::from_u64(1000000);

fn gen_fee(account: &Account) -> Fee {
    let mut rng = rand::thread_rng();

    let balance = account.balance;
    let lo_fee = MINIMUM_USER_COMMAND_FEE;
    let hi_fee = MINIMUM_USER_COMMAND_FEE.scale(2).unwrap();

    assert!(hi_fee <= (Fee::from_u64(balance.as_u64())));

    Fee::from_u64(rng.gen_range(lo_fee.as_u64()..hi_fee.as_u64()))
}

/// Fee payer balance change is Neg
fn fee_to_amt(fee: &Fee) -> Signed<Amount> {
    Signed::<Amount>::of_unsigned(Amount::from_u64(fee.as_u64())).negate()
}

/// takes an account id, if we want to sign this data
///
/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L1063
fn gen_account_update_body_fee_payer(
    failure: Option<&Failure>,
    permissions_auth: Option<ControlTag>,
    account_id: AccountId,
    vk: Option<&WithHash<VerificationKey>>,
    protocol_state_view: Option<&ProtocolStateView>,
    account_state_tbl: &mut HashMap<AccountId, (Account, Role)>,
) -> FeePayerBody {
    let account_precondition_gen = |account: &Account| account.nonce;

    let body_components = gen_account_update_body_components(
        BodyComponentsParams {
            update: None,
            account_id: Some(account_id),
            token_id: None,
            caller: None,
            account_ids_seen: None,
            account_state_tbl,
            vk,
            failure,
            new_account: None,
            zkapp_account: None,
            is_fee_payer: Some(true),
            available_public_keys: None,
            permissions_auth,
            required_balance_change: None,
            protocol_state_view,
            zkapp_account_ids: &[],
            increment_nonce: ((), true),
            authorization_tag: ControlTag::Signature,
            _phantom: PhantomData,
        },
        // gen_balance_change
        gen_fee,
        // gen_use_full_commitment
        |_account_precondition| {},
        // f_balance_change
        fee_to_amt,
        // f_token_id
        |token_id| {
            // make sure the fee payer's token id is the default,
            // which is represented by the unit value in the body
            assert!(token_id.is_default());
            // return unit
        },
        // f_account_precondition,
        |_, account| account_precondition_gen(account),
        // f_account_update_account_precondition,
        |nonce| AccountPreconditions::Nonce(*nonce),
    );

    body_components.to_fee_payer()
}

/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L1089
fn gen_fee_payer(
    failure: Option<&Failure>,
    permissions_auth: Option<ControlTag>,
    account_id: AccountId,
    protocol_state_view: Option<&ProtocolStateView>,
    vk: Option<&WithHash<VerificationKey>>,
    account_state_tbl: &mut HashMap<AccountId, (Account, Role)>,
) -> FeePayer {
    let body = gen_account_update_body_fee_payer(
        failure,
        permissions_auth,
        account_id,
        vk,
        protocol_state_view,
        account_state_tbl,
    );

    // real signature to be added when this data inserted into a Zkapp_command.t
    let authorization = Signature::dummy();

    FeePayer {
        body,
        authorization,
    }
}

/// `gen_zkapp_command_from` generates a zkapp_command and record the change of accounts accordingly
/// in `account_state_tbl`. Note that `account_state_tbl` is optional. If it's not provided
/// then it would be computed from the ledger. If you plan to generate several zkapp_command,
/// then please manually pass `account_state_tbl` to `gen_zkapp_command_from` function.
/// If you are generating several zkapp_command, it's better to pre-compute the
/// `account_state_tbl` before you call this function. This way you can manually set the
/// role of fee payer accounts to be `Fee_payer` in `account_state_tbl` which would prevent
/// those accounts being used as ordinary participants in other zkapp_command.
///
/// Generated zkapp_command uses dummy signatures and dummy proofs.
fn gen_zkapp_command_from(
    failure: Option<Failure>,
    max_account_updates: Option<usize>,
    max_token_updates: Option<usize>,
    fee_payer_keypair: Keypair,
    keymap: HashMap<HashableCompressedPubKey, Keypair>,
    account_state_tbl: Option<&mut HashMap<AccountId, (Account, Role)>>,
    ledger: Mask,
    protocol_state_view: Option<&ProtocolStateView>,
    vk: Option<&WithHash<VerificationKey>>,
) -> ZkAppCommand {
    let mut rng = rand::thread_rng();

    let failure = failure.as_ref();
    let max_account_updates = max_account_updates.unwrap_or(MAX_ACCOUNT_UPDATES);
    let max_token_updates = max_token_updates.unwrap_or(MAX_TOKEN_UPDATES);

    let fee_payer_pk = fee_payer_keypair.public.into_compressed();
    let fee_payer_account_id = AccountId::create(fee_payer_pk, TokenId::default());

    let ledger_accounts = ledger.to_list();

    // table of public keys to accounts, updated when generating each account_update
    // a Map would be more principled, but threading that map through the code
    // adds complexity

    let mut account_state_tbl = match account_state_tbl {
        Some(account_state_tbl) => MyCowMut::Borrow(account_state_tbl),
        None => MyCowMut::Own(HashMap::new()),
    };
    let account_state_tbl = &mut account_state_tbl;

    // make sure all ledger keys are in the keymap
    for account in ledger_accounts.into_iter() {
        let id = account.id();
        let pk = id.public_key.clone();

        // Initialize account states
        if let Vacant(entry) = account_state_tbl.entry(id.clone()) {
            entry.insert(if id == fee_payer_account_id {
                (account, Role::FeePayer)
            } else {
                (account, Role::OrdinaryParticipant)
            });
        };

        if keymap.get(&HashableCompressedPubKey(pk.clone())).is_none() {
            panic!(
                "gen_zkapp_command_from: public key {:?} is in ledger, but not keymap",
                pk
            );
        }
    }

    // table of public keys not in the ledger, to be used for new zkapp_command
    // we have the corresponding private keys, so we can create signatures for those
    // new zkapp_command
    let ledger_account_list: Vec<AccountId> = ledger
        .accounts()
        .iter()
        .chain(account_state_tbl.keys())
        .collect::<HashSet<&AccountId>>() // deduplicate
        .into_iter()
        .cloned()
        .collect(); // TODO: Not sure if it matches ocaml

    let ledger_pk_list: Vec<CompressedPubKey> = ledger_account_list
        .iter()
        .map(|id| id.public_key.clone())
        .collect();
    let ledger_pk_set: HashSet<HashableCompressedPubKey> = ledger_pk_list
        .iter()
        .map(|pk| HashableCompressedPubKey(pk.clone()))
        .collect();

    let mut available_public_keys: HashSet<HashableCompressedPubKey> = keymap
        .keys()
        .filter(|pk| !ledger_pk_set.contains(pk))
        .cloned()
        .collect();
    let available_public_keys = &mut available_public_keys;

    // account ids seen, to generate receipt chain hash precondition only if
    // a account_update with a given account id has not been encountered before
    let mut account_ids_seen = HashSet::<AccountId>::new();
    let account_ids_seen = &mut account_ids_seen;

    let fee_payer = gen_fee_payer(
        failure,
        Some(ControlTag::Signature),
        fee_payer_account_id.clone(),
        protocol_state_view,
        vk,
        account_state_tbl,
    );

    let zkapp_account_ids: Vec<AccountId> = account_state_tbl
        .iter()
        .filter(|(_, (a, role))| match role {
            Role::FeePayer | Role::NewAccount | Role::NewTokenAccount => false,
            Role::OrdinaryParticipant => a.zkapp.is_some(),
        })
        .map(|(id, _)| id.clone())
        .collect();
    let zkapp_account_ids = zkapp_account_ids.as_slice();

    account_ids_seen.insert(fee_payer_account_id);

    fn mk_forest<T: Clone>(
        ps: Vec<zkapp_command::Tree<T, AccountUpdateSimple>>,
    ) -> Vec<WithStackHash<T, AccountUpdateSimple>> {
        ps.into_iter()
            .map(|v| {
                WithStackHash {
                    elt: v,
                    stack_hash: Fp::zero(), // TODO: OCaml uses `()`
                }
            })
            .collect()
    }

    fn mk_node<T: Clone>(
        p: (AccountUpdateSimple, T),
        calls: Vec<zkapp_command::Tree<T, AccountUpdateSimple>>,
    ) -> zkapp_command::Tree<T, AccountUpdateSimple> {
        zkapp_command::Tree {
            account_update: p,
            account_update_digest: Fp::zero(), // TODO: OCaml uses `()`
            calls: zkapp_command::CallForest(mk_forest(calls)),
        }
    }

    let mut gen_zkapp_command_with_dynamic_balance =
        |new_account: bool, num_zkapp_command: usize| {
            let mut rng = rand::thread_rng();
            let mut commands = Vec::with_capacity(num_zkapp_command);

            for _ in 0..num_zkapp_command {
                // choose a random authorization
                // first Account_update.t updates the permissions, using the Signature authorization,
                //  according the random authorization
                // second Account_update.t uses the random authorization

                let (permissions_auth, update) = match failure {
                    Some(Failure::UpdateNotPermitted(ref update_type)) => {
                        let is_proof = rng.gen::<bool>();

                        let auth_tag = if is_proof {
                            ControlTag::Proof
                        } else {
                            ControlTag::Signature
                        };

                        let mut perm = Permissions::gen(auth_tag);

                        match &update_type {
                            NotPermitedOf::Delegate => {
                                perm.set_delegate = AuthRequired::from(auth_tag);
                            }
                            NotPermitedOf::AppState => {
                                perm.edit_state = AuthRequired::from(auth_tag);
                            }
                            NotPermitedOf::VerificationKey => {
                                perm.set_verification_key = AuthRequired::from(auth_tag);
                            }
                            NotPermitedOf::ZkappUri => {
                                perm.set_zkapp_uri = AuthRequired::from(auth_tag);
                            }
                            NotPermitedOf::TokenSymbol => {
                                perm.set_token_symbol = AuthRequired::from(auth_tag);
                            }
                            NotPermitedOf::VotingFor => {
                                perm.set_voting_for = AuthRequired::from(auth_tag);
                            }
                            NotPermitedOf::Send => {
                                perm.send = AuthRequired::from(auth_tag);
                            }
                            NotPermitedOf::Receive => {
                                perm.receive = AuthRequired::from(auth_tag);
                            }
                        };

                        (
                            auth_tag,
                            Some(Update {
                                permissions: SetOrKeep::Set(perm),
                                ..Update::dummy()
                            }),
                        )
                    }
                    _ => {
                        let tag = if new_account {
                            [ControlTag::Signature, ControlTag::NoneGiven]
                                .choose(&mut rng)
                                .cloned()
                                .unwrap()
                        } else {
                            ControlTag::gen(&mut rng)
                        };

                        (tag, None)
                    }
                };

                let zkapp_account = match permissions_auth {
                    ControlTag::Proof => true,
                    ControlTag::Signature | ControlTag::NoneGiven => false,
                };

                // Signature authorization to start
                let account_update0 = {
                    let authorization = zkapp_command::Control::Signature(Signature::dummy());
                    gen_account_update_from(AccountUpdateParams {
                        update,
                        failure,
                        new_account: Some(new_account),
                        zkapp_account: Some(zkapp_account),
                        account_id: None,
                        token_id: None,
                        caller: None,
                        permissions_auth: Some(permissions_auth),
                        required_balance_change: None,
                        zkapp_account_ids,
                        authorization,
                        account_ids_seen,
                        available_public_keys,
                        account_state_tbl,
                        protocol_state_view,
                        vk,
                    })
                };

                let account_update = {
                    // authorization according to chosen permissions auth
                    let (authorization, update) = match failure {
                        Some(Failure::UpdateNotPermitted(update_type)) => {
                            let auth = match permissions_auth {
                                ControlTag::Proof => Control::dummy_of_tag(ControlTag::Signature),
                                ControlTag::Signature => Control::dummy_of_tag(ControlTag::Proof),
                                _ => Control::dummy_of_tag(ControlTag::NoneGiven),
                            };

                            let mut update = Update::dummy();

                            match update_type {
                                NotPermitedOf::Delegate => {
                                    update.delegate = SetOrKeep::Set(gen_compressed());
                                }
                                NotPermitedOf::AppState => {
                                    update.app_state =
                                        std::array::from_fn(|_| SetOrKeep::Set(Fp::rand(&mut rng)));
                                }
                                NotPermitedOf::VerificationKey => {
                                    let data = VerificationKey::dummy();
                                    let hash = data.digest();
                                    update.verification_key =
                                        SetOrKeep::Set(WithHash { data, hash });
                                }
                                NotPermitedOf::ZkappUri => {
                                    update.zkapp_uri =
                                        SetOrKeep::Set("https://o1labs.org".to_string().into());
                                }
                                NotPermitedOf::TokenSymbol => {
                                    update.token_symbol = SetOrKeep::Set("CODA".to_string().into());
                                }
                                NotPermitedOf::VotingFor => {
                                    update.voting_for =
                                        SetOrKeep::Set(VotingFor(Fp::rand(&mut rng)));
                                }
                                NotPermitedOf::Send | NotPermitedOf::Receive => {}
                            };

                            let new_perm = Permissions::gen(ControlTag::Signature);
                            update.permissions = SetOrKeep::Set(new_perm);

                            (auth, Some(update))
                        }
                        _ => {
                            let auth = Control::dummy_of_tag(permissions_auth);
                            (auth, None)
                        }
                    };

                    let account_id = AccountId::create(
                        account_update0.body.public_key.clone(),
                        account_update0.body.token_id.clone(),
                    );

                    let permissions_auth = ControlTag::Signature;

                    gen_account_update_from(AccountUpdateParams {
                        update,
                        failure,
                        new_account: None,
                        zkapp_account: None,
                        account_id: Some(account_id),
                        token_id: None,
                        caller: None,
                        permissions_auth: Some(permissions_auth),
                        required_balance_change: None,
                        zkapp_account_ids,
                        authorization,
                        account_ids_seen,
                        available_public_keys,
                        account_state_tbl,
                        protocol_state_view,
                        vk,
                    })
                };

                commands.push(mk_node((account_update0, ()), vec![]));
                commands.push(mk_node((account_update, ()), vec![]));
            }

            commands
        };

    // at least 1 account_update
    let num_zkapp_command = rng.gen_range(1..max_account_updates);
    let num_new_accounts = rng.gen_range(0..num_zkapp_command);
    let num_old_zkapp_command = num_zkapp_command - num_new_accounts;

    let mut old_zkapp_command =
        gen_zkapp_command_with_dynamic_balance(false, num_old_zkapp_command);
    let mut new_zkapp_command = gen_zkapp_command_with_dynamic_balance(true, num_new_accounts);

    let account_updates0: Vec<_> = {
        old_zkapp_command.append(&mut new_zkapp_command);
        old_zkapp_command
    };

    let balance_change_sum = account_updates0.iter().fold(
        // init
        if num_new_accounts == 0 {
            Signed::<Amount>::zero()
        } else {
            let amount = Amount::from_u64(ACCOUNT_CREATION_FEE.as_u64());
            let amount = amount.scale(num_new_accounts as u64).unwrap();
            Signed::of_unsigned(amount)
        },
        |accum, node| {
            accum
                .add(&node.account_update.0.body.balance_change)
                .expect("Overflow adding other zkapp_command balances")
        },
    );

    // modify the balancing account_update with balance change to yield a zero sum
    // balancing account_update is created immediately after the fee payer
    // account_update is created. This is because the preconditions generation
    // is sensitive to the order of account_update generation.

    let balance_change = balance_change_sum.negate();

    let balancing_account_update = {
        let authorization = Control::Signature(Signature::dummy());
        gen_account_update_from(AccountUpdateParams {
            update: None,
            failure,
            new_account: Some(false),
            zkapp_account: None,
            account_id: None,
            token_id: None,
            caller: None,
            permissions_auth: Some(ControlTag::Signature),
            required_balance_change: Some(balance_change),
            zkapp_account_ids,
            authorization,
            account_ids_seen,
            available_public_keys,
            account_state_tbl,
            protocol_state_view,
            vk,
        })
    };

    let mut gen_zkapp_command_with_token_accounts = |num_zkapp_command: usize| {
        let authorization = Control::Signature(Signature::dummy());
        let permissions_auth = ControlTag::Signature;
        let caller = CallType::Call;

        (0..num_zkapp_command)
            .map(|_| {
                let parent = {
                    let required_balance_change = {
                        let amount = Amount::from_u64(ACCOUNT_CREATION_FEE.as_u64());
                        Some(Signed::of_unsigned(amount).negate())
                    };

                    gen_account_update_from(AccountUpdateParams {
                        update: None,
                        failure,
                        new_account: None,
                        zkapp_account: None,
                        account_id: None,
                        token_id: None,
                        caller: Some(caller.clone()),
                        permissions_auth: Some(permissions_auth),
                        required_balance_change,
                        zkapp_account_ids,
                        authorization: authorization.clone(),
                        account_ids_seen,
                        available_public_keys,
                        account_state_tbl,
                        protocol_state_view,
                        vk,
                    })
                };

                let token_id = Some(
                    AccountId::create(parent.body.public_key.clone(), parent.body.token_id.clone())
                        .derive_token_id(),
                );

                let child = gen_account_update_from(AccountUpdateParams {
                    update: None,
                    failure,
                    new_account: Some(true),
                    zkapp_account: None,
                    account_id: None,
                    token_id,
                    caller: Some(caller.clone()),
                    permissions_auth: Some(permissions_auth),
                    required_balance_change: None,
                    zkapp_account_ids,
                    authorization: authorization.clone(),
                    account_ids_seen,
                    available_public_keys,
                    account_state_tbl,
                    protocol_state_view,
                    vk,
                });

                mk_node((parent, ()), vec![mk_node((child, ()), vec![])])
            })
            .collect::<Vec<_>>()
    };

    let num_new_token_zkapp_command = rng.gen_range(0..max_token_updates);
    let new_token_zkapp_command =
        gen_zkapp_command_with_token_accounts(num_new_token_zkapp_command);

    let account_updates = mk_forest(
        account_updates0
            .into_iter()
            .chain([mk_node((balancing_account_update, ()), vec![])])
            .chain(new_token_zkapp_command)
            .collect(),
    );

    let memo = Memo::gen();
    let zkapp_command_dummy_authorizations = ZkAppCommand {
        fee_payer,
        account_updates: {
            // account_updates.add_callers();

            todo!()
        },
        memo,
    };

    todo!()
}

//   let zkapp_command_dummy_authorizations : Zkapp_command.t =
//     { fee_payer
//     ; account_updates =
//         account_updates |> Zkapp_command.Call_forest.add_callers_simple
//         |> Zkapp_command.Call_forest.accumulate_hashes_predicated
//     ; memo
//     }
//   in
//   (* update receipt chain hashes in accounts table *)
//   let receipt_elt =
//     let _txn_commitment, full_txn_commitment =
//       (* also computed in replace_authorizations, but easier just to re-compute here *)
//       Zkapp_command_builder.get_transaction_commitments
//         zkapp_command_dummy_authorizations
//     in
//     Receipt.Zkapp_command_elt.Zkapp_command_commitment full_txn_commitment
//   in
//   Account_id.Table.update account_state_tbl fee_payer_acct_id ~f:(function
//     | None ->
//         failwith "Expected fee payer account id to be in table"
//     | Some (account, _) ->
//         let receipt_chain_hash =
//           Receipt.Chain_hash.cons_zkapp_command_commitment
//             Mina_numbers.Index.zero receipt_elt
//             account.Account.Poly.receipt_chain_hash
//         in
//         ({ account with receipt_chain_hash }, `Fee_payer) ) ;
//   let account_updates =
//     Zkapp_command.Call_forest.to_account_updates
//       zkapp_command_dummy_authorizations.account_updates
//   in
//   List.iteri account_updates ~f:(fun ndx account_update ->
//       (* update receipt chain hash only for signature, proof authorizations *)
//       match Account_update.authorization account_update with
//       | Control.Proof _ | Control.Signature _ ->
//           let acct_id = Account_update.account_id account_update in
//           Account_id.Table.update account_state_tbl acct_id ~f:(function
//             | None ->
//                 failwith
//                   "Expected other account_update account id to be in table"
//             | Some (account, role) ->
//                 let receipt_chain_hash =
//                   let account_update_index =
//                     Mina_numbers.Index.of_int (ndx + 1)
//                   in
//                   Receipt.Chain_hash.cons_zkapp_command_commitment
//                     account_update_index receipt_elt
//                     account.Account.Poly.receipt_chain_hash
//                 in
//                 ({ account with receipt_chain_hash }, role) )
//       | Control.None_given ->
//           () ) ;
//   zkapp_command_dummy_authorizations
