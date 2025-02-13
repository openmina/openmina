use anyhow::Result;
use clap::{Parser, Subcommand};
use firestore::FirestoreDb;
use sqlx::SqlitePool;

mod config;
mod local_db;
mod remote_db;
mod time;

use config::Config;
use remote_db::ScoreDocument;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create database schema
    InitDb,
    /// Process heartbeats from Firestore
    Process,
    /// Toggle windows disabled state for a time range
    ToggleWindows {
        /// Start time in UTC (format: YYYY-MM-DD HH:MM:SS)
        #[arg(long)]
        start: String,
        /// End time in UTC (format: YYYY-MM-DD HH:MM:SS)
        #[arg(long)]
        end: String,
        #[arg(long)]
        disabled: bool,
    },
    /// View scores for all submitters
    ViewScores,
    /// Post scores to Firestore
    PostScores,
    /// Set the last processing time
    SetLastProcessed {
        /// Time in UTC (format: YYYY-MM-DD or YYYY-MM-DD HH:MM:SS)
        #[arg(long)]
        time: String,
    },
    /// Create time windows for a given time range
    CreateWindows {
        /// Start time in UTC (format: YYYY-MM-DD HH:MM:SS)
        #[arg(long)]
        start: String,
        /// End time in UTC (format: YYYY-MM-DD HH:MM:SS)
        #[arg(long)]
        end: String,
    },
    /// Run continuous processing loop
    ProcessLoop {
        #[arg(long, default_value = "300")]
        interval_seconds: u64,
    },
}

async fn post_scores_to_firestore(
    pool: &SqlitePool,
    db: &FirestoreDb,
    config: &Config,
) -> Result<()> {
    // Make sure scores are up to date
    local_db::update_scores(pool, config).await?;

    let scores = sqlx::query!(
        r#"
        SELECT
            pk.public_key,
            ss.score,
            ss.blocks_produced,
            ss.last_updated,
            ss.last_heartbeat
        FROM submitter_scores ss
        JOIN public_keys pk ON pk.id = ss.public_key_id
        ORDER BY ss.score DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let scores: Vec<ScoreDocument> = scores
        .into_iter()
        .map(|row| ScoreDocument {
            public_key: row.public_key,
            score: row.score,
            blocks_produced: row.blocks_produced,
            last_updated: row.last_updated,
            last_heartbeat: row.last_heartbeat,
        })
        .collect();

    let max_scores = local_db::get_max_scores(pool).await?;
    remote_db::post_scores(db, scores, (max_scores.current, max_scores.total)).await?;

    Ok(())
}

async fn run_process_loop(
    pool: &SqlitePool,
    db: &FirestoreDb,
    config: &Config,
    interval_seconds: u64,
) -> Result<()> {
    let interval = std::time::Duration::from_secs(interval_seconds);

    loop {
        println!("Processing heartbeats...");
        let count = local_db::process_heartbeats(db, pool, config).await?;

        if count > 0 {
            println!("Posting scores...");
            post_scores_to_firestore(pool, db, config).await?;
        }

        println!("Sleeping for {} seconds...", interval_seconds);
        tokio::time::sleep(interval).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    println!("\n{}\n", config);

    let cli = Cli::parse();

    local_db::ensure_db_exists(&config.database_url)?;
    let pool = SqlitePool::connect(&config.database_url).await?;

    match cli.command {
        Commands::InitDb => {
            local_db::create_tables_from_file(&pool).await?;
            local_db::ensure_initial_windows(&pool, &config).await?;
            local_db::mark_disabled_windows(&pool, &config).await?;
        }
        Commands::Process => {
            println!("Initializing firestore connection...");
            let db = remote_db::get_db(&config).await?;
            local_db::create_tables_from_file(&pool).await?;
            local_db::ensure_initial_windows(&pool, &config).await?;
            local_db::mark_disabled_windows(&pool, &config).await?;
            local_db::process_heartbeats(&db, &pool, &config).await?;
            println!("Processing completed successfully!");
        }
        Commands::ToggleWindows {
            start,
            end,
            disabled,
        } => {
            local_db::toggle_windows(&pool, start, end, disabled).await?;
        }
        Commands::ViewScores => {
            local_db::view_scores(&pool, &config).await?;
        }
        Commands::PostScores => {
            println!("Initializing firestore connection...");
            let db = remote_db::get_db(&config).await?;
            post_scores_to_firestore(&pool, &db, &config).await?;
        }
        Commands::SetLastProcessed { time } => {
            local_db::set_last_processed_time(&pool, &time).await?;
        }
        Commands::CreateWindows { start, end } => {
            local_db::create_windows(&pool, start, end).await?;
        }
        Commands::ProcessLoop { interval_seconds } => {
            println!("Initializing firestore connection...");
            let db = remote_db::get_db(&config).await?;
            local_db::create_tables_from_file(&pool).await?;
            local_db::ensure_initial_windows(&pool, &config).await?;
            local_db::mark_disabled_windows(&pool, &config).await?;
            run_process_loop(&pool, &db, &config, interval_seconds).await?;
        }
    }

    Ok(())
}
