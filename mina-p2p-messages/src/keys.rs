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
