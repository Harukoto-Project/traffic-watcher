mod render;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use clap::{Parser, Subcommand};
use monitor_core::model::{SnapshotInfo, SourceSummary, StatusInfo};
use monitor_core::{Event, now_ms};
use monitor_ipc::{DEFAULT_PIPE_NAME, Request, query};
use monitor_storage::{HourlyRow, Storage};
use serde::Deserialize;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(
    name = "monitorctl",
    version,
    about = "Control and inspect the monitord daemon"
)]
struct Cli {
    #[arg(long, global = true, default_value = DEFAULT_PIPE_NAME, help = "Named pipe of the daemon")]
    pipe: String,

    #[arg(
        long,
        global = true,
        help = "Read the SQLite file directly instead of asking the daemon (history/summary/hourly/snapshots only)"
    )]
    db: Option<PathBuf>,

    #[arg(long, global = true, help = "Print raw JSON")]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Show daemon status and the latest value of each source")]
    Status,
    #[command(about = "Reload the configuration file")]
    Reload,
    #[command(about = "Change the ping interval until the next reload")]
    SetInterval { interval_ms: u64 },
    #[command(about = "Show recent events")]
    History {
        #[arg(long, default_value_t = 5)]
        minutes: u64,
        #[arg(long, help = "Source filter, `*` is a wildcard (e.g. ping:*)")]
        source: Option<String>,
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },
    #[command(about = "Show ping loss and latency percentiles")]
    Summary {
        #[arg(long, default_value_t = 60)]
        minutes: u64,
    },
    #[command(about = "Show hourly aggregates kept after raw data is purged")]
    Hourly {
        #[arg(long, default_value_t = 24)]
        hours: u64,
        #[arg(long)]
        source: Option<String>,
    },
    #[command(about = "List alert snapshots")]
    Snapshots {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    #[command(about = "Show one alert snapshot")]
    Snapshot { id: i64 },
}

#[derive(Deserialize)]
struct SnapshotDetail {
    info: SnapshotInfo,
    events: Vec<Event>,
}

fn since(minutes: u64) -> i64 {
    now_ms()
        - i64::try_from(minutes)
            .unwrap_or(i64::MAX / 120_000)
            .saturating_mul(60_000)
}

async fn call(cli: &Cli, request: Request) -> anyhow::Result<Value> {
    if let Some(db) = &cli.db {
        anyhow::ensure!(
            query::is_query(&request),
            "this command needs the running daemon (omit --db)"
        );
        let storage = Storage::open_readonly(db)?;
        return query::execute(&storage, &request);
    }
    let response = monitor_ipc::request(&cli.pipe, &request).await?;
    if !response.ok {
        anyhow::bail!(
            "{}",
            response
                .error
                .unwrap_or_else(|| "request failed".to_owned())
        );
    }
    Ok(response.data)
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let request = match &cli.command {
        Command::Status => Request::Status,
        Command::Reload => Request::Reload,
        Command::SetInterval { interval_ms } => Request::SetPingInterval {
            interval_ms: *interval_ms,
        },
        Command::History {
            minutes,
            source,
            limit,
        } => Request::History {
            since_ms: since(*minutes),
            until_ms: None,
            source: source.clone(),
            limit: *limit,
        },
        Command::Summary { minutes } => Request::Summary {
            since_ms: since(*minutes),
        },
        Command::Hourly { hours, source } => Request::Hourly {
            since_ms: since(hours.saturating_mul(60)),
            source: source.clone(),
        },
        Command::Snapshots { limit } => Request::Snapshots { limit: *limit },
        Command::Snapshot { id } => Request::Snapshot { id: *id },
    };

    let data = call(&cli, request).await?;
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    match &cli.command {
        Command::Status => render::status(&serde_json::from_value::<StatusInfo>(data)?),
        Command::Reload => println!("configuration reloaded"),
        Command::SetInterval { interval_ms } => {
            println!("ping interval set to {interval_ms} ms (until next reload)")
        }
        Command::History { .. } => render::history(&serde_json::from_value::<Vec<Event>>(data)?),
        Command::Summary { minutes } => render::summary(
            *minutes,
            &serde_json::from_value::<Vec<SourceSummary>>(data)?,
        ),
        Command::Hourly { .. } => render::hourly(&serde_json::from_value::<Vec<HourlyRow>>(data)?),
        Command::Snapshots { .. } => {
            render::snapshots(&serde_json::from_value::<Vec<SnapshotInfo>>(data)?)
        }
        Command::Snapshot { .. } => {
            let detail: SnapshotDetail =
                serde_json::from_value(data).context("unexpected snapshot payload")?;
            render::snapshot(&detail.info, &detail.events);
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
