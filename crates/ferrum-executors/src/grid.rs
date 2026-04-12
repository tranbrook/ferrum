//! Grid Executor - multi-level grid trading.

use async_trait::async_trait;
use ferrum_core::config::GridConfig;
use ferrum_core::error::Result;
use ferrum_core::events::FerrumEvent;
use ferrum_core::traits::{Executor, MarketData};
use ferrum_core::types::*;
use std::time::Instant;

/// Grid executor configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridExecutorConfig {
    pub controller_id: String,
    pub connector: String,
    pub pair: TradingPair,
    pub side: Side,
    pub grid: GridConfig,
    pub keep_position: bool,
}

/// Grid level state
#[derive(Debug, Clone)]
pub struct GridLevel {
    pub price: Price,
    pub amount: Quantity,
    pub filled: bool,
}

/// Grid executor for range-bound markets
pub struct GridExecutor {
    id: ExecutorId,
    config: GridExecutorConfig,
    status: ExecutorStatus,
    levels: Vec<GridLevel>,
    net_pnl: f64,
    created_at: Instant,
}

impl GridExecutor {
    pub fn new(config: GridExecutorConfig) -> Self {
        let levels = Self::generate_levels(&config.grid, config.side);
        Self {
            id: ExecutorId::new(),
            status: ExecutorStatus::Created,
            levels,
            net_pnl: 0.0,
            created_at: Instant::now(),
            config,
        }
    }

    fn generate_levels(grid: &GridConfig, _side: Side) -> Vec<GridLevel> {
        let step = (grid.end_price - grid.start_price) / grid.levels as f64;
        let amount_per_level = grid.total_amount_quote / grid.levels as f64;
        (0..grid.levels)
            .map(|i| {
                let price = grid.start_price + step * (i as f64 + 0.5);
                GridLevel {
                    price: Price(price),
                    amount: Quantity(amount_per_level / price),
                    filled: false,
                }
            })
            .collect()
    }

    pub fn filled_count(&self) -> usize {
        self.levels.iter().filter(|l| l.filled).count()
    }
}

#[async_trait]
impl Executor for GridExecutor {
    fn id(&self) -> &ExecutorId { &self.id }
    fn executor_type(&self) -> ExecutorType { ExecutorType::Grid }
    fn status(&self) -> &ExecutorStatus { &self.status }
    fn controller_id(&self) -> &str { &self.config.controller_id }

    async fn tick(&mut self, market: &MarketData) -> Result<Vec<FerrumEvent>> {
        let mut events = Vec::new();
        if let Some(ob) = &market.orderbook {
            if let Some(current_price) = ob.mid_price() {
                if matches!(self.status, ExecutorStatus::Created) {
                    self.status = ExecutorStatus::Active;
                    events.push(FerrumEvent::ExecutorActivated { executor_id: self.id.clone() });
                }
                // Check if any grid level is hit
                for level in &mut self.levels {
                    if !level.filled && current_price.0 <= level.price.0 {
                        level.filled = true;
                    }
                }
            }
        }
        Ok(events)
    }

    fn metrics(&self) -> ExecutorMetrics {
        ExecutorMetrics {
            executor_id: self.id.clone(),
            controller_id: self.config.controller_id.clone(),
            executor_type: ExecutorType::Grid,
            net_pnl_quote: self.net_pnl,
            fees_paid_quote: 0.0,
            fees_earned_quote: 0.0,
            value_quote: 0.0,
            volume_quote: 0.0,
            duration_seconds: self.created_at.elapsed().as_secs(),
            close_type: match &self.status {
                ExecutorStatus::Terminated(ct) => Some(*ct),
                _ => None,
            },
        }
    }

    fn keep_position(&self) -> bool { self.config.keep_position }
    async fn stop(&mut self) -> Result<()> {
        self.status = ExecutorStatus::Terminated(CloseType::EarlyStop);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_level_generation() {
        let config = GridExecutorConfig {
            controller_id: "test".into(),
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            grid: GridConfig {
                start_price: 90000.0,
                end_price: 100000.0,
                levels: 5,
                total_amount_quote: 1000.0,
            },
            keep_position: true,
        };
        let executor = GridExecutor::new(config);
        assert_eq!(executor.levels.len(), 5);
        assert!(executor.levels[0].price.0 > 90000.0);
    }
}
