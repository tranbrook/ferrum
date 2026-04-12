//! Position Executor - directional trades with Triple Barrier exit.

use async_trait::async_trait;
use ferrum_core::config::TripleBarrierConfig;
use ferrum_core::error::Result;
use ferrum_core::events::FerrumEvent;
use ferrum_core::traits::{Executor, MarketData};
use ferrum_core::types::*;
use crate::triple_barrier::*;
use std::time::Instant;

/// Position executor configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PositionExecutorConfig {
    pub controller_id: String,
    pub connector: String,
    pub pair: TradingPair,
    pub side: Side,
    pub amount: Quantity,
    pub entry_price: Option<Price>,
    pub leverage: Option<u32>,
    pub triple_barrier: TripleBarrierConfig,
    pub keep_position: bool,
}

/// Position executor for directional trades
pub struct PositionExecutor {
    id: ExecutorId,
    config: PositionExecutorConfig,
    status: ExecutorStatus,
    entry_price: Option<Price>,
    current_price: Option<Price>,
    filled_quantity: Quantity,
    created_at: Instant,
    trailing_state: Option<TrailingStopState>,
    net_pnl: f64,
    fees_paid: f64,
    volume: f64,
}

impl PositionExecutor {
    pub fn new(config: PositionExecutorConfig) -> Self {
        Self {
            id: ExecutorId::new(),
            status: ExecutorStatus::Created,
            entry_price: config.entry_price,
            current_price: None,
            filled_quantity: Quantity::zero(),
            created_at: Instant::now(),
            trailing_state: None,
            net_pnl: 0.0,
            fees_paid: 0.0,
            volume: 0.0,
            config,
        }
    }

    pub fn config(&self) -> &PositionExecutorConfig { &self.config }

    pub fn entry_price(&self) -> Option<Price> { self.entry_price }

    pub fn unrealized_pnl(&self) -> f64 {
        match (self.entry_price, self.current_price) {
            (Some(entry), Some(current)) => {
                let pnl_pct = match self.config.side {
                    Side::Buy => (current.0 - entry.0) / entry.0,
                    Side::Sell => (entry.0 - current.0) / entry.0,
                    Side::Range => 0.0,
                };
                pnl_pct * self.config.amount.0 * entry.0
            }
            _ => 0.0,
        }
    }

    fn check_exit_conditions(&mut self) -> Option<CloseType> {
        let entry = match self.entry_price {
            Some(p) => p.0,
            None => return None,
        };
        let current = match self.current_price {
            Some(p) => p.0,
            None => return None,
        };
        let elapsed = self.created_at.elapsed().as_secs();

        let result = check_triple_barrier(
            &self.config.triple_barrier,
            entry,
            current,
            self.config.side,
            elapsed,
            &mut self.trailing_state,
        );

        match result {
            BarrierResult::Active => None,
            BarrierResult::TakeProfit => Some(CloseType::TakeProfit),
            BarrierResult::StopLoss => Some(CloseType::StopLoss),
            BarrierResult::TimeLimit => Some(CloseType::TimeLimit),
            BarrierResult::TrailingStop => Some(CloseType::TrailingStop),
        }
    }
}

#[async_trait]
impl Executor for PositionExecutor {
    fn id(&self) -> &ExecutorId { &self.id }

    fn executor_type(&self) -> ExecutorType { ExecutorType::Position }

    fn status(&self) -> &ExecutorStatus { &self.status }

    fn controller_id(&self) -> &str { &self.config.controller_id }

    async fn tick(&mut self, market: &MarketData) -> Result<Vec<FerrumEvent>> {
        let mut events = Vec::new();

        // Update current price from market data
        if let Some(ob) = &market.orderbook {
            if let Some(mid) = ob.mid_price() {
                self.current_price = Some(mid);

                // If just created and no entry price, use market price
                if matches!(self.status, ExecutorStatus::Created) {
                    self.entry_price = Some(mid);
                    self.filled_quantity = self.config.amount;
                    self.status = ExecutorStatus::Active;
                    events.push(FerrumEvent::ExecutorActivated {
                        executor_id: self.id.clone(),
                    });
                }
            }
        }

        // Check exit conditions if active
        if matches!(self.status, ExecutorStatus::Active) {
            if let Some(close_type) = self.check_exit_conditions() {
                self.net_pnl = self.unrealized_pnl() - self.fees_paid;
                self.status = ExecutorStatus::Terminated(close_type);
                events.push(FerrumEvent::ExecutorTerminated {
                    executor_id: self.id.clone(),
                    close_type,
                    pnl: self.net_pnl,
                });
            }
        }

        Ok(events)
    }

    fn metrics(&self) -> ExecutorMetrics {
        ExecutorMetrics {
            executor_id: self.id.clone(),
            controller_id: self.config.controller_id.clone(),
            executor_type: ExecutorType::Position,
            net_pnl_quote: self.net_pnl,
            fees_paid_quote: self.fees_paid,
            fees_earned_quote: 0.0,
            value_quote: self.unrealized_pnl(),
            volume_quote: self.volume,
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

    fn make_config() -> PositionExecutorConfig {
        PositionExecutorConfig {
            controller_id: "test-agent".into(),
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            amount: Quantity(0.01),
            entry_price: Some(Price(100000.0)),
            leverage: Some(1),
            triple_barrier: TripleBarrierConfig {
                take_profit: Some(0.02),
                stop_loss: Some(0.01),
                time_limit: Some(3600),
                trailing_stop_activation: None,
                trailing_stop_delta: None,
            },
            keep_position: false,
        }
    }

    fn make_market(price: f64) -> MarketData {
        MarketData {
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            orderbook: Some(OrderBook {
                pair: TradingPair::new("BTC", "USDT"),
                bids: vec![OrderBookLevel { price: Price(price - 1.0), quantity: Quantity(1.0) }],
                asks: vec![OrderBookLevel { price: Price(price + 1.0), quantity: Quantity(1.0) }],
                timestamp: 0,
            }),
            latest_candles: vec![],
            timestamp: 0,
        }
    }

    #[tokio::test]
    async fn test_position_executor_created_to_active() {
        let config = make_config();
        let mut executor = PositionExecutor::new(config);
        assert!(matches!(executor.status(), ExecutorStatus::Created));

        let market = make_market(100000.0);
        let events = executor.tick(&market).await.unwrap();
        assert!(matches!(executor.status(), ExecutorStatus::Active));
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_position_executor_stop() {
        let config = make_config();
        let mut executor = PositionExecutor::new(config);
        executor.stop().await.unwrap();
        assert!(matches!(executor.status(), ExecutorStatus::Terminated(CloseType::EarlyStop)));
    }
}
