//! Conversions between currency types and p2p message types.
//!
//! This module provides `From` implementations for converting between the
//! currency types in this crate and the wire format types from `mina-p2p-messages`.

use mina_p2p_messages::v2::{
    CurrencyAmountStableV1, CurrencyFeeStableV1, SgnStableV1, SignedAmount,
    UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};

use crate::currency::{Amount, Fee, Sgn, Signed};

// Amount conversions

impl From<CurrencyAmountStableV1> for Amount {
    fn from(value: CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<Amount> for CurrencyAmountStableV1 {
    fn from(value: Amount) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Amount> for CurrencyAmountStableV1 {
    fn from(value: &Amount) -> Self {
        CurrencyAmountStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Amount> for CurrencyFeeStableV1 {
    fn from(value: &Amount) -> Self {
        CurrencyFeeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

// Fee conversions

impl From<&CurrencyFeeStableV1> for Fee {
    fn from(value: &CurrencyFeeStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&CurrencyAmountStableV1> for Fee {
    fn from(value: &CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&Fee> for CurrencyFeeStableV1 {
    fn from(value: &Fee) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Fee> for CurrencyAmountStableV1 {
    fn from(value: &Fee) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

// Sgn conversions

impl From<SgnStableV1> for Sgn {
    fn from(value: SgnStableV1) -> Self {
        match value {
            SgnStableV1::Pos => Self::Pos,
            SgnStableV1::Neg => Self::Neg,
        }
    }
}

impl From<&Sgn> for SgnStableV1 {
    fn from(value: &Sgn) -> Self {
        match value {
            Sgn::Pos => Self::Pos,
            Sgn::Neg => Self::Neg,
        }
    }
}

// Signed<Amount> conversions

impl From<&SignedAmount> for Signed<Amount> {
    fn from(value: &SignedAmount) -> Self {
        Self {
            magnitude: Amount(value.magnitude.as_u64()),
            sgn: value.sgn.clone().into(),
        }
    }
}

impl From<&Signed<Amount>> for SignedAmount {
    fn from(value: &Signed<Amount>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn).into(),
        }
    }
}

// Signed<Fee> conversions

impl From<&SignedAmount> for Signed<Fee> {
    fn from(value: &SignedAmount) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: value.sgn.clone().into(),
        }
    }
}

impl From<&Signed<Fee>> for SignedAmount {
    fn from(value: &Signed<Fee>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn).into(),
        }
    }
}

// MinaStateBlockchainStateValueStableV2SignedAmount conversions
// (another signed amount type used in blockchain state)

type BlockchainStateSignedAmount =
    mina_p2p_messages::v2::MinaStateBlockchainStateValueStableV2SignedAmount;

impl From<&BlockchainStateSignedAmount> for Signed<Amount> {
    fn from(value: &BlockchainStateSignedAmount) -> Self {
        let BlockchainStateSignedAmount { magnitude, sgn } = value;

        Self {
            magnitude: (magnitude.clone()).into(),
            sgn: (sgn.clone()).into(),
        }
    }
}

impl From<&Signed<Amount>> for BlockchainStateSignedAmount {
    fn from(value: &Signed<Amount>) -> Self {
        let Signed::<Amount> { magnitude, sgn } = value;

        Self {
            magnitude: (*magnitude).into(),
            sgn: sgn.into(),
        }
    }
}
