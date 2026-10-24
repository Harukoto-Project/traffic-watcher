mod condition;
mod engine;
mod webhook;

pub use condition::{Condition, Operand, Operator};
pub use engine::{Alert, Rule, RuleEngine};
pub use webhook::{WebhookNotifier, format_alert};

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut mark = 0usize;

    while ti < t.len() {
        if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if pi < p.len() && p[pi] == t[ti] {
            pi += 1;
            ti += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::glob_match;

    #[test]
    fn glob_matching() {
        assert!(glob_match("ping:*", "ping:router"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("ping:router", "ping:router"));
        assert!(glob_match("*:router", "ping:router"));
        assert!(!glob_match("ping:*", "system"));
        assert!(!glob_match("ping:router", "ping:router2"));
        assert!(glob_match("a*b*c", "aXXbYYc"));
        assert!(!glob_match("a*b*c", "aXXbYY"));
    }
}
