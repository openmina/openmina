use super::{GenericCommand, GenericTransaction};
use crate::{
    scan_state::currency::{Fee, Nonce},
    AccountId,
};
use mina_curves::pasta::Fp;
use mina_p2p_messages::v2::MinaBaseUserCommandStableV2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VerificationKeyHash(pub Fp);

pub type SignedCommand = super::signed_command::SignedCommand;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "MinaBaseUserCommandStableV2")]
#[serde(try_from = "MinaBaseUserCommandStableV2")]
pub enum UserCommand {
    SignedCommand(Box<SignedCommand>),
    ZkAppCommand(Box<super::zkapp_command::valid::ZkAppCommand>),
}

impl UserCommand {
    /// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/user_command.ml#L277>
    pub fn forget_check(&self) -> super::UserCommand {
        match self {
            UserCommand::SignedCommand(cmd) => super::UserCommand::SignedCommand(cmd.clone()),
            UserCommand::ZkAppCommand(cmd) => {
                super::UserCommand::ZkAppCommand(Box::new(cmd.zkapp_command.clone()))
            }
        }
    }

    pub fn fee_payer(&self) -> AccountId {
        match self {
            UserCommand::SignedCommand(cmd) => cmd.fee_payer(),
            UserCommand::ZkAppCommand(cmd) => cmd.zkapp_command.fee_payer(),
        }
    }

    pub fn nonce(&self) -> Option<Nonce> {
        match self {
            UserCommand::SignedCommand(cmd) => Some(cmd.nonce()),
            UserCommand::ZkAppCommand(_) => None,
        }
    }
}

impl GenericCommand for UserCommand {
    fn fee(&self) -> Fee {
        match self {
            UserCommand::SignedCommand(cmd) => cmd.fee(),
            UserCommand::ZkAppCommand(cmd) => cmd.zkapp_command.fee(),
        }
    }

    fn forget(&self) -> super::UserCommand {
        match self {
            UserCommand::SignedCommand(cmd) => super::UserCommand::SignedCommand(cmd.clone()),
            UserCommand::ZkAppCommand(cmd) => {
                super::UserCommand::ZkAppCommand(Box::new(cmd.zkapp_command.clone()))
            }
        }
    }
}

impl GenericTransaction for Transaction {
    fn is_fee_transfer(&self) -> bool {
        matches!(self, Transaction::FeeTransfer(_))
    }
    fn is_coinbase(&self) -> bool {
        matches!(self, Transaction::Coinbase(_))
    }
    fn is_command(&self) -> bool {
        matches!(self, Transaction::Command(_))
    }
}

#[derive(Debug, derive_more::From)]
pub enum Transaction {
    Command(UserCommand),
    FeeTransfer(super::FeeTransfer),
    Coinbase(super::Coinbase),
}

impl Transaction {
    /// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/transaction/transaction.ml#L61>
    pub fn forget(&self) -> super::Transaction {
        match self {
            Transaction::Command(cmd) => super::Transaction::Command(cmd.forget_check()),
            Transaction::FeeTransfer(ft) => super::Transaction::FeeTransfer(ft.clone()),
            Transaction::Coinbase(cb) => super::Transaction::Coinbase(cb.clone()),
        }
    }
}
