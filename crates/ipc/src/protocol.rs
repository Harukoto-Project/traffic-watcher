use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::oneshot;

pub const DEFAULT_PIPE_NAME: &str = "monitord";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Status,
    Reload,
    SetPingInterval {
        interval_ms: u64,
    },
    History {
        since_ms: i64,
        until_ms: Option<i64>,
        source: Option<String>,
        limit: usize,
    },
    Summary {
        since_ms: i64,
    },
    Hourly {
        since_ms: i64,
        source: Option<String>,
    },
    Snapshots {
        limit: usize,
    },
    Snapshot {
        id: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    #[serde(default)]
    pub data: Value,
    #[serde(default)]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(data: Value) -> Self {
        Self {
            ok: true,
            data,
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: Value::Null,
            error: Some(message.into()),
        }
    }
}

pub struct Envelope {
    pub request: Request,
    pub reply: oneshot::Sender<Response>,
}
