use anyhow::Context;
use monitor_storage::Storage;
use serde_json::{Value, json};

use crate::protocol::Request;

pub const MAX_HISTORY_ROWS: usize = 20_000;
pub const MAX_SNAPSHOT_ROWS: usize = 1_000;

pub fn is_query(request: &Request) -> bool {
    matches!(
        request,
        Request::History { .. }
            | Request::Summary { .. }
            | Request::Hourly { .. }
            | Request::Snapshots { .. }
            | Request::Snapshot { .. }
    )
}

pub fn execute(storage: &Storage, request: &Request) -> anyhow::Result<Value> {
    match request {
        Request::History {
            since_ms,
            until_ms,
            source,
            limit,
        } => {
            let events = storage.query_events(
                *since_ms,
                *until_ms,
                source.as_deref(),
                (*limit).clamp(1, MAX_HISTORY_ROWS),
            )?;
            Ok(serde_json::to_value(events)?)
        }
        Request::Summary { since_ms } => Ok(serde_json::to_value(storage.summary(*since_ms)?)?),
        Request::Hourly { since_ms, source } => Ok(serde_json::to_value(
            storage.query_hourly(*since_ms, source.as_deref())?,
        )?),
        Request::Snapshots { limit } => Ok(serde_json::to_value(
            storage.list_snapshots((*limit).clamp(1, MAX_SNAPSHOT_ROWS))?,
        )?),
        Request::Snapshot { id } => {
            let (info, events) = storage
                .get_snapshot(*id)?
                .with_context(|| format!("snapshot {id} not found"))?;
            Ok(json!({ "info": info, "events": events }))
        }
        _ => anyhow::bail!("not a query request"),
    }
}
