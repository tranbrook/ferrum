//! Order Executor - simple limit/market order execution.

use async_trait::async_trait;
use ferrum_core::error::Result;
use ferrum_core::events::FerrumEvent;
use ferrum_core::traits::{Executor, MarketData};
use ferrum_core::types::*;
use std::time::Instant;

/// Order executor configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderExecutorConfig {
    pub controller_id: String,
    pub connector: String,
    pub pair: TradingPair,
    pub side: Side,
    pub amount: Quantity,
    pub price: Option<Price>,
    pub order_type: OrderType,
}

/// Order executor for limit/market orders
pub struct OrderExecutor {
    id: ExecutorId,
    config: OrderExecutorConfig,
    status: ExecutorStatus,
    filled_quantity: Quantity,
    avg_fill_price: Option<Price>,
    created_at: Instant,
}

impl OrderExecutor {
    pub fn new(config: OrderExecutorConfig) -> Self {
        Self {
            id: ExecutorId::new(),
            status: ExecutorStatus::Created,
            filled_quantity: Quantity::zero(),
            avg_fill_price: None,
            created_at: Instant::now(),
            config,
        }
    }
}

#[async_trait]
impl Executor for OrderExecutor {
    fn id(&self) -> &ExecutorId { &self.id }
    fn executor_type(&self) -> ExecutorType { ExecutorType::Order }
    fn status(&self) -> &ExecutorStatus { &self.status }
    fn controller_id(&self) -> &str { &self.config.controller_id }

    async fn tick(&mut self, market: &MarketData) -> Result<Vec<FerrumEvent>> {
        let mut events = Vec::new();
        if let Some(ob) = &market.orderbook {
            if matches!(self.status, ExecutorStatus::Created) {
                self.status = ExecutorStatus::Active;
                events.push(FerrumEvent::ExecutorActivated { executor_id: self.id.clone() });
                // Simulate immediate fill for market orders
                if self.config.order_type == OrderType::Market {
                    if let Some(mid) = ob.mid_price() {
                        self.filled_quantity = self.config.amount;
                        self.avg_fill_price = Some(mid);
                        self.status = ExecutorStatus::Terminated(CloseType::TakeProfit);
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
            executor_type: ExecutorType::Order,
            net_pnl_quote: 0.0,
            fees_paid_quote: 0.0,
            fees_earned_quote: 0.0,
            value_quote: self.filled_quantity.0 * self.avg_fill_price.map_or(0.0, |p| p.0),
            volume_quote: self.filled_quantity.0 * self.avg_fill_price.map_or(0.0, |p| p.0),
            duration_seconds: self.created_at.elapsed().as_secs(),
            close_type: match &self.status {
                ExecutorStatus::Terminated(ct) => Some(*ct),
                _ => None,
            },
        }
    }

    fn keep_position(&self) -> bool { true }
    async fn stop(&mut self) -> Result<()> {
        self.status = ExecutorStatus::Terminated(CloseType::EarlyStop);
        Ok(())
    }
}
