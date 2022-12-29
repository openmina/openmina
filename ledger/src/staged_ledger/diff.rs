use crate::{
    scan_state::{
        scan_state::{transaction_snark::work, VerifierError},
        transaction_logic::{
            valid, CoinbaseFeeTransfer, TransactionStatus, UserCommand, WithStatus,
        },
    },
    split_at_vec,
};

use super::pre_diff_info::PreDiffError;

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L5
#[derive(Debug)]
pub enum AtMostTwo<T> {
    Zero,
    One(Option<T>),
    Two(Option<(T, Option<T>)>),
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L20
#[derive(Debug)]
pub enum AtMostOne<T> {
    Zero,
    One(Option<T>),
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L37
#[derive(Debug)]
pub struct PreDiffTwo<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: AtMostTwo<CoinbaseFeeTransfer>,
    pub internal_command_statuses: Vec<TransactionStatus>,
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L54
#[derive(Debug)]
pub struct PreDiffOne<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: AtMostOne<CoinbaseFeeTransfer>,
    pub internal_command_statuses: Vec<TransactionStatus>,
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L68
type PreDiffWithAtMostTwoCoinbase = PreDiffTwo<work::Work, WithStatus<UserCommand>>;

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L82
type PreDiffWithAtMostOneCoinbase = PreDiffOne<work::Work, WithStatus<UserCommand>>;

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff_intf.ml#L107
pub struct Diff {
    diff: (
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
    pub struct Diff {
        diff: (
            PreDiffWithAtMostTwoCoinbase,
            Option<PreDiffWithAtMostOneCoinbase>,
        ),
    }
}

pub mod with_valid_signatures {
    use super::*;
    use crate::{
        scan_state::{
            currency::{Amount, Magnitude},
            scan_state::ConstraintConstants,
            transaction_logic::valid,
        },
        staged_ledger::staged_ledger::StagedLedger,
    };

    pub type PreDiffWithAtMostTwoCoinbase = PreDiffTwo<work::Work, WithStatus<valid::UserCommand>>;

    pub type PreDiffWithAtMostOneCoinbase = PreDiffOne<work::Work, WithStatus<valid::UserCommand>>;

    pub struct Diff {
        pub diff: (
            PreDiffWithAtMostTwoCoinbase,
            Option<PreDiffWithAtMostOneCoinbase>,
        ),
    }

    impl Diff {
        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger_diff/diff.ml#L278
        pub fn coinbase(
            &self,
            constraint_constants: &ConstraintConstants,
            supercharge_coinbase: bool,
        ) -> Option<Amount> {
            let (first_pre_diff, second_pre_diff_opt) = &self.diff;
            let coinbase_amount =
                StagedLedger::coinbase_amount(supercharge_coinbase, constraint_constants);

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
    }
}
