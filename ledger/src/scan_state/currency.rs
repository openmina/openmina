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
    fn wrapping_add(&self, rhs: &Self) -> Self;
    fn wrapping_mul(&self, rhs: &Self) -> Self;
    fn wrapping_sub(&self, rhs: &Self) -> Self;
    fn checked_add(&self, rhs: &Self) -> Option<Self>;
    fn checked_mul(&self, rhs: &Self) -> Option<Self>;
    fn checked_sub(&self, rhs: &Self) -> Option<Self>;
    fn is_zero(&self) -> bool;
    fn zero() -> Self;
}

/// Trait used for default values with `ClosedInterval`
pub trait MinMax {
    fn min() -> Self;
    fn max() -> Self;
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

    pub fn is_zero(&self) -> bool {
        self.magnitude.is_zero() && matches!(self.sgn, Sgn::Pos)
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
                Greater => self.sgn,
                Equal => return Some(Self::zero()),
            };
            let magnitude = self.magnitude.abs_diff(&rhs.magnitude);

            (magnitude, sgn)
        };

        Some(Self { magnitude, sgn })
    }
}

impl Amount {
    pub fn of_fee(fee: &Fee) -> Self {
        Self(fee.0)
    }
}

impl Balance {
    pub fn sub_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_sub(amount.0).map(Self)
    }

    pub fn add_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_add(amount.0).map(Self)
    }
}

impl Nonce {
    // TODO: Not sure if OCaml wraps around here
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

macro_rules! impl_number {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(impl_number!({$name32, u32, as_u32, from_u32, next_u32, append_u32},);)+
        $(impl_number!({$name64, u64, as_u64, from_u64, next_u64, append_u64},);)+
    };
    ($({ $name:ident, $inner:ty, $as_name:ident, $from_name:ident, $next_name:ident, $append_name:ident },)*) => ($(
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub(super) $inner);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!("{}({:?})", stringify!($name), self.0))
            }
        }

        impl Magnitude for $name {
            fn zero() -> Self {
                Self(0)
            }

            fn is_zero(&self) -> bool {
                self.0 == 0
            }

            fn wrapping_add(&self, rhs: &Self) -> Self {
                Self(self.0.wrapping_add(rhs.0))
            }

            fn wrapping_mul(&self, rhs: &Self) -> Self {
                Self(self.0.wrapping_mul(rhs.0))
            }

            fn wrapping_sub(&self, rhs: &Self) -> Self {
                Self(self.0.wrapping_sub(rhs.0))
            }

            fn checked_add(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_add(rhs.0).map(Self)
            }

            fn checked_mul(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_mul(rhs.0).map(Self)
            }

            fn checked_sub(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_sub(rhs.0).map(Self)
            }

            fn abs_diff(&self, rhs: &Self) -> Self {
                Self(self.0.abs_diff(rhs.0))
            }
        }

        impl MinMax for $name {
            fn min() -> Self { Self(0) }
            fn max() -> Self { Self(<$inner>::MAX) }
        }

        impl $name {
            pub fn $as_name(&self) -> $inner {
                self.0
            }

            pub const fn $from_name(value: $inner) -> Self {
                Self(value)
            }

            /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/currency/currency.ml#L379
            pub fn scale(&self, n: $inner) -> Option<Self> {
                self.checked_mul(&Self::$from_name(n))
            }

            pub fn min() -> Self {
                <Self as MinMax>::min()
            }

            pub fn max() -> Self {
                <Self as MinMax>::max()
            }
        }

        impl rand::distributions::Distribution<$name> for rand::distributions::Standard {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $name {
                $name(rng.$next_name())
            }
        }

        impl crate::ToInputs for $name {
            fn to_inputs(&self, inputs: &mut crate::Inputs) {
                inputs.$append_name(self.0);
            }
        }

    )+)
}

impl_number!(
    32: { Length, Slot, Nonce, Index, },
    64: { Amount, Balance, Fee, BlockTime, },
);
