use super::{
    signed_command, zkapp_command, Coinbase, FeeTransfer, Transaction, TransactionStatus,
    UserCommand, WithStatus,
};
use crate::{
    scan_state::currency::{Amount, Magnitude, Signed},
    Account, AccountId,
};
use mina_core::constants::ConstraintConstants;
use mina_curves::pasta::Fp;

pub mod signed_command_applied {
    use mina_signer::CompressedPubKey;

    use crate::AccountId;

    use super::{signed_command, WithStatus};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Common {
        pub user_command: WithStatus<signed_command::SignedCommand>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Body {
        Payments {
            new_accounts: Vec<AccountId>,
        },
        StakeDelegation {
            previous_delegate: Option<CompressedPubKey>,
        },
        Failed,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct SignedCommandApplied {
        pub common: Common,
        pub body: Body,
    }
}

pub use signed_command_applied::SignedCommandApplied;

impl SignedCommandApplied {
    pub fn new_accounts(&self) -> &[AccountId] {
        use signed_command_applied::Body::*;

        match &self.body {
            Payments { new_accounts } => new_accounts.as_slice(),
            StakeDelegation { .. } | Failed => &[],
        }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L65>
#[derive(Debug, Clone, PartialEq)]
pub struct ZkappCommandApplied {
    pub accounts: Vec<(AccountId, Option<Box<Account>>)>,
    pub command: WithStatus<zkapp_command::ZkAppCommand>,
    pub new_accounts: Vec<AccountId>,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L82>
#[derive(Debug, Clone, PartialEq)]
pub enum CommandApplied {
    SignedCommand(Box<SignedCommandApplied>),
    ZkappCommand(Box<ZkappCommandApplied>),
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L96>
#[derive(Debug, Clone, PartialEq)]
pub struct FeeTransferApplied {
    pub fee_transfer: WithStatus<FeeTransfer>,
    pub new_accounts: Vec<AccountId>,
    pub burned_tokens: Amount,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L112>
#[derive(Debug, Clone, PartialEq)]
pub struct CoinbaseApplied {
    pub coinbase: WithStatus<Coinbase>,
    pub new_accounts: Vec<AccountId>,
    pub burned_tokens: Amount,
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L142>
#[derive(Debug, Clone, PartialEq)]
pub enum Varying {
    Command(CommandApplied),
    FeeTransfer(FeeTransferApplied),
    Coinbase(CoinbaseApplied),
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L142>
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionApplied {
    pub previous_hash: Fp,
    pub varying: Varying,
}

impl TransactionApplied {
    /// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L639>
    pub fn transaction(&self) -> WithStatus<Transaction> {
        use CommandApplied::*;
        use Varying::*;

        match &self.varying {
            Command(SignedCommand(cmd)) => cmd
                .common
                .user_command
                .map(|c| Transaction::Command(UserCommand::SignedCommand(Box::new(c.clone())))),
            Command(ZkappCommand(cmd)) => cmd
                .command
                .map(|c| Transaction::Command(UserCommand::ZkAppCommand(Box::new(c.clone())))),
            FeeTransfer(f) => f.fee_transfer.map(|f| Transaction::FeeTransfer(f.clone())),
            Coinbase(c) => c.coinbase.map(|c| Transaction::Coinbase(c.clone())),
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L662>
    pub fn transaction_status(&self) -> &TransactionStatus {
        use CommandApplied::*;
        use Varying::*;

        match &self.varying {
            Command(SignedCommand(cmd)) => &cmd.common.user_command.status,
            Command(ZkappCommand(cmd)) => &cmd.command.status,
            FeeTransfer(f) => &f.fee_transfer.status,
            Coinbase(c) => &c.coinbase.status,
        }
    }

    pub fn burned_tokens(&self) -> Amount {
        match &self.varying {
            Varying::Command(_) => Amount::zero(),
            Varying::FeeTransfer(f) => f.burned_tokens,
            Varying::Coinbase(c) => c.burned_tokens,
        }
    }

    pub fn new_accounts(&self) -> &[AccountId] {
        use CommandApplied::*;
        use Varying::*;

        match &self.varying {
            Command(SignedCommand(cmd)) => cmd.new_accounts(),
            Command(ZkappCommand(cmd)) => cmd.new_accounts.as_slice(),
            FeeTransfer(f) => f.new_accounts.as_slice(),
            Coinbase(cb) => cb.new_accounts.as_slice(),
        }
    }

    /// <https://github.com/MinaProtocol/mina/blob/e5183ca1dde1c085b4c5d37d1d9987e24c294c32/src/lib/transaction_logic/mina_transaction_logic.ml#L176>
    pub fn supply_increase(
        &self,
        constraint_constants: &ConstraintConstants,
    ) -> Result<Signed<Amount>, String> {
        let burned_tokens = Signed::<Amount>::of_unsigned(self.burned_tokens());

        let account_creation_fees = {
            let account_creation_fee_int = constraint_constants.account_creation_fee;
            let num_accounts_created = self.new_accounts().len() as u64;

            // int type is OK, no danger of overflow
            let amount = account_creation_fee_int
                .checked_mul(num_accounts_created)
                .unwrap();
            Signed::<Amount>::of_unsigned(Amount::from_u64(amount))
        };

        let expected_supply_increase = match &self.varying {
            Varying::Coinbase(cb) => cb.coinbase.data.expected_supply_increase()?,
            _ => Amount::zero(),
        };
        let expected_supply_increase = Signed::<Amount>::of_unsigned(expected_supply_increase);

        // TODO: Make sure it's correct
        let total = [burned_tokens, account_creation_fees]
            .into_iter()
            .try_fold(expected_supply_increase, |total, amt| {
                total.add(&amt.negate())
            });

        total.ok_or_else(|| "overflow".to_string())
    }
}
