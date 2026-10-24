use chrono::{DateTime, Local};
use monitor_core::model::{SnapshotInfo, SourceSummary, StatusInfo};
use monitor_core::{Event, EventKind};
use monitor_storage::HourlyRow;

fn time(ms: i64) -> String {
    DateTime::from_timestamp_millis(ms)
        .map(|d| {
            d.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string()
        })
        .unwrap_or_else(|| ms.to_string())
}

fn opt(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"))
}

fn fields(event: &Event) -> String {
    event
        .fields
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn duration(secs: i64) -> String {
    format!(
        "{}d {:02}:{:02}:{:02}",
        secs / 86_400,
        secs % 86_400 / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

pub fn status(info: &StatusInfo) {
    println!("monitord {} (pid {})", info.version, info.pid);
    println!("  started : {}", time(info.started_ms));
    println!("  uptime  : {}", duration(info.uptime_secs));
    println!("  config  : {}", info.config_path);
    println!("  events  : {}", info.events_total);
    println!("  alerts  : {}", info.alerts_total);
    match (info.last_reload_ms, &info.last_reload_error) {
        (_, Some(err)) => println!("  reload  : FAILED - {err}"),
        (Some(ms), None) => println!("  reload  : {}", time(ms)),
        (None, None) => println!("  reload  : never"),
    }
    println!();
    println!("latest per source:");
    for source in &info.sources {
        let fields = source
            .fields
            .iter()
            .filter(|(k, _)| !k.contains("_top"))
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "  {:<18} {}  {}",
            source.source,
            time(source.last_ts_ms),
            fields
        );
    }
}

pub fn history(events: &[Event]) {
    if events.is_empty() {
        println!("no events");
        return;
    }
    for event in events {
        let mark = if event.kind == EventKind::Alert {
            "!"
        } else {
            " "
        };
        println!(
            "{mark} {}  {:<18} {}",
            time(event.ts_ms),
            event.source,
            fields(event)
        );
    }
    println!("{} events", events.len());
}

pub fn summary(minutes: u64, rows: &[SourceSummary]) {
    if rows.is_empty() {
        println!("no ping data in the last {minutes} minutes");
        return;
    }
    println!("ping summary (last {minutes} minutes)");
    println!(
        "{:<18} {:>8} {:>6} {:>7} {:>8} {:>8} {:>8} {:>8}",
        "SOURCE", "SAMPLES", "LOST", "LOSS%", "AVG ms", "P95 ms", "P99 ms", "MAX ms"
    );
    for r in rows {
        println!(
            "{:<18} {:>8} {:>6} {:>6.2}% {:>8} {:>8} {:>8} {:>8}",
            r.source,
            r.samples,
            r.lost,
            r.loss_pct,
            opt(r.avg_ms),
            opt(r.p95_ms),
            opt(r.p99_ms),
            opt(r.max_ms)
        );
    }
}

pub fn hourly(rows: &[HourlyRow]) {
    if rows.is_empty() {
        println!("no hourly aggregates (raw data is rolled up after retention_days)");
        return;
    }
    println!(
        "{:<16} {:<14} {:<14} {:>8} {:>10} {:>10} {:>10}",
        "HOUR", "SOURCE", "METRIC", "SAMPLES", "AVG", "MIN", "MAX"
    );
    for r in rows {
        let hour = DateTime::from_timestamp_millis(r.hour_ms)
            .map(|d| d.with_timezone(&Local).format("%m-%d %H:%M").to_string())
            .unwrap_or_default();
        println!(
            "{:<16} {:<14} {:<14} {:>8} {:>10.2} {:>10.2} {:>10.2}",
            hour, r.source, r.metric, r.samples, r.avg, r.min, r.max
        );
    }
}

pub fn snapshots(rows: &[SnapshotInfo]) {
    if rows.is_empty() {
        println!("no snapshots");
        return;
    }
    println!(
        "{:>5}  {:<23}  {:<14} {:<18} {:>7}  MESSAGE",
        "ID", "TRIGGER", "RULE", "SOURCE", "EVENTS"
    );
    for s in rows {
        println!(
            "{:>5}  {:<23}  {:<14} {:<18} {:>7}  {}",
            s.id,
            time(s.trigger_ms),
            s.rule,
            s.source,
            s.event_count,
            s.message
        );
    }
}

pub fn snapshot(info: &SnapshotInfo, events: &[Event]) {
    println!("snapshot #{} ({})", info.id, info.rule);
    println!("  source : {}", info.source);
    println!("  trigger: {}", time(info.trigger_ms));
    println!(
        "  window : {} .. {}",
        time(info.window_start_ms),
        time(info.window_end_ms)
    );
    println!("  message: {}", info.message);
    println!();
    for event in events {
        let mark = if event.kind == EventKind::Alert {
            "!"
        } else if event.ts_ms == info.trigger_ms && event.source == info.source {
            ">"
        } else {
            " "
        };
        println!(
            "{mark} {}  {:<18} {}",
            time(event.ts_ms),
            event.source,
            fields(event)
        );
    }
}
