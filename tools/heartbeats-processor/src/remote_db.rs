use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use firestore::*;
use mina_p2p_messages::v2;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;

const FIRESTORE_BATCH_SIZE: u32 = 1000; // Number of documents per batch
const MAX_TIME_CHUNK_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureJson {
    pub field: String,
    pub scalar: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatEntry {
    pub version: u8,
    pub payload: String,
    pub submitter: String,
    pub signature: SignatureJson,
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    pub decoded_payload: Option<Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProducedBlockInfo {
    pub height: u32,
    pub global_slot: u32,
    pub hash: String,
    pub base64_encoded_header: String,
}

#[derive(Debug)]
pub struct BlockInfo {
    pub hash: String,
    pub height: u64,
    pub global_slot: u64,
}

impl ProducedBlockInfo {
    pub fn block_header_decoded(&self) -> Result<v2::MinaBlockHeaderStableV2, String> {
        use base64::{engine::general_purpose::URL_SAFE, Engine as _};
        use mina_p2p_messages::binprot::BinProtRead;

        let decoded = URL_SAFE
            .decode(&self.base64_encoded_header)
            .map_err(|_| "Could not decode base64".to_string())?;
        let block_header = v2::MinaBlockHeaderStableV2::binprot_read(&mut &decoded[..])
            .map_err(|e| format!("Could not decode block header: {:?}", e))?;

        Ok(block_header)
    }
}

impl HeartbeatEntry {
    pub fn decode_payload(&mut self) -> Result<(), anyhow::Error> {
        let decoded = general_purpose::URL_SAFE.decode(&self.payload)?;
        let json_str = String::from_utf8(decoded)?;
        self.decoded_payload = Some(serde_json::from_str(&json_str)?);
        Ok(())
    }

    pub fn peer_id(&self) -> Option<String> {
        self.decoded_payload
            .as_ref()
            .and_then(|decoded| decoded.get("peer_id"))
            .and_then(|peer_id| peer_id.as_str())
            .map(|s| s.to_string())
    }

    pub fn last_produced_block_info(&self) -> Option<ProducedBlockInfo> {
        let result = self
            .decoded_payload
            .as_ref()
            .and_then(|status| status.get("last_produced_block_info"))
            .filter(|v| !v.is_null())
            .map(|block_info| serde_json::from_value(block_info.clone()))?;

        match result {
            Ok(info) => Some(info),
            Err(e) => {
                eprintln!("Invalid block header: {:?}", e);
                None
            }
        }
    }

    fn transition_frontier(&self) -> Option<&Value> {
        self.decoded_payload
            .as_ref()
            .and_then(|decoded| decoded.get("status"))
            .and_then(|status| status.get("transition_frontier"))
    }

    fn best_tip(&self) -> Option<&Value> {
        self.transition_frontier()
            .and_then(|tf| tf.get("best_tip"))
            .filter(|v| !v.is_null())
    }

    pub fn best_tip_block(&self) -> Option<BlockInfo> {
        self.best_tip().map(|best_tip| BlockInfo {
            hash: best_tip.get("hash").unwrap().as_str().unwrap().to_string(),
            height: best_tip.get("height").unwrap().as_u64().unwrap(),
            global_slot: best_tip.get("global_slot").unwrap().as_u64().unwrap(),
        })
    }

    #[allow(dead_code)]
    pub fn sync_status(&self) -> Option<String> {
        self.transition_frontier()
            .and_then(|tf| tf.get("sync"))
            .and_then(|sync| sync.get("status"))
            .map(|status| status.as_str().unwrap().to_string())
    }

    pub fn sync_phase(&self) -> Option<String> {
        self.transition_frontier()
            .and_then(|tf| tf.get("sync"))
            .and_then(|sync| sync.get("phase"))
            .map(|phase| phase.as_str().unwrap().to_string())
    }

    pub fn is_synced(&self) -> bool {
        self.sync_phase()
            .as_ref()
            .map(|status| status == "Synced")
            .unwrap_or(false)
    }

    pub fn is_catchup(&self) -> bool {
        self.sync_phase()
            .as_ref()
            .map(|status| status == "Catchup")
            .unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreDocument {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub score: i64,
    #[serde(rename = "blocksProduced")]
    pub blocks_produced: i64,
    #[serde(rename = "lastUpdated")]
    pub last_updated: i64,
    #[serde(rename = "lastHeartbeat")]
    pub last_heartbeat: i64,
}

pub async fn get_db(config: &Config) -> Result<FirestoreDb> {
    if let Some(emulator_host) = &config.firestore_emulator_host {
        // Using emulator
        std::env::set_var("GOOGLE_CLOUD_PROJECT", "staging");
        let emulator_url = format!("http://{}", emulator_host);
        let token_source = gcloud_sdk::TokenSourceType::Default;
        Ok(FirestoreDb::with_options_token_source(
            FirestoreDbOptions::new("staging".to_string()).with_firebase_api_url(emulator_url),
            vec!["http://127.0.0.1:9099".to_string()],
            token_source,
        )
        .await?)
    } else {
        // Production mode - requires auth
        Ok(FirestoreDb::new(&config.google_cloud_project).await?)
    }
}

pub struct HeartbeatChunkState {
    pub chunk_start: DateTime<Utc>,
    pub last_timestamp: Option<DateTime<Utc>>,
}

pub async fn fetch_heartbeat_chunk(
    db: &FirestoreDb,
    state: &mut HeartbeatChunkState,
    end_time: DateTime<Utc>,
) -> Result<Vec<HeartbeatEntry>> {
    let chunk_duration = Duration::try_hours(MAX_TIME_CHUNK_HOURS).unwrap();
    let chunk_end = (state.chunk_start + chunk_duration).min(end_time);

    if state.chunk_start >= end_time {
        println!("Reached end of testing window: {}", end_time);
        return Ok(Vec::new());
    }

    println!(
        "Fetching heartbeat chunk... {} to {}",
        state.chunk_start, chunk_end
    );

    let query = db
        .fluent()
        .select()
        .from("heartbeats")
        .filter(|q| {
            let mut conditions = vec![
                q.field("createTime")
                    .greater_than_or_equal(firestore::FirestoreTimestamp::from(state.chunk_start)),
                q.field("createTime")
                    .less_than(firestore::FirestoreTimestamp::from(chunk_end)),
            ];

            if let Some(ts) = &state.last_timestamp {
                conditions.push(
                    q.field("createTime")
                        .greater_than(firestore::FirestoreTimestamp::from(*ts)),
                );
            }

            q.for_all(conditions)
        })
        .order_by([
            ("createTime", FirestoreQueryDirection::Ascending),
            ("__name__", FirestoreQueryDirection::Ascending),
        ])
        .limit(FIRESTORE_BATCH_SIZE);

    let mut batch: Vec<HeartbeatEntry> = query.obj().query().await?;

    if batch.is_empty() {
        state.chunk_start = chunk_end;
        state.last_timestamp = None;
    } else {
        state.last_timestamp = batch.last().map(|doc| doc.create_time);
        if batch.len() < FIRESTORE_BATCH_SIZE as usize {
            state.chunk_start = chunk_end;
            state.last_timestamp = None;
        }
    }

    // Decode payloads
    for heartbeat in &mut batch {
        if let Err(e) = heartbeat.decode_payload() {
            eprintln!("Failed to decode payload: {:?}", e);
        }
    }

    Ok(batch)
}

pub async fn post_scores(
    db: &FirestoreDb,
    scores: Vec<ScoreDocument>,
    max_scores: (i64, i64),
) -> Result<()> {
    let scores_count = scores.len();
    let now = FirestoreTimestamp::from(Utc::now());
    let (current_max, total_max) = max_scores;

    let mut transaction = db.begin_transaction().await?;

    // Store max scores in separate documents
    db.fluent()
        .update()
        .in_col("maxScore")
        .document_id("current")
        .object(&serde_json::json!({
            "value": current_max,
            "lastUpdated": now,
        }))
        .add_to_transaction(&mut transaction)?;

    db.fluent()
        .update()
        .in_col("maxScore")
        .document_id("total")
        .object(&serde_json::json!({
            "value": total_max,
            "lastUpdated": now,
        }))
        .add_to_transaction(&mut transaction)?;

    // Per-key scores
    for doc in scores {
        db.fluent()
            .update()
            .in_col("scores")
            .document_id(&doc.public_key)
            .object(&doc)
            .add_to_transaction(&mut transaction)?;
    }

    println!(
        "Successfully posted {scores_count} scores and max scores (current: {}, total: {}) to Firestore",
        current_max, total_max
    );

    transaction.commit().await?;

    Ok(())
}
