use std::borrow::Cow;

use mina_p2p_messages::v2::NonZeroCurvePoint;

use super::{Snark, SnarkInfo, SnarkJobCommitment, SnarkJobId};

#[derive(Debug, Ord, Eq, PartialEq)]
pub struct SnarkCmp<'a> {
    pub job_id: Cow<'a, SnarkJobId>,
    pub fee: u64,
    pub prover: &'a NonZeroCurvePoint,
}

impl<'a> SnarkCmp<'a> {
    pub fn tie_breaker_hash(&self) -> [u8; 32] {
        super::tie_breaker_hash(&self.job_id, self.prover)
    }
}

impl<'a> PartialOrd for SnarkCmp<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.job_id
                .cmp(&other.job_id)
                .then_with(|| self.fee.cmp(&other.fee).reverse())
                .then_with(|| self.tie_breaker_hash().cmp(&other.tie_breaker_hash())),
        )
    }
}

impl<'a> From<&'a SnarkJobCommitment> for SnarkCmp<'a> {
    fn from(value: &'a SnarkJobCommitment) -> Self {
        Self {
            job_id: Cow::Borrowed(&value.job_id),
            fee: value.fee.0.as_u64(),
            prover: &value.snarker,
        }
    }
}

impl<'a> From<&'a Snark> for SnarkCmp<'a> {
    fn from(value: &'a Snark) -> Self {
        Self {
            job_id: Cow::Owned(value.job_id()),
            fee: value.fee.0.as_u64(),
            prover: &value.snarker,
        }
    }
}

impl<'a> From<&'a SnarkInfo> for SnarkCmp<'a> {
    fn from(value: &'a SnarkInfo) -> Self {
        Self {
            job_id: Cow::Borrowed(&value.job_id),
            fee: value.fee.0.as_u64(),
            prover: &value.prover,
        }
    }
}

pub fn snark_cmp<'a, A, B>(a: A, b: B) -> std::cmp::Ordering
where
    A: Into<SnarkCmp<'a>>,
    B: Into<SnarkCmp<'a>>,
{
    a.into().cmp(&b.into())
}

impl<T> PartialEq<T> for SnarkJobCommitment
where
    for<'a> &'a T: Into<SnarkCmp<'a>>,
{
    fn eq(&self, other: &T) -> bool {
        Into::<SnarkCmp<'_>>::into(&*self) == Into::<SnarkCmp<'_>>::into(other)
    }
}

impl<T> PartialOrd<T> for SnarkJobCommitment
where
    for<'a> &'a T: Into<SnarkCmp<'a>>,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        Some(snark_cmp(self, other))
    }
}

impl<T> PartialEq<T> for SnarkInfo
where
    for<'a> &'a T: Into<SnarkCmp<'a>>,
{
    fn eq(&self, other: &T) -> bool {
        Into::<SnarkCmp<'_>>::into(self) == Into::<SnarkCmp<'_>>::into(other)
    }
}

impl<T> PartialOrd<T> for SnarkInfo
where
    for<'a> &'a T: Into<SnarkCmp<'a>>,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        Some(snark_cmp(self, other))
    }
}

impl<T> PartialEq<T> for Snark
where
    for<'a> &'a T: Into<SnarkCmp<'a>>,
{
    fn eq(&self, other: &T) -> bool {
        Into::<SnarkCmp<'_>>::into(self) == Into::<SnarkCmp<'_>>::into(other)
    }
}

impl<T> PartialOrd<T> for Snark
where
    for<'a> &'a T: Into<SnarkCmp<'a>>,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        Some(snark_cmp(self, other))
    }
}
