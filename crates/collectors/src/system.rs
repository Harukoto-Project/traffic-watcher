use std::time::{Duration, Instant};

use monitor_core::{Collector, Event, EventKind, now_ms};
use sysinfo::{Disks, Networks, System};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::sampler::run_sampler;

const DISK_REFRESH_EVERY: u64 = 30;

#[derive(Debug, Clone)]
pub struct SystemCollector {
    interval: Duration,
}

impl SystemCollector {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }
}

struct SystemState {
    sys: System,
    nets: Networks,
    disks: Disks,
    last: Instant,
    tick: u64,
    disk_used_pct: f64,
    disk_total_gb: f64,
}

fn disk_usage(disks: &Disks) -> (f64, f64) {
    let mut total = 0u64;
    let mut available = 0u64;
    for disk in disks.list() {
        if disk.is_removable() {
            continue;
        }
        total += disk.total_space();
        available += disk.available_space();
    }
    if total == 0 {
        return (0.0, 0.0);
    }
    let used = total.saturating_sub(available);
    (
        used as f64 * 100.0 / total as f64,
        total as f64 / 1_073_741_824.0,
    )
}

impl SystemState {
    fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        let nets = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let (disk_used_pct, disk_total_gb) = disk_usage(&disks);
        Self {
            sys,
            nets,
            disks,
            last: Instant::now(),
            tick: 0,
            disk_used_pct,
            disk_total_gb,
        }
    }

    fn sample(&mut self) -> Event {
        let ts = now_ms();
        let elapsed = self.last.elapsed().as_secs_f64().max(0.001);
        self.last = Instant::now();
        self.tick += 1;

        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.nets.refresh(true);

        if self.tick.is_multiple_of(DISK_REFRESH_EVERY) {
            self.disks.refresh(true);
            let (pct, total) = disk_usage(&self.disks);
            self.disk_used_pct = pct;
            self.disk_total_gb = total;
        }

        let mut rx = 0u64;
        let mut tx = 0u64;
        let mut top_iface = String::new();
        let mut top_bytes = 0u64;
        for (name, data) in self.nets.list() {
            let r = data.received();
            let t = data.transmitted();
            rx += r;
            tx += t;
            if r + t > top_bytes {
                top_bytes = r + t;
                top_iface = name.clone();
            }
        }

        let mem_total = self.sys.total_memory();
        let mem_used = self.sys.used_memory();
        let mem_used_pct = if mem_total > 0 {
            mem_used as f64 * 100.0 / mem_total as f64
        } else {
            0.0
        };

        let mut event = Event::new(ts, "system", EventKind::Metric)
            .with("cpu_pct", f64::from(self.sys.global_cpu_usage()))
            .with("mem_used_pct", mem_used_pct)
            .with("mem_used_mb", (mem_used / 1_048_576) as i64)
            .with("mem_total_mb", (mem_total / 1_048_576) as i64)
            .with("rx_bytes_s", rx as f64 / elapsed)
            .with("tx_bytes_s", tx as f64 / elapsed)
            .with("disk_used_pct", self.disk_used_pct)
            .with("disk_total_gb", self.disk_total_gb);
        if !top_iface.is_empty() {
            event = event.with("net_top_iface", top_iface);
        }
        event
    }
}

impl Collector for SystemCollector {
    fn name(&self) -> &str {
        "system"
    }

    async fn run(&self, tx: Sender<Event>, shutdown: CancellationToken) -> anyhow::Result<()> {
        let state = tokio::task::spawn_blocking(SystemState::new).await?;
        run_sampler(
            "system",
            state,
            self.interval,
            tx,
            shutdown,
            |s: &mut SystemState| s.sample(),
        )
        .await
    }
}
