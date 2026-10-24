use rusqlite::params;

use super::model::{PingBucket, SystemBucket};
use crate::Storage;

impl Storage {
    pub fn ping_series(
        &self,
        since_ms: i64,
        until_ms: i64,
        bucket_ms: i64,
    ) -> anyhow::Result<Vec<PingBucket>> {
        let conn = self.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT (ts_ms / ?3) * ?3 AS bucket_ms,
                    source,
                    COUNT(*),
                    SUM(CASE WHEN json_extract(fields, '$.ok') = 1 THEN 0 ELSE 1 END),
                    AVG(json_extract(fields, '$.rtt_ms')),
                    MAX(json_extract(fields, '$.rtt_ms'))
             FROM events
             WHERE ts_ms >= ?1 AND ts_ms <= ?2 AND source LIKE 'ping:%' AND kind = 'metric'
             GROUP BY bucket_ms, source
             ORDER BY bucket_ms, source",
        )?;
        let rows = stmt.query_map(params![since_ms, until_ms, bucket_ms.max(1)], |row| {
            Ok(PingBucket {
                bucket_ms: row.get(0)?,
                source: row.get(1)?,
                samples: row.get(2)?,
                lost: row.get(3)?,
                avg_ms: row.get(4)?,
                max_ms: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn system_series(
        &self,
        since_ms: i64,
        until_ms: i64,
        bucket_ms: i64,
    ) -> anyhow::Result<Vec<SystemBucket>> {
        let conn = self.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT (ts_ms / ?3) * ?3 AS bucket_ms,
                    AVG(json_extract(fields, '$.cpu_pct')), MAX(json_extract(fields, '$.cpu_pct')),
                    AVG(json_extract(fields, '$.mem_used_pct')), MAX(json_extract(fields, '$.mem_used_pct')),
                    AVG(json_extract(fields, '$.rx_bytes_s')), MAX(json_extract(fields, '$.rx_bytes_s')),
                    AVG(json_extract(fields, '$.tx_bytes_s')), MAX(json_extract(fields, '$.tx_bytes_s'))
             FROM events
             WHERE ts_ms >= ?1 AND ts_ms <= ?2 AND source = 'system' AND kind = 'metric'
             GROUP BY bucket_ms
             ORDER BY bucket_ms",
        )?;
        let rows = stmt.query_map(params![since_ms, until_ms, bucket_ms.max(1)], |row| {
            Ok(SystemBucket {
                bucket_ms: row.get(0)?,
                cpu_avg: row.get(1)?,
                cpu_max: row.get(2)?,
                mem_avg: row.get(3)?,
                mem_max: row.get(4)?,
                rx_avg: row.get(5)?,
                rx_max: row.get(6)?,
                tx_avg: row.get(7)?,
                tx_max: row.get(8)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

#[cfg(test)]
mod tests {
    use monitor_core::{Event, EventKind};

    use crate::Storage;

    #[test]
    fn ping_series_groups_by_bucket_and_counts_loss() {
        let storage = Storage::open_in_memory().unwrap();
        let ping = |ts: i64, rtt: Option<f64>| {
            let mut event =
                Event::new(ts, "ping:router", EventKind::Metric).with("ok", rtt.is_some());
            if let Some(rtt) = rtt {
                event = event.with("rtt_ms", rtt);
            }
            event
        };
        storage
            .insert_batch(&[
                ping(1_000, Some(10.0)),
                ping(2_000, Some(30.0)),
                ping(3_000, None),
                ping(11_000, Some(50.0)),
            ])
            .unwrap();

        let series = storage.ping_series(0, i64::MAX, 10_000).unwrap();
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].bucket_ms, 0);
        assert_eq!(series[0].samples, 3);
        assert_eq!(series[0].lost, 1);
        assert_eq!(series[0].avg_ms, Some(20.0));
        assert_eq!(series[0].max_ms, Some(30.0));
        assert_eq!(series[1].bucket_ms, 10_000);
        assert_eq!(series[1].lost, 0);
    }

    #[test]
    fn system_series_groups_by_bucket() {
        let storage = Storage::open_in_memory().unwrap();
        let system =
            |ts: i64, cpu: f64| Event::new(ts, "system", EventKind::Metric).with("cpu_pct", cpu);
        storage
            .insert_batch(&[
                system(1_000, 10.0),
                system(2_000, 30.0),
                system(12_000, 5.0),
            ])
            .unwrap();

        let series = storage.system_series(0, i64::MAX, 10_000).unwrap();
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].cpu_avg, Some(20.0));
        assert_eq!(series[0].cpu_max, Some(30.0));
        assert!(series[0].mem_avg.is_none());
        assert_eq!(series[1].cpu_max, Some(5.0));
    }
}
