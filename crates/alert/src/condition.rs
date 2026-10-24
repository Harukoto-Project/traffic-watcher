use monitor_core::{Event, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Bool(bool),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub field: String,
    pub op: Operator,
    pub operand: Operand,
}

const OPERATORS: &[(&str, Operator)] = &[
    (">=", Operator::Ge),
    ("<=", Operator::Le),
    ("==", Operator::Eq),
    ("!=", Operator::Ne),
    (">", Operator::Gt),
    ("<", Operator::Lt),
];

fn parse_operand(raw: &str) -> Operand {
    match raw {
        "true" => Operand::Bool(true),
        "false" => Operand::Bool(false),
        _ => {
            if let Ok(n) = raw.parse::<f64>() {
                return Operand::Number(n);
            }
            let unquoted = raw
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                .unwrap_or(raw);
            Operand::Text(unquoted.to_owned())
        }
    }
}

impl Condition {
    pub fn parse(input: &str) -> anyhow::Result<Self> {
        let mut best: Option<(usize, &str, Operator)> = None;
        for (symbol, op) in OPERATORS {
            if let Some(pos) = input.find(symbol) {
                let better = match best {
                    None => true,
                    Some((best_pos, best_symbol, _)) => {
                        pos < best_pos || (pos == best_pos && symbol.len() > best_symbol.len())
                    }
                };
                if better {
                    best = Some((pos, symbol, *op));
                }
            }
        }
        let (pos, symbol, op) =
            best.ok_or_else(|| anyhow::anyhow!("no operator in condition: {input}"))?;
        let field = input[..pos].trim();
        let rhs = input[pos + symbol.len()..].trim();
        anyhow::ensure!(!field.is_empty(), "missing field in condition: {input}");
        anyhow::ensure!(!rhs.is_empty(), "missing value in condition: {input}");
        let operand = parse_operand(rhs);
        if let Operand::Bool(_) | Operand::Text(_) = operand {
            anyhow::ensure!(
                matches!(op, Operator::Eq | Operator::Ne),
                "operator {symbol} requires a numeric value: {input}"
            );
        }
        Ok(Self {
            field: field.to_owned(),
            op,
            operand,
        })
    }

    pub fn evaluate(&self, event: &Event) -> Option<bool> {
        let value = event.get(&self.field)?;
        match (&self.operand, value) {
            (Operand::Number(rhs), v) => {
                let lhs = v.as_f64()?;
                Some(match self.op {
                    Operator::Gt => lhs > *rhs,
                    Operator::Ge => lhs >= *rhs,
                    Operator::Lt => lhs < *rhs,
                    Operator::Le => lhs <= *rhs,
                    Operator::Eq => (lhs - rhs).abs() < f64::EPSILON,
                    Operator::Ne => (lhs - rhs).abs() >= f64::EPSILON,
                })
            }
            (Operand::Bool(rhs), Value::Bool(lhs)) => Some(match self.op {
                Operator::Eq => lhs == rhs,
                Operator::Ne => lhs != rhs,
                _ => return None,
            }),
            (Operand::Text(rhs), Value::Text(lhs)) => Some(match self.op {
                Operator::Eq => lhs == rhs,
                Operator::Ne => lhs != rhs,
                _ => return None,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monitor_core::EventKind;

    fn event() -> Event {
        Event::new(0, "ping:router", EventKind::Metric)
            .with("rtt_ms", 120.5)
            .with("ok", true)
            .with("host", "192.168.0.1")
            .with("count", 3i64)
    }

    #[test]
    fn parses_operators() {
        let c = Condition::parse("rtt_ms > 100").unwrap();
        assert_eq!(c.field, "rtt_ms");
        assert_eq!(c.op, Operator::Gt);
        assert_eq!(c.operand, Operand::Number(100.0));

        let c = Condition::parse("rtt_ms>=100").unwrap();
        assert_eq!(c.op, Operator::Ge);

        let c = Condition::parse("ok == false").unwrap();
        assert_eq!(c.operand, Operand::Bool(false));

        let c = Condition::parse("host != \"x\"").unwrap();
        assert_eq!(c.operand, Operand::Text("x".to_owned()));
    }

    #[test]
    fn rejects_invalid() {
        assert!(Condition::parse("rtt_ms").is_err());
        assert!(Condition::parse("> 100").is_err());
        assert!(Condition::parse("rtt_ms >").is_err());
        assert!(Condition::parse("ok > true").is_err());
    }

    #[test]
    fn evaluates() {
        let e = event();
        assert_eq!(
            Condition::parse("rtt_ms > 100").unwrap().evaluate(&e),
            Some(true)
        );
        assert_eq!(
            Condition::parse("rtt_ms < 100").unwrap().evaluate(&e),
            Some(false)
        );
        assert_eq!(
            Condition::parse("count >= 3").unwrap().evaluate(&e),
            Some(true)
        );
        assert_eq!(
            Condition::parse("ok == false").unwrap().evaluate(&e),
            Some(false)
        );
        assert_eq!(
            Condition::parse("host == 192.168.0.1")
                .unwrap()
                .evaluate(&e),
            Some(true)
        );
        assert_eq!(Condition::parse("missing > 1").unwrap().evaluate(&e), None);
        assert_eq!(Condition::parse("host > 1").unwrap().evaluate(&e), None);
    }
}
