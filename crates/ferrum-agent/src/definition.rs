//! Agent definition parser - parses agent.md files.

use ferrum_core::config::{AgentConfig, AgentDefinition, RiskLimits};
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::types::TradingPair;
use std::str::FromStr;

/// Parser for agent.md YAML frontmatter + Markdown format
pub struct AgentDefinitionParser;

impl AgentDefinitionParser {
    /// Parse agent.md file content into AgentDefinition
    pub fn parse(content: &str) -> Result<AgentDefinition> {
        // Split YAML frontmatter from markdown body
        let (frontmatter, body) = Self::split_frontmatter(content)?;

        // Parse YAML config
        let config_value: serde_yaml::Value = serde_yaml::from_str(&frontmatter)
            .map_err(|e| FerrumError::ConfigError(format!("Invalid YAML frontmatter: {}", e)))?;

        let name = config_value.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed-agent")
            .to_string();

        let tick_interval = config_value.get("tick_interval_secs")
            .and_then(|v| v.as_i64())
            .unwrap_or(60) as u64;

        let connectors = config_value.get("connectors")
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let trading_pair_str = config_value.get("trading_pair")
            .and_then(|v| v.as_str())
            .unwrap_or("BTC-USDT");
        let trading_pair = TradingPair::from_str(trading_pair_str)
            .map_err(|e| FerrumError::ConfigError(format!("Invalid trading_pair: {:?}", e)))?;

        // Parse configs (agent-suggestible)
        let spread = config_value.get("spread_percentage")
            .and_then(|v| v.as_f64());
        let grid_levels = config_value.get("grid_levels")
            .and_then(|v| v.as_i64())
            .map(|v| v as u32);
        let leverage = config_value.get("leverage")
            .and_then(|v| v.as_i64())
            .map(|v| v as u32);

        // Parse limits (user-only guardrails)
        let limits = config_value.get("limits")
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_default();

        // Parse markdown body for goal and rules
        let (goal, rules) = Self::parse_markdown_body(&body);

        let config = AgentConfig {
            name: name.clone(),
            tick_interval_secs: tick_interval,
            connectors,
            trading_pair,
            spread_percentage: spread,
            grid_levels,
            leverage,
        };

        Ok(AgentDefinition {
            name,
            config,
            limits,
            goal,
            rules,
        })
    }

    fn split_frontmatter(content: &str) -> Result<(String, String)> {
        let trimmed = content.trim();
        if !trimmed.starts_with("---") {
            return Err(FerrumError::ConfigError("Missing YAML frontmatter".into()));
        }
        let rest = &trimmed[3..];
        if let Some(end) = rest.find("---") {
            Ok((rest[..end].to_string(), rest[end + 3..].to_string()))
        } else {
            Err(FerrumError::ConfigError("Unclosed YAML frontmatter".into()))
        }
    }

    fn parse_markdown_body(body: &str) -> (String, Vec<String>) {
        let mut goal = String::new();
        let mut rules = Vec::new();
        let mut in_goal = false;
        let mut in_rules = false;

        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            
            if trimmed == "## Goal" {
                in_goal = true;
                in_rules = false;
                continue;
            }
            if trimmed == "## Rules" || trimmed == "## Strategy" || trimmed == "## Strategy Rules" {
                in_goal = false;
                in_rules = true;
                continue;
            }
            if trimmed.starts_with("## ") {
                in_goal = false;
                in_rules = false;
                continue;
            }
            
            if in_goal && goal.is_empty() {
                goal = trimmed.to_string();
                in_goal = false;
                continue;
            }
            
            if (in_rules) && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
                let rule = trimmed.trim_start_matches("- ").trim_start_matches("* ").to_string();
                if !rule.is_empty() {
                    rules.push(rule);
                }
            }
        }

        (goal, rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent_md() {
        let content = r#"---
name: grid-market-maker
tick_interval_secs: 30
connectors:
  - binance
trading_pair: BTC-USDT
spread_percentage: 0.5
limits:
  max_position_size_quote: 1000
  max_single_order_quote: 100
  max_daily_loss_quote: 50
  max_open_executors: 10
---

## Goal
Maintain a grid market making strategy on BTC-USDT

## Rules
- Place buy orders below mid price
- Place sell orders above mid price
- Never exceed 50 USDT daily loss
- Close all positions if drawdown exceeds 10%
"#;
        let def = AgentDefinitionParser::parse(content).unwrap();
        assert_eq!(def.name, "grid-market-maker");
        assert_eq!(def.config.tick_interval_secs, 30);
        assert_eq!(def.config.connectors, vec!["binance"]);
        assert_eq!(def.config.trading_pair, TradingPair::new("BTC", "USDT"));
        assert_eq!(def.limits.max_daily_loss_quote, 50.0);
        assert_eq!(def.rules.len(), 4);
    }
}
