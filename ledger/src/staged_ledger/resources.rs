use std::collections::HashMap;

use mina_signer::CompressedPubKey;

use crate::scan_state::{
    currency::{Amount, Fee, Magnitude},
    scan_state::{transaction_snark::work, ConstraintConstants},
    transaction_logic::{valid, CoinbaseFeeTransfer, WithStatus},
};

use super::{
    diff::AtMostTwo, pre_diff_info::HashableCompressedPubKey, staged_ledger::StagedLedger,
};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Resources {
    max_space: u64,
    max_jobs: u64,
    commands_rev: Vec<WithStatus<valid::UserCommand>>,
    completed_work_rev: Vec<work::Checked>,
    fee_transfers: HashMap<HashableCompressedPubKey, Fee>,
    add_coinbase: bool,
    coinbase: AtMostTwo<CoinbaseFeeTransfer>,
    supercharge_coinbase: bool,
    receiver_pk: CompressedPubKey,
    budget: Result<Fee, ()>,
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

    fn coinbase_work(
        constraint_constants: &ConstraintConstants,
        is_two: Option<bool>,
        works: Vec<work::Checked>,
        is_coinbase_receiver_new: bool,
        supercharge_coinbase: bool,
    ) -> Option<(AtMostTwo<CoinbaseFeeTransfer>, Vec<work::Work>)> {
        let is_two = is_two.unwrap_or(false);

        let (min1, min2) = Self::cheapest_two_work(&works);

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
                        Some((cb, diff(&works, &[stmt])))
                    } else {
                        let cb = AtMostTwo::Two(None);
                        Some((cb, works))
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

                        Some((cb, diff(&works, &[stmt1, stmt2])))
                    } else if Amount::of_fee(&w1.fee) <= coinbase_amount {
                        let stmt = w1.statement();
                        let cb = AtMostTwo::Two(Self::coinbase_ft(w1).map(|ft| (ft, None)));
                        Some((cb, diff(&works, &[stmt])))
                    } else {
                        let cb = AtMostTwo::Two(None);
                        Some((cb, works))
                    }
                }
            }
        } else {
            min1.map(|w| {
                if Amount::of_fee(&w.fee) <= budget {
                    let stmt = w.statement();
                    let cb = AtMostTwo::One(Self::coinbase_ft(w));
                    (cb, diff(&works, &[stmt]))
                } else {
                    let cb = AtMostTwo::One(None);
                    (cb, works)
                }
            })
        }
    }
}
