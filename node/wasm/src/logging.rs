use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use serde::Serialize;
use shared::log::inner as tracing;
use tracing::field::{Field, Visit};
use tracing::{Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};
use wasm_bindgen::prelude::*;

pub struct InMemLoggerConfig {
    pub max_level: Level,
    /// Maximum length for logs list stored in memory.
    pub max_len: usize,
}

pub fn setup_global_logger(config: InMemLoggerConfig) -> InMemLogs {
    let max_len = config.max_len;
    let logger = InMemLogger {
        config,
        logs: InMemLogs::new(max_len),
    };
    let logs = logger.logs.clone();

    tracing::subscriber::set_global_default(Registry::default().with(logger))
        .expect("error setting default global subscriber for ");

    logs
}

#[derive(Serialize, Debug, Clone, Copy)]
#[wasm_bindgen]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Default, Clone)]
pub struct InMemLogs {
    max_len: usize,
    list: Rc<RefCell<VecDeque<InMemLog>>>,
}

impl InMemLogs {
    pub fn new(max_len: usize) -> Self {
        Self {
            max_len,
            list: Default::default(),
        }
    }

    fn last_id(&self) -> usize {
        self.list.borrow().back().map_or(0, |v| v.id())
    }

    fn next_id(&self) -> usize {
        self.last_id() + 1
    }

    fn push(&self, mut log: InMemLog) {
        log.id = self.next_id();
        let mut list = self.list.borrow_mut();
        list.push_back(log);

        if list.len() > self.max_len {
            list.pop_front();
        }
    }

    pub fn get_range(&self, cursor: Option<usize>, limit: usize) -> Vec<InMemLog> {
        let last_id = self.last_id();
        let start = cursor
            .and_then(|id| self.last_id().checked_sub(id))
            .unwrap_or(0);

        self.list
            .borrow()
            .iter()
            .rev()
            .skip(start)
            .take(limit)
            .cloned()
            .collect()
    }
}

// TODO(binier): might be unsafe if used in webworkers.
unsafe impl Send for InMemLogs {}
unsafe impl Sync for InMemLogs {}

#[derive(Serialize, Debug, Clone)]
#[wasm_bindgen]
pub struct InMemLog {
    id: usize,
    level: LogLevel,
    #[serde(rename = "details")]
    fields: BTreeMap<&'static str, InMemLogValue>,
}

#[wasm_bindgen]
impl InMemLog {
    fn new(level: Level, fields: BTreeMap<&'static str, InMemLogValue>) -> Self {
        let level = match level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        };

        Self {
            id: 0,
            level,
            fields,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        Some(self.fields.get(key)?.to_string())
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum InMemLogValue {
    F64(f64),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    Bool(bool),
    Str(String),
}

impl fmt::Display for InMemLogValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F64(v) => write!(f, "{}", v),
            Self::I64(v) => write!(f, "{}", v),
            Self::U64(v) => write!(f, "{}", v),
            Self::I128(v) => write!(f, "{}", v),
            Self::U128(v) => write!(f, "{}", v),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
        }
    }
}

pub struct InMemLogger {
    config: InMemLoggerConfig,
    logs: InMemLogs,
}

// In wasm there are no threads, so we impl these traits in order to
// satisfy tracing library's requirements.
// TODO(binier): might be unsafe if used in webworkers.
unsafe impl Send for InMemLogger {}
unsafe impl Sync for InMemLogger {}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for InMemLogger {
    fn enabled(&self, metadata: &tracing::Metadata<'_>, _: Context<'_, S>) -> bool {
        let level = metadata.level();
        level <= &self.config.max_level
    }

    fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
        Some(self.config.max_level.into())
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
    }

    /// doc: Notifies this layer that a span with the given Id recorded the given values.
    fn on_record(&self, id: &tracing::Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        unimplemented!()
    }

    // /// doc: Notifies this layer that a span with the ID span recorded that it follows from the span with the ID follows.
    // fn on_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id, ctx: Context<'_, S>) {}
    /// doc: Notifies this layer that an event has occurred.
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = Visitor::default();
        event.record(&mut visitor);
        let level = event.metadata().level();
        let fields = visitor.finish();
        self.logs.push(InMemLog::new(*level, fields));
    }

    /// doc: Notifies this layer that a span with the given ID was entered.
    fn on_enter(&self, id: &tracing::Id, _ctx: Context<'_, S>) {}

    /// doc: Notifies this layer that the span with the given ID was exited.
    fn on_exit(&self, id: &tracing::Id, ctx: Context<'_, S>) {}
}

struct Visitor {
    fields: BTreeMap<&'static str, InMemLogValue>,
}

impl Visitor {
    pub fn finish(self) -> BTreeMap<&'static str, InMemLogValue> {
        self.fields
    }
}

impl Default for Visitor {
    fn default() -> Self {
        Self {
            fields: Default::default(),
        }
    }
}

impl Visit for Visitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        unreachable!()
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(field.name(), InMemLogValue::F64(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name(), InMemLogValue::I64(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name(), InMemLogValue::U64(value));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.fields.insert(field.name(), InMemLogValue::I128(value));
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.fields.insert(field.name(), InMemLogValue::U128(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name(), InMemLogValue::Bool(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name(), InMemLogValue::Str(value.to_string()));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn Error + 'static)) {
        self.fields
            .insert(field.name(), InMemLogValue::Str(value.to_string()));
    }
}
