pub use tracing::Level;

#[cfg(not(target_family = "wasm"))]
mod native {
    use std::{fmt::Result, path::PathBuf};
    use tracing::{field::Visit, level_filters::LevelFilter, Level};
    use tracing_appender::non_blocking::WorkerGuard;
    use tracing_subscriber::{
        field::{RecordFields, VisitOutput},
        fmt::{
            format::{PrettyVisitor, Writer},
            time::FormatTime,
            FormatFields,
        },
        layer::SubscriberExt,
        Layer,
    };

    #[allow(unused)]
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
            .with_test_writer();
        //.with_timer(ReduxTimer)

        if max_log_level != Level::TRACE {
            let subscriber = builder.fmt_fields(TracingFieldFormatter).finish();
            tracing::subscriber::set_global_default(subscriber)
        } else {
            let subscriber = builder.finish();
            tracing::subscriber::set_global_default(subscriber)
        }
        .expect("global subscriber should be configurable");
    }

    pub fn initialize_with_filesystem_output(
        max_log_level: Level,
        log_output_dir: PathBuf,
    ) -> WorkerGuard {
        let file_appender = tracing_appender::rolling::daily(log_output_dir, "openmina.log");
        let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);
        let level_filter = LevelFilter::from_level(max_log_level);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file_writer)
            .with_ansi(false)
            .with_filter(level_filter);

        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
            .with_filter(level_filter);

        let subscriber = tracing_subscriber::Registry::default()
            .with(file_layer)
            .with(stdout_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global subscriber");

        file_guard
    }
}

#[cfg(target_family = "wasm")]
mod web {
    use super::*;
    use tracing_wasm::{set_as_global_default_with_config, WASMLayerConfigBuilder};

    pub fn initialize(max_log_level: Level) {
        let mut config = WASMLayerConfigBuilder::new();
        config.set_max_level(max_log_level);
        set_as_global_default_with_config(config.build());
    }
}

#[cfg(not(target_family = "wasm"))]
pub use native::{initialize, initialize_with_filesystem_output};
#[cfg(target_family = "wasm")]
pub use web::initialize;
