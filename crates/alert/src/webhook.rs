use std::time::Duration;

use anyhow::Context;
use serde::Serialize;

use crate::engine::Alert;

#[derive(Serialize)]
struct DiscordMessage<'a> {
    username: &'a str,
    content: String,
}

pub fn format_alert(alert: &Alert) -> String {
    format!("**[ALERT]** {}", alert.message)
}

#[derive(Clone)]
pub struct WebhookNotifier {
    client: reqwest::Client,
    url: String,
}

impl WebhookNotifier {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let parsed = reqwest::Url::parse(url).context("invalid webhook url")?;
        let loopback = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
        anyhow::ensure!(
            parsed.scheme() == "https" || (parsed.scheme() == "http" && loopback),
            "webhook url must use https (http is allowed only for loopback)"
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            client,
            url: url.to_owned(),
        })
    }

    pub async fn send(&self, alert: &Alert) -> anyhow::Result<()> {
        let body = DiscordMessage {
            username: "traffic-watcher",
            content: format_alert(alert),
        };
        let response = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .context("webhook request failed")?;
        let status = response.status();
        anyhow::ensure!(status.is_success(), "webhook returned {status}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_validation() {
        assert!(WebhookNotifier::new("https://discord.com/api/webhooks/1/abc").is_ok());
        assert!(WebhookNotifier::new("http://127.0.0.1:8080/hook").is_ok());
        assert!(WebhookNotifier::new("http://example.com/hook").is_err());
        assert!(WebhookNotifier::new("not a url").is_err());
    }
}
