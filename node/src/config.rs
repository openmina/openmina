use std::ffi::OsString;
use std::str::FromStr;
use std::sync::Arc;

use mina_p2p_messages::v2::CurrencyFeeStableV1;
use serde::{Deserialize, Serialize};

use crate::account::AccountPublicKey;
pub use crate::block_producer::BlockProducerConfig;
pub use crate::ledger::LedgerConfig;
pub use crate::p2p::P2pConfig;
pub use crate::snark::SnarkConfig;
pub use crate::snark_pool::SnarkPoolConfig;
use crate::transition_frontier::genesis::GenesisConfig;
pub use crate::transition_frontier::TransitionFrontierConfig;
pub use mina_p2p_messages::v2::MinaBaseProtocolConstantsCheckedValueStableV1 as ProtocolConstants;

// TODO(binier): maybe make sure config is immutable.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ledger: LedgerConfig,
    pub snark: SnarkConfig,
    pub p2p: P2pConfig,
    pub transition_frontier: TransitionFrontierConfig,
    pub block_producer: Option<BlockProducerConfig>,
    pub global: GlobalConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalConfig {
    pub build: Box<BuildEnv>,
    pub snarker: Option<SnarkerConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkerConfig {
    pub public_key: AccountPublicKey,
    pub fee: CurrencyFeeStableV1,
    pub strategy: SnarkerStrategy,
    pub auto_commit: bool,
    /// External Mina snark worker executable path
    pub path: OsString,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SnarkerStrategy {
    Sequential,
    Random,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildEnv {
    pub time: String,
    pub git: GitBuildEnv,
    pub cargo: CargoBuildEnv,
    pub rustc: RustCBuildEnv,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct GitBuildEnv {
    pub commit_time: String,
    pub commit_hash: String,
    pub branch: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct CargoBuildEnv {
    pub features: String,
    pub opt_level: u8,
    pub target: String,
    pub is_debug: bool,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct RustCBuildEnv {
    pub channel: String,
    pub commit_date: String,
    pub commit_hash: String,
    pub host: String,
    pub version: String,
    pub llvm_version: String,
}

#[allow(clippy::eq_op)]
impl BuildEnv {
    pub fn get() -> Self {
        Self {
            time: env!("VERGEN_BUILD_TIMESTAMP").to_owned(),
            git: GitBuildEnv {
                commit_time: env!("VERGEN_GIT_COMMIT_TIMESTAMP").to_owned(),
                commit_hash: env!("VERGEN_GIT_SHA").to_owned(),
                branch: env!("VERGEN_GIT_BRANCH").to_owned(),
            },
            cargo: CargoBuildEnv {
                features: env!("VERGEN_CARGO_FEATURES").to_owned(),
                opt_level: env!("VERGEN_CARGO_OPT_LEVEL").parse().unwrap(),
                target: env!("VERGEN_CARGO_TARGET_TRIPLE").to_owned(),
                is_debug: env!("VERGEN_CARGO_DEBUG") == "true",
            },
            rustc: RustCBuildEnv {
                channel: env!("VERGEN_RUSTC_CHANNEL").to_owned(),
                commit_date: env!("VERGEN_RUSTC_COMMIT_DATE").to_owned(),
                commit_hash: env!("VERGEN_RUSTC_COMMIT_HASH").to_owned(),
                host: env!("VERGEN_RUSTC_HOST_TRIPLE").to_owned(),
                version: env!("VERGEN_RUSTC_SEMVER").to_owned(),
                llvm_version: env!("VERGEN_RUSTC_LLVM_VERSION").to_owned(),
            },
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid strategy: {0}! expected one of: seq/sequential/rand/random")]
pub struct SnarkerStrategyParseError(String);

impl FromStr for SnarkerStrategy {
    type Err = SnarkerStrategyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "seq" | "sequential" => SnarkerStrategy::Sequential,
            "rand" | "random" => SnarkerStrategy::Random,
            other => return Err(SnarkerStrategyParseError(other.to_owned())),
        })
    }
}

// Load static berkeley genesis ledger for testing
lazy_static::lazy_static! {
    pub static ref BERKELEY_CONFIG: Arc<GenesisConfig> = {
        let bytes = include_bytes!("../../genesis_ledgers/berkeley.bin");
        Arc::new(GenesisConfig::Prebuilt(
            std::borrow::Cow::Borrowed(bytes)
        ))
    };
}

#[cfg(test)]
mod tests {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    use super::BERKELEY_CONFIG;

    #[test]
    fn berkeley_config() {
        let (_mask, config) = BERKELEY_CONFIG.load().expect("should be loadable");

        assert_eq!(
            config.ledger_hash,
            "jwkqwgAC6MXgfiZmynHRqXV6PGbMbLwFCx56Y2rt5vwdumf6ofp"
                .parse()
                .unwrap()
        );
        assert_eq!(
            config.constants.genesis_state_timestamp,
            OffsetDateTime::parse("2024-02-02T14:01:01Z", &Rfc3339)
                .unwrap()
                .into()
        );
    }
}
