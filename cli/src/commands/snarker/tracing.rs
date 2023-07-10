use std::fmt::Result;

use tracing::Level;
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

fn redux_timer(w: &mut Writer<'_>) -> Result {
    match redux::SystemTime::now().duration_since(redux::SystemTime::UNIX_EPOCH) {
        Ok(v) => {
            write!(w, "{}", v.as_nanos())
        }
        Err(_) => write!(w, "unknown-time"),
    }
}

#[allow(dead_code)]
struct ReduxTimer;

impl FormatTime for ReduxTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> Result {
        redux_timer(w)
    }
}

pub fn initialize(max_log_level: Level) {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(max_log_level)
        //.with_timer(ReduxTimer)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("global subscriber should be configurable");
}
