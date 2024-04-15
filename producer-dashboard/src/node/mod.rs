use ::reqwest::Client;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use num_bigint::BigInt;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeSet, path::PathBuf, process::Command, str::FromStr};

use crate::StakingToolError;

pub mod epoch_ledgers;
pub mod watchdog;

use self::{daemon_status::SyncStatus, epoch_ledgers::LedgerEntry};

type PublicKey = String;
type StateHash = String;
type FeeTransferType = String;
type UserCommandKind = String;
type Amount = StringNumber;
type Fee = StringNumber;
type Epoch = String;
type Length = String;
type EpochSeed = String;
type Slot = String;
type Globalslot = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    best_tip: Option<BestTip>,
    best_chain: Vec<String>,
    transition_frontier_root_global_slot: u32,
    transition_frontier_best_tip_global_slot: u32,
    sync_status: SyncStatus,
    dumped_ledgers: BTreeSet<u32>,
}

impl NodeData {
    // TODO(adonagy): Hydrate from db
    // pub fn new()
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct StringNumber(BigInt);

#[allow(dead_code)]
impl StringNumber {
    pub fn to_bigint(&self) -> BigInt {
        self.0.clone()
    }
}

impl From<BigInt> for StringNumber {
    fn from(value: BigInt) -> Self {
        Self(value)
    }
}

impl<'de> Deserialize<'de> for StringNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BigInt::from_str(&s)
            .map(StringNumber)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for StringNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let as_string = self.0.to_string();
        serializer.serialize_str(&as_string)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/daemon_status.graphql",
    response_derives = "Debug, Clone"
)]
struct DaemonStatus;

impl DaemonStatus {
    pub async fn sync_status() -> SyncStatus {
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .build()
            .unwrap();

        let variables = daemon_status::Variables {};

        let response_body =
            post_graphql::<DaemonStatus, _>(&client, "http://adonagy.com:5000/graphql", variables)
                .await
                .unwrap();

        let response_data: daemon_status::ResponseData = response_body
            .data
            .ok_or(StakingToolError::EmptyGraphqlResponse)
            .unwrap();

        response_data.daemon_status.sync_status
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/genesis_timestamp.graphql",
    response_derives = "Debug, Clone"
)]
struct GenesisTimestamp;

pub async fn get_genesis_timestmap() -> Result<String, StakingToolError> {
    let client = Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .build()
        .unwrap();

    let variables = genesis_timestamp::Variables {};

    let response_body =
        post_graphql::<GenesisTimestamp, _>(&client, "http://adonagy.com:5000/graphql", variables)
            .await
            .unwrap();
    let response_data: genesis_timestamp::ResponseData = response_body
        .data
        .ok_or(StakingToolError::EmptyGraphqlResponse)?;
    Ok(response_data.genesis_constants.genesis_timestamp)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/best_chain.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
struct BestChain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestTip(best_chain::BestChainBestChain);

impl BestTip {
    pub fn consensus_state(&self) -> &best_chain::BestChainBestChainProtocolStateConsensusState {
        &self.0.protocol_state.consensus_state
    }

    pub fn epoch_bounds(&self) -> ((u32, u32), (u32, u32)) {
        // TODO(adonagy): get the data from the node + unwrap
        const SLOTS_PER_EPOCH: u32 = 7140;
        let current_epoch = self.consensus_state().epoch.parse::<u32>().unwrap();
        let current_start = current_epoch * SLOTS_PER_EPOCH;
        let current_end = current_epoch * SLOTS_PER_EPOCH + SLOTS_PER_EPOCH - 1;

        let next_epoch = current_epoch + 1;
        let next_start = next_epoch * SLOTS_PER_EPOCH;
        let next_end = next_start + SLOTS_PER_EPOCH - 1;

        ((current_start, current_end), (next_start, next_end))
    }
}

impl From<best_chain::BestChainBestChain> for BestTip {
    fn from(value: best_chain::BestChainBestChain) -> Self {
        BestTip(value)
    }
}

impl From<BestTip> for best_chain::BestChainBestChain {
    fn from(value: BestTip) -> Self {
        value.0
    }
}

#[allow(dead_code)]
pub async fn get_best_chain() -> Result<Vec<String>, StakingToolError> {
    let client = Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .build()
        .unwrap();

    let variables = best_chain::Variables { max_length: 290 };
    let response_body =
        post_graphql::<BestChain, _>(&client, "http://adonagy.com:5000/graphql", variables)
            .await
            .unwrap();

    let response_data: best_chain::ResponseData = response_body
        .data
        .ok_or(StakingToolError::EmptyGraphqlResponse)?;

    response_data
        .best_chain
        .ok_or(StakingToolError::EmptyGraphqlResponse)
        .map(|v| v.iter().map(|bc| bc.state_hash.clone()).collect())
}

pub async fn get_best_tip() -> Result<BestTip, StakingToolError> {
    let client = Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .build()
        .unwrap();

    let variables = best_chain::Variables { max_length: 1 };
    let response_body =
        post_graphql::<BestChain, _>(&client, "http://adonagy.com:5000/graphql", variables)
            .await
            .unwrap();

    let response_data: best_chain::ResponseData = response_body
        .data
        .ok_or(StakingToolError::EmptyGraphqlResponse)?;

    response_data
        .best_chain
        .map(|res| res.first().cloned().unwrap().into())
        .ok_or(StakingToolError::EmptyGraphqlResponse)
}

pub fn dump_current_staking_ledger(current_epoch_number: u32) -> impl AsRef<[u8]> {
    // if !ledger_dir.exists() {
    //     fs::create_dir_all(ledger_dir.clone()).unwrap();
    // }

    let output = Command::new("mina")
        .args(["ledger", "export", "staking-epoch-ledger"])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        panic!("Command execution failed with error: {}", error_message);
    }

    output.stdout

    // let mut file = fs::File::create(format!("{}/{current_epoch_number}-staking-ledger", ledger_dir.display())).unwrap();

    // file.write_all(&output.stdout).unwrap();
}
