use std::collections::BTreeSet;

use graphql_client::{reqwest::post_graphql, GraphQLQuery};

use producer_dashboard::evaluator::epoch::{SlotData, SlotStatus};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/graphql/explorer_schema.json",
    query_path = "tests/graphql/explorer_blocks_produced.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CanonicalBlocksProduced;

#[tokio::test]
async fn test_comapre_to_explorer() {
    let client = reqwest::Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .build()
        .unwrap();

    let variables = canonical_blocks_produced::Variables {
        limit: 7140,
        epoch: 10,
        pk: "B62qpfgnUm7zVqi8MJHNB2m37rtgMNDbFNhC2DpMmmVpQt8x6gKv9Ww".to_string(),
    };

    let response_body = post_graphql::<CanonicalBlocksProduced, _>(
        &client,
        "https://devnet.graphql.minaexplorer.com/",
        variables,
    )
    .await
    .unwrap();

    let response_data: canonical_blocks_produced::ResponseData = response_body.data.unwrap();

    let explorer_blocks = response_data
        .blocks
        .into_iter()
        .filter_map(|v| {
            if let Some(v) = v {
                Some(v.state_hash.unwrap())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    let dash_data: Vec<SlotData> = reqwest::get(
        "http://65.109.105.40:3000/epoch/10",
        // "https://staging-devnet-openmina-bp-0-dashboard.minaprotocol.network/epoch/10",
    )
    .await
    .unwrap()
    .json()
    .await
    .unwrap();

    let dash_blocks = dash_data
        .into_iter()
        .filter_map(|data| {
            if matches!(
                data.block_status(),
                SlotStatus::Canonical | SlotStatus::CanonicalPending
            ) {
                data.block()
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    for e_block in &explorer_blocks {
        if !dash_blocks.contains(e_block) {
            println!("Missing from dash: {}", e_block);
        }
    }

    assert_eq!(explorer_blocks.len(), dash_blocks.len());

    // assert_eq!(explorer_blocks, dash_blocks);
}
