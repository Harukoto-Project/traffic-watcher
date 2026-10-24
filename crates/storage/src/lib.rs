use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Context;
use monitor_core::model::{SnapshotInfo, SourceSummary};
use monitor_core::{Event, EventKind};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

mod analysis;

pub use analysis::{DatabaseStats, PingBucket, SystemBucket, SystemSummary};

const HOUR_MS: i64 = 3_600_000;

const ROLLUP_METRICS: &[(&str, &str)] = &[
    ("ping:%", "rtt_ms"),
    ("ping:%", "ok"),
    ("system", "cpu_pct"),
    ("system", "mem_used_pct"),
    ("system", "rx_bytes_s"),
    ("system", "tx_bytes_s"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyRow {
    pub source: String,
    pub hour_ms: i64,
    pub metric: String,
    pub samples: i64,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
}

pub struct NewSnapshot<'a> {
    pub created_ms: i64,
    pub rule: &'a str,
    pub source: &'a str,
    pub trigger_ms: i64,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub message: &'a str,
    pub events: &'a [Event],
}

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

fn glob_to_like(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    for ch in pattern.chars() {
        match ch {
            '*' => out.push('%'),
            '%' | '_' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            other => out.push(other),
        }
    }
    out
}

fn kind_from_str(s: &str) -> EventKind {
    match s {
        "alert" => EventKind::Alert,
        "info" => EventKind::Info,
        _ => EventKind::Metric,
    }
}

fn row_to_event(ts_ms: i64, source: String, kind: String, fields: String) -> Event {
    Event {
        ts_ms,
        source,
        kind: kind_from_str(&kind),
        fields: serde_json::from_str(&fields).unwrap_or_else(|_| BTreeMap::new()),
    }
}

fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (p * sorted.len() as f64).ceil() as usize;
    Some(sorted[rank.clamp(1, sorted.len()) - 1])
}

impl Storage {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let conn =
            Connection::open(path).with_context(|| format!("failed to open {}", path.display()))?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> anyhow::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    pub fn open_readonly(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("failed to open {}", path.display()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init(conn: Connection) -> anyhow::Result<Self> {
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts_ms INTEGER NOT NULL,
                source TEXT NOT NULL,
                kind TEXT NOT NULL,
                fields TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_events_ts ON events(ts_ms);
            CREATE INDEX IF NOT EXISTS idx_events_source_ts ON events(source, ts_ms);
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_ms INTEGER NOT NULL,
                rule TEXT NOT NULL,
                source TEXT NOT NULL,
                trigger_ms INTEGER NOT NULL,
                window_start_ms INTEGER NOT NULL,
                window_end_ms INTEGER NOT NULL,
                message TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS snapshot_events (
                snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
                ts_ms INTEGER NOT NULL,
                source TEXT NOT NULL,
                kind TEXT NOT NULL,
                fields TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_snapshot_events ON snapshot_events(snapshot_id, ts_ms);
            CREATE TABLE IF NOT EXISTS hourly (
                source TEXT NOT NULL,
                hour_ms INTEGER NOT NULL,
                metric TEXT NOT NULL,
                samples INTEGER NOT NULL,
                avg REAL NOT NULL,
                min REAL NOT NULL,
                max REAL NOT NULL,
                PRIMARY KEY (source, hour_ms, metric)
            );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn insert_batch(&self, events: &[Event]) -> anyhow::Result<usize> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO events (ts_ms, source, kind, fields) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for event in events {
                let fields = serde_json::to_string(&event.fields)?;
                stmt.execute(params![
                    event.ts_ms,
                    event.source,
                    event.kind.as_str(),
                    fields
                ])?;
            }
        }
        tx.commit()?;
        Ok(events.len())
    }

    pub fn query_events(
        &self,
        since_ms: i64,
        until_ms: Option<i64>,
        source: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<Event>> {
        let conn = self.lock();
        let pattern = source.map(glob_to_like);
        let mut stmt = conn.prepare_cached(
            "SELECT ts_ms, source, kind, fields FROM events
             WHERE ts_ms >= ?1 AND ts_ms <= ?2 AND (?3 IS NULL OR source LIKE ?3 ESCAPE '\\')
             ORDER BY ts_ms DESC, id DESC LIMIT ?4",
        )?;
        let rows = stmt.query_map(
            params![
                since_ms,
                until_ms.unwrap_or(i64::MAX),
                pattern,
                limit as i64
            ],
            |row| {
                Ok(row_to_event(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            },
        )?;
        let mut events = rows.collect::<Result<Vec<_>, _>>()?;
        events.reverse();
        Ok(events)
    }

    pub fn summary(&self, since_ms: i64) -> anyhow::Result<Vec<SourceSummary>> {
        self.summary_range(since_ms, i64::MAX)
    }

    pub fn query_hourly(
        &self,
        since_ms: i64,
        source: Option<&str>,
    ) -> anyhow::Result<Vec<HourlyRow>> {
        let conn = self.lock();
        let pattern = source.map(glob_to_like);
        let mut stmt = conn.prepare_cached(
            "SELECT source, hour_ms, metric, samples, avg, min, max FROM hourly
             WHERE hour_ms >= ?1 AND (?2 IS NULL OR source LIKE ?2 ESCAPE '\\')
             ORDER BY hour_ms, source, metric",
        )?;
        let rows = stmt.query_map(params![since_ms, pattern], |row| {
            Ok(HourlyRow {
                source: row.get(0)?,
                hour_ms: row.get(1)?,
                metric: row.get(2)?,
                samples: row.get(3)?,
                avg: row.get(4)?,
                min: row.get(5)?,
                max: row.get(6)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn insert_snapshot(&self, snapshot: &NewSnapshot<'_>) -> anyhow::Result<i64> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO snapshots
             (created_ms, rule, source, trigger_ms, window_start_ms, window_end_ms, message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.created_ms,
                snapshot.rule,
                snapshot.source,
                snapshot.trigger_ms,
                snapshot.window_start_ms,
                snapshot.window_end_ms,
                snapshot.message
            ],
        )?;
        let id = tx.last_insert_rowid();
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO snapshot_events (snapshot_id, ts_ms, source, kind, fields)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for event in snapshot.events {
                let fields = serde_json::to_string(&event.fields)?;
                stmt.execute(params![
                    id,
                    event.ts_ms,
                    event.source,
                    event.kind.as_str(),
                    fields
                ])?;
            }
        }
        tx.commit()?;
        Ok(id)
    }

    pub fn list_snapshots(&self, limit: usize) -> anyhow::Result<Vec<SnapshotInfo>> {
        let conn = self.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT s.id, s.created_ms, s.rule, s.source, s.trigger_ms,
                    s.window_start_ms, s.window_end_ms, s.message,
                    (SELECT COUNT(*) FROM snapshot_events e WHERE e.snapshot_id = s.id)
             FROM snapshots s ORDER BY s.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], Self::map_snapshot)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    fn map_snapshot(row: &rusqlite::Row<'_>) -> rusqlite::Result<SnapshotInfo> {
        Ok(SnapshotInfo {
            id: row.get(0)?,
            created_ms: row.get(1)?,
            rule: row.get(2)?,
            source: row.get(3)?,
            trigger_ms: row.get(4)?,
            window_start_ms: row.get(5)?,
            window_end_ms: row.get(6)?,
            message: row.get(7)?,
            event_count: row.get(8)?,
        })
    }

    pub fn get_snapshot(&self, id: i64) -> anyhow::Result<Option<(SnapshotInfo, Vec<Event>)>> {
        let conn = self.lock();
        let info = conn
            .query_row(
                "SELECT s.id, s.created_ms, s.rule, s.source, s.trigger_ms,
                        s.window_start_ms, s.window_end_ms, s.message,
                        (SELECT COUNT(*) FROM snapshot_events e WHERE e.snapshot_id = s.id)
                 FROM snapshots s WHERE s.id = ?1",
                params![id],
                Self::map_snapshot,
            )
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        let Some(info) = info else {
            return Ok(None);
        };
        let mut stmt = conn.prepare_cached(
            "SELECT ts_ms, source, kind, fields FROM snapshot_events
             WHERE snapshot_id = ?1 ORDER BY ts_ms, source",
        )?;
        let rows = stmt.query_map(params![id], |row| {
            Ok(row_to_event(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        let events = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(Some((info, events)))
    }

    pub fn rollup_and_purge(&self, cutoff_ms: i64) -> anyhow::Result<usize> {
        let cutoff = cutoff_ms.div_euclid(HOUR_MS) * HOUR_MS;
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        for (pattern, metric) in ROLLUP_METRICS {
            tx.execute(
                "INSERT OR REPLACE INTO hourly (source, hour_ms, metric, samples, avg, min, max)
                 SELECT source, (ts_ms / ?1) * ?1, ?2, COUNT(v), AVG(v), MIN(v), MAX(v)
                 FROM (
                     SELECT source, ts_ms, json_extract(fields, '$.' || ?2) AS v
                     FROM events WHERE ts_ms < ?3 AND source LIKE ?4 AND kind = 'metric'
                 )
                 WHERE v IS NOT NULL
                 GROUP BY source, (ts_ms / ?1)",
                params![HOUR_MS, metric, cutoff, pattern],
            )?;
        }
        let deleted = tx.execute("DELETE FROM events WHERE ts_ms < ?1", params![cutoff])?;
        tx.commit()?;
        Ok(deleted)
    }

    pub fn purge_snapshots(&self, cutoff_ms: i64) -> anyhow::Result<usize> {
        let conn = self.lock();
        Ok(conn.execute(
            "DELETE FROM snapshots WHERE created_ms < ?1",
            params![cutoff_ms],
        )?)
    }

    pub fn count(&self) -> anyhow::Result<i64> {
        Ok(self
            .lock()
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ping(ts: i64, source: &str, rtt: Option<f64>) -> Event {
        let mut e = Event::new(ts, source, EventKind::Metric).with("ok", rtt.is_some());
        if let Some(rtt) = rtt {
            e = e.with("rtt_ms", rtt);
        }
        e
    }

    #[test]
    fn insert_query_and_glob() {
        let storage = Storage::open_in_memory().unwrap();
        storage
            .insert_batch(&[
                ping(1_000, "ping:router", Some(1.5)),
                ping(2_000, "ping:cloudflare", Some(30.0)),
                ping(3_000, "ping:router", None),
            ])
            .unwrap();
        assert_eq!(storage.count().unwrap(), 3);

        let all = storage.query_events(0, None, Some("ping:*"), 100).unwrap();
        assert_eq!(all.len(), 3);
        assert!(all.windows(2).all(|w| w[0].ts_ms <= w[1].ts_ms));

        let router = storage
            .query_events(0, None, Some("ping:router"), 100)
            .unwrap();
        assert_eq!(router.len(), 2);

        let limited = storage.query_events(0, None, None, 1).unwrap();
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].ts_ms, 3_000);
    }

    #[test]
    fn summary_computes_loss_and_percentiles() {
        let storage = Storage::open_in_memory().unwrap();
        let mut events = Vec::new();
        for i in 1..=100 {
            events.push(ping(i * 10, "ping:router", Some(i as f64)));
        }
        events.push(ping(2_000, "ping:router", None));
        storage.insert_batch(&events).unwrap();

        let summary = storage.summary(0).unwrap();
        assert_eq!(summary.len(), 1);
        let s = &summary[0];
        assert_eq!(s.samples, 101);
        assert_eq!(s.lost, 1);
        assert_eq!(s.max_ms, Some(100.0));
        assert_eq!(s.p95_ms, Some(95.0));
        assert_eq!(s.p99_ms, Some(99.0));
    }

    #[test]
    fn rollup_keeps_hourly_aggregate_before_purge() {
        let storage = Storage::open_in_memory().unwrap();
        let hour = HOUR_MS;
        storage
            .insert_batch(&[
                ping(hour + 10, "ping:router", Some(10.0)),
                ping(hour + 20, "ping:router", Some(30.0)),
                ping(hour + 30, "ping:router", None),
                ping(5 * hour + 10, "ping:router", Some(5.0)),
            ])
            .unwrap();

        let deleted = storage.rollup_and_purge(3 * hour + 5).unwrap();
        assert_eq!(deleted, 3);
        assert_eq!(storage.count().unwrap(), 1);

        let rows = storage.query_hourly(0, Some("ping:router")).unwrap();
        let rtt = rows.iter().find(|r| r.metric == "rtt_ms").unwrap();
        assert_eq!(rtt.hour_ms, hour);
        assert_eq!(rtt.samples, 2);
        assert_eq!(rtt.avg, 20.0);
        assert_eq!(rtt.max, 30.0);
        let ok = rows.iter().find(|r| r.metric == "ok").unwrap();
        assert_eq!(ok.samples, 3);
    }

    #[test]
    fn snapshot_roundtrip() {
        let storage = Storage::open_in_memory().unwrap();
        let events = vec![
            ping(1_000, "ping:router", Some(1.0)),
            ping(2_000, "ping:router", Some(2.0)),
        ];
        let id = storage
            .insert_snapshot(&NewSnapshot {
                created_ms: 3_000,
                rule: "ping_spike",
                source: "ping:router",
                trigger_ms: 2_000,
                window_start_ms: 1_000,
                window_end_ms: 2_000,
                message: "spike",
                events: &events,
            })
            .unwrap();

        let list = storage.list_snapshots(10).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].event_count, 2);

        let (info, loaded) = storage.get_snapshot(id).unwrap().unwrap();
        assert_eq!(info.rule, "ping_spike");
        assert_eq!(loaded.len(), 2);
        assert!(storage.get_snapshot(id + 1).unwrap().is_none());

        assert_eq!(storage.purge_snapshots(4_000).unwrap(), 1);
        assert!(storage.list_snapshots(10).unwrap().is_empty());
    }
}
