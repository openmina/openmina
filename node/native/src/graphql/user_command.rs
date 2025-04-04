//! Implements GraphQL types and resolvers for user commands (payments and delegations).
//! This module handles the conversion between GraphQL input types and internal transaction representations.

use std::str::FromStr;

use juniper::{GraphQLInputObject, GraphQLObject};
use ledger::scan_state::{
    currency::{Amount, Fee, Magnitude, Nonce, Slot},
    transaction_logic::{signed_command, Memo},
};
use mina_p2p_messages::{
    bigint::BigInt,
    v2::{self, TokenIdKeyHash},
};
use mina_signer::CompressedPubKey;
use node::account::AccountPublicKey;
use o1_utils::field_helpers::FieldHelpers;

use super::zkapp::GraphQLFailureReason;

// #[derive(GraphQLInputObject, Debug)]
// pub struct InputGraphQLSendPayment {
//     pub input: InputGraphQLPayment,
//     pub signature: UserCommandSignature,
// }

#[derive(GraphQLInputObject, Debug)]
pub struct InputGraphQLPayment {
    pub from: String,
    pub to: String,
    pub amount: String,
    pub valid_until: Option<String>,
    pub fee: String,
    pub memo: Option<String>,
    pub nonce: Option<String>,
}

#[derive(GraphQLInputObject, Debug)]
pub struct InputGraphQLDelegation {
    pub from: String,
    pub to: String,
    pub valid_until: Option<String>,
    pub fee: String,
    pub memo: Option<String>,
    pub nonce: Option<String>,
}

#[derive(GraphQLInputObject, Debug, Clone)]
pub struct UserCommandSignature {
    pub field: Option<String>,
    pub scalar: Option<String>,
    // Note: either raw_signature or scalar + field must be provided
    pub raw_signature: Option<String>,
}

impl TryFrom<UserCommandSignature> for mina_signer::Signature {
    type Error = super::ConversionError;

    fn try_from(value: UserCommandSignature) -> Result<Self, Self::Error> {
        let UserCommandSignature {
            field,
            scalar,
            raw_signature,
        } = value;

        if let Some(raw_signature) = raw_signature {
            let sig_parts_len = raw_signature
                .len()
                .checked_div(2)
                .ok_or(super::ConversionError::InvalidLength)?;
            let (rx_hex, s_hex) = raw_signature.split_at(sig_parts_len);

            let rx_bytes = hex::decode(rx_hex).map_err(|_| super::ConversionError::InvalidHex)?;
            let s_bytes = hex::decode(s_hex).map_err(|_| super::ConversionError::InvalidHex)?;

            let rx = mina_signer::BaseField::from_bytes(&rx_bytes)?;
            let s = mina_signer::ScalarField::from_bytes(&s_bytes)?;

            Ok(Self { rx, s })
        } else if let (Some(field), Some(scalar)) = (field, scalar) {
            let sig = Self {
                rx: BigInt::from_decimal(&field)?
                    .try_into()
                    .map_err(|_| super::ConversionError::InvalidBigInt)?,
                s: BigInt::from_decimal(&scalar)?
                    .try_into()
                    .map_err(|_| super::ConversionError::InvalidBigInt)?,
            };

            Ok(sig)
        } else {
            Err(super::ConversionError::MissingField(
                "Either raw_signature or scalar + field must be provided".to_string(),
            ))
        }
    }
}

impl TryFrom<&UserCommandSignature> for mina_signer::Signature {
    type Error = super::ConversionError;

    fn try_from(value: &UserCommandSignature) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl UserCommandSignature {
    pub fn validate(&self) -> Result<(), super::Error> {
        if self.raw_signature.is_some() || (self.scalar.is_some() && self.field.is_some()) {
            Ok(())
        } else {
            Err(super::Error::Custom(
                "Either raw_signature or scalar + field must be provided".to_string(),
            ))
        }
    }
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLSendPaymentResponse {
    pub payment: GraphQLUserCommand,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLSendDelegationResponse {
    pub delegation: GraphQLUserCommand,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLUserCommand {
    pub amount: String,
    pub fee: String,
    pub failure_reason: Option<GraphQLFailureReason>,
    // TODO: add the account type
    pub fee_payer: String,
    pub fee_token: String,
    pub hash: String,
    pub id: String,
    pub is_delegation: bool,
    pub kind: String,
    pub memo: String,
    pub nonce: String,
    // TODO: add the account type
    pub receiver: String,
    // TODO: add the account type
    pub source: String,
    pub token: String,
    pub valid_until: String,
}

impl TryFrom<v2::MinaBaseUserCommandStableV2> for GraphQLSendPaymentResponse {
    type Error = super::ConversionError;
    fn try_from(value: v2::MinaBaseUserCommandStableV2) -> Result<Self, Self::Error> {
        if let v2::MinaBaseUserCommandStableV2::SignedCommand(ref signed_cmd) = value {
            if let v2::MinaBaseSignedCommandPayloadBodyStableV2::Payment(ref payment) =
                signed_cmd.payload.body
            {
                let res = GraphQLSendPaymentResponse {
                    payment: GraphQLUserCommand {
                        amount: payment.amount.to_string(),
                        fee: signed_cmd.payload.common.fee.to_string(),
                        failure_reason: None,
                        fee_payer: signed_cmd.payload.common.fee_payer_pk.to_string(),
                        fee_token: TokenIdKeyHash::default().to_string(),
                        hash: signed_cmd.hash()?.to_string(),
                        id: signed_cmd.to_base64()?,
                        is_delegation: false,
                        kind: "PAYMENT".to_string(),
                        memo: signed_cmd.payload.common.memo.to_base58check(),
                        nonce: signed_cmd.payload.common.nonce.to_string(),
                        receiver: payment.receiver_pk.to_string(),
                        source: signed_cmd.payload.common.fee_payer_pk.to_string(),
                        token: TokenIdKeyHash::default().to_string(),
                        valid_until: signed_cmd.payload.common.valid_until.as_u32().to_string(),
                    },
                };
                Ok(res)
            } else {
                Err(super::ConversionError::WrongVariant)
            }
        } else {
            Err(super::ConversionError::WrongVariant)
        }
    }
}

impl TryFrom<v2::MinaBaseUserCommandStableV2> for GraphQLSendDelegationResponse {
    type Error = super::ConversionError;
    fn try_from(value: v2::MinaBaseUserCommandStableV2) -> Result<Self, Self::Error> {
        if let v2::MinaBaseUserCommandStableV2::SignedCommand(ref signed_cmd) = value {
            if let v2::MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(ref delegation) =
                signed_cmd.payload.body
            {
                let v2::MinaBaseStakeDelegationStableV2::SetDelegate { new_delegate } = delegation;
                let res = GraphQLSendDelegationResponse {
                    delegation: GraphQLUserCommand {
                        amount: "0".to_string(),
                        fee: signed_cmd.payload.common.fee.to_string(),
                        failure_reason: None,
                        fee_payer: signed_cmd.payload.common.fee_payer_pk.to_string(),
                        fee_token: TokenIdKeyHash::default().to_string(),
                        hash: signed_cmd.hash()?.to_string(),
                        id: signed_cmd.to_base64()?,
                        is_delegation: true,
                        kind: "STAKE_DELEGATION".to_string(),
                        memo: signed_cmd.payload.common.memo.to_base58check(),
                        nonce: signed_cmd.payload.common.nonce.to_string(),
                        receiver: new_delegate.to_string(),
                        source: signed_cmd.payload.common.fee_payer_pk.to_string(),
                        token: TokenIdKeyHash::default().to_string(),
                        valid_until: signed_cmd.payload.common.valid_until.as_u32().to_string(),
                    },
                };
                Ok(res)
            } else {
                Err(super::ConversionError::WrongVariant)
            }
        } else {
            Err(super::ConversionError::WrongVariant)
        }
    }
}

impl InputGraphQLPayment {
    pub fn create_user_command(
        &self,
        infered_nonce: Nonce,
        signature: UserCommandSignature,
    ) -> Result<v2::MinaBaseUserCommandStableV2, super::ConversionError> {
        let infered_nonce = infered_nonce.incr();

        let nonce = if let Some(nonce) = &self.nonce {
            let input_nonce = Nonce::from_u32(
                nonce
                    .parse::<u32>()
                    .map_err(|_| super::ConversionError::InvalidBigInt)?,
            );

            if input_nonce.is_zero() || input_nonce > infered_nonce {
                return Err(super::ConversionError::Custom(
                    "Provided nonce is zero or greater than infered nonce".to_string(),
                ));
            } else {
                input_nonce
            }
        } else {
            infered_nonce
        };

        let valid_until = if let Some(valid_until) = &self.valid_until {
            Some(Slot::from_u32(
                valid_until
                    .parse::<u32>()
                    .map_err(|_| super::ConversionError::InvalidBigInt)?,
            ))
        } else {
            None
        };

        let memo = if let Some(memo) = &self.memo {
            Memo::from_str(memo)
                .map_err(|_| super::ConversionError::Custom("Invalid memo".to_string()))?
        } else {
            Memo::empty()
        };

        let from: CompressedPubKey = AccountPublicKey::from_str(&self.from)?
            .try_into()
            .map_err(|_| super::ConversionError::InvalidBigInt)?;

        let signature = signature.try_into()?;

        let sc: signed_command::SignedCommand = signed_command::SignedCommand {
            payload: signed_command::SignedCommandPayload::create(
                Fee::from_u64(
                    self.fee
                        .parse::<u64>()
                        .map_err(|_| super::ConversionError::InvalidBigInt)?,
                ),
                from.clone(),
                nonce,
                valid_until,
                memo,
                signed_command::Body::Payment(signed_command::PaymentPayload {
                    receiver_pk: AccountPublicKey::from_str(&self.to)?
                        .try_into()
                        .map_err(|_| super::ConversionError::InvalidBigInt)?,
                    amount: Amount::from_u64(
                        self.amount
                            .parse::<u64>()
                            .map_err(|_| super::ConversionError::InvalidBigInt)?,
                    ),
                }),
            ),
            signer: from.clone(),
            signature,
        };

        Ok(v2::MinaBaseUserCommandStableV2::SignedCommand(sc.into()))
    }
}

impl InputGraphQLDelegation {
    pub fn create_user_command(
        &self,
        infered_nonce: Nonce,
        signature: UserCommandSignature,
    ) -> Result<v2::MinaBaseUserCommandStableV2, super::ConversionError> {
        let infered_nonce = infered_nonce.incr();

        let nonce = if let Some(nonce) = &self.nonce {
            let input_nonce = Nonce::from_u32(
                nonce
                    .parse::<u32>()
                    .map_err(|_| super::ConversionError::InvalidBigInt)?,
            );

            if input_nonce.is_zero() || input_nonce > infered_nonce {
                return Err(super::ConversionError::Custom(
                    "Provided nonce is zero or greater than infered nonce".to_string(),
                ));
            } else {
                input_nonce
            }
        } else {
            infered_nonce
        };

        let valid_until = if let Some(valid_until) = &self.valid_until {
            Some(Slot::from_u32(
                valid_until
                    .parse::<u32>()
                    .map_err(|_| super::ConversionError::InvalidBigInt)?,
            ))
        } else {
            None
        };

        let memo = if let Some(memo) = &self.memo {
            Memo::from_str(memo)
                .map_err(|_| super::ConversionError::Custom("Invalid memo".to_string()))?
        } else {
            Memo::empty()
        };

        let from: CompressedPubKey = AccountPublicKey::from_str(&self.from)?
            .try_into()
            .map_err(|_| super::ConversionError::InvalidBigInt)?;

        let signature = signature.try_into()?;

        let sc: signed_command::SignedCommand = signed_command::SignedCommand {
            payload: signed_command::SignedCommandPayload::create(
                Fee::from_u64(
                    self.fee
                        .parse::<u64>()
                        .map_err(|_| super::ConversionError::InvalidBigInt)?,
                ),
                from.clone(),
                nonce,
                valid_until,
                memo,
                signed_command::Body::StakeDelegation(
                    signed_command::StakeDelegationPayload::SetDelegate {
                        new_delegate: AccountPublicKey::from_str(&self.to)?
                            .try_into()
                            .map_err(|_| super::ConversionError::InvalidBigInt)?,
                    },
                ),
            ),
            signer: from.clone(),
            signature,
        };

        Ok(v2::MinaBaseUserCommandStableV2::SignedCommand(sc.into()))
    }
}
