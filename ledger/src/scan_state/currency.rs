use std::cmp::Ordering::{Equal, Greater, Less};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sgn {
    Pos,
    Neg,
}

impl Sgn {
    fn negate(&self) -> Self {
        match self {
            Sgn::Pos => Sgn::Neg,
            Sgn::Neg => Sgn::Pos,
        }
    }
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
    T: Magnitude + PartialOrd + Ord + Clone,
{
    pub fn create(magnitude: T, sgn: Sgn) -> Self {
        Self { magnitude, sgn }
    }

    pub fn of_unsigned(magnitude: T) -> Self {
        Self::create(magnitude, Sgn::Pos)
    }

    pub fn negate(&self) -> Self {
        if self.magnitude.is_zero() {
            Self::zero()
        } else {
            Self {
                magnitude: self.magnitude.clone(),
                sgn: self.sgn.negate(),
            }
        }
    }

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

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl Fee {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl Amount {
    pub fn of_fee(fee: &Fee) -> Self {
        Self(fee.0)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(amount: u64) -> Self {
        Self(amount)
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl Balance {
    pub fn sub_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_sub(amount.0).map(Self)
    }

    pub fn add_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_add(amount.0).map(Self)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(balance: u64) -> Self {
        Self(balance)
    }
}

impl rand::distributions::Distribution<Balance> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Balance {
        Balance(rng.next_u64())
    }
}

impl rand::distributions::Distribution<Amount> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Amount {
        Amount(rng.next_u64())
    }
}

impl rand::distributions::Distribution<Fee> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Fee {
        Fee(rng.next_u64())
    }
}
