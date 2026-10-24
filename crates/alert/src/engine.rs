use std::collections::{BTreeMap, HashMap};

use monitor_core::{Event, EventKind, Value};

use crate::condition::Condition;
use crate::glob_match;

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub source: String,
    pub condition: Condition,
    pub condition_text: String,
    pub consecutive: u32,
    pub cooldown_ms: i64,
    pub notify: Vec<String>,
    pub snapshot: bool,
}

impl Rule {
    pub fn new(
        name: &str,
        source: &str,
        condition: &str,
        consecutive: u32,
        cooldown_sec: u64,
        notify: Vec<String>,
        snapshot: bool,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(!name.is_empty(), "rule name must not be empty");
        anyhow::ensure!(!source.is_empty(), "rule {name}: source must not be empty");
        anyhow::ensure!(consecutive >= 1, "rule {name}: consecutive must be >= 1");
        Ok(Self {
            name: name.to_owned(),
            source: source.to_owned(),
            condition: Condition::parse(condition)
                .map_err(|e| anyhow::anyhow!("rule {name}: {e}"))?,
            condition_text: condition.trim().to_owned(),
            consecutive,
            cooldown_ms: i64::try_from(cooldown_sec).unwrap_or(i64::MAX / 1000) * 1000,
            notify,
            snapshot,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub rule: String,
    pub source: String,
    pub ts_ms: i64,
    pub message: String,
    pub consecutive: u32,
    pub notify: Vec<String>,
    pub snapshot: bool,
    pub fields: BTreeMap<String, Value>,
}

impl Alert {
    pub fn to_event(&self) -> Event {
        let mut event = Event::new(self.ts_ms, format!("alert:{}", self.rule), EventKind::Alert)
            .with("rule", self.rule.as_str())
            .with("target", self.source.as_str())
            .with("message", self.message.as_str())
            .with("consecutive", i64::from(self.consecutive));
        if let Some(value) = self.fields.get("value") {
            event.fields.insert("value".to_owned(), value.clone());
        }
        event
    }
}

#[derive(Default)]
struct RuleState {
    count: u32,
    last_fired_ms: Option<i64>,
}

pub struct RuleEngine {
    rules: Vec<Rule>,
    state: HashMap<(String, String), RuleState>,
}

impl RuleEngine {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self {
            rules,
            state: HashMap::new(),
        }
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn replace_rules(&mut self, rules: Vec<Rule>) {
        let unchanged: Vec<&str> = rules
            .iter()
            .filter(|new| self.rules.iter().any(|old| old == *new))
            .map(|r| r.name.as_str())
            .collect();
        self.state
            .retain(|(name, _), _| unchanged.contains(&name.as_str()));
        self.rules = rules;
    }

    pub fn evaluate(&mut self, event: &Event) -> Vec<Alert> {
        let mut alerts = Vec::new();
        if event.kind != EventKind::Metric {
            return alerts;
        }

        for rule in &self.rules {
            if !glob_match(&rule.source, &event.source) {
                continue;
            }
            let state = self
                .state
                .entry((rule.name.clone(), event.source.clone()))
                .or_default();

            match rule.condition.evaluate(event) {
                Some(true) => state.count = state.count.saturating_add(1),
                _ => {
                    state.count = 0;
                    continue;
                }
            }

            if state.count < rule.consecutive {
                continue;
            }
            if let Some(last) = state.last_fired_ms
                && event.ts_ms - last < rule.cooldown_ms
            {
                continue;
            }
            state.last_fired_ms = Some(event.ts_ms);

            let value = event.get(&rule.condition.field).cloned();
            let shown = value.as_ref().map(ToString::to_string).unwrap_or_default();
            let mut fields = BTreeMap::new();
            if let Some(v) = value {
                fields.insert("value".to_owned(), v);
            }
            alerts.push(Alert {
                rule: rule.name.clone(),
                source: event.source.clone(),
                ts_ms: event.ts_ms,
                message: format!(
                    "{}({})が条件`{}`を{}回連続で満たしました(値: {})",
                    rule.name, event.source, rule.condition_text, state.count, shown
                ),
                consecutive: state.count,
                notify: rule.notify.clone(),
                snapshot: rule.snapshot,
                fields,
            });
        }
        alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(consecutive: u32, cooldown_sec: u64) -> Rule {
        Rule::new(
            "ping_spike",
            "ping:*",
            "rtt_ms > 100",
            consecutive,
            cooldown_sec,
            vec!["log".to_owned()],
            true,
        )
        .unwrap()
    }

    fn ping(ts: i64, source: &str, rtt: f64) -> Event {
        Event::new(ts, source, EventKind::Metric)
            .with("ok", true)
            .with("rtt_ms", rtt)
    }

    #[test]
    fn fires_after_consecutive_matches() {
        let mut engine = RuleEngine::new(vec![rule(3, 300)]);
        assert!(engine.evaluate(&ping(0, "ping:router", 150.0)).is_empty());
        assert!(
            engine
                .evaluate(&ping(1000, "ping:router", 150.0))
                .is_empty()
        );
        let alerts = engine.evaluate(&ping(2000, "ping:router", 150.0));
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].rule, "ping_spike");
        assert_eq!(alerts[0].consecutive, 3);
        assert!(alerts[0].snapshot);
    }

    #[test]
    fn non_matching_event_resets_counter() {
        let mut engine = RuleEngine::new(vec![rule(3, 300)]);
        engine.evaluate(&ping(0, "ping:router", 150.0));
        engine.evaluate(&ping(1000, "ping:router", 150.0));
        engine.evaluate(&ping(2000, "ping:router", 10.0));
        assert!(
            engine
                .evaluate(&ping(3000, "ping:router", 150.0))
                .is_empty()
        );
        assert!(
            engine
                .evaluate(&ping(4000, "ping:router", 150.0))
                .is_empty()
        );
        assert_eq!(engine.evaluate(&ping(5000, "ping:router", 150.0)).len(), 1);
    }

    #[test]
    fn missing_field_resets_counter() {
        let mut engine = RuleEngine::new(vec![rule(2, 300)]);
        engine.evaluate(&ping(0, "ping:router", 150.0));
        let lost = Event::new(1000, "ping:router", EventKind::Metric).with("ok", false);
        engine.evaluate(&lost);
        assert!(
            engine
                .evaluate(&ping(2000, "ping:router", 150.0))
                .is_empty()
        );
    }

    #[test]
    fn cooldown_suppresses_repeats() {
        let mut engine = RuleEngine::new(vec![rule(1, 10)]);
        assert_eq!(engine.evaluate(&ping(0, "ping:router", 150.0)).len(), 1);
        assert!(
            engine
                .evaluate(&ping(5_000, "ping:router", 150.0))
                .is_empty()
        );
        assert!(
            engine
                .evaluate(&ping(9_999, "ping:router", 150.0))
                .is_empty()
        );
        assert_eq!(
            engine.evaluate(&ping(10_000, "ping:router", 150.0)).len(),
            1
        );
    }

    #[test]
    fn replace_rules_keeps_state_of_unchanged_rules_only() {
        let mut engine = RuleEngine::new(vec![rule(1, 600)]);
        assert_eq!(engine.evaluate(&ping(0, "ping:router", 150.0)).len(), 1);

        engine.replace_rules(vec![rule(1, 600)]);
        assert!(
            engine
                .evaluate(&ping(1000, "ping:router", 150.0))
                .is_empty()
        );

        engine.replace_rules(vec![rule(1, 10)]);
        assert_eq!(engine.evaluate(&ping(2000, "ping:router", 150.0)).len(), 1);
    }

    #[test]
    fn state_is_per_source() {
        let mut engine = RuleEngine::new(vec![rule(2, 300)]);
        engine.evaluate(&ping(0, "ping:router", 150.0));
        engine.evaluate(&ping(0, "ping:cloudflare", 150.0));
        assert_eq!(engine.evaluate(&ping(1000, "ping:router", 150.0)).len(), 1);
        assert_eq!(
            engine.evaluate(&ping(1000, "ping:cloudflare", 150.0)).len(),
            1
        );
    }

    #[test]
    fn ignores_other_sources_and_alert_events() {
        let mut engine = RuleEngine::new(vec![rule(1, 0)]);
        let sys = Event::new(0, "system", EventKind::Metric).with("rtt_ms", 500.0);
        assert!(engine.evaluate(&sys).is_empty());
        let alert = Event::new(0, "ping:router", EventKind::Alert).with("rtt_ms", 500.0);
        assert!(engine.evaluate(&alert).is_empty());
    }

    #[test]
    fn rejects_bad_rule() {
        assert!(Rule::new("r", "ping:*", "nonsense", 1, 0, vec![], false).is_err());
        assert!(Rule::new("r", "ping:*", "rtt_ms > 1", 0, 0, vec![], false).is_err());
    }
}
