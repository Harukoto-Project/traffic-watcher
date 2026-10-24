use std::collections::BTreeMap;

use monitor_core::model::SourceSummary;
use rusqlite::params;

use super::model::SystemSummary;
use crate::{Storage, percentile};

impl Storage {
    pub fn summary_range(
        &self,
        since_ms: i64,
        until_ms: i64,
    ) -> anyhow::Result<Vec<SourceSummary>> {
        let conn = self.lock();
        let mut stmt = conn.prepare_cached(
            "SELECT source, json_extract(fields, '$.ok'), json_extract(fields, '$.rtt_ms')
             FROM events
             WHERE ts_ms >= ?1 AND ts_ms <= ?2 AND source LIKE 'ping:%' AND kind = 'metric'
             ORDER BY source",
        )?;
        let rows = stmt.query_map(params![since_ms, until_ms], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<f64>>(2)?,
            ))
        })?;

        let mut acc: BTreeMap<String, (i64, i64, Vec<f64>)> = BTreeMap::new();
        for row in rows {
            let (source, ok, rtt) = row?;
            let entry = acc.entry(source).or_insert((0, 0, Vec::new()));
            entry.0 += 1;
            if ok == Some(1) {
                if let Some(rtt) = rtt {
                    entry.2.push(rtt);
                }
            } else {
                entry.1 += 1;
            }
        }

        let mut out = Vec::new();
        for (source, (samples, lost, mut rtts)) in acc {
            rtts.sort_by(|a, b| a.total_cmp(b));
            let avg = if rtts.is_empty() {
                None
            } else {
                Some(rtts.iter().sum::<f64>() / rtts.len() as f64)
            };
            out.push(SourceSummary {
                source,
                samples,
                lost,
                loss_pct: if samples > 0 {
                    lost as f64 * 100.0 / samples as f64
                } else {
                    0.0
                },
                avg_ms: avg,
                p95_ms: percentile(&rtts, 0.95),
                p99_ms: percentile(&rtts, 0.99),
                max_ms: rtts.last().copied(),
            });
        }
        Ok(out)
    }

    pub fn alert_count_range(&self, since_ms: i64, until_ms: i64) -> anyhow::Result<i64> {
        let conn = self.lock();
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM events WHERE ts_ms >= ?1 AND ts_ms <= ?2 AND kind = 'alert'",
            params![since_ms, until_ms],
            |row| row.get(0),
        )?)
    }

    pub fn system_summary_range(
        &self,
        since_ms: i64,
        until_ms: i64,
    ) -> anyhow::Result<SystemSummary> {
        let conn = self.lock();
        let summary = conn.query_row(
            "SELECT COUNT(*),
                    AVG(json_extract(fields, '$.cpu_pct')), MAX(json_extract(fields, '$.cpu_pct')),
                    AVG(json_extract(fields, '$.mem_used_pct')), MAX(json_extract(fields, '$.mem_used_pct')),
                    AVG(json_extract(fields, '$.rx_bytes_s')), MAX(json_extract(fields, '$.rx_bytes_s')),
                    AVG(json_extract(fields, '$.tx_bytes_s')), MAX(json_extract(fields, '$.tx_bytes_s'))
             FROM events
             WHERE ts_ms >= ?1 AND ts_ms <= ?2 AND source = 'system' AND kind = 'metric'",
            params![since_ms, until_ms],
            |row| {
                Ok(SystemSummary {
                    samples: row.get(0)?,
                    cpu_avg: row.get(1)?,
                    cpu_max: row.get(2)?,
                    mem_avg: row.get(3)?,
                    mem_max: row.get(4)?,
                    rx_avg: row.get(5)?,
                    rx_max: row.get(6)?,
                    tx_avg: row.get(7)?,
                    tx_max: row.get(8)?,
                })
            },
        )?;
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use monitor_core::{Event, EventKind};

    use crate::Storage;

    fn ping(ts: i64, rtt: Option<f64>) -> Event {
        let mut event = Event::new(ts, "ping:router", EventKind::Metric).with("ok", rtt.is_some());
        if let Some(rtt) = rtt {
            event = event.with("rtt_ms", rtt);
        }
        event
    }

    #[test]
    fn summary_range_respects_both_bounds() {
        let storage = Storage::open_in_memory().unwrap();
        storage
            .insert_batch(&[
                ping(1_000, Some(10.0)),
                ping(2_000, None),
                ping(3_000, Some(30.0)),
                ping(9_000, Some(90.0)),
            ])
            .unwrap();

        let all = storage.summary_range(0, i64::MAX).unwrap();
        assert_eq!(all[0].samples, 4);

        let ranged = storage.summary_range(1_500, 3_500).unwrap();
        assert_eq!(ranged.len(), 1);
        assert_eq!(ranged[0].samples, 2);
        assert_eq!(ranged[0].lost, 0);
        assert_eq!(ranged[0].max_ms, Some(30.0));
    }

    #[test]
    fn system_summary_range_aggregates_metrics() {
        let storage = Storage::open_in_memory().unwrap();
        let system = |ts: i64, cpu: f64| {
            Event::new(ts, "system", EventKind::Metric)
                .with("cpu_pct", cpu)
                .with("mem_used_pct", 50.0)
                .with("rx_bytes_s", 100.0)
                .with("tx_bytes_s", 10.0)
        };
        storage
            .insert_batch(&[system(1_000, 10.0), system(2_000, 30.0)])
            .unwrap();

        let summary = storage.system_summary_range(0, i64::MAX).unwrap();
        assert_eq!(summary.samples, 2);
        assert_eq!(summary.cpu_avg, Some(20.0));
        assert_eq!(summary.cpu_max, Some(30.0));

        let empty = storage.system_summary_range(10_000, 20_000).unwrap();
        assert_eq!(empty.samples, 0);
        assert!(empty.cpu_avg.is_none());
    }
}
