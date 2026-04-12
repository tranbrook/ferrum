//! Configuration types for Ferrum agents.

use crate::types::TradingPair;
use serde::{Deserialize, Serialize};

/// Risk limits - user-only, agent CANNOT modify
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size_quote: f64,
    pub max_single_order_quote: f64,
    pub max_daily_loss_quote: f64,
    pub max_open_executors: u32,
    pub max_drawdown_pct: f64,
    pub max_cost_per_day_usd: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size_quote: 1000.0,
            max_single_order_quote: 100.0,
            max_daily_loss_quote: 50.0,
            max_open_executors: 10,
            max_drawdown_pct: 10.0,
            max_cost_per_day_usd: 10.0,
        }
    }
}

/// Agent configuration - agent can suggest, user must approve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub tick_interval_secs: u64,
    pub connectors: Vec<String>,
    pub trading_pair: TradingPair,
    pub spread_percentage: Option<f64>,
    pub grid_levels: Option<u32>,
    pub leverage: Option<u32>,
}

/// Agent definition parsed from agent.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub config: AgentConfig,
    pub limits: RiskLimits,
    pub goal: String,
    pub rules: Vec<String>,
}

/// Risk state computed at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskState {
    pub daily_pnl: f64,
    pub total_exposure: f64,
    pub executor_count: u32,
    pub drawdown_pct: f64,
    pub daily_cost: f64,
    pub is_blocked: bool,
    pub block_reason: Option<String>,
}

impl Default for RiskState {
    fn default() -> Self {
        Self {
            daily_pnl: 0.0,
            total_exposure: 0.0,
            executor_count: 0,
            drawdown_pct: 0.0,
            daily_cost: 0.0,
            is_blocked: false,
            block_reason: None,
        }
    }
}

/// Triple Barrier configuration for PositionExecutor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripleBarrierConfig {
    pub take_profit: Option<f64>,      // Percentage
    pub stop_loss: Option<f64>,        // Percentage
    pub time_limit: Option<u64>,       // Seconds
    pub trailing_stop_activation: Option<f64>,
    pub trailing_stop_delta: Option<f64>,
}

impl Default for TripleBarrierConfig {
    fn default() -> Self {
        Self {
            take_profit: Some(0.02),    // 2%
            stop_loss: Some(0.01),      // 1%
            time_limit: Some(3600),     // 1 hour
            trailing_stop_activation: None,
            trailing_stop_delta: None,
        }
    }
}

/// Grid executor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub start_price: f64,
    pub end_price: f64,
    pub levels: u32,
    pub total_amount_quote: f64,
}

/// Global Ferrum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FerrumConfig {
    pub database_path: String,
    pub log_level: String,
    pub api_port: u16,
    #[serde(default)]
    pub exchanges: Vec<ExchangeConfig>,
}

impl Default for FerrumConfig {
    fn default() -> Self {
        Self {
            database_path: "ferrum.db".into(),
            log_level: "info".into(),
            api_port: 8080,
            exchanges: vec![],
        }
    }
}

/// Exchange connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>,
    pub testnet: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_risk_limits() {
        let limits = RiskLimits::default();
        assert_eq!(limits.max_position_size_quote, 1000.0);
        assert_eq!(limits.max_open_executors, 10);
    }

    #[test]
    fn test_default_risk_state() {
        let state = RiskState::default();
        assert!(!state.is_blocked);
        assert_eq!(state.daily_pnl, 0.0);
    }

    #[test]
    fn test_triple_barrier_default() {
        let tb = TripleBarrierConfig::default();
        assert_eq!(tb.take_profit, Some(0.02));
        assert_eq!(tb.stop_loss, Some(0.01));
    }
}
