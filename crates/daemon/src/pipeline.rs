use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use monitor_alert::{Alert, Rule, RuleEngine, WebhookNotifier};
use monitor_core::model::SourceStatus;
use monitor_core::{Event, EventKind, now_ms};
use monitor_storage::{NewSnapshot, Storage};
use tokio::sync::{mpsc, watch};
use tokio::time::{MissedTickBehavior, interval};

use crate::config::{Config, NOTIFY_WEBHOOK};

const BATCH_LIMIT: usize = 500;
const BACKLOG_LIMIT: usize = BATCH_LIMIT * 20;
const MAX_PENDING_SNAPSHOTS: usize = 16;
const RETENTION_CHECK: Duration = Duration::from_secs(3600);
const DAY_MS: i64 = 86_400_000;

pub struct PipelineConfig {
    pub rules: Vec<Rule>,
    pub notifier: Option<WebhookNotifier>,
    pub pre_ms: i64,
    pub post_ms: i64,
    pub retention_days: u32,
    pub snapshot_retention_days: u32,
    pub flush_interval: Duration,
}

impl PipelineConfig {
    pub fn from_config(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            rules: config.build_rules()?,
            notifier: config.resolve_webhook()?,
            pre_ms: i64::try_from(config.alert.pre_seconds).unwrap_or(i64::MAX / 1000) * 1000,
            post_ms: i64::try_from(config.alert.post_seconds).unwrap_or(i64::MAX / 1000) * 1000,
            retention_days: config.storage.retention_days,
            snapshot_retention_days: config.storage.snapshot_retention_days,
            flush_interval: Duration::from_millis(config.storage.flush_interval_ms),
        })
    }
}

#[derive(Default)]
pub struct ReloadState {
    pub at_ms: Option<i64>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct Shared {
    pub events_total: AtomicU64,
    pub alerts_total: AtomicU64,
    pub sources: Mutex<HashMap<String, SourceStatus>>,
    pub reload: Mutex<ReloadState>,
}

struct Pending {
    rule: String,
    source: String,
    trigger_ms: i64,
    window_start_ms: i64,
    deadline_ms: i64,
    message: String,
    events: Vec<Event>,
}

struct Pipeline {
    storage: Storage,
    cfg: Arc<PipelineConfig>,
    engine: RuleEngine,
    ring: VecDeque<Event>,
    pending: Vec<Pending>,
    batch: Vec<Event>,
    shared: Arc<Shared>,
}

fn make_ticker(period: Duration) -> tokio::time::Interval {
    let mut ticker = interval(period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker
}

pub async fn run(
    storage: Storage,
    mut rx: mpsc::Receiver<Event>,
    mut cfg_rx: watch::Receiver<Arc<PipelineConfig>>,
    shared: Arc<Shared>,
) {
    let cfg = cfg_rx.borrow_and_update().clone();
    let mut flush_period = cfg.flush_interval;
    let mut flush_tick = make_ticker(flush_period);
    let mut purge_tick = make_ticker(RETENTION_CHECK);
    let mut snapshot_tick = make_ticker(Duration::from_secs(1));
    let mut cfg_open = true;

    let mut pipeline = Pipeline {
        storage,
        engine: RuleEngine::new(cfg.rules.clone()),
        cfg,
        ring: VecDeque::new(),
        pending: Vec::new(),
        batch: Vec::new(),
        shared,
    };

    loop {
        tokio::select! {
            received = rx.recv() => match received {
                Some(event) => pipeline.on_event(event).await,
                None => break,
            },
            changed = cfg_rx.changed(), if cfg_open => {
                if changed.is_err() {
                    cfg_open = false;
                    continue;
                }
                let new_cfg = cfg_rx.borrow_and_update().clone();
                if new_cfg.flush_interval != flush_period {
                    flush_period = new_cfg.flush_interval;
                    flush_tick = make_ticker(flush_period);
                }
                pipeline.engine.replace_rules(new_cfg.rules.clone());
                pipeline.cfg = new_cfg;
                tracing::info!("pipeline configuration updated");
            }
            _ = flush_tick.tick() => pipeline.flush().await,
            _ = purge_tick.tick() => pipeline.purge().await,
            _ = snapshot_tick.tick() => pipeline.finalize_due(now_ms(), false).await,
        }
    }

    pipeline.flush().await;
    pipeline.finalize_due(now_ms(), true).await;
}

impl Pipeline {
    async fn on_event(&mut self, event: Event) {
        self.shared.events_total.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut sources) = self.shared.sources.lock() {
            sources.insert(
                event.source.clone(),
                SourceStatus {
                    source: event.source.clone(),
                    last_ts_ms: event.ts_ms,
                    fields: event.fields.clone(),
                },
            );
        }

        if event.kind == EventKind::Metric {
            self.ring.push_back(event.clone());
            let keep_from = event.ts_ms - self.cfg.pre_ms - 1000;
            while self.ring.front().is_some_and(|e| e.ts_ms < keep_from) {
                self.ring.pop_front();
            }
        }

        for pending in &mut self.pending {
            if event.ts_ms <= pending.deadline_ms {
                pending.events.push(event.clone());
            }
        }

        for alert in self.engine.evaluate(&event) {
            self.handle_alert(alert);
        }

        let ts = event.ts_ms;
        self.batch.push(event);
        if self.batch.len() >= BATCH_LIMIT {
            self.flush().await;
        }
        if self.pending.iter().any(|p| ts >= p.deadline_ms) {
            self.finalize_due(ts, false).await;
        }
    }

    fn handle_alert(&mut self, alert: Alert) {
        self.shared.alerts_total.fetch_add(1, Ordering::Relaxed);
        tracing::warn!(rule = %alert.rule, source = %alert.source, "{}", alert.message);

        let alert_event = alert.to_event();

        if alert.snapshot {
            if self.pending.len() >= MAX_PENDING_SNAPSHOTS {
                tracing::warn!(rule = %alert.rule, "too many pending snapshots; skipping snapshot");
            } else {
                let window_start_ms = alert.ts_ms - self.cfg.pre_ms;
                let mut events: Vec<Event> = self
                    .ring
                    .iter()
                    .filter(|e| e.ts_ms >= window_start_ms)
                    .cloned()
                    .collect();
                events.push(alert_event.clone());
                self.pending.push(Pending {
                    rule: alert.rule.clone(),
                    source: alert.source.clone(),
                    trigger_ms: alert.ts_ms,
                    window_start_ms,
                    deadline_ms: alert.ts_ms + self.cfg.post_ms,
                    message: alert.message.clone(),
                    events,
                });
            }
        }

        self.batch.push(alert_event);
        self.notify(&alert);
    }

    fn notify(&self, alert: &Alert) {
        if !alert.notify.iter().any(|n| n == NOTIFY_WEBHOOK) {
            return;
        }
        let Some(notifier) = self.cfg.notifier.clone() else {
            tracing::warn!(rule = %alert.rule, "webhook requested but no webhook url is configured");
            return;
        };
        let alert = alert.clone();
        tokio::spawn(async move {
            for attempt in 1..=2 {
                match notifier.send(&alert).await {
                    Ok(()) => return,
                    Err(err) => {
                        tracing::error!(rule = %alert.rule, attempt, error = %err, "webhook notification failed");
                        if attempt < 2 {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        }
                    }
                }
            }
        });
    }

    async fn finalize_due(&mut self, now: i64, force: bool) {
        let mut due = Vec::new();
        let mut index = 0;
        while index < self.pending.len() {
            if force || now >= self.pending[index].deadline_ms {
                due.push(self.pending.remove(index));
            } else {
                index += 1;
            }
        }
        for mut pending in due {
            pending.events.sort_by_key(|e| e.ts_ms);
            let window_end_ms = pending
                .events
                .iter()
                .map(|e| e.ts_ms)
                .max()
                .unwrap_or(pending.trigger_ms);
            let storage = self.storage.clone();
            let rule = pending.rule.clone();
            let result = tokio::task::spawn_blocking(move || {
                storage.insert_snapshot(&NewSnapshot {
                    created_ms: now_ms(),
                    rule: &pending.rule,
                    source: &pending.source,
                    trigger_ms: pending.trigger_ms,
                    window_start_ms: pending.window_start_ms,
                    window_end_ms,
                    message: &pending.message,
                    events: &pending.events,
                })
            })
            .await;
            match result {
                Ok(Ok(id)) => tracing::info!(snapshot_id = id, rule = %rule, "snapshot saved"),
                Ok(Err(err)) => {
                    tracing::error!(rule = %rule, error = %err, "failed to save snapshot")
                }
                Err(err) => tracing::error!(error = %err, "snapshot task failed"),
            }
        }
    }

    async fn flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }
        let pending = std::mem::take(&mut self.batch);
        let storage = self.storage.clone();
        let joined = tokio::task::spawn_blocking(move || {
            let result = storage.insert_batch(&pending);
            (pending, result)
        })
        .await;
        match joined {
            Ok((_, Ok(count))) => tracing::debug!(count, "events saved"),
            Ok((mut pending, Err(err))) => {
                tracing::error!(error = %err, "failed to save events");
                if pending.len() > BACKLOG_LIMIT {
                    let excess = pending.len() - BACKLOG_LIMIT;
                    pending.drain(..excess);
                }
                pending.append(&mut self.batch);
                self.batch = pending;
            }
            Err(err) => tracing::error!(error = %err, "storage task failed"),
        }
    }

    async fn purge(&self) {
        let now = now_ms();
        let cutoff = now - i64::from(self.cfg.retention_days) * DAY_MS;
        let snapshot_cutoff = now - i64::from(self.cfg.snapshot_retention_days) * DAY_MS;
        let storage = self.storage.clone();
        let joined = tokio::task::spawn_blocking(move || {
            let events = storage.rollup_and_purge(cutoff);
            let snapshots = storage.purge_snapshots(snapshot_cutoff);
            (events, snapshots)
        })
        .await;
        match joined {
            Ok((events, snapshots)) => {
                match events {
                    Ok(n) if n > 0 => {
                        tracing::info!(deleted = n, "old events rolled up and purged")
                    }
                    Ok(_) => {}
                    Err(err) => tracing::error!(error = %err, "retention purge failed"),
                }
                match snapshots {
                    Ok(n) if n > 0 => tracing::info!(deleted = n, "old snapshots purged"),
                    Ok(_) => {}
                    Err(err) => tracing::error!(error = %err, "snapshot purge failed"),
                }
            }
            Err(err) => tracing::error!(error = %err, "storage task failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(rules: Vec<Rule>) -> Arc<PipelineConfig> {
        Arc::new(PipelineConfig {
            rules,
            notifier: None,
            pre_ms: 5_000,
            post_ms: 3_000,
            retention_days: 7,
            snapshot_retention_days: 90,
            flush_interval: Duration::from_millis(100),
        })
    }

    fn ping(ts: i64, rtt: f64) -> Event {
        Event::new(ts, "ping:router", EventKind::Metric)
            .with("ok", true)
            .with("rtt_ms", rtt)
    }

    #[tokio::test]
    async fn alert_creates_snapshot_with_pre_and_post_events() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = Rule::new("spike", "ping:*", "rtt_ms > 100", 2, 300, vec![], true).unwrap();
        let (tx, rx) = mpsc::channel(64);
        let (_cfg_tx, cfg_rx) = watch::channel(cfg(vec![rule]));
        let shared = Arc::new(Shared::default());

        let handle = tokio::spawn(run(storage.clone(), rx, cfg_rx, shared.clone()));

        let base = now_ms();
        for i in 0..10 {
            tx.send(ping(base + i * 1000, 10.0)).await.unwrap();
        }
        tx.send(ping(base + 10_000, 150.0)).await.unwrap();
        tx.send(ping(base + 11_000, 150.0)).await.unwrap();
        for i in 12..20 {
            tx.send(ping(base + i * 1000, 10.0)).await.unwrap();
        }
        drop(tx);
        handle.await.unwrap();

        assert_eq!(shared.alerts_total.load(Ordering::Relaxed), 1);
        let snapshots = storage.list_snapshots(10).unwrap();
        assert_eq!(snapshots.len(), 1);
        let (info, events) = storage.get_snapshot(snapshots[0].id).unwrap().unwrap();
        assert_eq!(info.rule, "spike");
        assert_eq!(info.trigger_ms, base + 11_000);
        assert!(events.iter().any(|e| e.kind == EventKind::Alert));
        let metrics: Vec<i64> = events
            .iter()
            .filter(|e| e.kind == EventKind::Metric)
            .map(|e| e.ts_ms)
            .collect();
        assert_eq!(*metrics.first().unwrap(), base + 6_000);
        assert_eq!(*metrics.last().unwrap(), base + 14_000);
        assert!(metrics.windows(2).all(|w| w[0] <= w[1]));

        let stored = storage.query_events(0, None, Some("alert:*"), 10).unwrap();
        assert_eq!(stored.len(), 1);
        assert!(storage.count().unwrap() >= 20);
    }

    #[tokio::test]
    async fn no_snapshot_when_rule_disables_it() {
        let storage = Storage::open_in_memory().unwrap();
        let rule = Rule::new("spike", "ping:*", "rtt_ms > 100", 1, 300, vec![], false).unwrap();
        let (tx, rx) = mpsc::channel(8);
        let (_cfg_tx, cfg_rx) = watch::channel(cfg(vec![rule]));
        let shared = Arc::new(Shared::default());
        let handle = tokio::spawn(run(storage.clone(), rx, cfg_rx, shared.clone()));
        tx.send(ping(1_000, 500.0)).await.unwrap();
        drop(tx);
        handle.await.unwrap();
        assert_eq!(shared.alerts_total.load(Ordering::Relaxed), 1);
        assert!(storage.list_snapshots(10).unwrap().is_empty());
    }
}
