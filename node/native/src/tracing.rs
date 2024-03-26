pub use tracing::Level;

use std::fmt::Result;

use tracing::field::Visit;
use tracing_subscriber::{
    field::{RecordFields, VisitOutput},
    fmt::{
        format::{PrettyVisitor, Writer},
        time::FormatTime,
        FormatFields,
    },
};

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

struct FilterVisit<T>(T);

impl<T> FilterVisit<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Visit for FilterVisit<T>
where
    T: Visit,
{
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if !field.name().starts_with("trace_") {
            self.0.record_debug(field, value);
        }
    }
}

#[derive(Default)]
struct TracingFieldFormatter;

impl<'writer> FormatFields<'writer> for TracingFieldFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        writer: Writer<'writer>,
        fields: R,
    ) -> std::fmt::Result {
        let mut v = FilterVisit(PrettyVisitor::new(writer, true));
        fields.record(&mut v);
        v.into_inner().finish()
    }
}

pub fn initialize(max_log_level: Level) {
    let builder = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(max_log_level)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .with_test_writer()
        //.with_timer(ReduxTimer)
        ;
    if max_log_level != Level::TRACE {
        let subscriber = builder.fmt_fields(TracingFieldFormatter).finish();
        tracing::subscriber::set_global_default(subscriber)
    } else {
        let subscriber = builder.finish();
        tracing::subscriber::set_global_default(subscriber)
    }
    .expect("global subscriber should be configurable");
}
