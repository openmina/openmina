use std::{
    ops::{Add, AddAssign},
    path::PathBuf,
};

use num_bigint::BigInt;
use num_traits::identities::Zero;
use num_traits::FromPrimitive;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

use crate::StakingToolError;

// TODO(adonagy): remove dead_code
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MinaLedgerDumpBalanceStringNumber(BigInt);

impl<'de> Deserialize<'de> for MinaLedgerDumpBalanceStringNumber {
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
            .map(MinaLedgerDumpBalanceStringNumber)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for MinaLedgerDumpBalanceStringNumber {
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

impl From<MinaLedgerDumpBalanceStringNumber> for BigInt {
    fn from(value: MinaLedgerDumpBalanceStringNumber) -> Self {
        value.0
    }
}

impl From<&MinaLedgerDumpBalanceStringNumber> for BigInt {
    fn from(value: &MinaLedgerDumpBalanceStringNumber) -> Self {
        value.0.clone()
    }
}

impl From<MinaLedgerDumpBalanceStringNumber> for NanoMina {
    fn from(value: MinaLedgerDumpBalanceStringNumber) -> Self {
        NanoMina(value.0)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NanoMina(BigInt);

impl Serialize for NanoMina {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for NanoMina {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        BigInt::from_str(&s)
            .map(NanoMina)
            .map_err(serde::de::Error::custom)
    }
}

impl AddAssign for NanoMina {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0.clone() + rhs.0)
    }
}

impl Add for NanoMina {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

// impl From<usize> for NanoMina {
//     fn from(value: usize) -> Self {
//         Self(BigInt::from_usize(value).unwrap())
//     }
// }

impl NanoMina {
    pub fn new(value: BigInt) -> Self {
        let nano_factor: BigInt = 1_000_000_000.into();
        Self(value * nano_factor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntry {
    pub pk: String,
    pub balance: MinaLedgerDumpBalanceStringNumber,
    pub token: String,
    pub token_symbol: String,
    pub delegate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ledger {
    inner: Vec<LedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Balances {
    balance_producer: NanoMina,
    balance_delegated: NanoMina,
    balance_staked: NanoMina,
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
        self.inner
            .iter()
            .filter(|entry| entry.token == "wSHV2S4qX9jFsLjQo8r1BsMLH2ZRKsZx6EJd1sbozGPieEC4Jf")
            .map(|entry| &entry.balance.0)
            .sum()
    }

    pub fn producer_balances(&self, producer: &str) -> Balances {
        let mut balances = Balances::default();

        for entry in &self.inner {
            if entry.pk == producer {
                balances.balance_producer = entry.balance.clone().into();
            } else if let Some(delegate) = &entry.delegate {
                if delegate == producer {
                    balances.balance_delegated += entry.balance.clone().into();
                }
            }
        }
        balances.balance_staked =
            balances.balance_delegated.clone() + balances.balance_producer.clone();
        balances
    }
}
