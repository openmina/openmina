use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Config {
    pub google_cloud_project: String,
    pub google_credentials_path: Option<String>,
    pub firestore_emulator_host: Option<String>,
    pub database_url: String,
    pub window_range_start: DateTime<Utc>,
    pub window_range_end: DateTime<Utc>,
    pub disabled_windows: Vec<(DateTime<Utc>, DateTime<Utc>)>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let google_cloud_project =
            std::env::var("GOOGLE_CLOUD_PROJECT").unwrap_or_else(|_| "local".to_string());

        let google_credentials_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();
        let firestore_emulator_host = std::env::var("FIRESTORE_EMULATOR_HOST").ok();

        let database_url = std::env::var("DATABASE_PATH")
            .map(|path| format!("sqlite:{}", path))
            .unwrap_or_else(|_| format!("sqlite:heartbeats-{}.db", google_cloud_project));

        let window_range_start =
            std::env::var("WINDOW_RANGE_START").context("WINDOW_RANGE_START must be set")?;
        let window_range_start = DateTime::parse_from_rfc3339(&window_range_start)
            .context("Failed to parse WINDOW_RANGE_START as RFC3339")?
            .with_timezone(&Utc);

        let window_range_end =
            std::env::var("WINDOW_RANGE_END").context("WINDOW_RANGE_END must be set")?;
        let window_range_end = DateTime::parse_from_rfc3339(&window_range_end)
            .context("Failed to parse WINDOW_RANGE_END as RFC3339")?
            .with_timezone(&Utc);

        if window_range_start >= window_range_end {
            anyhow::bail!("WINDOW_RANGE_START must be before WINDOW_RANGE_END");
        }

        let disabled_windows = if let Ok(ranges) = std::env::var("DISABLED_WINDOWS") {
            let mut windows = Vec::new();
            for range in ranges.split(',').filter(|s| !s.is_empty()) {
                let mut parts = range.split('/');
                let start = parts.next().ok_or_else(|| {
                    anyhow::anyhow!("Missing start time in disabled window range")
                })?;
                let end = parts
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Missing end time in disabled window range"))?;

                let start = DateTime::parse_from_rfc3339(start)
                    .with_context(|| {
                        format!("Failed to parse disabled window start time: {}", start)
                    })?
                    .with_timezone(&Utc);
                let end = DateTime::parse_from_rfc3339(end)
                    .with_context(|| format!("Failed to parse disabled window end time: {}", end))?
                    .with_timezone(&Utc);

                if start >= end {
                    anyhow::bail!(
                        "Disabled window start time must be before end time: {start} >= {end}"
                    );
                }
                if end < window_range_start || start > window_range_end {
                    println!("Warning: Disabled window {start} to {end} is outside the main window range");
                }

                windows.push((start, end));
            }
            windows
        } else {
            Vec::new()
        };

        Ok(Config {
            google_cloud_project,
            google_credentials_path,
            firestore_emulator_host,
            database_url,
            window_range_start,
            window_range_end,
            disabled_windows,
        })
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Configuration:")?;
        writeln!(f, "  Project: {}", self.google_cloud_project)?;
        if let Some(creds) = &self.google_credentials_path {
            writeln!(f, "  Credentials: {}", creds)?;
        }
        if let Some(emu) = &self.firestore_emulator_host {
            writeln!(f, "  Firestore Emulator: {}", emu)?;
        }
        writeln!(f, "  Database: {}", self.database_url)?;
        writeln!(f, "  Window Range:")?;
        writeln!(f, "    Start: {}", self.window_range_start)?;
        writeln!(f, "    End:   {}", self.window_range_end)?;
        if !self.disabled_windows.is_empty() {
            writeln!(f, "  Disabled Windows:")?;
            for (start, end) in &self.disabled_windows {
                writeln!(f, "    {} to {}", start, end)?;
            }
        }
        Ok(())
    }
}
