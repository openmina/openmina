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
    Self: Sized + PartialOrd + Copy,
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

    fn add_flagged(&self, rhs: &Self) -> (Self, bool) {
        let z = self.wrapping_add(rhs);
        (z, z < *self)
    }

    fn sub_flagged(&self, rhs: &Self) -> (Self, bool) {
        (self.wrapping_sub(rhs), self < rhs)
    }
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
                magnitude: self.magnitude,
                sgn: self.sgn.negate(),
            }
        }
    }

    pub fn is_pos(&self) -> bool {
        matches!(self.sgn, Sgn::Pos)
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

    pub fn add_flagged(&self, rhs: Self) -> (Self, bool) {
        match (self.sgn, rhs.sgn) {
            (Sgn::Neg, sgn @ Sgn::Neg) | (Sgn::Pos, sgn @ Sgn::Pos) => {
                let (magnitude, overflow) = self.magnitude.add_flagged(&rhs.magnitude);
                (Self { magnitude, sgn }, overflow)
            }
            (Sgn::Pos, Sgn::Neg) | (Sgn::Neg, Sgn::Pos) => {
                let sgn = match self.magnitude.cmp(&rhs.magnitude) {
                    Less => rhs.sgn,
                    Greater => self.sgn,
                    Equal => Sgn::Pos,
                };
                let magnitude = self.magnitude.abs_diff(&rhs.magnitude);
                (Self { magnitude, sgn }, false)
            }
        }
    }
}

impl Amount {
    pub fn of_fee(fee: &Fee) -> Self {
        Self(fee.0)
    }

    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }
}

impl Balance {
    pub fn sub_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_sub(amount.0).map(Self)
    }

    pub fn add_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_add(amount.0).map(Self)
    }

    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    pub fn add_signed_amount_flagged(&self, rhs: Signed<Amount>) -> (Self, bool) {
        let rhs = Signed {
            magnitude: Balance::from_u64(rhs.magnitude.0),
            sgn: rhs.sgn,
        };

        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    pub fn to_amount(self) -> Amount {
        Amount(self.0)
    }
}

impl Index {
    // TODO: Not sure if OCaml wraps around here
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl Nonce {
    // TODO: Not sure if OCaml wraps around here
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
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
            pub const fn scale(&self, n: $inner) -> Option<Self> {
                match self.0.checked_mul(n) {
                    Some(n) => Some(Self(n)),
                    None => None
                }
            }

            pub fn min() -> Self {
                <Self as MinMax>::min()
            }

            pub fn max() -> Self {
                <Self as MinMax>::max()
            }

            /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/currency/currency.ml#L118
            pub fn of_formatted_string(input: &str) -> Self {
                const PRECISION: usize = 9;

                let mut s = String::with_capacity(input.len() + 9);

                if !input.contains('.') {
                    let append = "000000000";
                    assert_eq!(append.len(), PRECISION);

                    s.push_str(append);
                } else {
                    let (whole, decimal) = {
                        let mut splitted = input.split('.');
                        let whole = splitted.next().unwrap();
                        let decimal = splitted.next().unwrap();
                        assert!(splitted.next().is_none(), "Currency.of_formatted_string: Invalid currency input");
                        (whole, decimal)
                    };

                    let decimal_length = decimal.len();

                    if decimal_length > PRECISION {
                        s.push_str(whole);
                        s.push_str(&decimal[0..PRECISION]);
                    } else {
                        s.push_str(whole);
                        s.push_str(decimal);
                        for _ in 0..PRECISION - decimal_length {
                            s.push('0');
                        }
                    }
                }

                let n = s.parse::<$inner>().unwrap();
                Self(n)
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
