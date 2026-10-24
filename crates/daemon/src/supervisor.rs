use std::sync::Arc;
use std::time::Duration;

use monitor_core::{Collector, Event};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    pub initial: Duration,
    pub max: Duration,
    pub healthy_after: Duration,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(60),
            healthy_after: Duration::from_secs(60),
        }
    }
}

pub fn spawn_supervised<C: Collector>(
    collector: C,
    tx: Sender<Event>,
    token: CancellationToken,
    backoff: Backoff,
) -> JoinHandle<()> {
    let collector = Arc::new(collector);
    tokio::spawn(async move {
        let name = collector.name().to_owned();
        let mut delay = backoff.initial;

        loop {
            if token.is_cancelled() {
                return;
            }
            let started = Instant::now();
            let run = {
                let collector = collector.clone();
                let tx = tx.clone();
                let token = token.clone();
                tokio::spawn(async move { collector.run(tx, token).await })
            };

            match run.await {
                Ok(Ok(())) if token.is_cancelled() => return,
                Ok(Ok(())) => tracing::warn!(collector = %name, "collector exited unexpectedly"),
                Ok(Err(err)) => {
                    tracing::error!(collector = %name, error = %err, "collector failed")
                }
                Err(err) if err.is_panic() => {
                    tracing::error!(collector = %name, "collector panicked")
                }
                Err(err) => {
                    tracing::error!(collector = %name, error = %err, "collector task aborted")
                }
            }

            if token.is_cancelled() {
                return;
            }
            if started.elapsed() >= backoff.healthy_after {
                delay = backoff.initial;
            }
            tracing::info!(collector = %name, retry_in_ms = delay.as_millis() as u64, "restarting collector");
            tokio::select! {
                _ = token.cancelled() => return,
                _ = tokio::time::sleep(delay) => {}
            }
            delay = (delay * 2).min(backoff.max);
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use monitor_core::EventKind;
    use tokio::sync::mpsc;

    use super::*;

    struct Flaky {
        runs: Arc<AtomicU32>,
        panic_first: bool,
    }

    impl Collector for Flaky {
        fn name(&self) -> &str {
            "flaky"
        }

        async fn run(&self, tx: Sender<Event>, shutdown: CancellationToken) -> anyhow::Result<()> {
            let n = self.runs.fetch_add(1, Ordering::SeqCst);
            if n == 0 && self.panic_first {
                panic!("boom");
            }
            if n < 2 {
                anyhow::bail!("transient failure {n}");
            }
            tx.send(Event::new(1, "flaky", EventKind::Metric))
                .await
                .ok();
            shutdown.cancelled().await;
            Ok(())
        }
    }

    #[tokio::test]
    async fn restarts_after_error_and_panic() {
        let runs = Arc::new(AtomicU32::new(0));
        let (tx, mut rx) = mpsc::channel(8);
        let token = CancellationToken::new();
        let handle = spawn_supervised(
            Flaky {
                runs: runs.clone(),
                panic_first: true,
            },
            tx,
            token.clone(),
            Backoff {
                initial: Duration::from_millis(10),
                max: Duration::from_millis(40),
                healthy_after: Duration::from_secs(60),
            },
        );

        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("collector should recover")
            .expect("event");
        assert_eq!(event.source, "flaky");
        assert_eq!(runs.load(Ordering::SeqCst), 3);

        token.cancel();
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("supervisor should stop")
            .unwrap();
    }
}
