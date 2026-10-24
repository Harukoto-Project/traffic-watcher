use std::collections::BTreeMap;
use std::future::Future;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(v) => Some(*v as f64),
            Value::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(v) => Some(v),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{v}"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v:.2}"),
            Value::Text(v) => write!(f, "{v}"),
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Int(i64::from(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    Metric,
    Alert,
    Info,
}

impl EventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::Metric => "metric",
            EventKind::Alert => "alert",
            EventKind::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts_ms: i64,
    pub source: String,
    pub kind: EventKind,
    pub fields: BTreeMap<String, Value>,
}

pub fn now_ms() -> i64 {
    (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
}

impl Event {
    pub fn new(ts_ms: i64, source: impl Into<String>, kind: EventKind) -> Self {
        Self {
            ts_ms,
            source: source.into(),
            kind,
            fields: BTreeMap::new(),
        }
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }
}

pub mod model {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};

    use crate::Value;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SourceSummary {
        pub source: String,
        pub samples: i64,
        pub lost: i64,
        pub loss_pct: f64,
        pub avg_ms: Option<f64>,
        pub p95_ms: Option<f64>,
        pub p99_ms: Option<f64>,
        pub max_ms: Option<f64>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SnapshotInfo {
        pub id: i64,
        pub created_ms: i64,
        pub rule: String,
        pub source: String,
        pub trigger_ms: i64,
        pub window_start_ms: i64,
        pub window_end_ms: i64,
        pub message: String,
        pub event_count: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SourceStatus {
        pub source: String,
        pub last_ts_ms: i64,
        pub fields: BTreeMap<String, Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StatusInfo {
        pub version: String,
        pub pid: u32,
        pub started_ms: i64,
        pub uptime_secs: i64,
        pub config_path: String,
        pub events_total: u64,
        pub alerts_total: u64,
        pub last_reload_ms: Option<i64>,
        pub last_reload_error: Option<String>,
        pub sources: Vec<SourceStatus>,
    }
}

pub trait Collector: Send + Sync + 'static {
    fn name(&self) -> &str;

    fn run(
        &self,
        tx: Sender<Event>,
        shutdown: CancellationToken,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
