use std::path::{Path, PathBuf};

use anyhow::Context;
use monitor_alert::{Rule, WebhookNotifier};
use serde::Deserialize;

use crate::secret;

pub const NOTIFY_LOG: &str = "log";
pub const NOTIFY_WEBHOOK: &str = "webhook";

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DaemonConfig {
    pub data_dir: PathBuf,
    pub log_level: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            log_level: "info".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PingTargetConfig {
    pub name: String,
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PingConfig {
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub targets: Vec<PingTargetConfig>,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            timeout_ms: 1000,
            targets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SystemConfig {
    pub enabled: bool,
    pub interval_ms: u64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProcessConfig {
    pub enabled: bool,
    pub interval_ms: u64,
    pub top_n: usize,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 10_000,
            top_n: 5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    pub retention_days: u32,
    pub snapshot_retention_days: u32,
    pub flush_interval_ms: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            retention_days: 7,
            snapshot_retention_days: 90,
            flush_interval_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleConfig {
    pub name: String,
    pub source: String,
    pub condition: String,
    #[serde(default = "default_consecutive")]
    pub consecutive: u32,
    #[serde(default = "default_cooldown")]
    pub cooldown_sec: u64,
    #[serde(default = "default_notify")]
    pub notify: Vec<String>,
    #[serde(default = "default_true")]
    pub snapshot: bool,
}

fn default_consecutive() -> u32 {
    1
}

fn default_cooldown() -> u64 {
    300
}

fn default_notify() -> Vec<String> {
    vec![NOTIFY_LOG.to_owned()]
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlertConfig {
    pub pre_seconds: u64,
    pub post_seconds: u64,
    pub rules: Vec<RuleConfig>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            pre_seconds: 30,
            post_seconds: 15,
            rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WebhookConfig {
    pub url: Option<String>,
    pub url_protected: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IpcConfig {
    pub enabled: bool,
    pub pipe_name: String,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pipe_name: monitor_ipc::DEFAULT_PIPE_NAME.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub ping: PingConfig,
    pub system: SystemConfig,
    pub process: ProcessConfig,
    pub storage: StorageConfig,
    pub alert: AlertConfig,
    pub webhook: WebhookConfig,
    pub ipc: IpcConfig,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("invalid config {}", path.display()))
    }

    pub fn parse(text: &str) -> anyhow::Result<Self> {
        let config: Config = toml::from_str(text)?;
        config.validate()?;
        Ok(config)
    }

    pub fn db_path(&self) -> PathBuf {
        self.daemon.data_dir.join("monitor.db")
    }

    fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.ping.interval_ms >= 100,
            "ping.interval_ms must be >= 100"
        );
        anyhow::ensure!(self.ping.timeout_ms >= 1, "ping.timeout_ms must be >= 1");
        anyhow::ensure!(
            self.system.interval_ms >= 500,
            "system.interval_ms must be >= 500"
        );
        anyhow::ensure!(
            self.process.interval_ms >= 1000,
            "process.interval_ms must be >= 1000"
        );
        anyhow::ensure!(self.process.top_n >= 1, "process.top_n must be >= 1");
        anyhow::ensure!(
            self.storage.flush_interval_ms >= 100,
            "storage.flush_interval_ms must be >= 100"
        );
        anyhow::ensure!(
            self.storage.retention_days >= 1,
            "storage.retention_days must be >= 1"
        );
        anyhow::ensure!(
            !self.ipc.pipe_name.is_empty() && !self.ipc.pipe_name.contains(['\\', '/']),
            "ipc.pipe_name must be a plain name"
        );
        for target in &self.ping.targets {
            anyhow::ensure!(
                !target.name.is_empty() && !target.host.is_empty(),
                "ping target requires name and host"
            );
        }
        let mut names: Vec<&str> = self.ping.targets.iter().map(|t| t.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        anyhow::ensure!(
            names.len() == self.ping.targets.len(),
            "ping target names must be unique"
        );
        anyhow::ensure!(
            !self.ping.targets.is_empty() || self.system.enabled || self.process.enabled,
            "no collector is enabled"
        );

        let rules = self.build_rules()?;
        let uses_webhook = rules
            .iter()
            .any(|r| r.notify.iter().any(|n| n == NOTIFY_WEBHOOK));
        if uses_webhook {
            self.resolve_webhook()?;
        }
        Ok(())
    }

    pub fn build_rules(&self) -> anyhow::Result<Vec<Rule>> {
        let mut rules = Vec::new();
        for rc in &self.alert.rules {
            for channel in &rc.notify {
                anyhow::ensure!(
                    channel == NOTIFY_LOG || channel == NOTIFY_WEBHOOK,
                    "rule {}: unsupported notify channel `{channel}` (supported: log, webhook)",
                    rc.name
                );
            }
            rules.push(Rule::new(
                &rc.name,
                &rc.source,
                &rc.condition,
                rc.consecutive,
                rc.cooldown_sec,
                rc.notify.clone(),
                rc.snapshot,
            )?);
        }
        let mut names: Vec<&str> = rules.iter().map(|r| r.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        anyhow::ensure!(
            names.len() == rules.len(),
            "alert rule names must be unique"
        );
        Ok(rules)
    }

    pub fn resolve_webhook(&self) -> anyhow::Result<Option<WebhookNotifier>> {
        let url = match (&self.webhook.url_protected, &self.webhook.url) {
            (Some(protected), _) => Some(
                secret::unprotect(protected).context("failed to decrypt webhook.url_protected")?,
            ),
            (None, Some(url)) => Some(url.clone()),
            (None, None) => None,
        };
        url.map(|u| WebhookNotifier::new(u.trim())).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[daemon]
data_dir = "C:\\ProgramData\\monitor"

[ping]
interval_ms = 1000

[[ping.targets]]
name = "router"
host = "192.168.0.1"

[[alert.rules]]
name = "ping_spike"
source = "ping:*"
condition = "rtt_ms > 100"
consecutive = 3
cooldown_sec = 300
notify = ["log"]
"#;

    #[test]
    fn parses_sample() {
        let config = Config::parse(SAMPLE).unwrap();
        assert_eq!(config.ping.targets.len(), 1);
        assert_eq!(config.alert.rules[0].consecutive, 3);
        assert!(config.system.enabled);
        assert_eq!(config.build_rules().unwrap().len(), 1);
    }

    #[test]
    fn rejects_unknown_keys_and_channels() {
        assert!(Config::parse("[daemon]\nbogus = 1\n").is_err());
        let toast = SAMPLE.replace("[\"log\"]", "[\"toast\"]");
        assert!(Config::parse(&toast).is_err());
    }

    #[test]
    fn rejects_bad_condition_and_duplicates() {
        let bad = SAMPLE.replace("rtt_ms > 100", "nonsense");
        assert!(Config::parse(&bad).is_err());
        let dup = format!("{SAMPLE}\n[[ping.targets]]\nname = \"router\"\nhost = \"1.1.1.1\"\n");
        assert!(Config::parse(&dup).is_err());
    }

    #[test]
    fn rejects_webhook_without_url_only_when_url_invalid() {
        let with_hook = SAMPLE.replace("[\"log\"]", "[\"log\", \"webhook\"]");
        assert!(Config::parse(&with_hook).is_ok());
        let bad = format!("{with_hook}\n[webhook]\nurl = \"http://example.com/x\"\n");
        assert!(Config::parse(&bad).is_err());
        let good =
            format!("{with_hook}\n[webhook]\nurl = \"https://discord.com/api/webhooks/1/a\"\n");
        assert!(Config::parse(&good).is_ok());
    }

    #[test]
    fn requires_some_collector() {
        let text = "[system]\nenabled = false\n[process]\nenabled = false\n";
        assert!(Config::parse(text).is_err());
    }
}
