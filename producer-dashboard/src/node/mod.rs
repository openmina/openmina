use ::reqwest::Client;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use num_bigint::BigInt;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

use crate::StakingToolError;

type PublicKey = String;
type StateHash = String;
type FeeTransferType = String;
type UserCommandKind = String;
type Amount = StringNumber;
type Fee = StringNumber;
type Epoch = String;
type Length = String;
type EpochSeed = String;

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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/best_chain.graphql",
    response_derives = "Debug, Clone"
)]
struct BestChain;

#[derive(Debug, Clone)]
pub struct BestTip(best_chain::BestChainBestChain);

impl BestTip {
    pub fn consensus_state(&self) -> &best_chain::BestChainBestChainProtocolStateConsensusState {
        &self.0.protocol_state.consensus_state
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
pub async fn get_best_chain() -> Result<Vec<best_chain::BestChainBestChain>, StakingToolError> {
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
