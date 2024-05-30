use std::{env, sync::atomic::AtomicBool};

use openmina_core::log::inner::Level;
use tracing::Subscriber;
use tracing_subscriber::{layer::SubscriberExt, Layer};

lazy_static::lazy_static! {
    pub(crate) static ref LOG: () = initialize_logging();

    pub(crate) static ref ERROR: AtomicBool = Default::default();
}

/// Tracing subscriber that panics on `Error` level events.
struct ErrorPanicLayer;

impl<S> Layer<S> for ErrorPanicLayer
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if event.metadata().level() == &Level::ERROR {
            ERROR.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

fn initialize_logging() {
    let level = std::env::var("OPENMINA_TRACING_LEVEL")
        .ok()
        .and_then(|level| match level.parse() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("cannot parse {level} as tracing level: {e}");
                None
            }
        })
        .unwrap_or(Level::INFO);
    let builder = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .with_test_writer()
        //.with_timer(ReduxTimer)
        ;
    let error_panic_layer = env::var("PANIC_ON_TRACING_ERROR")
        .ok()
        .and_then(|var| var.parse::<bool>().ok())
        .unwrap_or(true)
        .then_some(ErrorPanicLayer);
    let subscriber = builder.finish().with(error_panic_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("global subscriber should be configurable");

    if env::var("OPENMINA_LOG_TRACER")
        .ok()
        .and_then(|var| var.parse::<bool>().ok())
        .unwrap_or_default()
    {
        if let Err(err) = tracing_log::LogTracer::init() {
            eprintln!("cannot initialize log tracing bridge: {err}");
        }
    }
}
