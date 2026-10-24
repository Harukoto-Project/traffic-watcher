use std::time::Duration;

use monitor_core::Event;
use tokio::sync::mpsc::Sender;
use tokio::time::{Instant, MissedTickBehavior, interval_at};
use tokio_util::sync::CancellationToken;

pub(crate) async fn run_sampler<S, F>(
    name: &'static str,
    state: S,
    period: Duration,
    tx: Sender<Event>,
    shutdown: CancellationToken,
    sample: F,
) -> anyhow::Result<()>
where
    S: Send + 'static,
    F: Fn(&mut S) -> Event + Send + Sync + Clone + 'static,
{
    let mut slot = Some(state);
    let mut ticker = interval_at(Instant::now() + period, period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => return Ok(()),
            _ = ticker.tick() => {}
        }

        let mut current = slot
            .take()
            .ok_or_else(|| anyhow::anyhow!("{name} sampler state lost"))?;
        let f = sample.clone();
        let (returned, event) = tokio::task::spawn_blocking(move || {
            let event = f(&mut current);
            (current, event)
        })
        .await?;
        slot = Some(returned);

        if tx.send(event).await.is_err() {
            return Ok(());
        }
    }
}
