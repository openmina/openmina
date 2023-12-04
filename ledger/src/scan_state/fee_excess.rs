//! Fee excesses associated with transactions or transitions.
//!
//! These are represented as a 'left' and 'right' excess, which describe the
//! unresolved fee excesses in the fee tokens of the first (or leftmost) and
//! last (or rightmost) transactions in the transition.
//!
//! Assumptions:
//! * Transactions are grouped by their fee token.
//! * The 'fee transfer' transaction to dispense those fees is part of this
//!   group.
//! * The fee excess for each token is 0 across the group.
//! * No transactions with fees paid in another token are executed while the
//!   previous fee token's excess is non-zero.
//!
//! By maintaining these assumptions, we can ensure that the un-settled fee
//! excesses can be represented by excesses in (at most) 2 tokens.
//! Consider, for example, any consecutive subsequence of the transactions
//!
//! ..[txn@2][ft@2][txn@3][txn@3][ft@3][txn@4][ft@4][txn@5][txn@5][ft@5][txn@6][ft@6]..
//!
//! where [txn@i] and [ft@i] are transactions and fee transfers respectively
//! paid in token i.
//! The only groups which may have non-zero fee excesses are those which
//! contain the start and end of the subsequence.
//!
//! The code below also defines a canonical representation where fewer than 2
//! tokens have non-zero excesses. See [rebalance] below for details and the
//! implementation.
//!
//!
//! Port of the implementation from:
//! https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/fee_excess.ml#L1

use ark_ff::{BigInteger, BigInteger256, Zero};
use mina_hasher::Fp;

use crate::{
    proofs::{
        numbers::currency::{CheckedFee, CheckedSigned},
        witness::{field, Boolean, FieldWitness, Witness},
    },
    ToInputs, TokenId,
};

use super::{
    currency::{Fee, Magnitude, Sgn, Signed},
    scan_state::transaction_snark::OneOrTwo,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeExcess {
    pub fee_token_l: TokenId,
    pub fee_excess_l: Signed<Fee>,
    pub fee_token_r: TokenId,
    pub fee_excess_r: Signed<Fee>,
}

#[derive(Debug, Clone)]
pub struct CheckedFeeExcess<F: FieldWitness> {
    pub fee_token_l: TokenId,
    pub fee_excess_l: CheckedSigned<F, CheckedFee<F>>,
    pub fee_token_r: TokenId,
    pub fee_excess_r: CheckedSigned<F, CheckedFee<F>>,
}

impl ToInputs for FeeExcess {
    /// https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/mina_base/fee_excess.ml#L162
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        let Self {
            fee_token_l,
            fee_excess_l,
            fee_token_r,
            fee_excess_r,
        } = self;

        inputs.append(fee_token_l);
        inputs.append(fee_excess_l);
        inputs.append(fee_token_r);
        inputs.append(fee_excess_r);
    }
}

impl FeeExcess {
    pub fn empty() -> Self {
        Self {
            fee_token_l: TokenId::default(),
            fee_excess_l: Signed::<Fee>::zero(),
            fee_token_r: TokenId::default(),
            fee_excess_r: Signed::<Fee>::zero(),
        }
    }

    pub fn is_zero(&self) -> bool {
        let Self {
            fee_token_l,
            fee_excess_l,
            fee_token_r,
            fee_excess_r,
        } = self;

        fee_token_l.is_default()
            && fee_token_r.is_default()
            && fee_excess_l.is_zero()
            && fee_excess_r.is_zero()
    }

    /// https://github.com/MinaProtocol/mina/blob/e5183ca1dde1c085b4c5d37d1d9987e24c294c32/src/lib/mina_base/fee_excess.ml#L536
    pub fn of_one_or_two(excesses: OneOrTwo<(TokenId, Signed<Fee>)>) -> Result<Self, String> {
        match excesses {
            OneOrTwo::One((fee_token_l, fee_excess_l)) => Self {
                fee_token_l,
                fee_excess_l,
                fee_token_r: TokenId::default(),
                fee_excess_r: Signed::<Fee>::zero(),
            },
            OneOrTwo::Two(((fee_token_l, fee_excess_l), (fee_token_r, fee_excess_r))) => Self {
                fee_token_l,
                fee_excess_l,
                fee_token_r,
                fee_excess_r,
            },
        }
        .rebalance()
    }

    /// https://github.com/MinaProtocol/mina/blob/e5183ca1dde1c085b4c5d37d1d9987e24c294c32/src/lib/mina_base/fee_excess.ml#L526
    pub fn of_single((fee_token_l, fee_excess_l): (TokenId, Signed<Fee>)) -> Self {
        Self {
            fee_token_l,
            fee_excess_l,
            fee_token_r: TokenId::default(),
            fee_excess_r: Signed::<Fee>::zero(),
        }
    }

    /// 'Rebalance' to a canonical form, where
    /// - if there is only 1 nonzero excess, it is to the left
    /// - any zero fee excess has the default token
    /// - if the fee tokens are the same, the excesses are combined
    ///
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/fee_excess.ml#L301
    fn rebalance(&self) -> Result<Self, String> {
        let Self {
            fee_token_l,
            fee_excess_l,
            fee_token_r,
            fee_excess_r,
        } = self;

        // Use the same token for both if [fee_excess_l] is zero.
        let fee_token_l = if fee_excess_l.magnitude.is_zero() {
            fee_token_r
        } else {
            fee_token_l
        };

        // Rebalancing.
        let (fee_excess_l, fee_excess_r) = if fee_token_l == fee_token_r {
            match fee_excess_l.add(fee_excess_r) {
                Some(fee_excess_l) => (fee_excess_l, Signed::<Fee>::zero()),
                None => return Err("Error adding fees: overflow".to_string()),
            }
        } else {
            (*fee_excess_l, *fee_excess_r)
        };

        // Use the default token if the excess is zero.
        // This allows [verify_complete_merge] to verify a proof without knowledge of
        // the particular fee tokens used.

        let fee_token_l = if fee_excess_l.magnitude.is_zero() {
            TokenId::default()
        } else {
            fee_token_l.clone()
        };
        let fee_token_r = if fee_excess_r.magnitude.is_zero() {
            TokenId::default()
        } else {
            fee_token_r.clone()
        };

        Ok(Self {
            fee_token_l,
            fee_excess_l,
            fee_token_r,
            fee_excess_r,
        })
    }

    fn rebalance_checked(
        fee_token_l: TokenId,
        fee_excess_l: Fp,
        fee_token_r: TokenId,
        fee_excess_r: Fp,
        w: &mut Witness<Fp>,
    ) -> (TokenId, Fp, TokenId, Fp) {
        let fee_token_l = {
            let excess_is_zero = field::equal(Fp::zero(), fee_excess_l, w);
            w.exists_no_check(match excess_is_zero {
                Boolean::True => &fee_token_r,
                Boolean::False => &fee_token_l,
            })
        };

        let (fee_excess_l, fee_excess_r) = {
            let tokens_equal = field::equal(fee_token_l.0, fee_token_r.0, w);
            let amount_to_move = w.exists_no_check(match tokens_equal {
                Boolean::True => fee_excess_r,
                Boolean::False => Fp::zero(),
            });

            (fee_excess_l + amount_to_move, fee_excess_r + amount_to_move)
        };

        let fee_token_l = {
            let excess_is_zero = field::equal(Fp::zero(), fee_excess_l, w);
            w.exists_no_check(match excess_is_zero {
                Boolean::True => TokenId::default(),
                Boolean::False => fee_token_l.clone(),
            })
        };

        let fee_token_r = {
            let excess_is_zero = field::equal(Fp::zero(), fee_excess_r, w);
            w.exists_no_check(match excess_is_zero {
                Boolean::True => TokenId::default(),
                Boolean::False => fee_token_r.clone(),
            })
        };

        (fee_token_l, fee_excess_l, fee_token_r, fee_excess_r)
    }

    /// Combine the fee excesses from two transitions.
    ///
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/fee_excess.ml#L380
    pub fn combine(
        Self {
            fee_token_l: fee_token1_l,
            fee_excess_l: fee_excess1_l,
            fee_token_r: fee_token1_r,
            fee_excess_r: fee_excess1_r,
        }: &Self,
        Self {
            fee_token_l: fee_token2_l,
            fee_excess_l: fee_excess2_l,
            fee_token_r: fee_token2_r,
            fee_excess_r: fee_excess2_r,
        }: &Self,
    ) -> Result<Self, String> {
        // Eliminate fee_excess1_r.
        // [1l; 1r; 2l; 2r] -> [1l; 2l; 2r]
        let ((fee_token1_l, fee_excess1_l), (fee_token2_l, fee_excess2_l)) = eliminate_fee_excess(
            (fee_token1_l, fee_excess1_l),
            (fee_token1_r, fee_excess1_r),
            (fee_token2_l, fee_excess2_l),
        )?;

        // Eliminate fee_excess2_l.
        // [1l; 2l; 2r] -> [1l; 2r]
        let ((fee_token1_l, fee_excess1_l), (fee_token2_r, fee_excess2_r)) = eliminate_fee_excess(
            (fee_token1_l, &fee_excess1_l),
            (fee_token2_l, &fee_excess2_l),
            (fee_token2_r, fee_excess2_r),
        )?;

        Self {
            fee_token_l: fee_token1_l.clone(),
            fee_excess_l: fee_excess1_l,
            fee_token_r: fee_token2_r.clone(),
            fee_excess_r: fee_excess2_r,
        }
        .rebalance()
    }

    pub fn combine_checked(
        Self {
            fee_token_l: fee_token1_l,
            fee_excess_l: fee_excess1_l,
            fee_token_r: fee_token1_r,
            fee_excess_r: fee_excess1_r,
        }: &Self,
        Self {
            fee_token_l: fee_token2_l,
            fee_excess_l: fee_excess2_l,
            fee_token_r: fee_token2_r,
            fee_excess_r: fee_excess2_r,
        }: &Self,
        w: &mut Witness<Fp>,
    ) -> (TokenId, Signed<Fee>, TokenId, Signed<Fee>) {
        // Represent amounts as field elements.
        let fee_excess1_l = fee_excess1_l.to_checked::<Fp>().value(w);
        let fee_excess1_r = fee_excess1_r.to_checked::<Fp>().value(w);
        let fee_excess2_l = fee_excess2_l.to_checked::<Fp>().value(w);
        let fee_excess2_r = fee_excess2_r.to_checked::<Fp>().value(w);

        let ((fee_token1_l, fee_excess1_l), (fee_token2_l, fee_excess2_l)) =
            eliminate_fee_excess_checked(
                (fee_token1_l, fee_excess1_l),
                (fee_token1_r, fee_excess1_r),
                (fee_token2_l, fee_excess2_l),
                w,
            );

        let ((fee_token1_l, fee_excess1_l), (fee_token2_r, fee_excess2_r)) =
            eliminate_fee_excess_checked(
                (&fee_token1_l, fee_excess1_l),
                (&fee_token2_l, fee_excess2_l),
                (&fee_token2_r, fee_excess2_r),
                w,
            );

        let (fee_token_l, fee_excess_l, fee_token_r, fee_excess_r) =
            Self::rebalance_checked(fee_token1_l, fee_excess1_l, fee_token2_r, fee_excess2_r, w);

        let convert_to_currency = |excess: Fp| {
            let bigint: BigInteger256 = excess.into();
            let is_neg = bigint.get_bit(255 - 1);
            let sgn = if is_neg { Sgn::Neg } else { Sgn::Pos };
            let magnitude = Fee::from_u64(bigint.0[0]);
            Signed::create(magnitude, sgn)
        };

        let fee_excess_l = w.exists(convert_to_currency(fee_excess_l));
        fee_excess_l.to_checked::<Fp>().value(w); // Made by `Fee.Signed.Checked.to_field_var` call

        let fee_excess_r = w.exists(convert_to_currency(fee_excess_r));
        fee_excess_r.to_checked::<Fp>().value(w); // Made by `Fee.Signed.Checked.to_field_var` call

        (fee_token_l, fee_excess_l, fee_token_r, fee_excess_r)
    }
}

/// Eliminate a fee excess, either by combining it with one to the left/right,
/// or by checking that it is zero.
///
/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/fee_excess.ml#L200
fn eliminate_fee_excess<'a>(
    (fee_token_l, fee_excess_l): (&'a TokenId, &'a Signed<Fee>),
    (fee_token_m, fee_excess_m): (&'a TokenId, &'a Signed<Fee>),
    (fee_token_r, fee_excess_r): (&'a TokenId, &'a Signed<Fee>),
) -> Result<((&'a TokenId, Signed<Fee>), (&'a TokenId, Signed<Fee>)), String> {
    let add_err = |x: &Signed<Fee>, y: &Signed<Fee>| -> Result<Signed<Fee>, String> {
        x.add(y)
            .ok_or_else(|| "Error adding fees: Overflow".to_string())
    };

    if fee_token_l == fee_token_m || fee_excess_l.magnitude.is_zero() {
        let fee_excess_l = add_err(fee_excess_l, fee_excess_m)?;
        Ok(((fee_token_m, fee_excess_l), (fee_token_r, *fee_excess_r)))
    } else if fee_token_r == fee_token_m || fee_excess_r.magnitude.is_zero() {
        let fee_excess_r = add_err(fee_excess_r, fee_excess_m)?;
        Ok(((fee_token_l, *fee_excess_l), (fee_token_m, fee_excess_r)))
    } else if fee_excess_m.magnitude.is_zero() {
        Ok(((fee_token_l, *fee_excess_l), (fee_token_r, *fee_excess_r)))
    } else {
        Err(format!(
            "Error eliminating fee excess: Excess for token {:?} \
             {:?} was nonzero",
            fee_token_m, fee_excess_m
        ))
    }
}

pub fn assert_equal_checked(_t1: &FeeExcess, t2: &FeeExcess, w: &mut Witness<Fp>) {
    t2.fee_excess_l.to_checked::<Fp>().value(w);
    t2.fee_excess_r.to_checked::<Fp>().value(w);
}

fn eliminate_fee_excess_checked<'a>(
    (fee_token_l, fee_excess_l): (&'a TokenId, Fp),
    (fee_token_m, fee_excess_m): (&'a TokenId, Fp),
    (fee_token_r, fee_excess_r): (&'a TokenId, Fp),
    w: &mut Witness<Fp>,
) -> ((TokenId, Fp), (TokenId, Fp)) {
    let mut combine = |fee_token: &TokenId, fee_excess: Fp, fee_excess_m: Fp| {
        let fee_token_equal = field::equal(fee_token.0, fee_token_m.0, w);
        let fee_excess_zero = field::equal(Fp::zero(), fee_excess, w);

        let may_move = fee_token_equal.or(&fee_excess_zero, w);

        let fee_token = w.exists_no_check(match fee_excess_zero {
            Boolean::True => fee_token_m,
            Boolean::False => fee_token,
        });

        let fee_excess_to_move = w.exists_no_check(match may_move {
            Boolean::True => fee_excess_m,
            Boolean::False => Fp::zero(),
        });

        (
            (fee_token.clone(), fee_excess + fee_excess_to_move),
            fee_excess_m - fee_excess_to_move,
        )
    };

    let ((fee_token_l, fee_excess_l), fee_excess_m) =
        combine(fee_token_l, fee_excess_l, fee_excess_m);
    let ((fee_token_r, fee_excess_r), _fee_excess_m) =
        combine(fee_token_r, fee_excess_r, fee_excess_m);

    ((fee_token_l, fee_excess_l), (fee_token_r, fee_excess_r))
}
