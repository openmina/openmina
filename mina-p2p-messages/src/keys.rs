use ark_ff::fields::arithmetic::InvalidBigInt;

impl TryFrom<crate::v2::NonZeroCurvePointUncompressedStableV1> for mina_signer::CompressedPubKey {
    type Error = InvalidBigInt;

    fn try_from(
        val: crate::v2::NonZeroCurvePointUncompressedStableV1,
    ) -> Result<Self, Self::Error> {
        Ok(mina_signer::CompressedPubKey {
            x: val.x.try_into()?,
            is_odd: val.is_odd,
        })
    }
}

impl From<mina_signer::CompressedPubKey> for crate::v2::NonZeroCurvePointUncompressedStableV1 {
    fn from(v: mina_signer::CompressedPubKey) -> crate::v2::NonZeroCurvePointUncompressedStableV1 {
        Self {
            x: v.x.into(),
            is_odd: v.is_odd,
        }
    }
}
impl TryFrom<&crate::v2::NonZeroCurvePointUncompressedStableV1> for mina_signer::CompressedPubKey {
    type Error = InvalidBigInt;

    fn try_from(
        val: &crate::v2::NonZeroCurvePointUncompressedStableV1,
    ) -> Result<Self, Self::Error> {
        Ok(mina_signer::CompressedPubKey {
            x: (&val.x).try_into()?,
            is_odd: val.is_odd,
        })
    }
}

impl From<&mina_signer::CompressedPubKey> for crate::v2::NonZeroCurvePointUncompressedStableV1 {
    fn from(v: &mina_signer::CompressedPubKey) -> crate::v2::NonZeroCurvePointUncompressedStableV1 {
        Self {
            x: v.x.into(),
            is_odd: v.is_odd,
        }
    }
}

impl TryFrom<&crate::v2::NonZeroCurvePoint> for mina_signer::CompressedPubKey {
    type Error = InvalidBigInt;

    fn try_from(val: &crate::v2::NonZeroCurvePoint) -> Result<Self, Self::Error> {
        Ok(mina_signer::CompressedPubKey {
            x: (&val.x).try_into()?,
            is_odd: val.is_odd,
        })
    }
}

impl From<&mina_signer::CompressedPubKey> for crate::v2::NonZeroCurvePoint {
    fn from(v: &mina_signer::CompressedPubKey) -> crate::v2::NonZeroCurvePoint {
        let key = crate::v2::NonZeroCurvePointUncompressedStableV1 {
            x: (&v.x).into(),
            is_odd: v.is_odd,
        };
        key.into()
    }
}

impl From<&mina_signer::Signature> for crate::v2::MinaBaseSignatureStableV1 {
    fn from(value: &mina_signer::Signature) -> Self {
        Self(value.rx.into(), value.s.into())
    }
}

impl TryFrom<&crate::v2::MinaBaseSignatureStableV1> for mina_signer::Signature {
    type Error = InvalidBigInt;

    fn try_from(value: &crate::v2::MinaBaseSignatureStableV1) -> Result<Self, Self::Error> {
        Ok(Self {
            rx: value.0.to_field()?,
            s: value.1.to_field()?,
        })
    }
}
