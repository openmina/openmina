use std::collections::HashMap;

use mina_signer::CompressedPubKey;

use crate::{
    scan_state::{
        currency::{Amount, Fee, Magnitude},
        scan_state::{group_list, transaction_snark::work, ConstraintConstants},
        transaction_logic::{
            valid, Coinbase, CoinbaseFeeTransfer, FeeTransfer, GenericCommand, GenericTransaction,
            SingleFeeTransfer, Transaction, TransactionStatus, UserCommand, WithStatus,
        },
    },
    verifier::VerifierError,
    TokenId,
};

use super::diff::{self, with_valid_signatures_and_proofs, PreDiffOne, PreDiffTwo};

#[derive(Debug)]
pub enum PreDiffError {
    VerificationFailed(VerifierError),
    CoinbaseError(String),
    InsufficientFee((Fee, Fee)),
    InternalCommandStatusMismatch,
    Unexpected(String),
}

impl From<VerifierError> for PreDiffError {
    fn from(value: VerifierError) -> Self {
        Self::VerificationFailed(value)
    }
}

impl From<String> for PreDiffError {
    fn from(value: String) -> Self {
        Self::Unexpected(value)
    }
}

// type t =
//   | Verification_failed of Verifier.Failure.t
//   | Coinbase_error of string
//   | Insufficient_fee of Currency.Fee.t * Currency.Fee.t
//   | Internal_command_status_mismatch
//   | Unexpected of Error.t
// [@@deriving sexp]

struct PreDiffInfo<T> {
    transactions: Vec<WithStatus<T>>,
    work: Vec<work::Work>,
    commands_count: usize,
    coinbases: Vec<Amount>,
}

impl<T> PreDiffInfo<T> {
    fn empty() -> Self {
        Self {
            transactions: Vec::new(),
            work: Vec::new(),
            commands_count: 0,
            coinbases: Vec::new(),
        }
    }
}

enum CoinbaseParts {
    Zero,
    One(Option<CoinbaseFeeTransfer>),
    Two(Option<(CoinbaseFeeTransfer, Option<CoinbaseFeeTransfer>)>),
}

/// A Coinbase is a single transaction that accommodates the coinbase amount
/// and a fee transfer for the work required to add the coinbase. It also
/// contains the state body hash corresponding to a particular protocol state.
/// Unlike a transaction, a coinbase (including the fee transfer) just requires one slot
/// in the jobs queue.
/// The minimum number of slots required to add a single transaction is three (at
/// worst case number of provers: when each pair of proofs is from a different
/// prover). One slot for the transaction and two slots for fee transfers.
/// When the diff is split into two prediffs (why? refer to #687) and if after
/// adding transactions, the first prediff has two slots remaining which cannot
/// not accommodate transactions, then those slots are filled by splitting the
/// coinbase into two parts.
/// If it has one slot, then we simply add one coinbase. It is also possible that
/// the first prediff may have no slots left after adding transactions (for
/// example, when there are three slots and maximum number of provers), in which case,
/// we simply add one coinbase as part of the second prediff.
///
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L95
fn create_coinbase(
    constraint_constants: &ConstraintConstants,
    coinbase_parts: CoinbaseParts,
    receiver: &CompressedPubKey,
    coinbase_amount: Amount,
) -> Result<Vec<Coinbase>, PreDiffError> {
    let coinbase_or_error = |cb: Result<Coinbase, String>| -> Result<Coinbase, PreDiffError> {
        cb.map_err(PreDiffError::CoinbaseError)
    };

    let underflow_err = |a1: Amount, a2: Amount| {
        a1.checked_sub(&a2).ok_or_else(|| {
            PreDiffError::CoinbaseError(format!(
                "underflow when splitting coinbase: Minuend: {:?} Subtrahend: {:?}",
                a1, a2
            ))
        })
    };

    let two_parts = |amt: Amount,
                     ft1: Option<CoinbaseFeeTransfer>,
                     ft2: Option<CoinbaseFeeTransfer>|
     -> Result<Vec<Coinbase>, PreDiffError> {
        let rem_coinbase = underflow_err(coinbase_amount, amt)?;

        let ft2_amount = ft2
            .as_ref()
            .map(|ft| Amount::of_fee(&ft.fee))
            .unwrap_or_else(Amount::zero);
        underflow_err(rem_coinbase, ft2_amount)?;

        let cb1 = coinbase_or_error(Coinbase::create(amt, receiver.clone(), ft1))?;
        let cb2 = coinbase_or_error(Coinbase::create(rem_coinbase, receiver.clone(), ft2))?;

        Ok(vec![cb1, cb2])
    };

    let coinbases = match coinbase_parts {
        CoinbaseParts::Zero => vec![],
        CoinbaseParts::One(x) => vec![coinbase_or_error(Coinbase::create(
            coinbase_amount,
            receiver.clone(),
            x,
        ))?],
        CoinbaseParts::Two(None) => two_parts(
            Amount::of_fee(&constraint_constants.account_creation_fee),
            None,
            None,
        )?,
        CoinbaseParts::Two(Some((ft1, ft2))) => {
            let fee = constraint_constants
                .account_creation_fee
                .checked_add(&ft1.fee)
                .ok_or_else(|| {
                    PreDiffError::CoinbaseError(format!(
                        "Overflow when trying to add account_creation_fee \
                     {:?} to a fee transfer {:?}",
                        constraint_constants.account_creation_fee, ft1.fee,
                    ))
                })?;

            two_parts(Amount::of_fee(&fee), Some(ft1), ft2)?
        }
    };

    Ok(coinbases)
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L166
fn sum_fees<I, T, F>(xs: I, fun: F) -> Result<Fee, PreDiffError>
where
    F: Fn(&T) -> Fee,
    I: IntoIterator<Item = T>,
{
    xs.into_iter()
        .try_fold(Fee::zero(), |acc, elem| acc.checked_add(&fun(&elem)))
        .ok_or_else(|| PreDiffError::Unexpected("Fee overflow".to_string()))
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L179
fn fee_remainder<'a, Cmd>(
    commands: &[WithStatus<Cmd>],
    completed_works: impl IntoIterator<Item = &'a work::Unchecked>,
    coinbase_fee: Fee,
) -> Result<Fee, PreDiffError>
where
    Cmd: GenericCommand,
{
    let budget = sum_fees(commands, |v| v.data.fee())?;
    let work_fee = sum_fees(completed_works, |w| w.fee)?;

    let total_work_fee = work_fee
        .checked_sub(&coinbase_fee)
        .unwrap_or_else(Fee::zero);

    budget
        .checked_sub(&total_work_fee)
        .ok_or(PreDiffError::InsufficientFee((budget, total_work_fee)))
}

#[derive(Clone, Debug, Eq, derive_more::From)]
pub struct HashableCompressedPubKey(pub CompressedPubKey);

impl PartialEq for HashableCompressedPubKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::hash::Hash for HashableCompressedPubKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.x.hash(state);
        self.0.is_odd.hash(state);
    }
}

impl PartialOrd for HashableCompressedPubKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.x.partial_cmp(&other.0.x) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        };
        self.0.is_odd.partial_cmp(&other.0.is_odd)
    }
}

pub fn fee_transfers_map<I, T>(singles: I) -> Option<HashMap<HashableCompressedPubKey, Fee>>
where
    I: IntoIterator<Item = (T, Fee)>,
    T: Into<HashableCompressedPubKey>,
{
    use std::collections::hash_map::Entry::{Occupied, Vacant};

    let singles = singles.into_iter();

    let mut map = HashMap::with_capacity(singles.size_hint().0);

    for (pk, fee) in singles {
        let pk: HashableCompressedPubKey = pk.into();

        match map.entry(pk) {
            Occupied(mut entry) => {
                entry.insert(fee.checked_add(entry.get())?);
            }
            Vacant(e) => {
                e.insert(fee);
            }
        }
    }

    Some(map)
}

/// TODO: This method is a mess, need to add tests
///
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L199
fn create_fee_transfers<'a>(
    completed_works: impl Iterator<Item = &'a work::Unchecked>,
    delta: Fee,
    public_key: &CompressedPubKey,
    coinbase_fts: impl Iterator<Item = &'a CoinbaseFeeTransfer>,
) -> Result<Vec<FeeTransfer>, PreDiffError> {
    use std::collections::hash_map::Entry::Occupied;

    let mut singles = Vec::with_capacity(256); // Should be `completed_works.len() + 1`
    if !delta.is_zero() {
        singles.push((HashableCompressedPubKey(public_key.clone()), delta));
    }

    singles.extend(
        completed_works.filter_map(|work::Unchecked { fee, prover, .. }| {
            if fee.is_zero() {
                None
            } else {
                Some((HashableCompressedPubKey(prover.clone()), *fee))
            }
        }),
    );

    let mut singles_map = fee_transfers_map(singles)
        .ok_or_else(|| PreDiffError::Unexpected("fee overflow".to_string()))?;

    for CoinbaseFeeTransfer {
        receiver_pk,
        fee: cb_fee,
    } in coinbase_fts
    {
        if let Occupied(mut entry) =
            singles_map.entry(HashableCompressedPubKey(receiver_pk.clone()))
        {
            let new_fee = entry
                .get()
                .checked_sub(cb_fee)
                .ok_or_else(|| PreDiffError::Unexpected("fee underflow".to_string()))?;

            if !new_fee.is_zero() {
                entry.insert(new_fee);
            } else {
                entry.remove();
            }
        }
    }

    let mut list: Vec<_> = singles_map.into_iter().collect();
    // TODO: panic + check how OCaml sort those keys
    list.sort_by(|(pk1, _), (pk2, _)| pk1.partial_cmp(pk2).unwrap());

    let sft: Vec<_> = list
        .into_iter()
        .map(|(receiver_pk, fee)| SingleFeeTransfer::create(receiver_pk.0, fee, TokenId::default()))
        .collect();

    let res: Result<Vec<_>, _> = group_list(&sft, |v| v.clone())
        .map(FeeTransfer::of_singles)
        .collect();

    Ok(res?)
}

fn check_coinbase<A, B>(
    (fst, snd): &(PreDiffTwo<A, B>, Option<PreDiffOne<A, B>>),
) -> Result<(), PreDiffError> {
    use diff::AtMostOne as O;
    use diff::AtMostTwo::*;

    match (
        &fst.coinbase,
        snd.as_ref()
            .map(|s| &s.coinbase)
            .unwrap_or(&diff::AtMostOne::Zero),
    ) {
        // (Zero, Zero) | (Zero, One _) | One _, Zero | Two _, Zero ->
        (Zero, O::Zero) | (Zero, O::One(_)) | (One(_), O::Zero) | (Two(_), O::Zero) => Ok(()),
        (x, y) => Err(PreDiffError::CoinbaseError(format!(
            "Invalid coinbase value in staged ledger prediffs {:?} and {:?}",
            x, y,
        ))),
    }
}

fn generate_statuses<F, Cmd, Tx>(
    constraint_constants: &ConstraintConstants,
    coinbase_parts: CoinbaseParts,
    receiver: &CompressedPubKey,
    coinbase_amount: Amount,
    commands: &[WithStatus<Cmd>],
    completed_works: &[work::Unchecked],
    generate_status: &mut F,
) -> Result<(Vec<WithStatus<Cmd>>, Vec<TransactionStatus>), PreDiffError>
where
    Cmd: GenericCommand + Clone,
    F: FnMut(Transaction) -> Result<TransactionStatus, String>,
    Tx: GenericTransaction + From<Coinbase> + From<FeeTransfer>,
{
    let TransactionData {
        commands,
        coinbases,
        fee_transfers,
    } = get_transaction_data::<Cmd, Tx>(
        constraint_constants,
        coinbase_parts,
        receiver,
        coinbase_amount,
        commands.to_vec(),
        completed_works,
    )?;

    let transactions = commands
        .into_iter()
        .map(|cmd| {
            let status = generate_status(Transaction::Command(cmd.data.forget()))?;

            Ok(WithStatus {
                data: cmd.data,
                status,
            })
        })
        .collect::<Result<Vec<_>, PreDiffError>>()?;

    // Order of application is user-commands, coinbase, fee transfers. See [get_individual_info]

    let internal_commands = coinbases
        .into_iter()
        .map(Transaction::Coinbase)
        .chain(fee_transfers.into_iter().map(Transaction::FeeTransfer));

    let internal_command_statuses = internal_commands
        .map(|cmd| Ok(generate_status(cmd)?))
        .collect::<Result<Vec<_>, PreDiffError>>()?;

    Ok((transactions, internal_command_statuses))
}

pub fn compute_statuses<F, Tx>(
    constraint_constants: &ConstraintConstants,
    diff: (
        PreDiffTwo<work::Work, WithStatus<valid::UserCommand>>,
        Option<PreDiffOne<work::Work, WithStatus<valid::UserCommand>>>,
    ),
    coinbase_receiver: CompressedPubKey,
    coinbase_amount: Amount,
    generate_status: &mut F,
) -> Result<
    (
        PreDiffTwo<work::Work, WithStatus<valid::UserCommand>>,
        Option<PreDiffOne<work::Work, WithStatus<valid::UserCommand>>>,
    ),
    PreDiffError,
>
where
    F: FnMut(Transaction) -> Result<TransactionStatus, String>,
    Tx: GenericTransaction + From<Coinbase> + From<FeeTransfer>,
{
    let get_statuses_pre_diff_with_at_most_two =
        |t1: PreDiffTwo<work::Work, WithStatus<valid::UserCommand>>, generate_status: &mut F| {
            let coinbase_parts = match &t1.coinbase {
                diff::AtMostTwo::Zero => CoinbaseParts::Zero,
                diff::AtMostTwo::One(x) => CoinbaseParts::One(x.clone()),
                diff::AtMostTwo::Two(x) => CoinbaseParts::Two(x.clone()),
            };

            let (commands, internal_command_statuses) = generate_statuses::<_, _, Tx>(
                constraint_constants,
                coinbase_parts,
                &coinbase_receiver,
                coinbase_amount,
                &t1.commands,
                &t1.completed_works,
                generate_status,
            )?;

            Ok::<_, PreDiffError>(PreDiffTwo {
                completed_works: t1.completed_works,
                commands,
                coinbase: t1.coinbase,
                internal_command_statuses,
            })
        };

    let get_statuses_pre_diff_with_at_most_one =
        |t2: PreDiffOne<work::Work, WithStatus<valid::UserCommand>>, generate_status: &mut F| {
            let coinbase_added = match &t2.coinbase {
                diff::AtMostOne::Zero => CoinbaseParts::Zero,
                diff::AtMostOne::One(x) => CoinbaseParts::One(x.clone()),
            };

            let (commands, internal_command_statuses) = generate_statuses::<_, _, Tx>(
                constraint_constants,
                coinbase_added,
                &coinbase_receiver,
                coinbase_amount,
                &t2.commands,
                &t2.completed_works,
                generate_status,
            )?;

            Ok::<_, PreDiffError>(PreDiffOne {
                completed_works: t2.completed_works,
                commands,
                coinbase: t2.coinbase,
                internal_command_statuses,
            })
        };

    let p1 = get_statuses_pre_diff_with_at_most_two(diff.0, generate_status)?;
    let p2 = match diff.1 {
        Some(d2) => Some(get_statuses_pre_diff_with_at_most_one(d2, generate_status)?),
        None => None,
    };

    Ok((p1, p2))
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L237
struct TransactionData<T> {
    commands: Vec<WithStatus<T>>,
    coinbases: Vec<Coinbase>,
    fee_transfers: Vec<FeeTransfer>,
}

fn get_transaction_data<Cmd, Tx>(
    constraint_constants: &ConstraintConstants,
    coinbase_parts: CoinbaseParts,
    receiver: &CompressedPubKey,
    coinbase_amount: Amount,
    commands: Vec<WithStatus<Cmd>>,
    completed_works: &[work::Unchecked],
) -> Result<TransactionData<Cmd>, PreDiffError>
where
    Cmd: GenericCommand,
    Tx: GenericTransaction + From<Coinbase> + From<FeeTransfer>,
{
    let coinbases = create_coinbase(
        constraint_constants,
        coinbase_parts,
        receiver,
        coinbase_amount,
    )?;

    let coinbase_fts_iterator = coinbases.iter().flat_map(|cb| cb.fee_transfer.iter());

    let coinbase_work_fees: Fee =
        sum_fees(coinbase_fts_iterator.clone(), |ft| ft.fee).expect("OCaml throw here");

    let txn_works_others_iterator = completed_works.iter().filter(|w| &w.prover != receiver);

    let delta: Fee = fee_remainder(
        commands.as_slice(),
        txn_works_others_iterator.clone(),
        coinbase_work_fees,
    )?;

    let fee_transfers: Vec<FeeTransfer> = create_fee_transfers(
        txn_works_others_iterator,
        delta,
        receiver,
        coinbase_fts_iterator,
    )?;

    Ok(TransactionData {
        commands,
        coinbases,
        fee_transfers,
    })
}

fn get_individual_info<Cmd, Tx>(
    constraint_constants: &ConstraintConstants,
    coinbase_parts: CoinbaseParts,
    receiver: &CompressedPubKey,
    coinbase_amount: Amount,
    commands: Vec<WithStatus<Cmd>>,
    completed_works: Vec<work::Unchecked>,
    internal_command_statuses: Vec<TransactionStatus>,
) -> Result<PreDiffInfo<Tx>, PreDiffError>
where
    Cmd: GenericCommand,
    Tx: GenericTransaction + From<Coinbase> + From<FeeTransfer> + From<Cmd>,
{
    let TransactionData {
        commands,
        coinbases: coinbase_parts,
        fee_transfers,
    } = get_transaction_data::<Cmd, Tx>(
        constraint_constants,
        coinbase_parts,
        receiver,
        coinbase_amount,
        commands,
        &completed_works,
    )?;

    let commands_count = commands.len();
    let coinbases_amount: Vec<Amount> = coinbase_parts.iter().map(|cb| cb.amount).collect();

    let internal_commands = coinbase_parts
        .into_iter()
        .map(Tx::from)
        .chain(fee_transfers.into_iter().map(Into::into));

    let internal_commands_with_statuses = internal_command_statuses
        .into_iter()
        .zip(internal_commands)
        .map(|(status, cmd)| {
            if cmd.is_coinbase() || cmd.is_fee_transfer() {
                Ok(WithStatus { data: cmd, status })
            } else {
                Err(PreDiffError::InternalCommandStatusMismatch)
            }
        });

    let transactions: Vec<WithStatus<Tx>> = commands
        .into_iter()
        .map(|cmd| Ok(cmd.into_map(Into::into)))
        .chain(internal_commands_with_statuses)
        .collect::<Result<_, _>>()?;

    Ok(PreDiffInfo {
        transactions,
        work: completed_works,
        commands_count,
        coinbases: coinbases_amount,
    })
}

fn get_impl<Cmd, Tx>(
    constraint_constants: &ConstraintConstants,
    diff: (
        PreDiffTwo<work::Work, WithStatus<Cmd>>,
        Option<PreDiffOne<work::Work, WithStatus<Cmd>>>,
    ),
    coinbase_receiver: CompressedPubKey,
    coinbase_amount: Option<Amount>,
) -> Result<(Vec<WithStatus<Tx>>, Vec<work::Work>, usize, Vec<Amount>), PreDiffError>
where
    Cmd: GenericCommand,
    Tx: GenericTransaction + From<Coinbase> + From<FeeTransfer> + From<Cmd>,
{
    let coinbase_amount = match coinbase_amount {
        Some(amount) => amount,
        None => {
            return Err(PreDiffError::CoinbaseError(format!(
                "Overflow when calculating coinbase amount: Supercharged \
                 coinbase factor ({:?}) x coinbase amount ({:?})",
                constraint_constants.supercharged_coinbase_factor,
                constraint_constants.coinbase_amount,
            )))
        }
    };

    let apply_pre_diff_with_at_most_two = |t1: PreDiffTwo<_, _>| {
        let coinbase_parts = match t1.coinbase {
            diff::AtMostTwo::Zero => CoinbaseParts::Zero,
            diff::AtMostTwo::One(x) => CoinbaseParts::One(x),
            diff::AtMostTwo::Two(x) => CoinbaseParts::Two(x),
        };

        get_individual_info::<Cmd, Tx>(
            constraint_constants,
            coinbase_parts,
            &coinbase_receiver,
            coinbase_amount,
            t1.commands,
            t1.completed_works,
            t1.internal_command_statuses,
        )
    };

    let apply_pre_diff_with_at_most_one = |t2: PreDiffOne<_, _>| {
        let coinbase_added = match t2.coinbase {
            diff::AtMostOne::Zero => CoinbaseParts::Zero,
            diff::AtMostOne::One(x) => CoinbaseParts::One(x),
        };
        get_individual_info::<Cmd, Tx>(
            constraint_constants,
            coinbase_added,
            &coinbase_receiver,
            coinbase_amount,
            t2.commands,
            t2.completed_works,
            t2.internal_command_statuses,
        )
    };

    check_coinbase(&diff)?;

    let p1 = apply_pre_diff_with_at_most_two(diff.0)?;

    let p2 = if let Some(d) = diff.1 {
        apply_pre_diff_with_at_most_one(d)?
    } else {
        PreDiffInfo::empty()
    };

    Ok((
        p1.transactions.into_iter().chain(p2.transactions).collect(),
        p1.work.into_iter().chain(p2.work).collect(),
        p1.commands_count + p2.commands_count,
        p1.coinbases.into_iter().chain(p2.coinbases).collect(),
    ))
}

impl diff::Diff {
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L457
    pub fn get<F>(
        self,
        check: F,
        constraint_constants: &ConstraintConstants,
        coinbase_receiver: CompressedPubKey,
        supercharge_coinbase: bool,
    ) -> Result<
        (
            Vec<WithStatus<valid::Transaction>>,
            Vec<work::Work>,
            usize,
            Vec<Amount>,
        ),
        PreDiffError,
    >
    where
        F: Fn(Vec<&UserCommand>) -> Result<Vec<valid::UserCommand>, VerifierError>,
    {
        let diff = self.validate_commands(check)?;

        let coinbase_amount =
            diff::coinbase(&diff.diff, constraint_constants, supercharge_coinbase);

        get_impl::<valid::UserCommand, valid::Transaction>(
            constraint_constants,
            diff.diff,
            coinbase_receiver,
            coinbase_amount,
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L481
    pub fn get_transactions(
        self,
        constraint_constants: &ConstraintConstants,
        coinbase_receiver: CompressedPubKey,
        supercharge_coinbase: bool,
    ) -> Result<Vec<WithStatus<Transaction>>, PreDiffError> {
        let coinbase_amount =
            diff::coinbase(&self.diff, constraint_constants, supercharge_coinbase);

        let (transactions, _, _, _) = get_impl::<UserCommand, Transaction>(
            constraint_constants,
            self.diff,
            coinbase_receiver,
            coinbase_amount,
        )?;

        Ok(transactions)
    }
}

impl with_valid_signatures_and_proofs::Diff {
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L472
    pub fn get_unchecked(
        self,
        constraint_constants: &ConstraintConstants,
        coinbase_receiver: CompressedPubKey,
        supercharge_coinbase: bool,
    ) -> Result<
        (
            Vec<WithStatus<valid::Transaction>>,
            Vec<work::Work>,
            usize,
            Vec<Amount>,
        ),
        PreDiffError,
    > {
        let diff = self.forget_proof_checks();

        let coinbase_amount =
            diff::coinbase(&diff.diff, constraint_constants, supercharge_coinbase);

        get_impl::<valid::UserCommand, valid::Transaction>(
            constraint_constants,
            diff.diff,
            coinbase_receiver,
            coinbase_amount,
        )
    }
}
