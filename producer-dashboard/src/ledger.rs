use num_bigint::BigInt;
use num_traits::identities::Zero;
use num_traits::FromPrimitive;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;
use std::str::FromStr;

use crate::StakingToolError;

// TODO(adonagy): remove dead_code
#[allow(dead_code)]
#[derive(Debug)]
pub struct MinaBalanceStringNumber(BigInt);

impl<'de> Deserialize<'de> for MinaBalanceStringNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let parts: Vec<&str> = s.split('.').collect();
        let mut normalized = String::new();
        normalized.push_str(parts[0]);

        if parts.len() > 1 {
            if parts[1].len() > 9 {
                // If the fractional part is longer than 9 digits, raise an error, should not happen
                return Err(serde::de::Error::custom(
                    "Fractional part longer than 9 digits is not supported",
                ));
            }
            // Extend the fractional part to 9 digits if necessary
            normalized.push_str(parts[1]);
            (parts[1].len()..9).for_each(|_| normalized.push('0'))
        } else {
            // If no fractional part, just append 9 zeros
            normalized.extend(std::iter::repeat('0').take(9));
        }

        BigInt::from_str(&normalized)
            .map(MinaBalanceStringNumber)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for MinaBalanceStringNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Divide by 1_000_000_000 to get the integral part and remainder
        let divisor = BigInt::from_u64(1_000_000_000).unwrap();
        let (integral, fractional) = (&self.0 / &divisor, &self.0 % &divisor);

        // Convert integral part to string
        let integral_str = integral.to_string();

        // For the fractional part, only proceed if it's not zero
        let fractional_str = if !fractional.is_zero() {
            // Convert fractional part to string
            let mut fractional_str = fractional.to_string();
            // Ensure the fractional part is not omitted due to leading zeros
            while fractional_str.len() < 9 {
                fractional_str.insert(0, '0');
            }
            // Trim trailing zeros if necessary
            fractional_str.trim_end_matches('0').to_string()
        } else {
            String::new()
        };

        // Combine integral and fractional parts
        let result = if !fractional_str.is_empty() {
            format!("{}.{}", integral_str, fractional_str)
        } else {
            integral_str
        };

        // Serialize the combined string
        serializer.serialize_str(&result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerEntry {
    pk: String,
    balance: MinaBalanceStringNumber,
    delegate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ledger {
    inner: Vec<LedgerEntry>,
}

#[allow(dead_code)]
impl Ledger {
    pub fn load_from_file(path: PathBuf) -> Result<Self, StakingToolError> {
        let f = std::fs::File::open(path)?;

        Ok(Self {
            inner: serde_json::from_reader(f)?,
        })
    }

    pub fn gather_producer_and_delegates(&self, producer: &str) -> Vec<(usize, &LedgerEntry)> {
        self.inner
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.pk == producer || entry.delegate.as_deref() == Some(producer) {
                    Some((index, entry))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn total_currency(&self) -> BigInt {
        self.inner.iter().map(|entry| &entry.balance.0).sum()
    }
}

// {
//     "pk": "B62qiy32p8kAKnny8ZFwoMhYpBppM1DWVCqAPBYNcXnsAHhnfAAuXgg",
//     "balance": "0.000001",
//     "delegate": "B62qiy32p8kAKnny8ZFwoMhYpBppM1DWVCqAPBYNcXnsAHhnfAAuXgg",
//     "token": "wSHV2S4qX9jFsLjQo8r1BsMLH2ZRKsZx6EJd1sbozGPieEC4Jf",
//     "receipt_chain_hash":
//       "2n1hGCgg3jCKQJzVBgfujGqyV6D9riKgq27zhXqYgTRVZM5kqfkm",
//     "voting_for": "3NK2tkzqqK5spR2sZ7tujjqPksL45M3UUrcA4WhCkeiPtnugyE2x",
//     "permissions": {
//       "edit_state": "signature",
//       "send": "signature",
//       "set_delegate": "signature",
//       "set_permissions": "signature",
//       "set_verification_key": { "auth": "signature", "txn_version": "2" },
//       "set_zkapp_uri": "signature",
//       "edit_action_state": "signature",
//       "set_token_symbol": "signature",
//       "increment_nonce": "signature",
//       "set_voting_for": "signature",
//       "set_timing": "signature"
//     },
//     "token_symbol": ""
//   },
