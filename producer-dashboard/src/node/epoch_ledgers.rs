use std::{fs, io::Write, path::PathBuf, process::Command};

use num_bigint::BigInt;
use num_traits::identities::Zero;
use num_traits::FromPrimitive;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

use crate::StakingToolError;

// TODO(adonagy): remove dead_code
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
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

impl From<MinaBalanceStringNumber> for BigInt {
    fn from(value: MinaBalanceStringNumber) -> Self {
        value.0
    }
}

impl From<&MinaBalanceStringNumber> for BigInt {
    fn from(value: &MinaBalanceStringNumber) -> Self {
        value.0.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntry {
    pub pk: String,
    pub balance: MinaBalanceStringNumber,
    pub delegate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ledger {
    inner: Vec<LedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Balances {
    #[serde(serialize_with = "serialize_bigint_as_string")]
    balance_producer: BigInt,
    #[serde(serialize_with = "serialize_bigint_as_string")]
    balance_delegated: BigInt,
    #[serde(serialize_with = "serialize_bigint_as_string")]
    balance_staked: BigInt,
}

fn serialize_bigint_as_string<S>(num: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&num.to_string())
}

#[allow(dead_code)]
impl Ledger {
    pub fn new(inner: Vec<LedgerEntry>) -> Self {
        Self { inner }
    }

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

    pub fn producer_balances(&self, producer: &str) -> Balances {
        let mut balances = Balances::default();

        for entry in &self.inner {
            if entry.pk == producer {
                balances.balance_producer = entry.balance.clone().into();
            } else if let Some(delegate) = &entry.delegate {
                if delegate == producer {
                    balances.balance_delegated += &entry.balance.clone().into();
                }
            }
        }
        balances.balance_staked = &balances.balance_delegated + &balances.balance_producer;
        balances
    }
}
