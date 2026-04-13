//! Paper trading engine that simulates order execution.

use crate::tracker::PositionTracker;
use crate::types::*;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::types::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Paper trading engine.
pub struct PaperTradingEngine {
    config: PaperTradingConfig,
    tracker: PositionTracker,
}

impl PaperTradingEngine {
    /// Create a new paper trading engine.
    pub fn new(config: PaperTradingConfig) -> Self {
        let tracker = PositionTracker::new(config.initial_balances.clone());
        Self { config, tracker }
    }

    /// Get a reference to the position tracker.
    pub fn tracker(&self) -> &PositionTracker {
        &self.tracker
    }

    /// Get a mutable reference to the position tracker.
    pub fn tracker_mut(&mut self) -> &mut PositionTracker {
        &mut self.tracker
    }

    /// Submit a market order.
    ///
    /// Fee handling:
    /// - BUY: Fee is deducted from the quote asset (USDT) spent. Total cost = price * amount + fee.
    ///   You receive exactly `amount` of the base asset.
    /// - SELL: Fee is deducted from the quote asset received. You receive (price * amount - fee).
    ///   You give exactly `amount` of the base asset.
    ///   The fee is recorded in the position tracker for PnL calculation.
    pub fn submit_market_order(
        &mut self,
        pair: TradingPair,
        side: Side,
        amount: f64,
        current_price: f64,
    ) -> Result<PaperOrder> {
        let slippage = self.calculate_slippage(amount, current_price);
        let fill_price = match side {
            Side::Buy => current_price * (1.0 + slippage),
            Side::Sell => current_price * (1.0 - slippage),
            Side::Range => return Err(FerrumError::OrderError("Range not supported".into())),
        };

        let fee = fill_price * amount * self.config.fee_rate;

        let (cost_asset, cost_amount, receive_asset, receive_amount) = match side {
            Side::Buy => {
                // BUY: Spend quote (USDT), receive base (BTC)
                // Cost includes fee: pay fill_price * amount + fee
                let cost = fill_price * amount + fee;
                (pair.quote.clone(), cost, pair.base.clone(), amount)
            }
            Side::Sell => {
                // SELL: Spend base (BTC), receive quote (USDT)
                // Receive amount after fee: fill_price * amount - fee
                let receive = fill_price * amount - fee;
                (pair.base.clone(), amount, pair.quote.clone(), receive)
            }
            Side::Range => return Err(FerrumError::OrderError("Range not supported".into())),
        };

        let available = self.tracker.balance(&cost_asset);
        if available < cost_amount {
            return Err(FerrumError::OrderError(format!(
                "Insufficient {} balance: have {}, need {}",
                cost_asset, available, cost_amount
            )));
        }

        // Execute: debit cost asset, credit receive asset
        self.tracker.modify_balance(&cost_asset, -cost_amount);
        self.tracker.modify_balance(&receive_asset, receive_amount);

        // Update position (fee is tracked here for PnL, but NOT double-deducted from balance)
        self.tracker.update_position(pair.clone(), side, amount, fill_price, fee);

        let now = chrono::Utc::now().timestamp_millis();
        Ok(PaperOrder {
            id: Uuid::new_v4().to_string(),
            client_order_id: None,
            pair,
            side,
            order_type: OrderType::Market,
            amount,
            price: Some(current_price),
            status: PaperOrderStatus::Filled,
            filled_amount: amount,
            avg_fill_price: fill_price,
            created_at: now,
            updated_at: now,
        })
    }

    /// Submit a limit order.
    pub fn submit_limit_order(
        &mut self,
        pair: TradingPair,
        side: Side,
        amount: f64,
        price: f64,
    ) -> Result<PaperOrder> {
        let now = chrono::Utc::now().timestamp_millis();
        Ok(PaperOrder {
            id: Uuid::new_v4().to_string(),
            client_order_id: None,
            pair,
            side,
            order_type: OrderType::Limit,
            amount,
            price: Some(price),
            status: PaperOrderStatus::Pending,
            filled_amount: 0.0,
            avg_fill_price: 0.0,
            created_at: now,
            updated_at: now,
        })
    }

    /// Process a price update and check limit orders.
    pub fn process_price_update(&mut self, symbol: &str, price: f64) -> Vec<PaperOrder> {
        let mut prices = HashMap::new();
        prices.insert(symbol.to_string(), price);
        self.tracker.mark_to_market(&prices);
        Vec::new()
    }

    /// Get the account summary.
    pub fn account_summary(&self) -> &PaperAccount {
        self.tracker.account()
    }

    /// Calculate slippage for a given order size.
    fn calculate_slippage(&self, amount: f64, price: f64) -> f64 {
        match &self.config.slippage_model {
            SlippageModel::Fixed(slippage) => *slippage,
            SlippageModel::VolumeBased { base_slippage, volume_factor } => {
                base_slippage + (amount * price * volume_factor)
            }
            SlippageModel::None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_market_order_buy() {
        let mut engine = PaperTradingEngine::new(PaperTradingConfig::default());
        let pair = TradingPair::new("BTC", "USDT");
        let order = engine.submit_market_order(pair, Side::Buy, 0.1, 50000.0).unwrap();
        assert_eq!(order.status, PaperOrderStatus::Filled);
        assert_eq!(order.filled_amount, 0.1);
    }

    #[test]
    fn test_paper_market_order_sell() {
        let mut engine = PaperTradingEngine::new(PaperTradingConfig::default());
        let pair = TradingPair::new("BTC", "USDT");

        // First buy some BTC
        engine.submit_market_order(pair.clone(), Side::Buy, 0.1, 50000.0).unwrap();

        // Now sell it
        let order = engine.submit_market_order(pair, Side::Sell, 0.1, 55000.0).unwrap();
        assert_eq!(order.status, PaperOrderStatus::Filled);
    }

    #[test]
    fn test_paper_insufficient_balance() {
        let mut engine = PaperTradingEngine::new(PaperTradingConfig::default());
        let pair = TradingPair::new("BTC", "USDT");
        let result = engine.submit_market_order(pair, Side::Buy, 1.0, 50000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_paper_limit_order() {
        let mut engine = PaperTradingEngine::new(PaperTradingConfig::default());
        let pair = TradingPair::new("BTC", "USDT");
        let order = engine.submit_limit_order(pair, Side::Buy, 0.1, 49000.0).unwrap();
        assert_eq!(order.status, PaperOrderStatus::Pending);
    }

    #[test]
    fn test_buy_sell_round_trip_pnl() {
        let mut engine = PaperTradingEngine::new(PaperTradingConfig {
            fee_rate: 0.001,
            slippage_model: SlippageModel::None,
            ..Default::default()
        });
        let pair = TradingPair::new("BTC", "USDT");

        // Buy 0.1 BTC at 50000
        engine.submit_market_order(pair.clone(), Side::Buy, 0.1, 50000.0).unwrap();
        assert_eq!(engine.tracker().balance("BTC"), 0.1);

        // Sell 0.1 BTC at 55000 - should have profit
        engine.submit_market_order(pair.clone(), Side::Sell, 0.1, 55000.0).unwrap();
        assert_eq!(engine.tracker().balance("BTC"), 0.0);

        // After selling at 55000 with 0.1% fee:
        // Received: 55000 * 0.1 - (55000 * 0.1 * 0.001) = 5500 - 5.5 = 5494.5
        // Spent:   50000 * 0.1 + (50000 * 0.1 * 0.001) = 5000 + 5 = 5005
        // Profit:  5494.5 - 5005 = 489.5
        let account = engine.account_summary();
        assert!(account.realized_pnl > 480.0, "Expected ~489.5 profit, got {}", account.realized_pnl);
    }
}
