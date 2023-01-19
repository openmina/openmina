#[cfg(feature = "hashing")]
impl From<crate::v2::NonZeroCurvePointUncompressedStableV1> for mina_signer::CompressedPubKey {
    fn from(val: crate::v2::NonZeroCurvePointUncompressedStableV1) -> Self {
        mina_signer::CompressedPubKey {
            x: val.x.into(),
            is_odd: val.is_odd,
        }
    }
}

#[cfg(feature = "hashing")]
impl From<mina_signer::CompressedPubKey> for crate::v2::NonZeroCurvePointUncompressedStableV1 {
    fn from(v: mina_signer::CompressedPubKey) -> crate::v2::NonZeroCurvePointUncompressedStableV1 {
        Self {
            x: v.x.into(),
            is_odd: v.is_odd,
        }
    }
}

#[cfg(feature = "hashing")]
impl From<&crate::v2::NonZeroCurvePoint> for mina_signer::CompressedPubKey {
    fn from(val: &crate::v2::NonZeroCurvePoint) -> Self {
        mina_signer::CompressedPubKey {
            x: (&val.x).into(),
            is_odd: val.is_odd,
        }
    }
}

#[cfg(feature = "hashing")]
impl From<&mina_signer::CompressedPubKey> for crate::v2::NonZeroCurvePoint {
    fn from(v: &mina_signer::CompressedPubKey) -> crate::v2::NonZeroCurvePoint {
        let key = crate::v2::NonZeroCurvePointUncompressedStableV1 {
            x: (&v.x).into(),
            is_odd: v.is_odd,
        };
        key.into()
    }
}

#[cfg(feature = "hashing")]
impl From<&mina_signer::Signature> for crate::v2::MinaBaseSignatureStableV1 {
    fn from(value: &mina_signer::Signature) -> Self {
        Self(value.rx.into(), value.s.into())
    }
}

#[cfg(feature = "hashing")]
impl From<&crate::v2::MinaBaseSignatureStableV1> for mina_signer::Signature {
    fn from(value: &crate::v2::MinaBaseSignatureStableV1) -> Self {
        Self {
            rx: value.0.to_field(),
            s: value.1.to_field(),
        }
    }
}
