use crate::scan_state::currency::{Fee, Magnitude};
use crate::{
    scan_state::{
        scan_state::transaction_snark::work,
        transaction_logic::{valid, CoinbaseFeeTransfer, GenericCommand, WithStatus},
    },
    staged_ledger::diff::AtMostTwo,
};

use self::detail::Detail;
use self::summary::Summary;

type CountAndFee = (u64, Fee);

type FeeSummable = Fee;

fn add_fee(fee1: FeeSummable, fee2: FeeSummable) -> FeeSummable {
    fee1.checked_add(&fee2).unwrap()
}

#[derive(Copy, Debug, Clone)]
pub enum Reason {
    NoWork,
    NoSpace,
    InsufficientFees,
    ExtraWork,
    Init,
    End,
}

#[derive(Clone, Debug)]
pub enum Partition {
    First,
    Second,
}

mod summary {

    use super::*;

    #[derive(Clone, Debug)]
    pub struct Resources {
        pub completed_work: CountAndFee,
        pub commands: CountAndFee,
        pub coinbase_work_fees: AtMostTwo<Fee>,
    }

    #[derive(Clone, Debug)]
    pub struct CommandConstraints {
        insufficient_work: u64,
        insufficient_space: u64,
    }

    #[derive(Clone, Debug)]
    pub struct CompletedWorkConstraints {
        insufficient_fees: u64,
        extra_work: u64,
    }

    #[derive(Clone, Debug)]
    pub struct Summary {
        partition: Partition,
        start_resources: Resources,
        available_slots: u64,
        required_work_count: u64,
        discarded_commands: CommandConstraints,
        discarded_completed_work: CompletedWorkConstraints,
        end_resources: Resources,
    }

    pub fn coinbase_fees(coinbase: &AtMostTwo<CoinbaseFeeTransfer>) -> AtMostTwo<FeeSummable> {
        use AtMostTwo::{One, Two, Zero};

        match coinbase {
            One(Some(x)) => One(Some(x.fee)),
            Two(Some((x, None))) => Two(Some((x.fee, None))),
            Two(Some((x, Some(x2)))) => Two(Some((x.fee, Some(x2.fee)))),
            Zero => Zero,
            One(None) => One(None),
            Two(None) => Two(None),
        }
    }

    impl Resources {
        pub fn init_resources(
            completed_work: &[work::Checked],
            commands: &[WithStatus<valid::UserCommand>],
            coinbase: &AtMostTwo<CoinbaseFeeTransfer>,
        ) -> Resources {
            let completed_work: CountAndFee = (
                completed_work.len() as u64,
                completed_work
                    .iter()
                    .fold(Fee::zero(), |accum, cmd| add_fee(cmd.fee, accum)),
            );

            let commands: CountAndFee = (
                commands.len() as u64,
                commands
                    .iter()
                    .fold(Fee::zero(), |accum, cmd| add_fee(cmd.data.fee(), accum)),
            );

            let coinbase_work_fees = coinbase_fees(coinbase);

            Self {
                completed_work,
                commands,
                coinbase_work_fees,
            }
        }
    }

    impl Summary {
        pub fn init(
            completed_work: &[work::Checked],
            commands: &[WithStatus<valid::UserCommand>],
            coinbase: &AtMostTwo<CoinbaseFeeTransfer>,
            partition: Partition,
            available_slots: u64,
            required_work_count: u64,
        ) -> Summary {
            let start_resources = Resources::init_resources(completed_work, commands, coinbase);
            let discarded_commands = CommandConstraints {
                insufficient_work: 0,
                insufficient_space: 0,
            };

            let discarded_completed_work = CompletedWorkConstraints {
                insufficient_fees: 0,
                extra_work: 0,
            };

            let end_resources = Resources {
                completed_work: (0, Fee::zero()),
                commands: (0, Fee::zero()),
                coinbase_work_fees: AtMostTwo::Zero,
            };

            Self {
                partition,
                start_resources,
                available_slots,
                required_work_count,
                discarded_commands,
                discarded_completed_work,
                end_resources,
            }
        }

        pub fn end_log(
            &mut self,
            completed_work: &[work::Checked],
            commands: &[WithStatus<valid::UserCommand>],
            coinbase: &AtMostTwo<CoinbaseFeeTransfer>,
        ) {
            self.end_resources = Resources::init_resources(completed_work, commands, coinbase);
        }

        pub fn discard_command(&mut self, why: Reason) {
            match why {
                Reason::NoWork => {
                    self.discarded_commands.insufficient_work += 1;
                }
                Reason::NoSpace => {
                    self.discarded_commands.insufficient_space += 1;
                }
                _ => {}
            }
        }

        pub fn discard_completed_work(&mut self, why: Reason) {
            match why {
                Reason::InsufficientFees => {
                    self.discarded_completed_work.insufficient_fees += 1;
                }
                Reason::ExtraWork => {
                    self.discarded_completed_work.extra_work += 1;
                }
                _ => {}
            }
        }
    }
}

mod detail {
    use crate::staged_ledger::diff::AtMostTwo;

    use super::*;

    #[derive(Debug, Clone)]
    struct Line {
        reason: Reason,
        commands: CountAndFee,
        completed_work: CountAndFee,
        coinbase: AtMostTwo<Fee>,
    }

    #[derive(Clone, Debug)]
    pub struct Detail(Vec<Line>);

    impl Detail {
        pub fn init(
            completed_work: &[work::Checked],
            commands: &[WithStatus<valid::UserCommand>],
            coinbase: &AtMostTwo<CoinbaseFeeTransfer>,
        ) -> Detail {
            let mut lines = Vec::with_capacity(256);
            let init = summary::Resources::init_resources(completed_work, commands, coinbase);

            lines.push(Line {
                reason: Reason::Init,
                commands: init.commands,
                completed_work: init.completed_work,
                coinbase: init.coinbase_work_fees,
            });

            Self(lines)
        }

        pub fn discard_command(&mut self, why: Reason, command: &valid::UserCommand) {
            assert!(!self.0.is_empty());

            let last = self.0.last().unwrap();

            let new_line = Line {
                reason: why,
                commands: (
                    last.commands.0 - 1,
                    last.commands.1.checked_sub(&command.fee()).unwrap(),
                ),
                ..last.clone()
            };

            self.0.push(new_line);
        }

        pub fn discard_completed_work(&mut self, why: Reason, completed_work: &work::Unchecked) {
            assert!(!self.0.is_empty());

            let last = self.0.last().unwrap();

            let new_line = Line {
                reason: why,
                completed_work: (
                    last.completed_work.0 - 1,
                    last.completed_work
                        .1
                        .checked_sub(&completed_work.fee)
                        .unwrap(),
                ),
                ..last.clone()
            };

            self.0.push(new_line);
        }

        pub fn end_log(&mut self, coinbase: &AtMostTwo<CoinbaseFeeTransfer>) {
            assert!(!self.0.is_empty());

            let last = self.0.last().unwrap();

            // Because coinbase could be updated ooutside of the check_constraints_and_update function
            let new_line = Line {
                reason: Reason::End,
                coinbase: summary::coinbase_fees(coinbase),
                ..last.clone()
            };

            self.0.push(new_line);
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiffCreationLog {
    pub summary: Summary,
    pub detail: Detail,
}

type LogList = Vec<DiffCreationLog>;
type SummaryList = Vec<Summary>;
type DetailList = Vec<Detail>;

impl DiffCreationLog {
    pub fn init(
        completed_work: &[work::Checked],
        commands: &[WithStatus<valid::UserCommand>],
        coinbase: &AtMostTwo<CoinbaseFeeTransfer>,
        partition: Partition,
        available_slots: u64,
        required_work_count: u64,
    ) -> Self {
        let summary = Summary::init(
            completed_work,
            commands,
            coinbase,
            partition,
            available_slots,
            required_work_count,
        );
        let detail = Detail::init(completed_work, commands, coinbase);

        Self { summary, detail }
    }

    pub fn discard_command(&mut self, why: Reason, command: &valid::UserCommand) {
        self.detail.discard_command(why, command);
        self.summary.discard_command(why);
    }

    pub fn discard_completed_work(&mut self, why: Reason, completed_work: &work::Unchecked) {
        self.detail.discard_completed_work(why, completed_work);
        self.summary.discard_command(why);
    }

    pub fn end_log(
        &mut self,
        completed_work: &[work::Checked],
        commands: &[WithStatus<valid::UserCommand>],
        coinbase: &AtMostTwo<CoinbaseFeeTransfer>,
    ) {
        self.summary.end_log(completed_work, commands, coinbase);
        self.detail.end_log(coinbase);
    }
}
