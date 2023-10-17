use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
pub struct ScenarioId(String);

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

impl<'de> Deserialize<'de> for ScenarioId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        for c in s.chars() {
            if ('A'..'Z').contains(&c) {
                return Err(serde::de::Error::custom(
                    "scenario id can't contain upper-case characters",
                ));
            }
            if ('a'..'z').contains(&c) || ('0'..'9').contains(&c) || c == '-' || c == '_' {
                continue;
            }
            return Err(serde::de::Error::custom(
                "scenario id must match pattern /[a-z0-9_-]*/",
            ));
        }

        Ok(ScenarioId(s))
    }
}
