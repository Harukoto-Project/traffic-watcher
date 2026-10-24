use monitor_core::Event;
use monitor_storage::Storage;

use crate::error::CommandError;

pub const MAX_EVENT_LIMIT: usize = 5000;
const KINDS: &[&str] = &["metric", "alert", "info"];

pub fn normalize_kind(kind: Option<String>) -> Result<Option<String>, CommandError> {
    match kind.filter(|k| !k.is_empty()) {
        Some(k) if KINDS.contains(&k.as_str()) => Ok(Some(k)),
        Some(k) => Err(CommandError::invalid_argument(format!(
            "不明なイベント種別です: {k}"
        ))),
        None => Ok(None),
    }
}

pub fn list(
    storage: &Storage,
    since_ms: i64,
    until_ms: i64,
    source: Option<&str>,
    kind: Option<&str>,
    limit: usize,
) -> anyhow::Result<Vec<Event>> {
    storage.query_events_filtered(
        since_ms,
        until_ms,
        source.filter(|s| !s.is_empty()),
        kind,
        limit.clamp(1, MAX_EVENT_LIMIT),
    )
}

#[cfg(test)]
mod tests {
    use super::normalize_kind;

    #[test]
    fn kind_is_validated() {
        assert_eq!(normalize_kind(None).unwrap(), None);
        assert_eq!(normalize_kind(Some(String::new())).unwrap(), None);
        assert_eq!(
            normalize_kind(Some("alert".to_owned())).unwrap(),
            Some("alert".to_owned())
        );
        assert!(normalize_kind(Some("bogus".to_owned())).is_err());
    }
}
