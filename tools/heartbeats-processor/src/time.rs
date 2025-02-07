use chrono::{DateTime, Duration, Utc};

const WINDOW_SIZE_MINUTES: i64 = 5;

#[derive(Debug)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

// Helper function to convert DateTime to Unix timestamp
pub fn to_unix_timestamp(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}

// Helper function to convert Unix timestamp to DateTime
pub fn from_unix_timestamp(ts: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(ts, 0).unwrap()
}

pub fn parse_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    // Try parsing with different formats
    if let Ok(dt) = DateTime::parse_from_str(&format!("{} +0000", s), "%Y-%m-%d %H:%M:%S %z") {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
        return Ok(dt.with_timezone(&Utc));
    }

    Err(anyhow::anyhow!(
        "Invalid datetime format. Expected YYYY-MM-DD HH:MM:SS or YYYY-MM-DDThh:mm:ssZ"
    ))
}

pub fn generate_fixed_time_windows(start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<TimeWindow> {
    let window_duration = Duration::try_minutes(WINDOW_SIZE_MINUTES).unwrap();
    let mut windows = Vec::new();
    let mut current = start;

    while current < end {
        let window_end = current + window_duration;
        windows.push(TimeWindow {
            start: current,
            end: window_end,
        });
        current = window_end;
    }

    windows
}
