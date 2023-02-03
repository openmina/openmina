use crate::{
    scan_state::{
        currency::{Amount, Magnitude},
        scan_state::{transaction_snark::work, ConstraintConstants},
        transaction_logic::{
            valid, CoinbaseFeeTransfer, TransactionStatus, UserCommand, WithStatus,
        },
    },
    split_at_vec,
    verifier::VerifierError,
};

use super::{pre_diff_info::PreDiffError, staged_ledger::StagedLedger};

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L5
#[derive(Clone)]
pub enum AtMostTwo<T> {
    Zero,
    One(Option<T>),
    Two(Option<(T, Option<T>)>),
}

impl<T> std::fmt::Debug for AtMostTwo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "Zero"),
            Self::One(_) => f.debug_tuple("One(_)").finish(),
            Self::Two(_) => f.debug_tuple("Two(_, _)").finish(),
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L20
#[derive(Clone)]
pub enum AtMostOne<T> {
    Zero,
    One(Option<T>),
}

impl<T> std::fmt::Debug for AtMostOne<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "Zero"),
            Self::One(_) => f.debug_tuple("One(_)").finish(),
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L37
#[derive(Clone)]
pub struct PreDiffTwo<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: AtMostTwo<CoinbaseFeeTransfer>,
    pub internal_command_statuses: Vec<TransactionStatus>,
}

impl<A, B> std::fmt::Debug for PreDiffTwo<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = self;

        f.debug_struct("PreDiffTwo")
            .field("completed_works", &completed_works.len())
            .field("commands", &commands.len())
            .field("coinbase", coinbase)
            .field("internal_command_statuses", internal_command_statuses)
            .finish()
    }
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L54
#[derive(Clone)]
pub struct PreDiffOne<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: AtMostOne<CoinbaseFeeTransfer>,
    pub internal_command_statuses: Vec<TransactionStatus>,
}

impl<A, B> std::fmt::Debug for PreDiffOne<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = self;

        f.debug_struct("PreDiffOne")
            .field("completed_works", &completed_works.len())
            .field("commands", &commands.len())
            .field("coinbase", coinbase)
            .field("internal_command_statuses", internal_command_statuses)
            .finish()
    }
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L68
pub type PreDiffWithAtMostTwoCoinbase = PreDiffTwo<work::Work, WithStatus<UserCommand>>;

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L82
pub type PreDiffWithAtMostOneCoinbase = PreDiffOne<work::Work, WithStatus<UserCommand>>;

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L107
#[derive(Debug, Clone)]
pub struct Diff {
    pub diff: (
        PreDiffWithAtMostTwoCoinbase,
        Option<PreDiffWithAtMostOneCoinbase>,
    ),
}

impl Diff {
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L429
    pub fn completed_works(&self) -> Vec<work::Work> {
        let first = self.diff.0.completed_works.as_slice();

        let second = match self.diff.1.as_ref() {
            Some(second) => second.completed_works.as_slice(),
            None => &[],
        };

        first.iter().chain(second).cloned().collect()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L425
    pub fn commands(&self) -> Vec<WithStatus<UserCommand>> {
        let first = self.diff.0.commands.as_slice();

        let second = match self.diff.1.as_ref() {
            Some(second) => second.commands.as_slice(),
            None => &[],
        };

        first.iter().chain(second).cloned().collect()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L333
    pub fn validate_commands<F>(self, check: F) -> Result<with_valid_signatures::Diff, PreDiffError>
    where
        F: Fn(Vec<&UserCommand>) -> Result<Vec<valid::UserCommand>, VerifierError>,
    {
        let validate = |cmds: Vec<WithStatus<UserCommand>>| -> Result<Vec<WithStatus<valid::UserCommand>>, VerifierError> {
            let valids = check(cmds.iter().map(|c| &c.data).collect())?;
            Ok(valids.into_iter().zip(cmds).map(|(data, c)| {
                WithStatus { data, status: c.status  }
            }).collect())
        };

        let commands = self.commands();

        let (d1, d2) = self.diff;

        let commands_all = validate(commands)?;

        let (commands1, commands2) = split_at_vec(commands_all, d1.commands.len());

        let p1 = with_valid_signatures::PreDiffWithAtMostTwoCoinbase {
            completed_works: d1.completed_works,
            commands: commands1,
            coinbase: d1.coinbase,
            internal_command_statuses: d1.internal_command_statuses,
        };

        let p2 = d2.map(|d2| with_valid_signatures::PreDiffWithAtMostOneCoinbase {
            completed_works: d2.completed_works,
            commands: commands2,
            coinbase: d2.coinbase,
            internal_command_statuses: d2.internal_command_statuses,
        });

        Ok(with_valid_signatures::Diff { diff: (p1, p2) })
    }

    pub fn empty() -> Self {
        Self {
            diff: (
                PreDiffWithAtMostTwoCoinbase {
                    completed_works: Vec::new(),
                    commands: Vec::new(),
                    coinbase: AtMostTwo::Zero,
                    internal_command_statuses: Vec::new(),
                },
                None,
            ),
        }
    }
}

pub mod with_valid_signatures_and_proofs {
    use crate::scan_state::transaction_logic::valid;

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L123
    pub type PreDiffWithAtMostTwoCoinbase =
        PreDiffTwo<work::Checked, WithStatus<valid::UserCommand>>;

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L129
    pub type PreDiffWithAtMostOneCoinbase =
        PreDiffOne<work::Checked, WithStatus<valid::UserCommand>>;

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L140
    #[derive(Debug)]
    pub struct Diff {
        pub diff: (
            PreDiffWithAtMostTwoCoinbase,
            Option<PreDiffWithAtMostOneCoinbase>,
        ),
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L268
    fn forget_cw(list: Vec<work::Checked>) -> Vec<work::Unchecked> {
        list.into_iter().map(work::Checked::forget).collect()
    }

    impl Diff {
        pub fn commands(&self) -> Vec<WithStatus<valid::UserCommand>> {
            let first = self.diff.0.commands.as_slice();

            let second = match self.diff.1.as_ref() {
                Some(second) => second.commands.as_slice(),
                None => &[],
            };

            first.iter().chain(second).cloned().collect()
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L373
        pub fn forget_proof_checks(self) -> super::with_valid_signatures::Diff {
            let d1 = self.diff.0;

            let p1 = with_valid_signatures::PreDiffWithAtMostTwoCoinbase {
                completed_works: forget_cw(d1.completed_works),
                commands: d1.commands,
                coinbase: d1.coinbase,
                internal_command_statuses: d1.internal_command_statuses,
            };

            let p2 = self
                .diff
                .1
                .map(|d2| with_valid_signatures::PreDiffWithAtMostOneCoinbase {
                    completed_works: forget_cw(d2.completed_works),
                    commands: d2.commands,
                    coinbase: d2.coinbase,
                    internal_command_statuses: d2.internal_command_statuses,
                });

            super::with_valid_signatures::Diff { diff: (p1, p2) }
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L419
        pub fn forget(self) -> super::Diff {
            let d1 = self.diff.0;
            let p1 = super::PreDiffWithAtMostTwoCoinbase {
                completed_works: forget_cw(d1.completed_works),
                commands: d1
                    .commands
                    .into_iter()
                    .map(|c| c.map(|c| c.forget_check()))
                    .collect(),
                coinbase: d1.coinbase,
                internal_command_statuses: d1.internal_command_statuses,
            };

            let d2 = self.diff.1;
            let p2 = d2.map(|d2| super::PreDiffWithAtMostOneCoinbase {
                completed_works: forget_cw(d2.completed_works),
                commands: d2
                    .commands
                    .into_iter()
                    .map(|c| c.map(|c| c.forget_check()))
                    .collect(),
                coinbase: d2.coinbase,
                internal_command_statuses: d2.internal_command_statuses,
            });

            super::Diff { diff: (p1, p2) }
        }

        pub fn empty() -> Self {
            Self {
                diff: (
                    PreDiffWithAtMostTwoCoinbase {
                        completed_works: Vec::new(),
                        commands: Vec::new(),
                        coinbase: AtMostTwo::Zero,
                        internal_command_statuses: Vec::new(),
                    },
                    None,
                ),
            }
        }
    }
}

pub mod with_valid_signatures {
    use super::*;
    use crate::scan_state::transaction_logic::valid;

    pub type PreDiffWithAtMostTwoCoinbase = PreDiffTwo<work::Work, WithStatus<valid::UserCommand>>;

    pub type PreDiffWithAtMostOneCoinbase = PreDiffOne<work::Work, WithStatus<valid::UserCommand>>;

    pub struct Diff {
        pub diff: (
            PreDiffWithAtMostTwoCoinbase,
            Option<PreDiffWithAtMostOneCoinbase>,
        ),
    }

    impl Diff {
        pub fn empty() -> Self {
            Self {
                diff: (
                    PreDiffWithAtMostTwoCoinbase {
                        completed_works: Vec::new(),
                        commands: Vec::new(),
                        coinbase: AtMostTwo::Zero,
                        internal_command_statuses: Vec::new(),
                    },
                    None,
                ),
            }
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L278
pub fn coinbase<A, B>(
    diff: &(PreDiffTwo<A, B>, Option<PreDiffOne<A, B>>),
    constraint_constants: &ConstraintConstants,
    supercharge_coinbase: bool,
) -> Option<Amount> {
    let (first_pre_diff, second_pre_diff_opt) = &diff;
    let coinbase_amount = StagedLedger::coinbase_amount(supercharge_coinbase, constraint_constants);

    match (
        &first_pre_diff.coinbase,
        second_pre_diff_opt
            .as_ref()
            .map(|s| &s.coinbase)
            .unwrap_or(&AtMostOne::Zero),
    ) {
        (AtMostTwo::Zero, AtMostOne::Zero) => Some(Amount::zero()),
        _ => coinbase_amount,
    }
}
