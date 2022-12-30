use std::collections::HashMap;

use mina_signer::CompressedPubKey;

use crate::scan_state::{
    currency::{Amount, Fee, Magnitude},
    scan_state::{transaction_snark::work, ConstraintConstants},
    transaction_logic::{valid, CoinbaseFeeTransfer, GenericCommand, WithStatus},
};

use super::{
    diff::AtMostTwo,
    pre_diff_info::{fee_transfers_map, HashableCompressedPubKey},
    staged_ledger::StagedLedger,
};

#[derive(Clone, Debug)]
struct Discarded {
    pub commands_rev: Vec<WithStatus<valid::UserCommand>>,
    pub completed_work: Vec<work::Checked>,
}

impl Discarded {
    fn add_user_command(&mut self, cmd: WithStatus<valid::UserCommand>) {
        self.commands_rev.push(cmd);
    }

    fn add_completed_work(&mut self, work: work::Checked) {
        self.completed_work.push(work);
    }
}

enum IncreaseBy {
    One,
    Two,
}

#[derive(Clone, Debug)]
pub struct Resources {
    max_space: u64,
    max_jobs: u64,
    commands_rev: Vec<WithStatus<valid::UserCommand>>,
    completed_work_rev: Vec<work::Checked>, // TODO: Use another container (VecDeque ?)
    fee_transfers: HashMap<HashableCompressedPubKey, Fee>,
    add_coinbase: bool,
    coinbase: AtMostTwo<CoinbaseFeeTransfer>,
    supercharge_coinbase: bool,
    receiver_pk: CompressedPubKey,
    budget: Result<Fee, String>,
    discarded: Discarded,
    is_coinbase_receiver_new: bool,
    _logger: (),
}

impl Resources {
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1149
    fn coinbase_ft(work: work::Unchecked) -> Option<CoinbaseFeeTransfer> {
        // Here we could not add the fee transfer if the prover=receiver_pk but
        // retaining it to preserve that information in the
        // staged_ledger_diff. It will be checked in apply_diff before adding*)

        if !work.fee.is_zero() {
            Some(CoinbaseFeeTransfer::create(work.prover, work.fee))
        } else {
            None
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1157
    fn cheapest_two_work(works: &[work::Checked]) -> (Option<work::Work>, Option<work::Work>) {
        let (w1, w2) = works
            .iter()
            .fold((None, None), |(w1, w2), w| match (w1, w2) {
                (None, _) => (Some(w), None),
                (Some(x), None) => {
                    if w.fee < x.fee {
                        (Some(w), w1)
                    } else {
                        (w1, Some(w))
                    }
                }
                (Some(x), Some(y)) => {
                    if w.fee < x.fee {
                        (Some(w), w1)
                    } else if w.fee < y.fee {
                        (w1, Some(w))
                    } else {
                        (w1, w2)
                    }
                }
            });

        (w1.cloned(), w2.cloned())
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1171
    fn coinbase_work(
        constraint_constants: &ConstraintConstants,
        is_two: Option<bool>,
        works: &[work::Checked],
        is_coinbase_receiver_new: bool,
        supercharge_coinbase: bool,
    ) -> Option<(AtMostTwo<CoinbaseFeeTransfer>, Vec<work::Work>)> {
        let is_two = is_two.unwrap_or(false);

        let (min1, min2) = Self::cheapest_two_work(works);

        let diff = |ws: &[work::Unchecked], ws2: &[work::Statement]| -> Vec<work::Unchecked> {
            ws.iter()
                .filter(|w| {
                    let wstatement = w.statement();
                    !ws2.iter().any(|w2| &wstatement == w2)
                })
                .cloned()
                .collect()
        };

        let coinbase_amount =
            StagedLedger::coinbase_amount(supercharge_coinbase, constraint_constants)?;

        // if the coinbase receiver is new then the account creation fee will
        // be deducted from the reward
        let budget = if is_coinbase_receiver_new {
            coinbase_amount
                .checked_sub(&Amount::of_fee(&constraint_constants.account_creation_fee))?
        } else {
            coinbase_amount
        };

        if is_two {
            match (min1, min2) {
                (None, _) => None,
                (Some(w), None) => {
                    if Amount::of_fee(&w.fee) <= budget {
                        let stmt = w.statement();
                        let cb = AtMostTwo::Two(Self::coinbase_ft(w).map(|ft| (ft, None)));
                        Some((cb, diff(works, &[stmt])))
                    } else {
                        let cb = AtMostTwo::Two(None);
                        Some((cb, works.to_vec()))
                    }
                }
                (Some(w1), Some(w2)) => {
                    let sum = w1.fee.checked_add(&w2.fee)?;

                    if Amount::of_fee(&sum) < budget {
                        let stmt1 = w1.statement();
                        let stmt2 = w2.statement();
                        let cb = AtMostTwo::Two(
                            Self::coinbase_ft(w1).map(|ft| (ft, Self::coinbase_ft(w2))),
                        );

                        // Why add work without checking if work constraints are
                        // satisfied? If we reach here then it means that we are trying to
                        // fill the last two slots of the tree with coinbase trnasactions
                        // and if there's any work in [works] then that has to be included,
                        // either in the coinbase or as fee transfers that gets paid by
                        // the transaction fees. So having it as coinbase ft will at least
                        // reduce the slots occupied by fee transfers*)

                        Some((cb, diff(works, &[stmt1, stmt2])))
                    } else if Amount::of_fee(&w1.fee) <= coinbase_amount {
                        let stmt = w1.statement();
                        let cb = AtMostTwo::Two(Self::coinbase_ft(w1).map(|ft| (ft, None)));
                        Some((cb, diff(works, &[stmt])))
                    } else {
                        let cb = AtMostTwo::Two(None);
                        Some((cb, works.to_vec()))
                    }
                }
            }
        } else {
            min1.map(|w| {
                if Amount::of_fee(&w.fee) <= budget {
                    let stmt = w.statement();
                    let cb = AtMostTwo::One(Self::coinbase_ft(w));
                    (cb, diff(works, &[stmt]))
                } else {
                    let cb = AtMostTwo::One(None);
                    (cb, works.to_vec())
                }
            })
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1244
    fn init_coinbase_and_fee_transfers(
        constraint_constants: &ConstraintConstants,
        cw_seq: &[work::Unchecked],
        add_coinbase: bool,
        job_count: u64,
        slots: u64,
        is_coinbase_receiver_new: bool,
        supercharge_coinbase: bool,
    ) -> (AtMostTwo<CoinbaseFeeTransfer>, Vec<(CompressedPubKey, Fee)>) {
        let cw_unchecked =
            |works: Vec<work::Unchecked>| works.into_iter().map(|w| w.forget()).collect::<Vec<_>>();

        let (coinbase, rem_cw) = match (
            add_coinbase,
            Self::coinbase_work(
                constraint_constants,
                None,
                cw_seq,
                is_coinbase_receiver_new,
                supercharge_coinbase,
            ),
        ) {
            (true, Some((ft, rem_cw))) => (ft, rem_cw),
            (true, None) => {
                // Coinbase could not be added because work-fees > coinbase-amount
                if job_count == 0 || slots - job_count >= 1 {
                    // Either no jobs are required or there is a free slot that can be filled
                    // without having to include any work
                    (AtMostTwo::One(None), cw_seq.to_vec())
                } else {
                    (AtMostTwo::Zero, cw_seq.to_vec())
                }
            }
            _ => (AtMostTwo::Zero, cw_seq.to_vec()),
        };

        let rem_cw = cw_unchecked(rem_cw);
        let singles = rem_cw
            .into_iter()
            .filter_map(|work::Unchecked { fee, prover, .. }| {
                if fee.is_zero() {
                    None
                } else {
                    Some((prover, fee))
                }
            })
            .rev()
            .collect();

        (coinbase, singles)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1276
    fn init(
        constraint_constants: &ConstraintConstants,
        uc_seq: Vec<WithStatus<valid::UserCommand>>,
        mut cw_seq: Vec<work::Checked>,
        (slots, job_count): (u64, u64),
        receiver_pk: CompressedPubKey,
        add_coinbase: bool,
        supercharge_coinbase: bool,
        _logger: (),
        is_coinbase_receiver_new: bool,
    ) -> Self {
        let (coinbase, singles) = Self::init_coinbase_and_fee_transfers(
            constraint_constants,
            &cw_seq,
            add_coinbase,
            job_count,
            slots,
            is_coinbase_receiver_new,
            supercharge_coinbase,
        );

        let fee_transfers = fee_transfers_map(singles.clone()).expect("OCaml throw here");

        let budget1 = StagedLedger::sum_fees(&uc_seq, |c| c.data.fee());
        let budget2 =
            StagedLedger::sum_fees(singles.iter().filter(|(k, _)| k != &receiver_pk), |c| c.1);

        let budget = match (budget1, budget2) {
            (Ok(r), Ok(c)) => r
                .checked_sub(&c)
                .ok_or_else(|| "budget did not suffice".to_string()),
            (_, Err(e)) | (Err(e), _) => Err(e),
        };

        let discarded = Discarded {
            commands_rev: Vec::with_capacity(256),
            completed_work: Vec::with_capacity(256),
        };

        Self {
            max_space: slots,
            max_jobs: job_count,
            commands_rev: uc_seq,
            completed_work_rev: {
                // Completed work in reverse order for faster removal of proofs if budget doesn't suffice
                cw_seq.reverse();
                cw_seq
            },
            fee_transfers,
            add_coinbase,
            coinbase,
            supercharge_coinbase,
            receiver_pk,
            budget,
            discarded,
            is_coinbase_receiver_new,
            _logger,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1335
    fn reselect_coinbase_work(&mut self, constraint_constants: &ConstraintConstants) {
        // `std::borrow::Cow` has a `ToOwned` constraints
        enum MyCow<'a, T> {
            Borrow(&'a T),
            Own(T),
        }

        impl<'a, T> AsRef<T> for MyCow<'a, T> {
            fn as_ref(&self) -> &T {
                match self {
                    MyCow::Borrow(v) => v,
                    MyCow::Own(v) => v,
                }
            }
        }

        let cw_unchecked = |works: &[work::Unchecked]| {
            works.iter().map(|w| w.clone().forget()).collect::<Vec<_>>()
        };

        let (coinbase, rem_cw) = match &self.coinbase {
            AtMostTwo::Zero => (None, MyCow::Borrow(&self.completed_work_rev)),
            AtMostTwo::One(_) => {
                match Self::coinbase_work(
                    constraint_constants,
                    None,
                    &self.completed_work_rev,
                    self.is_coinbase_receiver_new,
                    self.supercharge_coinbase,
                ) {
                    None => (
                        Some(AtMostTwo::One(None)),
                        MyCow::Borrow(&self.completed_work_rev),
                    ),
                    Some((ft, rem_cw)) => (Some(ft), MyCow::Own(rem_cw)),
                }
            }
            AtMostTwo::Two(_) => {
                match Self::coinbase_work(
                    constraint_constants,
                    Some(true),
                    &self.completed_work_rev,
                    self.is_coinbase_receiver_new,
                    self.supercharge_coinbase,
                ) {
                    None => (
                        Some(AtMostTwo::Two(None)),
                        MyCow::Borrow(&self.completed_work_rev),
                    ),
                    Some((fts, rem_cw)) => (Some(fts), MyCow::Own(rem_cw)),
                }
            }
        };

        let rem_cw = cw_unchecked(rem_cw.as_ref());

        let singles = rem_cw
            .into_iter()
            .filter_map(|work::Unchecked { fee, prover, .. }| {
                if fee.is_zero() {
                    None
                } else {
                    Some((prover, fee))
                }
            })
            .rev();

        let fee_transfers = fee_transfers_map(singles).expect("OCaml throw here");

        if let Some(coinbase) = coinbase {
            self.coinbase = coinbase;
        };
        self.fee_transfers = fee_transfers;
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1379
    fn rebudget(&self) -> Result<Fee, String> {
        // get the correct coinbase and calculate the fee transfers
        let payment_fees = StagedLedger::sum_fees(&self.commands_rev, |c| c.data.fee());

        let prover_fee_others =
            self.fee_transfers
                .iter()
                .fold(Ok(Fee::zero()), |accum, (key, fee)| {
                    let others = accum?;
                    if self.receiver_pk == key.0 {
                        Ok(others)
                    } else {
                        others
                            .checked_add(fee)
                            .ok_or_else(|| "Fee overflow".to_string())
                    }
                });

        let revenue = payment_fees;
        let cost = prover_fee_others;

        match (revenue, cost) {
            (Ok(r), Ok(c)) => r
                .checked_sub(&c)
                .ok_or_else(|| "budget did not suffice".to_string()),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1399
    fn budget_sufficient(&self) -> bool {
        self.budget.is_ok()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1402
    fn coinbase_added(&self) -> u64 {
        match &self.coinbase {
            AtMostTwo::Zero => 0,
            AtMostTwo::One(_) => 1,
            AtMostTwo::Two(_) => 2,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1411
    #[allow(clippy::bool_to_int_with_if)]
    fn slots_occupied(&self) -> u64 {
        let fee_for_self = match &self.budget {
            Err(_) => 0,
            Ok(b) => {
                if b.is_zero() {
                    0
                } else {
                    1
                }
            }
        };

        let other_provers = self
            .fee_transfers
            .iter()
            .filter(|(pk, _)| pk.0 != self.receiver_pk)
            .count() as u64;

        let total_fee_transfer_pks = other_provers + fee_for_self;

        self.commands_rev.len() as u64 + ((total_fee_transfer_pks + 1) / 2) + self.coinbase_added()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1430
    fn space_available(&self) -> bool {
        let slots = self.slots_occupied();
        self.max_space > slots
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1434
    fn work_done(&self) -> bool {
        let no_of_proof_bundles = self.completed_work_rev.len() as u64;
        let slots = self.slots_occupied();

        // If more jobs were added in the previous diff then ( t.max_space-t.max_jobs)
        // slots can go for free in this diff
        no_of_proof_bundles == self.max_jobs || slots <= self.max_space - self.max_jobs
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1440
    fn space_constraint_satisfied(&self) -> bool {
        let occupied = self.slots_occupied();
        occupied <= self.max_space
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1444
    fn work_constraint_satisfied(&self) -> bool {
        // Are we doing all the work available
        let all_proofs = self.work_done();
        // enough work
        let slots = self.slots_occupied();
        let cw_count = self.completed_work_rev.len() as u64;
        let enough_work = cw_count >= slots;
        // if there are no transactions then don't need any proofs
        all_proofs || slots == 0 || enough_work
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1454
    fn available_space(&self) -> u64 {
        self.max_space - self.slots_occupied()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1456
    fn discard_last_work(
        &mut self,
        constraint_constants: &ConstraintConstants,
    ) -> Option<work::Work> {
        if self.completed_work_rev.is_empty() {
            return None;
        }

        let w = self.completed_work_rev.remove(0);

        let to_be_discarded_fee = w.fee;
        self.discarded.add_completed_work(w.clone());

        let current_budget = self.budget.clone();

        self.reselect_coinbase_work(constraint_constants);

        let budget = match current_budget {
            Ok(b) => b
                .checked_add(&to_be_discarded_fee)
                .ok_or_else(|| "Currency overflow".to_string()),
            Err(_) => self.rebudget(),
        };

        self.budget = budget;

        Some(w)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1476
    fn discard_user_command(&mut self) -> Option<WithStatus<valid::UserCommand>> {
        if self.commands_rev.is_empty() {
            let update_fee_transfers =
                |this: &mut Self,
                 ft: CoinbaseFeeTransfer,
                 coinbase: AtMostTwo<CoinbaseFeeTransfer>| {
                    this.fee_transfers.insert(ft.receiver_pk.into(), ft.fee);
                    this.coinbase = coinbase;
                    this.budget = this.rebudget();
                };

            match &self.coinbase {
                AtMostTwo::Zero => {}
                AtMostTwo::One(None) => {
                    self.coinbase = AtMostTwo::Zero;
                }
                AtMostTwo::Two(None) => {
                    self.coinbase = AtMostTwo::One(None);
                }
                AtMostTwo::Two(Some((ft, None))) => {
                    self.coinbase = AtMostTwo::One(Some(ft.clone()));
                }
                AtMostTwo::One(Some(ft)) => {
                    update_fee_transfers(self, ft.clone(), AtMostTwo::Zero);
                }
                AtMostTwo::Two(Some((ft1, Some(ft2)))) => {
                    update_fee_transfers(self, ft2.clone(), AtMostTwo::One(Some(ft1.clone())));
                }
            }

            None
        } else {
            let uc = self.commands_rev.remove(0);

            let current_budget = self.budget.clone();

            let to_be_discarded_fee = uc.data.fee();
            self.discarded.add_user_command(uc.clone());

            let budget = match current_budget {
                Ok(b) => b
                    .checked_add(&to_be_discarded_fee)
                    .ok_or_else(|| "Currency overflow".to_string()),
                Err(_) => self.rebudget(),
            };

            self.budget = budget;

            Some(uc)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1521
    fn worked_more(&self, constraint_constants: &ConstraintConstants) -> bool {
        // Is the work constraint satisfied even after discarding a work bundle?
        // We reach here after having more than enough work

        let mut this = self.clone(); // TODO: Do this without cloning
        this.discard_last_work(constraint_constants);

        let slots = this.slots_occupied();
        let cw_count = this.completed_work_rev.len() as u64;
        cw_count > 0 && cw_count >= slots
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1532
    fn incr_coinbase_part_by(
        &mut self,
        constraint_constants: &ConstraintConstants,
        count: IncreaseBy,
    ) {
        let incr = |cb: &AtMostTwo<CoinbaseFeeTransfer>| {
            Ok(match cb {
                AtMostTwo::Zero => AtMostTwo::One(None),
                AtMostTwo::One(None) => AtMostTwo::Two(None),
                AtMostTwo::One(Some(ft)) => AtMostTwo::Two(Some((ft.clone(), None))),
                AtMostTwo::Two(_) => {
                    return Err("Coinbase count cannot be more than two".to_string())
                }
            })
        };

        let by_one = |this: &mut Self| -> Result<(), String> {
            if !this.discarded.completed_work.is_empty() {
                let coinbase = incr(&this.coinbase)?;
                let w = this.discarded.completed_work.remove(0);

                // TODO: Not sure if it should push at the end here
                this.completed_work_rev.insert(0, w);
                this.coinbase = coinbase;

                this.reselect_coinbase_work(constraint_constants);
            } else {
                let coinbase = incr(&this.coinbase)?;
                this.coinbase = coinbase;

                if !this.work_done() {
                    return Err("Could not increment coinbase transaction count because of \
                         insufficient work"
                        .to_string());
                }
            }

            Ok(())
        };

        let apply = |this: &mut Self| match count {
            IncreaseBy::One => by_one(this),
            IncreaseBy::Two => {
                by_one(this)?;
                by_one(this)
            }
        };

        if let Err(e) = apply(self) {
            eprintln!("Error when increasing coinbase: {:?}", e);
        };
    }
}
