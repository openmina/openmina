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

use crate::TokenId;

use super::{
    currency::{Fee, Magnitude, Signed},
    scan_state::transaction_snark::OneOrTwo,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeExcess {
    pub(super) fee_token_l: TokenId,
    pub(super) fee_excess_l: Signed<Fee>,
    pub(super) fee_token_r: TokenId,
    pub(super) fee_excess_r: Signed<Fee>,
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
        self == &Self::empty()
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
