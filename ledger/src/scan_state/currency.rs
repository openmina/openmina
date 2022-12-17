use std::cmp::Ordering::{Equal, Greater, Less};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sgn {
    Pos,
    Neg,
}

pub trait Magnitude
where
    Self: Sized,
{
    fn abs_diff(&self, rhs: &Self) -> Self;
    fn checked_add(&self, rhs: &Self) -> Option<Self>;
    fn checked_sub(&self, rhs: &Self) -> Option<Self>;
    fn is_zero(&self) -> bool;
    fn zero() -> Self;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Signed<T: Magnitude> {
    pub magnitude: T,
    pub sgn: Sgn,
}

impl<T> Signed<T>
where
    T: Magnitude + PartialOrd + Ord,
{
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/currency/currency.ml#L441
    pub fn zero() -> Self {
        Self {
            magnitude: T::zero(),
            sgn: Sgn::Pos,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/currency/currency.ml#L460
    pub fn add(&self, rhs: &Self) -> Option<Self> {
        let (magnitude, sgn) = if self.sgn == rhs.sgn {
            let magnitude = self.magnitude.checked_add(&rhs.magnitude)?;
            let sgn = self.sgn;

            (magnitude, sgn)
        } else {
            let sgn = match self.magnitude.cmp(&rhs.magnitude) {
                Less => rhs.sgn,
                Equal => self.sgn,
                Greater => return Some(Self::zero()),
            };
            let magnitude = self.magnitude.abs_diff(&rhs.magnitude);

            (magnitude, sgn)
        };

        Some(Self { magnitude, sgn })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fee(pub(super) u64);

impl Magnitude for Fee {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    fn abs_diff(&self, rhs: &Self) -> Self {
        Self(self.0.abs_diff(rhs.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(pub(super) u64);

impl Magnitude for Amount {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    fn abs_diff(&self, rhs: &Self) -> Self {
        Self(self.0.abs_diff(rhs.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance(pub(super) u64);

impl Magnitude for Balance {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    fn abs_diff(&self, rhs: &Self) -> Self {
        Self(self.0.abs_diff(rhs.0))
    }
}
