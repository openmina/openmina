use crate::scan_state::currency::Fee;

pub mod user_command;
pub mod zkapp_command;
pub mod zkapp_command_builder;

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

const MINIMUM_USER_COMMAND_FEE: Fee = Fee::from_u64(1000000);

/// Value of `ledger_depth` when we run `dune runtest src/lib/staged_ledger -f`
///
/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/user_command_generators.ml#L15
const LEDGER_DEPTH: usize = 35;
