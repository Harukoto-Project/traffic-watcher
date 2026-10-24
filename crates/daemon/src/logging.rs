use std::path::Path;

use anyhow::Context;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};

const LOG_FILES_KEPT: usize = 14;

fn parse_level(level: &str) -> anyhow::Result<tracing::Level> {
    level
        .parse::<tracing::Level>()
        .map_err(|_| anyhow::anyhow!("invalid log level `{level}`"))
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("panic: {info}");
    }));
}

pub fn init_console(level: &str) -> anyhow::Result<()> {
    let level = parse_level(level)?;
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .init();
    install_panic_hook();
    Ok(())
}

pub fn init_file(level: &str, dir: &Path) -> anyhow::Result<WorkerGuard> {
    let level = parse_level(level)?;
    std::fs::create_dir_all(dir)
        .with_context(|| format!("failed to create log directory {}", dir.display()))?;
    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix("monitord")
        .filename_suffix("log")
        .max_log_files(LOG_FILES_KEPT)
        .build(dir)
        .context("failed to create log file appender")?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(false)
        .with_writer(writer)
        .init();
    install_panic_hook();
    Ok(guard)
}
