use monitor_core::Event;
use rusqlite::params;

use crate::{Storage, glob_to_like, row_to_event};

impl Storage {
    pub fn query_events_filtered(
        &self,
        since_ms: i64,
        until_ms: i64,
        source: Option<&str>,
        kind: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<Event>> {
        let conn = self.lock();
        let pattern = source.map(glob_to_like);
        let mut stmt = conn.prepare_cached(
            "SELECT ts_ms, source, kind, fields FROM events
             WHERE ts_ms >= ?1 AND ts_ms <= ?2
               AND (?3 IS NULL OR source LIKE ?3 ESCAPE '\\')
               AND (?4 IS NULL OR kind = ?4)
             ORDER BY ts_ms DESC, id DESC LIMIT ?5",
        )?;
        let rows = stmt.query_map(
            params![since_ms, until_ms, pattern, kind, limit as i64],
            |row| {
                Ok(row_to_event(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            },
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

#[cfg(test)]
mod tests {
    use monitor_core::{Event, EventKind};

    use crate::Storage;

    #[test]
    fn filtered_query_applies_kind_source_and_order() {
        let storage = Storage::open_in_memory().unwrap();
        storage
            .insert_batch(&[
                Event::new(1_000, "ping:router", EventKind::Metric).with("ok", true),
                Event::new(2_000, "ping:cloudflare", EventKind::Metric).with("ok", true),
                Event::new(3_000, "alert:ping_spike", EventKind::Alert).with("rule", "ping_spike"),
            ])
            .unwrap();

        let all = storage
            .query_events_filtered(0, i64::MAX, None, None, 10)
            .unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].ts_ms, 3_000);

        let alerts = storage
            .query_events_filtered(0, i64::MAX, None, Some("alert"), 10)
            .unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(storage.alert_count_range(0, i64::MAX).unwrap(), 1);

        let pings = storage
            .query_events_filtered(0, i64::MAX, Some("ping:*"), Some("metric"), 1)
            .unwrap();
        assert_eq!(pings.len(), 1);
        assert_eq!(pings[0].source, "ping:cloudflare");
    }
}
