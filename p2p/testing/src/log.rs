use openmina_core::log::inner::Level;

lazy_static::lazy_static! {
    pub(crate) static ref LOG: () = initialize_logging();
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
        .unwrap_or(Level::DEBUG);
    let builder = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .with_test_writer()
        //.with_timer(ReduxTimer)
        ;
    let subscriber = builder.finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("global subscriber should be configurable");
}
