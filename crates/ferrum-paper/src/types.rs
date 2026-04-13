//! Paper trading types.

use ferrum_core::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Paper trading configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradingConfig {
    /// Initial balances per asset.
    pub initial_balances: Vec<(String, f64)>,
    /// Trading fee rate.
    pub fee_rate: f64,
    /// Slippage model.
    pub slippage_model: SlippageModel,
    /// Enable or disable short selling.
    pub allow_shorting: bool,
    /// Maximum position size as fraction of portfolio.
    pub max_position_fraction: f64,
}

impl Default for PaperTradingConfig {
    fn default() -> Self {
        Self {
            initial_balances: vec![("USDT".to_string(), 10000.0)],
            fee_rate: 0.001,
            slippage_model: SlippageModel::Fixed(0.0005),
            allow_shorting: false,
            max_position_fraction: 0.25,
        }
    }
}

/// Slippage model for simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlippageModel {
    /// Fixed slippage (percentage).
    Fixed(f64),
    /// Volume-based slippage.
    VolumeBased { base_slippage: f64, volume_factor: f64 },
    /// No slippage.
    None,
}

/// Paper order state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperOrder {
    pub id: String,
    pub client_order_id: Option<String>,
    pub pair: TradingPair,
    pub side: Side,
    pub order_type: OrderType,
    pub amount: f64,
    pub price: Option<f64>,
    pub status: PaperOrderStatus,
    pub filled_amount: f64,
    pub avg_fill_price: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Status of a paper order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

/// Paper trading account state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAccount {
    pub balances: HashMap<String, f64>,
    pub positions: Vec<PaperPosition>,
    pub open_orders: Vec<PaperOrder>,
    pub total_pnl: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_fees: f64,
    pub trade_count: usize,
}

/// A paper trading position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPosition {
    pub pair: TradingPair,
    pub side: Side,
    pub amount: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_config_default() {
        let config = PaperTradingConfig::default();
        assert_eq!(config.fee_rate, 0.001);
    }
}
