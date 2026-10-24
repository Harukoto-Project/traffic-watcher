use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub events: i64,
    pub snapshots: i64,
    pub hourly_rows: i64,
    pub first_ms: Option<i64>,
    pub last_ms: Option<i64>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingBucket {
    pub bucket_ms: i64,
    pub source: String,
    pub samples: i64,
    pub lost: i64,
    pub avg_ms: Option<f64>,
    pub max_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBucket {
    pub bucket_ms: i64,
    pub cpu_avg: Option<f64>,
    pub cpu_max: Option<f64>,
    pub mem_avg: Option<f64>,
    pub mem_max: Option<f64>,
    pub rx_avg: Option<f64>,
    pub rx_max: Option<f64>,
    pub tx_avg: Option<f64>,
    pub tx_max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSummary {
    pub samples: i64,
    pub cpu_avg: Option<f64>,
    pub cpu_max: Option<f64>,
    pub mem_avg: Option<f64>,
    pub mem_max: Option<f64>,
    pub rx_avg: Option<f64>,
    pub rx_max: Option<f64>,
    pub tx_avg: Option<f64>,
    pub tx_max: Option<f64>,
}
