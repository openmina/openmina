use anyhow::Result;
use chrono::{DateTime, Utc};
use firestore::FirestoreDb;
use serde::Serialize;

use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::config::Config;
use crate::remote_db::BlockInfo;
use crate::remote_db::HeartbeatChunkState;
use crate::time::*;
use mina_tree::proofs::verification::verify_block;

#[derive(Debug)]
pub struct HeartbeatPresence {
    pub window_id: i64,
    pub public_key_id: i64,
    pub best_tip: BlockInfo,
    pub heartbeat_time: i64,
}

#[derive(Debug)]
pub struct ProducedBlock {
    pub window_id: i64,
    pub public_key_id: i64,
    pub block_hash: String,
    pub block_height: u32,
    pub block_global_slot: u32,
    pub block_data: String,
}

pub async fn get_last_processed_time(
    pool: &SqlitePool,
    config: Option<&Config>,
) -> Result<DateTime<Utc>> {
    let record = sqlx::query!("SELECT last_processed_time FROM processing_state WHERE id = 1")
        .fetch_one(pool)
        .await?;

    let db_time = from_unix_timestamp(record.last_processed_time);

    Ok(match config {
        Some(cfg) => db_time.max(cfg.window_range_start),
        None => db_time,
    })
}

pub async fn update_last_processed_time(pool: &SqlitePool, time: DateTime<Utc>) -> Result<()> {
    let current = get_last_processed_time(pool, None).await?;
    let ts = to_unix_timestamp(time);

    println!("Updating last processed time: {} -> {}", current, time);

    sqlx::query!(
        "UPDATE processing_state SET last_processed_time = ? WHERE id = 1",
        ts
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn ensure_time_windows(
    pool: &SqlitePool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<i64>> {
    let windows = generate_fixed_time_windows(start, end);
    let mut window_ids = Vec::new();

    for window in windows {
        let start_ts = to_unix_timestamp(window.start);
        let end_ts = to_unix_timestamp(window.end);

        // Try to get existing window ID first
        let existing_id = sqlx::query!(
            "SELECT id FROM time_windows WHERE start_time = ?1 AND end_time = ?2",
            start_ts,
            end_ts,
        )
        .fetch_optional(pool)
        .await?;

        let id = if let Some(record) = existing_id {
            record
                .id
                .expect("ID should not be None for an existing record")
        } else {
            sqlx::query!(
                "INSERT INTO time_windows (start_time, end_time) VALUES (?1, ?2) RETURNING id",
                start_ts,
                end_ts,
            )
            .fetch_one(pool)
            .await?
            .id
        };

        window_ids.push(id);
    }

    Ok(window_ids)
}

pub async fn ensure_public_keys(
    pool: &SqlitePool,
    public_keys: &[&str],
) -> Result<HashMap<String, i64>> {
    let mut map = HashMap::new();

    // Create a single query with multiple values
    let values = public_keys
        .iter()
        .map(|k| format!("('{}')", k))
        .collect::<Vec<_>>()
        .join(",");

    let query = format!(
        r#"
        INSERT INTO public_keys (public_key)
        VALUES {}
        ON CONFLICT (public_key) DO UPDATE SET
            public_key = excluded.public_key
        RETURNING id, public_key
        "#,
        values
    );

    let rows = sqlx::query(&query).fetch_all(pool).await?;

    for row in rows {
        let id: i64 = row.get("id");
        let key: String = row.get("public_key");
        map.insert(key, id);
    }

    Ok(map)
}

pub async fn batch_insert_presence(
    pool: &SqlitePool,
    presences: &[HeartbeatPresence],
) -> Result<()> {
    if presences.is_empty() {
        return Ok(());
    }

    let values = presences
        .iter()
        .map(|p| {
            format!(
                "({}, {}, '{}', {}, {}, {})",
                p.window_id,
                p.public_key_id,
                p.best_tip.hash,
                p.best_tip.height,
                p.best_tip.global_slot,
                p.heartbeat_time
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let query = format!(
        r#"
        INSERT INTO heartbeat_presence (
            window_id, public_key_id,
            best_tip_hash, best_tip_height, best_tip_global_slot,
            heartbeat_time
        )
        VALUES {}
        ON CONFLICT(window_id, public_key_id)
        DO UPDATE SET
            best_tip_hash = CASE
                WHEN excluded.best_tip_global_slot >= best_tip_global_slot
                THEN excluded.best_tip_hash
                ELSE best_tip_hash
            END,
            best_tip_height = CASE
                WHEN excluded.best_tip_global_slot >= best_tip_global_slot
                THEN excluded.best_tip_height
                ELSE best_tip_height
            END,
            best_tip_global_slot = CASE
                WHEN excluded.best_tip_global_slot >= best_tip_global_slot
                THEN excluded.best_tip_global_slot
                ELSE best_tip_global_slot
            END,
            heartbeat_time = CASE
                WHEN excluded.best_tip_global_slot >= best_tip_global_slot
                THEN excluded.heartbeat_time
                ELSE heartbeat_time
            END
        "#,
        values
    );

    sqlx::query(&query).execute(pool).await?;

    Ok(())
}

async fn batch_insert_produced_blocks(pool: &SqlitePool, blocks: &[ProducedBlock]) -> Result<()> {
    if blocks.is_empty() {
        return Ok(());
    }

    let values = blocks
        .iter()
        .map(|b| {
            format!(
                "({}, {}, '{}', {}, {}, '{}')",
                b.window_id,
                b.public_key_id,
                b.block_hash,
                b.block_height,
                b.block_global_slot,
                b.block_data.replace('\'', "''")
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let query = format!(
        r#"
        INSERT INTO produced_blocks (
            window_id, public_key_id,
            block_hash, block_height, block_global_slot,
            block_data_blob
        )
        VALUES {}
        ON CONFLICT(public_key_id, block_hash) DO NOTHING
        "#,
        values
    );

    sqlx::query(&query).execute(pool).await?;

    Ok(())
}

/// Marks heartbeat presence entries as outdated (disabled) based on global slot comparisons.
///
/// This function performs the following steps:
/// 1. Finds the maximum global slot for each window (considering only non-disabled entries).
/// 2. Identifies the previous window's maximum global slot for each window.
/// 3. Marks a presence entry as disabled if its global slot is less than:
///    - The maximum global slot of the previous window (if it exists).
///
/// This approach allows for a full window of tolerance in synchronization:
/// - Entries matching or exceeding the previous window's max slot are considered up-to-date.
/// - This allows for slight delays in propagation between windows.
///
/// Note: The first window in the sequence will not have any entries marked as disabled,
/// as there is no previous window to compare against.
///
/// Returns the number of presence entries marked as disabled.
async fn mark_outdated_presence(pool: &SqlitePool) -> Result<usize> {
    let affected = sqlx::query!(
        r#"
        WITH MaxSlots AS (
            SELECT
                window_id,
                MAX(best_tip_global_slot) as max_slot
            FROM heartbeat_presence
            WHERE disabled = FALSE
            GROUP BY window_id
        ),
        PrevMaxSlots AS (
            -- Get the max slot from the immediate previous window
            SELECT
                tw.id as window_id,
                prev.max_slot as prev_max_slot
            FROM time_windows tw
            LEFT JOIN time_windows prev_tw ON prev_tw.id = tw.id - 1
            LEFT JOIN MaxSlots prev ON prev.window_id = prev_tw.id
        )
        UPDATE heartbeat_presence
        SET disabled = TRUE
        WHERE (window_id, best_tip_global_slot) IN (
            SELECT
                hp.window_id,
                hp.best_tip_global_slot
            FROM heartbeat_presence hp
            JOIN PrevMaxSlots pms ON pms.window_id = hp.window_id
            WHERE hp.disabled = FALSE
            AND pms.prev_max_slot IS NOT NULL  -- Ensure there is a previous window
            AND hp.best_tip_global_slot < pms.prev_max_slot  -- Less than previous window max
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(affected.rows_affected() as usize)
}

pub async fn process_heartbeats(
    db: &FirestoreDb,
    pool: &SqlitePool,
    config: &Config,
) -> Result<usize> {
    let last_processed_time = get_last_processed_time(pool, Some(config)).await?;
    let now = Utc::now();
    // Don't fetch heartbeats beyond window range end
    let end_time = config.window_range_end.min(now);

    let mut total_heartbeats = 0;
    let mut latest_time = last_processed_time;
    let mut seen_blocks: HashMap<(i64, String), DateTime<Utc>> = HashMap::new();

    // Statistics
    let mut total_presence_count = 0;
    let mut total_skipped_count = 0;
    let mut total_blocks_recorded = 0;
    let mut total_blocks_duplicate = 0;
    let mut total_outside_windows = 0;

    let mut chunk_state = HeartbeatChunkState {
        chunk_start: last_processed_time,
        last_timestamp: None,
    };

    // Its ok to call these functions multiple times because the result is cached
    let verifier_index = snark::BlockVerifier::make();
    let verifier_srs = snark::get_srs();

    loop {
        let heartbeats =
            crate::remote_db::fetch_heartbeat_chunk(db, &mut chunk_state, end_time).await?;
        if heartbeats.is_empty() {
            break;
        }

        total_heartbeats += heartbeats.len();
        println!("Processing batch of {} heartbeats...", heartbeats.len());

        latest_time = latest_time.max(
            heartbeats
                .iter()
                .map(|h| h.create_time)
                .max()
                .unwrap_or(latest_time),
        );

        let start_ts = to_unix_timestamp(last_processed_time);
        let end_ts = to_unix_timestamp(latest_time);

        let existing_windows = sqlx::query!(
            r#"
            SELECT id, start_time, end_time
            FROM time_windows
            WHERE start_time <= ?2 AND end_time >= ?1 AND disabled = FALSE
            ORDER BY start_time ASC
            "#,
            start_ts,
            end_ts
        )
        .fetch_all(pool)
        .await?;

        let unique_submitters: HashSet<&str> = heartbeats
            .iter()
            .map(|entry| entry.submitter.as_str())
            .collect();

        let public_key_map =
            ensure_public_keys(pool, &unique_submitters.into_iter().collect::<Vec<_>>()).await?;

        let mut presence_count = 0;
        let mut skipped_count = 0;
        let mut blocks_recorded = 0;
        let mut blocks_duplicate = 0;
        let mut processed_heartbeats = HashSet::new();
        let mut produced_blocks_batch = Vec::new();

        for window in existing_windows {
            let window_start = from_unix_timestamp(window.start_time);
            let window_end = from_unix_timestamp(window.end_time);
            let mut presence_batch = Vec::new();

            for (idx, entry) in heartbeats.iter().enumerate() {
                if entry.create_time >= window_start && entry.create_time < window_end {
                    processed_heartbeats.insert(idx);

                    let best_tip = entry.best_tip_block();
                    let public_key_id = *public_key_map.get(&entry.submitter).unwrap();

                    // Record presence only if node is synced and has a best tip
                    if entry.is_synced() && best_tip.is_some() {
                        presence_batch.push(HeartbeatPresence {
                            window_id: window.id.unwrap(),
                            public_key_id,
                            best_tip: best_tip.unwrap(), // Cannot fail due to the above check
                            heartbeat_time: to_unix_timestamp(entry.create_time),
                        });
                        presence_count += 1;
                    } else {
                        skipped_count += 1;
                    }

                    // Process produced blocks regardless of sync status
                    match entry
                        .last_produced_block_info()
                        .map(|bi| (bi.clone(), bi.block_header_decoded()))
                    {
                        None => (), // No block to process
                        Some((block_info, Ok(block_header))) => {
                            let key = (public_key_id, block_info.hash.clone());

                            if let Some(first_seen) = seen_blocks.get(&key) {
                                blocks_duplicate += 1;
                                println!(
                                    "Duplicate block detected: {} (height: {}, producer: {}, peer_id: {}) [first seen at {}, now at {}]",
                                    key.1,
                                    block_info.height,
                                    entry.submitter,
                                    entry.peer_id().unwrap_or_else(|| "unknown".to_string()),
                                    first_seen,
                                    entry.create_time
                                );
                                continue;
                            }

                            // Verify that the block slot matches the expected one for the current time
                            // TODO: maybe we can be a bit more lenient here?
                            let expected_slot = global_slot_at_time(entry.create_time);
                            if block_info.global_slot != expected_slot {
                                println!(
                                    "WARNING: Invalid block slot: {} (height: {}, producer: {}, expected slot: {}, actual slot: {})",
                                    block_info.hash, block_info.height, entry.submitter, expected_slot, block_info.global_slot
                                );
                                continue;
                            }

                            // Verify block proof
                            if !verify_block(&block_header, &verifier_index, &verifier_srs) {
                                println!(
                                    "WARNING: Invalid block proof: {} (height: {}, producer: {})",
                                    block_info.hash, block_info.height, entry.submitter
                                );
                                continue;
                            }

                            seen_blocks.insert(key.clone(), entry.create_time);
                            produced_blocks_batch.push(ProducedBlock {
                                window_id: window.id.unwrap(),
                                public_key_id,
                                block_hash: block_info.hash,
                                block_height: block_info.height,
                                block_global_slot: block_info.global_slot,
                                block_data: block_info.base64_encoded_header,
                            });
                        }
                        Some((_block_info, Err(e))) => {
                            println!(
                                "WARNING: Failed to decode block from {}: {}",
                                entry.submitter, e
                            )
                        }
                    }
                }
            }

            if !presence_batch.is_empty() {
                batch_insert_presence(pool, &presence_batch).await?;
            }
        }

        if !produced_blocks_batch.is_empty() {
            blocks_recorded = produced_blocks_batch.len();
            batch_insert_produced_blocks(pool, &produced_blocks_batch).await?;
        }

        let outside_windows = heartbeats.len() - processed_heartbeats.len();

        println!(
            "Batch complete: {} presences, {} blocks ({} duplicates), {} skipped, {} outside windows",
            presence_count,
            blocks_recorded,
            blocks_duplicate,
            skipped_count,
            outside_windows
        );

        total_presence_count += presence_count;
        total_skipped_count += skipped_count;
        total_blocks_recorded += blocks_recorded;
        total_blocks_duplicate += blocks_duplicate;
        total_outside_windows += outside_windows;
    }

    println!(
        "Processed {} total heartbeats ({} synced presences recorded, {} unique blocks recorded ({} duplicates skipped), {} unsynced skipped), {} outside of defined windows",
        total_heartbeats,
        total_presence_count,
        total_blocks_recorded,
        total_blocks_duplicate,
        total_skipped_count,
        total_outside_windows,
    );

    if latest_time > last_processed_time {
        update_last_processed_time(pool, latest_time).await?;

        // Mark outdated presence entries as disabled
        let disabled_count = mark_outdated_presence(pool).await?;
        if disabled_count > 0 {
            println!(
                "Marked {} outdated presence entries as disabled",
                disabled_count
            );
        }
    }

    Ok(total_heartbeats)
}

pub async fn create_tables_from_file(pool: &SqlitePool) -> Result<()> {
    println!("Initializing SQLite database schema...");
    let schema = fs::read_to_string("schema.sql")?;
    sqlx::query(&schema).execute(pool).await?;
    Ok(())
}

pub async fn toggle_windows(
    pool: &SqlitePool,
    start: String,
    end: String,
    disabled: bool,
) -> Result<()> {
    let start_time = parse_datetime(&start)?;
    let end_time = parse_datetime(&end)?;

    if start_time >= end_time {
        return Err(anyhow::anyhow!("Start time must be before end time"));
    }

    let start_ts = to_unix_timestamp(start_time);
    let end_ts = to_unix_timestamp(end_time);

    let affected = sqlx::query!(
        r#"
        UPDATE time_windows
        SET disabled = ?1
        WHERE start_time >= ?2 AND end_time < ?3
        "#,
        disabled,
        start_ts,
        end_ts
    )
    .execute(pool)
    .await?;

    if affected.rows_affected() > 0 {
        println!(
            "{} windows {} successfully between {} and {}",
            affected.rows_affected(),
            if disabled { "disabled" } else { "enabled" },
            start_time,
            end_time
        );
    } else {
        println!("No windows found in the specified range");
    }
    Ok(())
}

// TODO: take into account the validated flag to count blocks
pub async fn update_scores(pool: &SqlitePool, config: &Config) -> Result<()> {
    let window_start = to_unix_timestamp(config.window_range_start);
    let current_time = chrono::Utc::now().timestamp();

    sqlx::query!(
        r#"
        WITH ValidWindows AS (
            SELECT id, start_time, end_time
            FROM time_windows
            WHERE disabled = FALSE
            AND end_time <= ?2
            AND start_time >= ?1
        ),
        BlockCounts AS (
            -- Count one block per global slot per producer
            SELECT
                public_key_id,
                COUNT(DISTINCT block_global_slot) as blocks
            FROM (
                -- Deduplicate blocks per global slot
                SELECT
                    pb.public_key_id,
                    pb.block_global_slot
                FROM produced_blocks pb
                JOIN ValidWindows vw ON vw.id = pb.window_id
                -- TODO: enable once block proof validation has been implemented
                -- WHERE pb.validated = TRUE
                GROUP BY pb.public_key_id, pb.block_global_slot
            ) unique_blocks
            GROUP BY public_key_id
        ),
        HeartbeatCounts AS (
            -- Count heartbeats only within valid windows and not disabled
            SELECT
                hp.public_key_id,
                COUNT(DISTINCT hp.window_id) as heartbeats
            FROM heartbeat_presence hp
            JOIN ValidWindows vw ON vw.id = hp.window_id
            WHERE hp.disabled = FALSE
            GROUP BY hp.public_key_id
        ),
        LastHeartbeats AS (
            -- Get last heartbeat time across all windows, including disabled entries
            SELECT
                public_key_id,
                MAX(heartbeat_time) as last_heartbeat
            FROM heartbeat_presence
            GROUP BY public_key_id
        )
        INSERT INTO submitter_scores (
            public_key_id,
            score,
            blocks_produced,
            last_heartbeat
        )
        SELECT
            pk.id,
            COALESCE(hc.heartbeats, 0) as score,
            COALESCE(bc.blocks, 0) as blocks_produced,
            COALESCE(lh.last_heartbeat, 0) as last_heartbeat
        FROM public_keys pk
        LEFT JOIN HeartbeatCounts hc ON hc.public_key_id = pk.id
        LEFT JOIN BlockCounts bc ON bc.public_key_id = pk.id
        LEFT JOIN LastHeartbeats lh ON lh.public_key_id = pk.id
        WHERE hc.heartbeats > 0 OR bc.blocks > 0
        ON CONFLICT(public_key_id) DO UPDATE SET
            score = excluded.score,
            blocks_produced = excluded.blocks_produced,
            last_heartbeat = excluded.last_heartbeat,
            last_updated = strftime('%s', 'now')
        "#,
        window_start,
        current_time
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct MaxScores {
    pub total: i64,
    pub current: i64,
}

pub async fn get_max_scores(pool: &SqlitePool) -> Result<MaxScores> {
    let total = sqlx::query!("SELECT COUNT(*) as count FROM time_windows WHERE disabled = FALSE")
        .fetch_one(pool)
        .await?
        .count as i64;

    let current = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*) as count 
        FROM time_windows 
        WHERE end_time <= strftime('%s', 'now')
        AND disabled = FALSE
        "#,
    )
    .fetch_one(pool)
    .await?
    .0;

    Ok(MaxScores { total, current })
}

pub async fn view_scores(pool: &SqlitePool, config: &Config) -> Result<()> {
    // Make sure scores are up to date
    update_scores(pool, config).await?;

    let scores = sqlx::query!(
        r#"
        SELECT
            pk.public_key,
            ss.score,
            ss.blocks_produced,
            datetime(ss.last_updated, 'unixepoch') as last_updated,
            datetime(ss.last_heartbeat, 'unixepoch') as last_heartbeat
        FROM submitter_scores ss
        JOIN public_keys pk ON pk.id = ss.public_key_id
        ORDER BY ss.score DESC, ss.blocks_produced DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let max_scores = get_max_scores(pool).await?;

    println!("\nSubmitter Scores:");
    println!("--------------------------------------------------------");
    println!(
        "Public Key                                              | Score | Blocks | Current Max | Total Max | Last Updated | Last Heartbeat"
    );
    println!("--------------------------------------------------------");

    for row in scores {
        println!(
            "{:<40} | {:>5} | {:>6} | {:>11} | {:>9} | {} | {}",
            row.public_key,
            row.score,
            row.blocks_produced,
            max_scores.current,
            max_scores.total,
            row.last_updated.unwrap_or_default(),
            row.last_heartbeat.unwrap_or_default()
        );
    }

    Ok(())
}

pub fn ensure_db_exists(db_path: &str) -> Result<()> {
    let file_path = db_path.strip_prefix("sqlite:").unwrap_or(db_path);

    if !std::path::Path::new(file_path).exists() {
        std::fs::File::create(file_path)?;
    }

    Ok(())
}

pub async fn set_last_processed_time(pool: &SqlitePool, time_str: &str) -> Result<()> {
    // Try parsing with different formats
    let dt = if let Ok(dt) = DateTime::parse_from_str(
        &format!("{} 00:00:00 +0000", time_str),
        "%Y-%m-%d %H:%M:%S %z",
    ) {
        dt.with_timezone(&Utc)
    } else if let Ok(dt) =
        DateTime::parse_from_str(&format!("{} +0000", time_str), "%Y-%m-%d %H:%M:%S %z")
    {
        dt.with_timezone(&Utc)
    } else {
        return Err(anyhow::anyhow!(
            "Invalid time format. Expected YYYY-MM-DD or YYYY-MM-DD HH:MM:SS"
        ));
    };

    let ts = to_unix_timestamp(dt);
    sqlx::query!(
        "UPDATE processing_state SET last_processed_time = ? WHERE id = 1",
        ts
    )
    .execute(pool)
    .await?;

    println!("Last processed time set to: {}", dt);
    Ok(())
}

pub async fn create_windows(pool: &SqlitePool, start: String, end: String) -> Result<()> {
    let start_time = parse_datetime(&start)?;
    let end_time = parse_datetime(&end)?;

    if start_time >= end_time {
        return Err(anyhow::anyhow!("Start time must be before end time"));
    }

    let window_ids = ensure_time_windows(pool, start_time, end_time).await?;
    println!("Created {} time windows", window_ids.len());
    Ok(())
}

/// Ensures time windows exist in the database for a configured time range.
///
/// This function uses environment variables to determine the range of windows to create:
/// - `WINDOW_RANGE_START`: The start time for window creation (RFC3339 format)
///   If not set, defaults to the current time
/// - `WINDOW_RANGE_END`: The end time for window creation (RFC3339 format)
///   If not set, defaults to start + 28 days
///
/// Time windows are created at 5-minute intervals within this range.
/// Windows that already exist will be preserved, new ones will be created.
/// Any windows outside this range will be disabled.
pub async fn ensure_initial_windows(pool: &SqlitePool, config: &Config) -> Result<()> {
    let start = config.window_range_start;
    let end = config.window_range_end;

    println!("Ensuring time windows exist from {} to {}", start, end);
    let window_ids = ensure_time_windows(pool, start, end).await?;
    println!("Created/verified {} time windows", window_ids.len());

    // Disable windows outside the configured range
    let start_ts = to_unix_timestamp(start);
    let end_ts = to_unix_timestamp(end);

    let affected = sqlx::query!(
        r#"
        UPDATE time_windows 
        SET disabled = TRUE 
        WHERE (start_time < ?1 OR end_time > ?2) 
        AND disabled = FALSE
        "#,
        start_ts,
        end_ts
    )
    .execute(pool)
    .await?;

    if affected.rows_affected() > 0 {
        println!(
            "Disabled {} windows outside the configured range",
            affected.rows_affected()
        );
    }

    Ok(())
}

pub async fn mark_disabled_windows(pool: &SqlitePool, config: &Config) -> Result<()> {
    if !config.disabled_windows.is_empty() {
        println!("Processing disabled window ranges:");
        let mut affected_total = 0;

        for (start, end) in &config.disabled_windows {
            println!("  {} to {}", start, end);
            let start_ts = to_unix_timestamp(*start);
            let end_ts = to_unix_timestamp(*end);

            let result = sqlx::query!(
                r#"
                UPDATE time_windows
                SET disabled = TRUE
                WHERE start_time >= ? AND end_time <= ?
                "#,
                start_ts,
                end_ts
            )
            .execute(pool)
            .await?;

            affected_total += result.rows_affected();
        }

        if affected_total > 0 {
            println!("✓ Disabled {} windows in the above ranges", affected_total);
        } else {
            println!("! No windows found in the configured disabled ranges");
        }
    }
    Ok(())
}

fn global_slot_at_time(time: DateTime<Utc>) -> u32 {
    use chrono::FixedOffset;
    let slot_duration = 180_000;
    let genesis_state_timestamp =
        DateTime::<FixedOffset>::parse_from_rfc3339("2024-04-09T21:00:00Z")
            .unwrap()
            .to_utc();
    let slot_start_ms = genesis_state_timestamp.timestamp_millis() as u64;
    let time_ms = time.timestamp_millis() as u64;

    let slot_diff = (time_ms - slot_start_ms) / slot_duration;
    slot_diff as u32
}
