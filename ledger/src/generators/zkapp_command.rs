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
    scan_state::{
        currency::{
            Amount, Balance, BlockTime, BlockTimeSpan, Fee, Length, Magnitude, Nonce, Sgn, Signed,
            Slot,
        },
        transaction_logic::{
            protocol_state::{self, ProtocolStateView},
            zkapp_command::{
                self, AccountPreconditions, AccountUpdate, AuthorizationKind, ClosedInterval,
                FeePayer, FeePayerBody, OrIgnore, SetOrKeep, Update, WithHash, WithStackHash,
                ZkAppCommand, ZkAppPreconditions,
            },
            Signature,
        },
        zkapp_logic,
    },
    staged_ledger::pre_diff_info::HashableCompressedPubKey,
    Account, AccountId, BaseLedger, ControlTag, Mask, MyCowMut, TokenId, VerificationKey,
    ZkAppAccount,
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
#[derive(Debug)]
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
#[derive(Debug)]
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

struct BodyComponentsParams<'a, A, B, C, D> {
    update: Option<Update>,
    account_id: Option<AccountId>,
    token_id: Option<TokenId>,
    caller: Option<CallType>,
    account_ids_seen: Option<HashSet<AccountId>>,
    account_state_tbl: &'a mut HashMap<AccountId, (Account, Role)>,
    vk: Option<WithHash<VerificationKey>>,
    failure: Option<Failure>,
    new_account: Option<bool>,
    zkapp_account: Option<bool>,
    is_fee_payer: Option<bool>,
    available_public_keys: Option<HashSet<HashableCompressedPubKey>>,
    permissions_auth: Option<ControlTag>,
    required_balance_change: Option<A>,
    protocol_state_view: Option<&'a ProtocolStateView>,
    zkapp_account_ids: Vec<AccountId>,
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
            vk.as_ref(),
            permissions_auth,
        ),
        Some(update) => update,
    };

    // account_update_increment_nonce for fee payer is unit and increment_nonce is true
    let (account_update_increment_nonce, increment_nonce) = increment_nonce;

    let verification_key = match vk {
        Some(vk) => vk,
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
        let mut available_pks = match available_public_keys {
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
    failure: Option<Failure>,
    permissions_auth: Option<ControlTag>,
    account_id: AccountId,
    vk: Option<WithHash<VerificationKey>>,
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
            zkapp_account_ids: vec![],
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
    failure: Option<Failure>,
    permissions_auth: Option<ControlTag>,
    account_id: AccountId,
    protocol_state_view: Option<&ProtocolStateView>,
    vk: Option<WithHash<VerificationKey>>,
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
    vk: Option<WithHash<VerificationKey>>,
) -> ZkAppCommand {
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

    let available_public_keys: HashSet<HashableCompressedPubKey> = keymap
        .keys()
        .filter(|pk| !ledger_pk_set.contains(pk))
        .cloned()
        .collect();

    // account ids seen, to generate receipt chain hash precondition only if
    // a account_update with a given account id has not been encountered before
    let mut account_ids_seen = HashSet::<AccountId>::new();

    let fee_payer = gen_fee_payer(
        failure,
        Some(ControlTag::Signature),
        fee_payer_account_id.clone(),
        protocol_state_view,
        vk,
        &mut account_state_tbl,
    );

    let zkapp_account_ids: Vec<AccountId> = account_state_tbl
        .iter()
        .filter(|(_, (a, role))| match role {
            Role::FeePayer | Role::NewAccount | Role::NewTokenAccount => false,
            Role::OrdinaryParticipant => a.zkapp.is_some(),
        })
        .map(|(id, _)| id.clone())
        .collect();

    account_ids_seen.insert(fee_payer_account_id);

    fn mk_forest<T: Clone>(ps: Vec<zkapp_command::Tree<T>>) -> Vec<WithStackHash<T>> {
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
        p: (AccountUpdate, T),
        calls: Vec<zkapp_command::Tree<T>>,
    ) -> zkapp_command::Tree<T> {
        zkapp_command::Tree {
            account_update: p,
            account_update_digest: Fp::zero(), // TODO: OCaml uses `()`
            calls: zkapp_command::CallForest(mk_forest(calls)),
        }
    }

    todo!()
}

//   let gen_zkapp_command_with_dynamic_balance ~new_account num_zkapp_command =
//     let rec go acc n =
//       let open Zkapp_basic in
//       let open Permissions in
//       if n <= 0 then return (List.rev acc)
//       else
//         (* choose a random authorization

//            first Account_update.t updates the permissions, using the Signature authorization,
//             according the random authorization

//            second Account_update.t uses the random authorization
//         *)
//         let%bind permissions_auth, update =
//           match failure with
//           | Some (Update_not_permitted update_type) ->
//               let%bind is_proof = Bool.quickcheck_generator in
//               let auth_tag =
//                 if is_proof then Control.Tag.Proof else Control.Tag.Signature
//               in
//               let%map perm = Permissions.gen ~auth_tag in
//               let update =
//                 match update_type with
//                 | `Delegate ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with
//                             set_delegate = Auth_required.from ~auth_tag
//                           }
//                     }
//                 | `App_state ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with
//                             edit_state = Auth_required.from ~auth_tag
//                           }
//                     }
//                 | `Verification_key ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with
//                             set_verification_key = Auth_required.from ~auth_tag
//                           }
//                     }
//                 | `Zkapp_uri ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with
//                             set_zkapp_uri = Auth_required.from ~auth_tag
//                           }
//                     }
//                 | `Token_symbol ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with
//                             set_token_symbol = Auth_required.from ~auth_tag
//                           }
//                     }
//                 | `Voting_for ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with
//                             set_voting_for = Auth_required.from ~auth_tag
//                           }
//                     }
//                 | `Send ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with send = Auth_required.from ~auth_tag }
//                     }
//                 | `Receive ->
//                     { Account_update.Update.dummy with
//                       permissions =
//                         Set_or_keep.Set
//                           { perm with receive = Auth_required.from ~auth_tag }
//                     }
//               in
//               (auth_tag, Some update)
//           | _ ->
//               let%map tag =
//                 if new_account then
//                   Quickcheck.Generator.of_list
//                     [ Control.Tag.Signature; None_given ]
//                 else Control.Tag.gen
//               in
//               (tag, None)
//         in
//         let zkapp_account =
//           match permissions_auth with
//           | Proof ->
//               true
//           | Signature | None_given ->
//               false
//         in
//         let%bind account_update0 =
//           (* Signature authorization to start *)
//           let authorization = Control.Signature Signature.dummy in
//           gen_account_update_from ~zkapp_account_ids ~account_ids_seen ~update
//             ?failure ~authorization ~new_account ~permissions_auth
//             ~zkapp_account ~available_public_keys ~account_state_tbl
//             ?protocol_state_view ?vk ()
//         in
//         let%bind account_update =
//           (* authorization according to chosen permissions auth *)
//           let%bind authorization, update =
//             match failure with
//             | Some (Update_not_permitted update_type) ->
//                 let auth =
//                   match permissions_auth with
//                   | Proof ->
//                       Control.(dummy_of_tag Signature)
//                   | Signature ->
//                       Control.(dummy_of_tag Proof)
//                   | _ ->
//                       Control.(dummy_of_tag None_given)
//                 in
//                 let%bind update =
//                   match update_type with
//                   | `Delegate ->
//                       let%map delegate =
//                         Signature_lib.Public_key.Compressed.gen
//                       in
//                       { Account_update.Update.dummy with
//                         delegate = Set_or_keep.Set delegate
//                       }
//                   | `App_state ->
//                       let%map app_state =
//                         let%map fields =
//                           let field_gen =
//                             Snark_params.Tick.Field.gen
//                             >>| fun x -> Set_or_keep.Set x
//                           in
//                           Quickcheck.Generator.list_with_length 8 field_gen
//                         in
//                         Zkapp_state.V.of_list_exn fields
//                       in
//                       { Account_update.Update.dummy with app_state }
//                   | `Verification_key ->
//                       let data = Pickles.Side_loaded.Verification_key.dummy in
//                       let hash = Zkapp_account.digest_vk data in
//                       let verification_key =
//                         Set_or_keep.Set { With_hash.data; hash }
//                       in
//                       return
//                         { Account_update.Update.dummy with verification_key }
//                   | `Zkapp_uri ->
//                       let zkapp_uri = Set_or_keep.Set "https://o1labs.org" in
//                       return { Account_update.Update.dummy with zkapp_uri }
//                   | `Token_symbol ->
//                       let token_symbol = Set_or_keep.Set "CODA" in
//                       return { Account_update.Update.dummy with token_symbol }
//                   | `Voting_for ->
//                       let%map field = Snark_params.Tick.Field.gen in
//                       let voting_for = Set_or_keep.Set field in
//                       { Account_update.Update.dummy with voting_for }
//                   | `Send | `Receive ->
//                       return Account_update.Update.dummy
//                 in
//                 let%map new_perm =
//                   Permissions.gen ~auth_tag:Control.Tag.Signature
//                 in
//                 ( auth
//                 , Some { update with permissions = Set_or_keep.Set new_perm } )
//             | _ ->
//                 return (Control.dummy_of_tag permissions_auth, None)
//           in
//           let account_id =
//             Account_id.create account_update0.body.public_key
//               account_update0.body.token_id
//           in
//           let permissions_auth = Control.Tag.Signature in
//           gen_account_update_from ~update ?failure ~zkapp_account_ids
//             ~account_ids_seen ~account_id ~authorization ~permissions_auth
//             ~zkapp_account ~available_public_keys ~account_state_tbl
//             ?protocol_state_view ?vk ()
//         in
//         (* this list will be reversed, so `account_update0` will execute before `account_update` *)
//         go
//           (mk_node account_update [] :: mk_node account_update0 [] :: acc)
//           (n - 1)
//     in
//     go [] num_zkapp_command
//   in
//   (* at least 1 account_update *)
//   let%bind num_zkapp_command = Int.gen_uniform_incl 1 max_account_updates in
//   let%bind num_new_accounts = Int.gen_uniform_incl 0 num_zkapp_command in
//   let num_old_zkapp_command = num_zkapp_command - num_new_accounts in
//   let%bind old_zkapp_command =
//     gen_zkapp_command_with_dynamic_balance ~new_account:false
//       num_old_zkapp_command
//   in
//   let%bind new_zkapp_command =
//     gen_zkapp_command_with_dynamic_balance ~new_account:true num_new_accounts
//   in
//   let account_updates0 = old_zkapp_command @ new_zkapp_command in
//   let balance_change_sum =
//     List.fold account_updates0
//       ~init:
//         ( if num_new_accounts = 0 then Currency.Amount.Signed.zero
//         else
//           Currency.Amount.(
//             Signed.of_unsigned
//               ( scale
//                   (of_fee
//                      Genesis_constants.Constraint_constants.compiled
//                        .account_creation_fee )
//                   num_new_accounts
//               |> Option.value_exn )) )
//       ~f:(fun acc node ->
//         match
//           Currency.Amount.Signed.add acc node.account_update.body.balance_change
//         with
//         | Some sum ->
//             sum
//         | None ->
//             failwith "Overflow adding other zkapp_command balances" )
//   in

//   (* modify the balancing account_update with balance change to yield a zero sum

//      balancing account_update is created immediately after the fee payer
//      account_update is created. This is because the preconditions generation
//      is sensitive to the order of account_update generation.
//   *)
//   let balance_change = Currency.Amount.Signed.negate balance_change_sum in
//   let%bind balancing_account_update =
//     let authorization = Control.Signature Signature.dummy in
//     gen_account_update_from ?failure ~permissions_auth:Control.Tag.Signature
//       ~zkapp_account_ids ~account_ids_seen ~authorization ~new_account:false
//       ~available_public_keys ~account_state_tbl
//       ~required_balance_change:balance_change ?protocol_state_view ?vk ()
//   in
//   let gen_zkapp_command_with_token_accounts ~num_zkapp_command =
//     let authorization = Control.Signature Signature.dummy in
//     let permissions_auth = Control.Tag.Signature in
//     let caller = Account_update.Call_type.Call in
//     let rec gen_tree acc n =
//       if n <= 0 then return (List.rev acc)
//       else
//         let%bind parent =
//           let required_balance_change =
//             Currency.Amount.(
//               Signed.negate
//                 (Signed.of_unsigned
//                    (of_fee
//                       Genesis_constants.Constraint_constants.compiled
//                         .account_creation_fee ) ))
//           in
//           gen_account_update_from ~zkapp_account_ids ~account_ids_seen
//             ~authorization ~permissions_auth ~available_public_keys ~caller
//             ~account_state_tbl ~required_balance_change ?protocol_state_view ?vk
//             ()
//         in
//         let token_id =
//           Account_id.derive_token_id
//             ~owner:
//               (Account_id.create parent.body.public_key parent.body.token_id)
//         in
//         let%bind child =
//           gen_account_update_from ~zkapp_account_ids ~account_ids_seen
//             ~new_account:true ~token_id ~caller ~authorization ~permissions_auth
//             ~available_public_keys ~account_state_tbl ?protocol_state_view ?vk
//             ()
//         in
//         gen_tree (mk_node parent [ mk_node child [] ] :: acc) (n - 1)
//     in
//     gen_tree [] num_zkapp_command
//   in
//   let%bind num_new_token_zkapp_command =
//     Int.gen_uniform_incl 0 max_token_updates
//   in
//   let%bind new_token_zkapp_command =
//     gen_zkapp_command_with_token_accounts
//       ~num_zkapp_command:num_new_token_zkapp_command
//   in
//   let account_updates =
//     account_updates0
//     @ [ mk_node balancing_account_update [] ]
//     @ new_token_zkapp_command
//     |> mk_forest
//   in
//   let%map memo = Signed_command_memo.gen in
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
