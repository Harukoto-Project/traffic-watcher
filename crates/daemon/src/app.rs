use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Context;
use monitor_collectors::ping::{PingCollector, PingTarget};
use monitor_collectors::process::ProcessCollector;
use monitor_collectors::system::SystemCollector;
use monitor_core::model::StatusInfo;
use monitor_core::{Event, now_ms};
use monitor_ipc::{Envelope, Request, Response, query};
use monitor_storage::Storage;
use notify::{EventKind as NotifyKind, RecursiveMode, Watcher};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::pipeline::{self, PipelineConfig, Shared};
use crate::supervisor::{Backoff, spawn_supervised};

const EVENT_CHANNEL: usize = 4096;
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(500);
const MIN_PING_INTERVAL_MS: u64 = 100;

struct CollectorGroup {
    token: CancellationToken,
    handles: Vec<JoinHandle<()>>,
}

impl CollectorGroup {
    fn start(config: &Config, tx: &mpsc::Sender<Event>, parent: &CancellationToken) -> Self {
        let token = parent.child_token();
        let mut handles = Vec::new();

        if !config.ping.targets.is_empty() {
            let collector = PingCollector::new(
                config
                    .ping
                    .targets
                    .iter()
                    .map(|t| PingTarget {
                        name: t.name.clone(),
                        host: t.host.clone(),
                    })
                    .collect(),
                Duration::from_millis(config.ping.interval_ms),
                Duration::from_millis(config.ping.timeout_ms),
            );
            handles.push(spawn_supervised(
                collector,
                tx.clone(),
                token.clone(),
                Backoff::default(),
            ));
        }
        if config.system.enabled {
            handles.push(spawn_supervised(
                SystemCollector::new(Duration::from_millis(config.system.interval_ms)),
                tx.clone(),
                token.clone(),
                Backoff::default(),
            ));
        }
        if config.process.enabled {
            handles.push(spawn_supervised(
                ProcessCollector::new(
                    Duration::from_millis(config.process.interval_ms),
                    config.process.top_n,
                ),
                tx.clone(),
                token.clone(),
                Backoff::default(),
            ));
        }
        tracing::info!(collectors = handles.len(), "collectors started");
        Self { token, handles }
    }

    async fn stop(self) {
        self.token.cancel();
        for handle in self.handles {
            let _ = handle.await;
        }
    }
}

fn collectors_changed(old: &Config, new: &Config) -> bool {
    old.ping != new.ping || old.system != new.system || old.process != new.process
}

struct App {
    config_path: PathBuf,
    config: Config,
    storage: Storage,
    shared: Arc<Shared>,
    started_ms: i64,
    cfg_tx: watch::Sender<Arc<PipelineConfig>>,
    tx: mpsc::Sender<Event>,
    group: Option<CollectorGroup>,
    shutdown: CancellationToken,
}

impl App {
    async fn restart_collectors(&mut self, new: &Config) {
        if let Some(group) = self.group.take() {
            group.stop().await;
        }
        self.group = Some(CollectorGroup::start(new, &self.tx, &self.shutdown));
    }

    async fn reload(&mut self) -> anyhow::Result<()> {
        let loaded = Config::load(&self.config_path).and_then(|config| {
            let pipeline = PipelineConfig::from_config(&config)?;
            Ok((config, pipeline))
        });
        let (mut new, pipeline_cfg) = match loaded {
            Ok(v) => v,
            Err(err) => {
                let message = format!("{err:#}");
                tracing::error!(error = %message, "config reload failed; keeping previous configuration");
                if let Ok(mut state) = self.shared.reload.lock() {
                    state.error = Some(message);
                }
                return Err(err);
            }
        };

        if new.daemon.data_dir != self.config.daemon.data_dir
            || new.daemon.log_level != self.config.daemon.log_level
        {
            tracing::warn!("daemon.data_dir and daemon.log_level changes require a restart");
        }
        if new.ipc.enabled != self.config.ipc.enabled
            || new.ipc.pipe_name != self.config.ipc.pipe_name
        {
            tracing::warn!("ipc settings changes require a restart");
        }
        new.daemon = self.config.daemon.clone();
        new.ipc = self.config.ipc.clone();

        let _ = self.cfg_tx.send(Arc::new(pipeline_cfg));
        if collectors_changed(&self.config, &new) {
            self.restart_collectors(&new).await;
        }
        self.config = new;

        if let Ok(mut state) = self.shared.reload.lock() {
            state.at_ms = Some(now_ms());
            state.error = None;
        }
        tracing::info!("configuration reloaded");
        Ok(())
    }

    async fn set_ping_interval(&mut self, interval_ms: u64) -> anyhow::Result<()> {
        anyhow::ensure!(
            interval_ms >= MIN_PING_INTERVAL_MS,
            "interval_ms must be >= {MIN_PING_INTERVAL_MS}"
        );
        anyhow::ensure!(
            !self.config.ping.targets.is_empty(),
            "no ping targets configured"
        );
        let mut new = self.config.clone();
        new.ping.interval_ms = interval_ms;
        self.restart_collectors(&new).await;
        self.config = new;
        tracing::info!(interval_ms, "ping interval changed (until next reload)");
        Ok(())
    }

    fn status(&self) -> StatusInfo {
        let now = now_ms();
        let (last_reload_ms, last_reload_error) = self
            .shared
            .reload
            .lock()
            .map(|s| (s.at_ms, s.error.clone()))
            .unwrap_or((None, None));
        let mut sources: Vec<_> = self
            .shared
            .sources
            .lock()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        sources.sort_by(|a, b| a.source.cmp(&b.source));
        StatusInfo {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            pid: std::process::id(),
            started_ms: self.started_ms,
            uptime_secs: (now - self.started_ms) / 1000,
            config_path: self.config_path.display().to_string(),
            events_total: self.shared.events_total.load(Ordering::Relaxed),
            alerts_total: self.shared.alerts_total.load(Ordering::Relaxed),
            last_reload_ms,
            last_reload_error,
            sources,
        }
    }

    async fn handle_request(&mut self, envelope: Envelope) {
        let Envelope { request, reply } = envelope;
        if query::is_query(&request) {
            spawn_query(self.storage.clone(), request, reply);
            return;
        }
        let response = match request {
            Request::Status => match serde_json::to_value(self.status()) {
                Ok(v) => Response::ok(v),
                Err(err) => Response::err(err.to_string()),
            },
            Request::Reload => match self.reload().await {
                Ok(()) => Response::ok(serde_json::Value::Null),
                Err(err) => Response::err(format!("{err:#}")),
            },
            Request::SetPingInterval { interval_ms } => {
                match self.set_ping_interval(interval_ms).await {
                    Ok(()) => Response::ok(serde_json::Value::Null),
                    Err(err) => Response::err(format!("{err:#}")),
                }
            }
            _ => Response::err("unsupported request"),
        };
        let _ = reply.send(response);
    }
}

fn spawn_query(storage: Storage, request: Request, reply: oneshot::Sender<Response>) {
    tokio::task::spawn_blocking(move || {
        let response = match query::execute(&storage, &request) {
            Ok(value) => Response::ok(value),
            Err(err) => Response::err(format!("{err:#}")),
        };
        let _ = reply.send(response);
    });
}

fn watch_config(path: &Path, tx: mpsc::Sender<()>) -> anyhow::Result<notify::RecommendedWatcher> {
    let file_name = path.file_name().map(|n| n.to_owned());
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_owned();
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        let Ok(event) = result else { return };
        let relevant = matches!(event.kind, NotifyKind::Create(_) | NotifyKind::Modify(_));
        if relevant
            && event
                .paths
                .iter()
                .any(|p| p.file_name() == file_name.as_deref())
        {
            let _ = tx.try_send(());
        }
    })?;
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}

pub async fn run(config_path: PathBuf, shutdown: CancellationToken) -> anyhow::Result<()> {
    let config = Config::load(&config_path)?;
    let pipeline_cfg = PipelineConfig::from_config(&config)?;

    let db_path = config.db_path();
    let storage = Storage::open(&db_path)?;
    tracing::info!(path = %db_path.display(), "storage opened");

    let shared = Arc::new(Shared::default());
    let (tx, rx) = mpsc::channel::<Event>(EVENT_CHANNEL);
    let (cfg_tx, cfg_rx) = watch::channel(Arc::new(pipeline_cfg));
    let pipeline_task = tokio::spawn(pipeline::run(storage.clone(), rx, cfg_rx, shared.clone()));

    let (ipc_tx, mut ipc_rx) = mpsc::channel::<Envelope>(64);
    let mut ipc_task: Option<JoinHandle<anyhow::Result<()>>> = if config.ipc.enabled {
        let pipe_name = config.ipc.pipe_name.clone();
        let token = shutdown.child_token();
        Some(tokio::spawn(async move {
            monitor_ipc::serve(&pipe_name, ipc_tx, token).await
        }))
    } else {
        drop(ipc_tx);
        None
    };

    let (watch_tx, mut watch_rx) = mpsc::channel::<()>(16);
    let _watcher = match watch_config(&config_path, watch_tx) {
        Ok(w) => Some(w),
        Err(err) => {
            tracing::warn!(error = %err, "config file watching disabled");
            None
        }
    };

    let group = CollectorGroup::start(&config, &tx, &shutdown);
    let mut app = App {
        config_path,
        config,
        storage,
        shared,
        started_ms: now_ms(),
        cfg_tx,
        tx,
        group: Some(group),
        shutdown: shutdown.clone(),
    };
    tracing::info!("monitord started");

    let outcome: anyhow::Result<()> = loop {
        tokio::select! {
            _ = shutdown.cancelled() => break Ok(()),
            Some(()) = watch_rx.recv() => {
                tokio::time::sleep(RELOAD_DEBOUNCE).await;
                while watch_rx.try_recv().is_ok() {}
                tracing::info!("config file change detected");
                let _ = app.reload().await;
            }
            Some(envelope) = ipc_rx.recv() => app.handle_request(envelope).await,
            finished = async { ipc_task.as_mut().unwrap().await }, if ipc_task.is_some() => {
                ipc_task = None;
                match finished {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => break Err(err.context("ipc server failed")),
                    Err(err) => break Err(anyhow::Error::new(err).context("ipc server task failed")),
                }
            }
        }
    };

    shutdown.cancel();
    tracing::info!("shutting down");
    let App { group, tx, .. } = app;
    if let Some(group) = group {
        group.stop().await;
    }
    drop(tx);
    let _ = tokio::time::timeout(Duration::from_secs(10), pipeline_task)
        .await
        .context("pipeline did not stop in time");
    if let Some(task) = ipc_task {
        let _ = task.await;
    }
    tracing::info!("stopped");
    outcome
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    fn write_config(path: &Path, data_dir: &Path, system_interval_ms: u64, pipe: &str) {
        let text = format!(
            r#"
[daemon]
data_dir = '{}'

[ping]
interval_ms = 200
timeout_ms = 500

[[ping.targets]]
name = "loopback"
host = "127.0.0.1"

[system]
interval_ms = {system_interval_ms}

[process]
enabled = false

[alert]
pre_seconds = 5
post_seconds = 60

[[alert.rules]]
name = "loopback_up"
source = "ping:loopback"
condition = "ok == true"
consecutive = 2
cooldown_sec = 600

[ipc]
pipe_name = "{pipe}"
"#,
            data_dir.display()
        );
        std::fs::write(path, text).unwrap();
    }

    async fn call(pipe: &str, request: Request) -> Response {
        monitor_ipc::request(pipe, &request).await.unwrap()
    }

    async fn wait_for_events(pipe: &str) {
        for _ in 0..100 {
            if let Ok(resp) = monitor_ipc::request(pipe, &Request::Status).await
                && resp.ok
            {
                let status: StatusInfo = serde_json::from_value(resp.data).unwrap();
                if status.events_total >= 4 && status.alerts_total >= 1 {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("daemon did not produce events and an alert in time");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn serves_ipc_reloads_and_shuts_down_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("data");
        let config_path = dir.path().join("config.toml");
        let pipe = format!("monitord-test-{}", std::process::id());
        write_config(&config_path, &data_dir, 500, &pipe);

        let token = CancellationToken::new();
        let daemon = tokio::spawn(run(config_path.clone(), token.clone()));
        wait_for_events(&pipe).await;

        let mut stored = 0;
        for _ in 0..50 {
            let history = call(
                &pipe,
                Request::History {
                    since_ms: 0,
                    until_ms: None,
                    source: Some("ping:*".to_owned()),
                    limit: 100,
                },
            )
            .await;
            assert!(history.ok, "{:?}", history.error);
            stored = history.data.as_array().map_or(0, Vec::len);
            if stored > 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert!(stored > 0, "events should be flushed to storage");

        let second = monitor_ipc::request("monitord-test-missing", &Request::Status).await;
        assert!(second.is_err());

        write_config(&config_path, &data_dir, 1000, &pipe);
        let reload = call(&pipe, Request::Reload).await;
        assert!(reload.ok, "{:?}", reload.error);

        std::fs::write(&config_path, "[bogus]\n").unwrap();
        let bad = call(&pipe, Request::Reload).await;
        assert!(!bad.ok);
        let status: StatusInfo =
            serde_json::from_value(call(&pipe, Request::Status).await.data).unwrap();
        assert!(status.last_reload_error.is_some());
        assert!(status.last_reload_ms.is_some());

        let interval = call(&pipe, Request::SetPingInterval { interval_ms: 10 }).await;
        assert!(!interval.ok);
        let interval = call(&pipe, Request::SetPingInterval { interval_ms: 300 }).await;
        assert!(interval.ok, "{:?}", interval.error);

        token.cancel();
        let result = tokio::time::timeout(Duration::from_secs(15), daemon)
            .await
            .expect("daemon should stop")
            .unwrap();
        assert!(result.is_ok(), "{result:?}");

        let storage = Storage::open(&data_dir.join("monitor.db")).unwrap();
        assert!(storage.count().unwrap() > 0);
        let snapshots = storage.list_snapshots(10).unwrap();
        assert_eq!(snapshots.len(), 1);
        assert!(snapshots[0].event_count > 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn second_instance_fails_on_pipe_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let pipe = format!("monitord-test-dup-{}", std::process::id());
        let first_cfg = dir.path().join("a.toml");
        let second_cfg = dir.path().join("b.toml");
        write_config(&first_cfg, &dir.path().join("data-a"), 1000, &pipe);
        write_config(&second_cfg, &dir.path().join("data-b"), 1000, &pipe);

        let token = CancellationToken::new();
        let first = tokio::spawn(run(first_cfg, token.clone()));
        tokio::time::sleep(Duration::from_millis(500)).await;

        let second = tokio::time::timeout(
            Duration::from_secs(10),
            run(second_cfg, CancellationToken::new()),
        )
        .await
        .expect("second instance should fail fast");
        assert!(second.is_err());

        token.cancel();
        first.await.unwrap().unwrap();
    }
}
