//! Position tracker - tracks virtual portfolio positions.

use ferrum_core::types::*;
use std::collections::HashMap;
use parking_lot::RwLock;

/// Portfolio position summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortfolioSummary {
    pub total_value_quote: f64,
    pub total_unrealized_pnl: f64,
    pub total_realized_pnl: f64,
    pub positions: Vec<Position>,
    pub balances: Vec<Balance>,
}

/// Tracks all positions across connectors
pub struct PositionTracker {
    positions: RwLock<HashMap<String, Position>>,
}

impl PositionTracker {
    pub fn new() -> Self {
        Self { positions: RwLock::new(HashMap::new()) }
    }

    pub fn update_position(&self, position: Position) {
        self.positions.write().insert(position.id.clone(), position);
    }

    pub fn remove_position(&self, id: &str) -> Option<Position> {
        self.positions.write().remove(id)
    }

    pub fn get_position(&self, id: &str) -> Option<Position> {
        self.positions.read().get(id).cloned()
    }

    pub fn all_positions(&self) -> Vec<Position> {
        self.positions.read().values().cloned().collect()
    }

    pub fn positions_by_pair(&self, pair: &TradingPair) -> Vec<Position> {
        self.positions.read().values()
            .filter(|p| p.pair == *pair)
            .cloned()
            .collect()
    }

    pub fn positions_by_connector(&self, connector: &str) -> Vec<Position> {
        self.positions.read().values()
            .filter(|p| p.connector == connector)
            .cloned()
            .collect()
    }

    pub fn total_exposure(&self) -> f64 {
        self.positions.read().values()
            .map(|p| p.amount.0 * p.entry_price.0)
            .sum()
    }

    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.read().values()
            .map(|p| p.unrealized_pnl)
            .sum()
    }

    pub fn portfolio_summary(&self, balances: Vec<Balance>) -> PortfolioSummary {
        let positions = self.all_positions();
        let total_unrealized = positions.iter().map(|p| p.unrealized_pnl).sum();
        let total_realized = positions.iter().map(|p| p.realized_pnl).sum();
        let total_value = balances.iter().map(|b| b.total().0).sum(); // Simplified

        PortfolioSummary {
            total_value_quote: total_value,
            total_unrealized_pnl: total_unrealized,
            total_realized_pnl: total_realized,
            positions,
            balances,
        }
    }
}

impl Default for PositionTracker {
    fn default() -> Self { Self::new() }
}
