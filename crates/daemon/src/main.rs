mod app;
mod config;
mod logging;
mod pipeline;
mod secret;
mod service;
mod supervisor;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context;
use clap::{Parser, Subcommand};
use monitor_alert::WebhookNotifier;
use tokio_util::sync::CancellationToken;

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(
    name = "monitord",
    version,
    about = "Network and system monitoring daemon"
)]
struct Args {
    #[arg(long, global = true, default_value = "config/default.toml")]
    config: PathBuf,

    #[arg(long, global = true)]
    log_level: Option<String>,

    #[arg(long, help = "Run in the foreground instead of as a service")]
    console: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Register the Windows service (requires admin rights)")]
    Install,
    #[command(about = "Remove the Windows service (requires admin rights)")]
    Uninstall,
    #[command(about = "Start the installed Windows service")]
    Start,
    #[command(about = "Stop the installed Windows service")]
    Stop,
    #[command(about = "Encrypt a webhook URL with DPAPI for webhook.url_protected")]
    ProtectSecret,
    #[command(hide = true)]
    Service,
}

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    let args = Args::parse();
    match &args.command {
        Some(Command::Install) => install(&args),
        Some(Command::Uninstall) => {
            service::uninstall()?;
            println!("service uninstalled");
            Ok(())
        }
        Some(Command::Start) => {
            service::start()?;
            println!("service started");
            Ok(())
        }
        Some(Command::Stop) => {
            service::stop()?;
            println!("service stopped");
            Ok(())
        }
        Some(Command::ProtectSecret) => protect_secret(),
        Some(Command::Service) => run_as_service(&args),
        None if args.console => run_console(&args),
        None => anyhow::bail!("specify --console or a subcommand (see --help)"),
    }
}

fn absolute_config_path(path: &Path) -> anyhow::Result<PathBuf> {
    std::path::absolute(path).with_context(|| format!("invalid config path {}", path.display()))
}

fn load_service_config(args: &Args) -> anyhow::Result<(PathBuf, Config)> {
    let path = absolute_config_path(&args.config)?;
    let config = Config::load(&path)?;
    anyhow::ensure!(
        config.daemon.data_dir.is_absolute(),
        "daemon.data_dir must be an absolute path when running as a service (got {})",
        config.daemon.data_dir.display()
    );
    Ok((path, config))
}

fn install(args: &Args) -> anyhow::Result<()> {
    let (path, config) = load_service_config(args)?;
    service::install(&path)?;
    println!("service installed");
    println!("  config:   {}", path.display());
    println!("  data_dir: {}", config.daemon.data_dir.display());
    println!("start it with: monitord start");
    Ok(())
}

fn protect_secret() -> anyhow::Result<()> {
    eprint!("Webhook URL: ");
    std::io::stderr().flush().ok();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let url = line.trim();
    anyhow::ensure!(!url.is_empty(), "no URL entered");
    WebhookNotifier::new(url)?;
    println!("url_protected = \"{}\"", secret::protect(url)?);
    Ok(())
}

fn run_console(args: &Args) -> anyhow::Result<()> {
    let path = absolute_config_path(&args.config)?;
    let config = Config::load(&path)?;
    let level = args
        .log_level
        .as_deref()
        .unwrap_or(&config.daemon.log_level);
    logging::init_console(level)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let token = CancellationToken::new();
    let signal_token = token.clone();
    runtime.spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("shutdown requested");
        signal_token.cancel();
    });
    runtime.block_on(app::run(path, token))
}

fn startup_error_log(message: &str) {
    let base = std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("monitor");
    let _ = std::fs::create_dir_all(&base);
    let _ = std::fs::write(base.join("monitord-startup-error.log"), message);
}

fn run_as_service(args: &Args) -> anyhow::Result<()> {
    let (path, config) = match load_service_config(args) {
        Ok(v) => v,
        Err(err) => {
            startup_error_log(&format!("{err:#}\n"));
            return Err(err);
        }
    };
    let level = args
        .log_level
        .as_deref()
        .unwrap_or(&config.daemon.log_level);
    let _guard = match logging::init_file(level, &config.daemon.data_dir.join("logs")) {
        Ok(guard) => guard,
        Err(err) => {
            startup_error_log(&format!("{err:#}\n"));
            return Err(err);
        }
    };
    service::run_dispatcher(path)
}
