use ::reqwest::Client;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use num_bigint::BigInt;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeSet, process::Command, str::FromStr};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
    evaluator::epoch::{RawGlobalSlot, RawSlot},
    StakingToolError,
};

pub mod epoch_ledgers;
pub mod watchdog;

use self::{daemon_status::SyncStatus, epoch_ledgers::Ledger};

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

pub fn calc_slot_timestamp(genesis_timestamp: i64, global_slot: u32) -> i64 {
    genesis_timestamp + ((global_slot as i64) * 60 * 3)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    graphql_url: String,
    client_url: String,
}

impl Node {
    pub fn new(graphql_url: String, client_url: String) -> Self {
        // If we specify the url as http://<IP>:<PORT> we need to strip http:// as the client handler code expexts <IP>:<PORT>
        let client_url = client_url.strip_prefix("http://").unwrap_or(&client_url);
        Self {
            graphql_url,
            client_url: client_url.to_string(),
        }
    }

    pub async fn wait_for_graphql(&self) -> Result<(), StakingToolError> {
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .build()
            .unwrap();

        let timeout_duration = tokio::time::Duration::from_secs(120); // 2 minutes
        let start_time = tokio::time::Instant::now();

        while tokio::time::Instant::now() - start_time < timeout_duration {
            match client.get(&self.graphql_url).send().await {
                Ok(response) => {
                    println!("[wait_for_graphql] Response status: {}", response.status());
                    if response.status().is_client_error() {
                        return Ok(()); // URL is reachable and returns a successful status
                    }
                }
                Err(_) => {
                    println!("Waiting for node...");
                }
            }
            // Wait for some time before the next retry
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Err(StakingToolError::NodeOffline)
    }

    pub async fn sync_status(&self) -> SyncStatus {
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .build()
            .unwrap();

        let variables = daemon_status::Variables {};

        let response_body = post_graphql::<DaemonStatus, _>(&client, &self.graphql_url, variables)
            .await
            .unwrap();

        let response_data: daemon_status::ResponseData = response_body
            .data
            .ok_or(StakingToolError::EmptyGraphqlResponse)
            .unwrap();

        response_data.daemon_status.sync_status
    }

    pub async fn get_genesis_timestmap(&self) -> Result<i64, StakingToolError> {
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .build()
            .unwrap();

        let variables = genesis_timestamp::Variables {};

        let response_body =
            post_graphql::<GenesisTimestamp, _>(&client, &self.graphql_url, variables)
                .await
                .unwrap();
        let response_data: genesis_timestamp::ResponseData = response_body
            .data
            .ok_or(StakingToolError::EmptyGraphqlResponse)?;

        let timestamp_formatted = response_data.genesis_constants.genesis_timestamp;
        let datetime = OffsetDateTime::parse(&timestamp_formatted, &Rfc3339).unwrap();
        Ok(datetime.unix_timestamp())
    }

    pub async fn get_genesis_slot_since_genesis(&self) -> Result<u32, StakingToolError> {
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .build()
            .unwrap();

        let variables = genesis_block::Variables {};

        let response_body = post_graphql::<GenesisBlock, _>(&client, &self.graphql_url, variables)
            .await
            .unwrap();

        let response_data: genesis_block::ResponseData = response_body
            .data
            .ok_or(StakingToolError::EmptyGraphqlResponse)?;

        let slot_since_genesis = response_data
            .genesis_block
            .protocol_state
            .consensus_state
            .slot_since_genesis;
        Ok(slot_since_genesis.parse()?)
    }

    #[allow(dead_code)]
    pub async fn get_best_chain(&self) -> Result<Vec<(u32, String)>, StakingToolError> {
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .build()
            .unwrap();

        let variables = best_chain::Variables { max_length: 290 };
        let response_body = post_graphql::<BestChain, _>(&client, &self.graphql_url, variables)
            .await
            .unwrap();

        let response_data: best_chain::ResponseData = response_body
            .data
            .ok_or(StakingToolError::EmptyGraphqlResponse)?;

        response_data
            .best_chain
            .ok_or(StakingToolError::EmptyGraphqlResponse)
            .map(|v| {
                v.iter()
                    .map(|bc| {
                        (
                            bc.protocol_state
                                .consensus_state
                                .slot_since_genesis
                                .parse()
                                .unwrap(),
                            bc.state_hash.clone(),
                        )
                    })
                    .collect()
            })
    }

    pub async fn get_best_tip(&self) -> Result<BestTip, StakingToolError> {
        let client = Client::builder()
            .user_agent("graphql-rust/0.10.0")
            .build()
            .unwrap();

        let variables = best_chain::Variables { max_length: 1 };
        let response_body = post_graphql::<BestChain, _>(&client, &self.graphql_url, variables)
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

    fn dump_current_staking_ledger(&self) -> impl AsRef<[u8]> {
        let output = Command::new("mina")
            .args([
                "ledger",
                "export",
                "--daemon-port",
                &self.client_url,
                "staking-epoch-ledger",
            ])
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            panic!("Command execution failed with error: {}", error_message);
        }

        output.stdout
    }

    pub fn get_staking_ledger(&self, _epoch_number: u32) -> Ledger {
        let raw = self.dump_current_staking_ledger();
        let inner = serde_json::from_slice(raw.as_ref()).unwrap();
        Ledger::new(inner)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    best_tip: Option<BestTip>,
    best_chain: Vec<(u32, String)>,
    sync_status: SyncStatus,
    dumped_ledgers: BTreeSet<u32>,
    // TODO: make sure it's available, make it an option
    genesis_timestamp: i64,
    genesis_slot_since_genesis: u32,
}

impl Default for NodeData {
    fn default() -> Self {
        Self {
            best_tip: Default::default(),
            best_chain: Default::default(),
            sync_status: SyncStatus::OFFLINE,
            dumped_ledgers: Default::default(),
            genesis_timestamp: 0,
            genesis_slot_since_genesis: 0,
        }
    }
}

impl NodeData {
    // TODO(adonagy): Hydrate from db
    // pub fn new()

    pub fn transition_frontier_root(&self) -> Option<u32> {
        self.best_chain.first().map(|v| v.0)
    }

    pub fn best_tip(&self) -> Option<BestTip> {
        // TODO
        self.best_tip.clone()
    }

    pub fn to_global_slot_since_genesis(&self, global_slot: u32) -> u32 {
        global_slot + self.genesis_slot_since_genesis
    }

    pub fn current_slot(&self) -> MinaSlot {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let elapsed = now - self.genesis_timestamp;

        let slot = (elapsed / (3 * 60)) as u32;
        MinaSlot::new(slot, self.genesis_slot_since_genesis)
    }

    pub fn best_chain(&self) -> &[(u32, String)] {
        self.best_chain.as_slice()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinaSlot {
    slot: RawSlot,
    global_slot: RawGlobalSlot,
    global_slot_since_genesis: u32,
}

impl MinaSlot {
    pub fn new(global_slot: u32, genesis_slot_since_genesis: u32) -> Self {
        let global_slot_since_genesis = global_slot + genesis_slot_since_genesis;
        let global_slot: RawGlobalSlot = global_slot.into();

        Self {
            global_slot: global_slot.clone(),
            slot: global_slot.into(),
            global_slot_since_genesis,
        }
    }

    pub fn global_slot(&self) -> RawGlobalSlot {
        self.global_slot.clone()
    }
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/genesis_timestamp.graphql",
    response_derives = "Debug, Clone"
)]
struct GenesisTimestamp;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/best_chain.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
struct BestChain;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/genesis_block.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
struct GenesisBlock;

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

    pub fn height(&self) -> u32 {
        self.consensus_state().block_height.parse().unwrap()
    }

    pub fn epoch(&self) -> u32 {
        self.consensus_state().epoch.parse().unwrap()
    }

    pub fn state_hash(&self) -> String {
        self.0.state_hash.clone()
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
