use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
pub struct ScenarioId(String);

#[derive(thiserror::Error, Debug)]
pub enum ScenarioIdParseError {
    #[error("scenario id can't contain upper-case characters")]
    ContainsUpperCaseCharacters,
    #[error("scenario id must match pattern /[a-z0-9_-]*/")]
    AcceptedPatternMismatch,
}

#[cfg(feature = "scenario-generators")]
impl From<crate::scenarios::Scenarios> for ScenarioId {
    fn from(value: crate::scenarios::Scenarios) -> Self {
        Self(value.to_str().to_owned())
    }
}

impl std::fmt::Display for ScenarioId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ScenarioId {
    type Err = ScenarioIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for c in s.chars() {
            if c.is_ascii_uppercase() {
                return Err(ScenarioIdParseError::ContainsUpperCaseCharacters);
            }
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' {
                continue;
            }
            return Err(ScenarioIdParseError::AcceptedPatternMismatch);
        }
        Ok(Self(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for ScenarioId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        Self::from_str(s).map_err(serde::de::Error::custom)
    }
}
