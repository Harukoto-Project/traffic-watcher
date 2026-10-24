use std::time::Duration;

use monitor_core::{Collector, Event, EventKind, now_ms};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::sampler::run_sampler;

#[derive(Debug, Clone)]
pub struct ProcessCollector {
    interval: Duration,
    top_n: usize,
}

impl ProcessCollector {
    pub fn new(interval: Duration, top_n: usize) -> Self {
        Self { interval, top_n }
    }
}

struct ProcessState {
    sys: System,
    cpus: f32,
    top_n: usize,
}

struct Entry {
    pid: u32,
    name: String,
    cpu_pct: f64,
    mem_mb: i64,
}

fn refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing().with_cpu().with_memory()
}

impl ProcessState {
    fn new(top_n: usize) -> Self {
        let mut sys = System::new();
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get() as f32)
            .unwrap_or(1.0);
        Self { sys, cpus, top_n }
    }

    fn sample(&mut self) -> Event {
        let ts = now_ms();
        self.sys
            .refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());

        let entries: Vec<Entry> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| Entry {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu_pct: f64::from(p.cpu_usage() / self.cpus),
                mem_mb: (p.memory() / 1_048_576) as i64,
            })
            .collect();

        let mut event = Event::new(ts, "process", EventKind::Metric)
            .with("process_count", entries.len() as i64);

        let mut by_cpu: Vec<&Entry> = entries.iter().collect();
        by_cpu.sort_by(|a, b| b.cpu_pct.total_cmp(&a.cpu_pct));
        for (i, e) in by_cpu.iter().take(self.top_n).enumerate() {
            let n = i + 1;
            event = event
                .with(format!("cpu_top{n}_name"), e.name.as_str())
                .with(format!("cpu_top{n}_pid"), i64::from(e.pid))
                .with(format!("cpu_top{n}_pct"), e.cpu_pct);
        }

        let mut by_mem: Vec<&Entry> = entries.iter().collect();
        by_mem.sort_by_key(|e| std::cmp::Reverse(e.mem_mb));
        for (i, e) in by_mem.iter().take(self.top_n).enumerate() {
            let n = i + 1;
            event = event
                .with(format!("mem_top{n}_name"), e.name.as_str())
                .with(format!("mem_top{n}_pid"), i64::from(e.pid))
                .with(format!("mem_top{n}_mb"), e.mem_mb);
        }
        event
    }
}

impl Collector for ProcessCollector {
    fn name(&self) -> &str {
        "process"
    }

    async fn run(&self, tx: Sender<Event>, shutdown: CancellationToken) -> anyhow::Result<()> {
        let top_n = self.top_n;
        let state = tokio::task::spawn_blocking(move || ProcessState::new(top_n)).await?;
        run_sampler(
            "process",
            state,
            self.interval,
            tx,
            shutdown,
            |s: &mut ProcessState| s.sample(),
        )
        .await
    }
}
