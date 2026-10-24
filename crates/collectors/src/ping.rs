use std::net::{Ipv4Addr, ToSocketAddrs};
use std::time::Duration;

use monitor_core::{Collector, Event, EventKind, now_ms};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct PingTarget {
    pub name: String,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct PingCollector {
    targets: Vec<PingTarget>,
    interval: Duration,
    timeout: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct ProbeResult {
    pub status: u32,
    pub rtt_ms: Option<f64>,
}

impl PingCollector {
    pub fn new(targets: Vec<PingTarget>, interval: Duration, timeout: Duration) -> Self {
        Self {
            targets,
            interval,
            timeout,
        }
    }
}

const RESOLVE_FAILED: u32 = u32::MAX;

fn resolve_v4(host: &str) -> Option<Ipv4Addr> {
    if let Ok(ip) = host.parse::<Ipv4Addr>() {
        return Some(ip);
    }
    (host, 0u16)
        .to_socket_addrs()
        .ok()?
        .find_map(|addr| match addr.ip() {
            std::net::IpAddr::V4(v4) => Some(v4),
            std::net::IpAddr::V6(_) => None,
        })
}

#[cfg(windows)]
fn probe_blocking(addr: Ipv4Addr, timeout: Duration) -> anyhow::Result<ProbeResult> {
    use std::ffi::c_void;
    use std::time::Instant;
    use windows::Win32::NetworkManagement::IpHelper::{
        ICMP_ECHO_REPLY, IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
    };

    const PAYLOAD: [u8; 32] = *b"traffic-watcher-ping-payload-001";
    const IP_SUCCESS: u32 = 0;

    let handle = unsafe { IcmpCreateFile()? };
    let mut reply = vec![0u8; size_of::<ICMP_ECHO_REPLY>() + PAYLOAD.len() + 8 + 64];
    let dest = u32::from_ne_bytes(addr.octets());
    let timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);

    let start = Instant::now();
    let count = unsafe {
        IcmpSendEcho(
            handle,
            dest,
            PAYLOAD.as_ptr() as *const c_void,
            PAYLOAD.len() as u16,
            None,
            reply.as_mut_ptr() as *mut c_void,
            reply.len() as u32,
            timeout_ms,
        )
    };
    let elapsed = start.elapsed();

    let result = if count > 0 {
        let echo = unsafe { &*(reply.as_ptr() as *const ICMP_ECHO_REPLY) };
        if echo.Status == IP_SUCCESS {
            ProbeResult {
                status: IP_SUCCESS,
                rtt_ms: Some(elapsed.as_secs_f64() * 1000.0),
            }
        } else {
            ProbeResult {
                status: echo.Status,
                rtt_ms: None,
            }
        }
    } else {
        let code = unsafe { windows::Win32::Foundation::GetLastError() }.0;
        ProbeResult {
            status: code,
            rtt_ms: None,
        }
    };

    let _ = unsafe { IcmpCloseHandle(handle) };
    Ok(result)
}

#[cfg(not(windows))]
fn probe_blocking(_addr: Ipv4Addr, _timeout: Duration) -> anyhow::Result<ProbeResult> {
    anyhow::bail!("ICMP ping is only supported on Windows")
}

impl Collector for PingCollector {
    fn name(&self) -> &str {
        "ping"
    }

    async fn run(&self, tx: Sender<Event>, shutdown: CancellationToken) -> anyhow::Result<()> {
        let mut resolved: Vec<Option<Ipv4Addr>> = vec![None; self.targets.len()];
        let mut ticker = interval(self.interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => return Ok(()),
                _ = ticker.tick() => {}
            }

            let ts = now_ms();
            let mut set = JoinSet::new();
            for (idx, target) in self.targets.iter().enumerate() {
                let host = target.host.clone();
                let cached = resolved[idx];
                let timeout = self.timeout;
                set.spawn_blocking(move || {
                    let addr = cached.or_else(|| resolve_v4(&host));
                    let outcome = match addr {
                        Some(a) => probe_blocking(a, timeout),
                        None => Ok(ProbeResult {
                            status: RESOLVE_FAILED,
                            rtt_ms: None,
                        }),
                    };
                    (idx, addr, outcome)
                });
            }

            while let Some(joined) = set.join_next().await {
                let (idx, addr, outcome) = match joined {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::error!(%err, "ping task failed");
                        continue;
                    }
                };
                resolved[idx] = addr;
                let target = &self.targets[idx];
                let mut event = Event::new(ts, format!("ping:{}", target.name), EventKind::Metric)
                    .with("host", target.host.as_str());
                if let Some(a) = addr {
                    event = event.with("addr", a.to_string());
                }
                match outcome {
                    Ok(res) => {
                        event = event
                            .with("ok", res.rtt_ms.is_some())
                            .with("status", res.status);
                        if let Some(rtt) = res.rtt_ms {
                            event = event.with("rtt_ms", rtt);
                        }
                    }
                    Err(err) => {
                        tracing::error!(target = %target.name, %err, "ping probe error");
                        event = event.with("ok", false).with("error", err.to_string());
                    }
                }
                if tx.send(event).await.is_err() {
                    return Ok(());
                }
            }
        }
    }
}
