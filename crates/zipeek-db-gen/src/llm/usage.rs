use aimux_core::types::LanguageModelUsage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub cached_input_tokens: u64,
}

impl TurnUsage {
    pub fn from_usage(usage: &LanguageModelUsage) -> Result<Self, serde_json::Error> {
        let value = serde_json::to_value(usage)?;
        Ok(Self::from_value(&value))
    }

    fn from_value(value: &serde_json::Value) -> Self {
        fn count(value: &serde_json::Value, keys: &[&str]) -> u64 {
            keys.iter()
                .find_map(|key| value.get(*key).and_then(serde_json::Value::as_u64))
                .unwrap_or(0)
        }
        let input = value
            .get("inputTokens")
            .or_else(|| value.get("input_tokens"));
        let output = value
            .get("outputTokens")
            .or_else(|| value.get("output_tokens"));
        let input = input.unwrap_or(&serde_json::Value::Null);
        let output = output.unwrap_or(&serde_json::Value::Null);
        let output_total = output.as_u64().unwrap_or_else(|| count(output, &["total"]));
        let reasoning = count(output, &["reasoning"]);
        let text = count(output, &["text"]);
        Self {
            input_tokens: input.as_u64().unwrap_or_else(|| count(input, &["total"])),
            output_tokens: if output_total == 0 {
                text + reasoning
            } else {
                output_total
            },
            reasoning_tokens: reasoning,
            cached_input_tokens: count(input, &["cacheRead", "cache_read"]),
        }
    }

    pub fn add(&mut self, other: &Self) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.reasoning_tokens += other.reasoning_tokens;
        self.cached_input_tokens += other.cached_input_tokens;
    }

    pub fn is_empty(&self) -> bool {
        self.input_tokens == 0
            && self.output_tokens == 0
            && self.reasoning_tokens == 0
            && self.cached_input_tokens == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_usage_snapshot() {
        let usage = TurnUsage::from_value(&json!({
            "inputTokens": {"total": 100, "cacheRead": 40},
            "outputTokens": {"total": 25, "text": 20, "reasoning": 5}
        }));
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 25);
        assert_eq!(usage.reasoning_tokens, 5);
        assert_eq!(usage.cached_input_tokens, 40);
    }

    #[test]
    fn totals_missing_are_derived() {
        let usage = TurnUsage::from_value(&json!({
            "outputTokens": {"text": 12, "reasoning": 3}
        }));
        assert_eq!(usage.output_tokens, 15);
        assert!(TurnUsage::from_value(&json!({})).is_empty());
    }

    #[test]
    fn add_accumulates() {
        let mut usage = TurnUsage {
            input_tokens: 10,
            output_tokens: 5,
            ..Default::default()
        };
        usage.add(&TurnUsage {
            input_tokens: 3,
            output_tokens: 2,
            ..Default::default()
        });
        assert_eq!(usage.input_tokens, 13);
        assert_eq!(usage.output_tokens, 7);
    }
}
