use mina_p2p_messages::v2::MinaBaseSignedCommandStableV2;
use mina_signer::{CompressedPubKey, Signature};

use crate::{
    decompress_pk,
    scan_state::{
        currency::{Amount, Fee, Nonce, Signed, Slot},
        fee_excess::FeeExcess,
    },
    AccountId, TokenId,
};

use super::{zkapp_command::AccessedOrNot, Memo, TransactionStatus};

/// Common fields shared by all signed command payloads.
///
/// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:34-48
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
#[derive(Debug, Clone, PartialEq)]
pub struct Common {
    /// Fee paid to the block producer
    pub fee: Fee,
    /// Public key paying the fee
    pub fee_payer_pk: CompressedPubKey,
    /// Account nonce for replay protection
    pub nonce: Nonce,
    /// Slot after which the transaction expires
    pub valid_until: Slot,
    /// Optional memo field (34 bytes)
    pub memo: Memo,
}

/// Payment payload for transferring MINA tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentPayload {
    /// Recipient's public key
    pub receiver_pk: CompressedPubKey,
    /// Amount to transfer
    pub amount: Amount,
}

/// Stake delegation payload for delegating stake to another account.
///
/// OCaml reference: src/lib/mina_base/stake_delegation.ml L:11-13
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakeDelegationPayload {
    /// Delegate stake to a new delegate
    SetDelegate {
        /// Public key of the new delegate
        new_delegate: CompressedPubKey,
    },
}

impl StakeDelegationPayload {
    /// OCaml reference: src/lib/mina_base/stake_delegation.ml L:35-37
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn receiver(&self) -> AccountId {
        let Self::SetDelegate { new_delegate } = self;
        AccountId::new(new_delegate.clone(), TokenId::default())
    }

    /// OCaml reference: src/lib/mina_base/stake_delegation.ml L:33-33
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn receiver_pk(&self) -> &CompressedPubKey {
        let Self::SetDelegate { new_delegate } = self;
        new_delegate
    }
}

/// The body of a signed command, which can be either a payment or stake
/// delegation.
///
/// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:179-181
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    /// Transfer MINA tokens from fee payer to receiver
    Payment(PaymentPayload),
    /// Delegate fee payer's stake to another account
    StakeDelegation(StakeDelegationPayload),
}

/// Signed command payload containing common fields and the transaction body.
///
/// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:239-243
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
#[derive(Debug, Clone, PartialEq)]
pub struct SignedCommandPayload {
    /// Common fields (fee, fee payer, nonce, valid_until, memo)
    pub common: Common,
    /// Transaction body (payment or stake delegation)
    pub body: Body,
}

impl SignedCommandPayload {
    pub fn create(
        fee: Fee,
        fee_payer_pk: CompressedPubKey,
        nonce: Nonce,
        valid_until: Option<Slot>,
        memo: Memo,
        body: Body,
    ) -> Self {
        Self {
            common: Common {
                fee,
                fee_payer_pk,
                nonce,
                valid_until: valid_until.unwrap_or_else(Slot::max),
                memo,
            },
            body,
        }
    }
}

/// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:352-362
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
mod weight {
    use super::*;

    fn payment(_: &PaymentPayload) -> u64 {
        1
    }
    fn stake_delegation(_: &StakeDelegationPayload) -> u64 {
        1
    }
    pub fn of_body(body: &Body) -> u64 {
        match body {
            Body::Payment(p) => payment(p),
            Body::StakeDelegation(s) => stake_delegation(s),
        }
    }
}

/// A signed command is a transaction that transfers value or delegates stake.
///
/// Signed commands are authorized by a cryptographic signature and consist of a
/// payload (containing the transaction details) and the signature proving
/// authorization.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(into = "MinaBaseSignedCommandStableV2")]
#[serde(try_from = "MinaBaseSignedCommandStableV2")]
pub struct SignedCommand {
    /// The transaction payload (common fields and body)
    pub payload: SignedCommandPayload,
    /// The public key that signed the transaction
    pub signer: CompressedPubKey, // TODO: This should be a `mina_signer::PubKey`
    /// The cryptographic signature
    pub signature: Signature,
}

impl SignedCommand {
    pub fn valid_until(&self) -> Slot {
        self.payload.common.valid_until
    }

    /// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:292-292
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn fee_payer(&self) -> AccountId {
        let public_key = self.payload.common.fee_payer_pk.clone();
        AccountId::new(public_key, TokenId::default())
    }

    /// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:290-290
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn fee_payer_pk(&self) -> &CompressedPubKey {
        &self.payload.common.fee_payer_pk
    }

    pub fn weight(&self) -> u64 {
        let Self {
            payload: SignedCommandPayload { common: _, body },
            signer: _,
            signature: _,
        } = self;
        weight::of_body(body)
    }

    /// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:288-288
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn fee_token(&self) -> TokenId {
        TokenId::default()
    }

    pub fn fee(&self) -> Fee {
        self.payload.common.fee
    }

    /// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:304-304
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn receiver(&self) -> AccountId {
        match &self.payload.body {
            Body::Payment(payload) => {
                AccountId::new(payload.receiver_pk.clone(), TokenId::default())
            }
            Body::StakeDelegation(payload) => payload.receiver(),
        }
    }

    /// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:302-302
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn receiver_pk(&self) -> &CompressedPubKey {
        match &self.payload.body {
            Body::Payment(payload) => &payload.receiver_pk,
            Body::StakeDelegation(payload) => payload.receiver_pk(),
        }
    }

    pub fn amount(&self) -> Option<Amount> {
        match &self.payload.body {
            Body::Payment(payload) => Some(payload.amount),
            Body::StakeDelegation(_) => None,
        }
    }

    pub fn nonce(&self) -> Nonce {
        self.payload.common.nonce
    }

    pub fn fee_excess(&self) -> FeeExcess {
        FeeExcess::of_single((self.fee_token(), Signed::<Fee>::of_unsigned(self.fee())))
    }

    /// OCaml reference: src/lib/mina_base/signed_command_payload.ml L:320-338
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn account_access_statuses(
        &self,
        status: &TransactionStatus,
    ) -> Vec<(AccountId, AccessedOrNot)> {
        use AccessedOrNot::*;
        use TransactionStatus::*;

        match status {
            Applied => vec![(self.fee_payer(), Accessed), (self.receiver(), Accessed)],
            // Note: The fee payer is always accessed, even if the transaction fails
            // OCaml reference: src/lib/mina_base/signed_command_payload.mli L:205-209
            Failed(_) => vec![(self.fee_payer(), Accessed), (self.receiver(), NotAccessed)],
        }
    }

    pub fn accounts_referenced(&self) -> Vec<AccountId> {
        self.account_access_statuses(&TransactionStatus::Applied)
            .into_iter()
            .map(|(id, _status)| id)
            .collect()
    }

    /// OCaml reference: src/lib/mina_base/signed_command.ml L:417-420
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn public_keys(&self) -> [&CompressedPubKey; 2] {
        [self.fee_payer_pk(), self.receiver_pk()]
    }

    /// OCaml reference: src/lib/mina_base/signed_command.ml L:422-424
    /// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
    /// Last verified: 2025-10-10
    pub fn check_valid_keys(&self) -> bool {
        self.public_keys()
            .into_iter()
            .all(|pk| decompress_pk(pk).is_some())
    }
}
