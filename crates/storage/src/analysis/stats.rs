use rusqlite::OptionalExtension;

use super::model::DatabaseStats;
use crate::{HOUR_MS, Storage};

const EXPECTED_TABLES: i64 = 4;

impl Storage {
    pub fn has_expected_schema(&self) -> anyhow::Result<bool> {
        let conn = self.lock();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'table' AND name IN ('events', 'snapshots', 'snapshot_events', 'hourly')",
            [],
            |row| row.get(0),
        )?;
        Ok(count == EXPECTED_TABLES)
    }

    pub fn database_stats(&self) -> anyhow::Result<DatabaseStats> {
        let conn = self.lock();
        let events: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
        let snapshots: i64 =
            conn.query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))?;
        let hourly_rows: i64 =
            conn.query_row("SELECT COUNT(*) FROM hourly", [], |row| row.get(0))?;

        let event_bounds: (Option<i64>, Option<i64>) = conn
            .query_row("SELECT MIN(ts_ms), MAX(ts_ms) FROM events", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .optional()?
            .unwrap_or((None, None));
        let hourly_bounds: (Option<i64>, Option<i64>) = conn
            .query_row("SELECT MIN(hour_ms), MAX(hour_ms) FROM hourly", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .optional()?
            .unwrap_or((None, None));

        let first_ms = [event_bounds.0, hourly_bounds.0]
            .into_iter()
            .flatten()
            .min();
        let last_ms = [event_bounds.1, hourly_bounds.1.map(|h| h + HOUR_MS)]
            .into_iter()
            .flatten()
            .max();

        let mut stmt = conn.prepare_cached("SELECT DISTINCT source FROM events ORDER BY source")?;
        let mut sources = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let mut hourly_stmt =
            conn.prepare_cached("SELECT DISTINCT source FROM hourly ORDER BY source")?;
        for source in hourly_stmt.query_map([], |row| row.get::<_, String>(0))? {
            let source = source?;
            if !sources.contains(&source) {
                sources.push(source);
            }
        }
        sources.sort();

        Ok(DatabaseStats {
            events,
            snapshots,
            hourly_rows,
            first_ms,
            last_ms,
            sources,
        })
    }
}

#[cfg(test)]
mod tests {
    use monitor_core::{Event, EventKind};

    use crate::Storage;

    #[test]
    fn stats_report_bounds_and_sources() {
        let storage = Storage::open_in_memory().unwrap();
        assert!(storage.has_expected_schema().unwrap());

        let empty = storage.database_stats().unwrap();
        assert_eq!(empty.events, 0);
        assert!(empty.first_ms.is_none());
        assert!(empty.last_ms.is_none());

        let events = vec![
            Event::new(1_000, "ping:router", EventKind::Metric).with("ok", true),
            Event::new(5_000, "system", EventKind::Metric).with("cpu_pct", 1.0),
        ];
        storage.insert_batch(&events).unwrap();

        let stats = storage.database_stats().unwrap();
        assert_eq!(stats.events, 2);
        assert_eq!(stats.first_ms, Some(1_000));
        assert_eq!(stats.last_ms, Some(5_000));
        assert_eq!(stats.sources, vec!["ping:router", "system"]);
    }
}
