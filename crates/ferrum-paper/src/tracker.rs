//! Position tracker for paper trading.

use ferrum_core::types::*;
use std::collections::HashMap;
use crate::types::{PaperAccount, PaperPosition};

/// Tracks positions and balances for paper trading.
pub struct PositionTracker {
    account: PaperAccount,
}

impl PositionTracker {
    pub fn new(balances: Vec<(String, f64)>) -> Self {
        let balance_map: HashMap<String, f64> = balances.into_iter().collect();
        Self {
            account: PaperAccount {
                balances: balance_map,
                positions: Vec::new(),
                open_orders: Vec::new(),
                total_pnl: 0.0,
                realized_pnl: 0.0,
                unrealized_pnl: 0.0,
                total_fees: 0.0,
                trade_count: 0,
            },
        }
    }

    /// Get a reference to the account.
    pub fn account(&self) -> &PaperAccount {
        &self.account
    }

    /// Get the balance of a specific asset.
    pub fn balance(&self, asset: &str) -> f64 {
        *self.account.balances.get(asset).unwrap_or(&0.0)
    }

    /// Modify the balance of an asset.
    pub fn modify_balance(&mut self, asset: &str, delta: f64) {
        let current = self.balance(asset);
        self.account.balances.insert(asset.to_string(), (current + delta).max(0.0));
    }

    /// Update position after a fill.
    ///
    /// Fee handling: The fee is already deducted from the balance in the engine.
    /// Here we only track fee for accounting purposes (total_fees). PnL is
    /// calculated purely from price difference * amount, without re-deducting fee.
    pub fn update_position(&mut self, pair: TradingPair, side: Side, amount: f64, price: f64, fee: f64) {
        self.account.total_fees += fee;
        self.account.trade_count += 1;

        if let Some(pos) = self.account.positions.iter_mut().find(|p| p.pair == pair) {
            if pos.side == side {
                // Adding to existing position in same direction - average entry price
                let total_cost = pos.entry_price * pos.amount + price * amount;
                pos.amount += amount;
                pos.entry_price = total_cost / pos.amount;
                pos.current_price = price;
            } else {
                // Closing or reducing position in opposite direction
                let close_amount = amount.min(pos.amount);
                let pnl = match pos.side {
                    Side::Buy => (price - pos.entry_price) * close_amount,
                    Side::Sell => (pos.entry_price - price) * close_amount,
                    _ => 0.0,
                };

                // Deduct fee from PnL (fee was already deducted from balance, this is for accounting)
                let net_pnl = pnl - fee;

                self.account.realized_pnl += net_pnl;
                self.account.total_pnl += net_pnl;
                pos.realized_pnl += net_pnl;
                pos.amount -= close_amount;

                // If closing more than position (reversal), the remainder opens a new position
                if amount > pos.amount {
                    let remaining = amount - close_amount;
                    // Position will be cleaned up below, new one added
                    if remaining > 0.0 {
                        self.account.positions.push(PaperPosition {
                            pair: pair.clone(),
                            side,
                            amount: remaining,
                            entry_price: price,
                            current_price: price,
                            unrealized_pnl: 0.0,
                            realized_pnl: 0.0,
                        });
                    }
                }
            }
        } else {
            // New position
            self.account.positions.push(PaperPosition {
                pair,
                side,
                amount,
                entry_price: price,
                current_price: price,
                unrealized_pnl: 0.0,
                realized_pnl: 0.0,
            });
        }

        // Remove zero-amount positions
        self.account.positions.retain(|p| p.amount > 0.0);
    }

    /// Mark positions to market with current prices.
    pub fn mark_to_market(&mut self, prices: &HashMap<String, f64>) {
        let mut total_unrealized = 0.0;
        for pos in &mut self.account.positions {
            let symbol = format!("{}{}", pos.pair.base, pos.pair.quote);
            if let Some(&price) = prices.get(&symbol) {
                pos.unrealized_pnl = match pos.side {
                    Side::Buy => (price - pos.entry_price) * pos.amount,
                    Side::Sell => (pos.entry_price - price) * pos.amount,
                    _ => 0.0,
                };
                pos.current_price = price;
                total_unrealized += pos.unrealized_pnl;
            }
        }
        self.account.unrealized_pnl = total_unrealized;
    }

    /// Get total portfolio value.
    pub fn portfolio_value(&self, prices: &HashMap<String, f64>) -> f64 {
        let mut value: f64 = self.account.balances.values().sum();
        for pos in &self.account.positions {
            value += pos.current_price * pos.amount;
        }
        value + self.account.unrealized_pnl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_tracker_new() {
        let tracker = PositionTracker::new(vec![("USDT".to_string(), 10000.0)]);
        assert_eq!(tracker.balance("USDT"), 10000.0);
    }

    #[test]
    fn test_modify_balance() {
        let mut tracker = PositionTracker::new(vec![("USDT".to_string(), 10000.0)]);
        tracker.modify_balance("USDT", -1000.0);
        assert_eq!(tracker.balance("USDT"), 9000.0);
        tracker.modify_balance("BTC", 0.5);
        assert_eq!(tracker.balance("BTC"), 0.5);
    }

    #[test]
    fn test_open_and_close_position() {
        let mut tracker = PositionTracker::new(vec![("USDT".to_string(), 10000.0)]);
        let pair = TradingPair::new("BTC", "USDT");

        // Open long at 50000
        tracker.update_position(pair.clone(), Side::Buy, 0.1, 50000.0, 5.0);
        assert_eq!(tracker.account().positions.len(), 1);
        assert_eq!(tracker.account().positions[0].entry_price, 50000.0);

        // Close long at 55000
        tracker.update_position(pair.clone(), Side::Sell, 0.1, 55000.0, 5.5);
        assert_eq!(tracker.account().positions.len(), 0);
        // PnL: (55000 - 50000) * 0.1 - 5.5 = 500 * 0.1 - 5.5 = 50 - 5.5 = 44.5
        let expected_pnl = (55000.0 - 50000.0) * 0.1 - 5.5;
        assert!(
            (tracker.account().realized_pnl - expected_pnl).abs() < 0.01,
            "Expected PnL ~{}, got {}",
            expected_pnl,
            tracker.account().realized_pnl
        );
    }

    #[test]
    fn test_add_to_existing_position() {
        let mut tracker = PositionTracker::new(vec![("USDT".to_string(), 10000.0)]);
        let pair = TradingPair::new("BTC", "USDT");

        tracker.update_position(pair.clone(), Side::Buy, 0.1, 50000.0, 5.0);
        tracker.update_position(pair.clone(), Side::Buy, 0.1, 60000.0, 6.0);

        assert_eq!(tracker.account().positions.len(), 1);
        let pos = &tracker.account().positions[0];
        assert_eq!(pos.amount, 0.2);
        // Average entry: (50000*0.1 + 60000*0.1) / 0.2 = 55000
        assert!(
            (pos.entry_price - 55000.0).abs() < 0.01,
            "Expected avg entry 55000, got {}",
            pos.entry_price
        );
    }

    #[test]
    fn test_partial_close() {
        let mut tracker = PositionTracker::new(vec![("USDT".to_string(), 10000.0)]);
        let pair = TradingPair::new("BTC", "USDT");

        // Open 0.2 BTC at 50000
        tracker.update_position(pair.clone(), Side::Buy, 0.2, 50000.0, 10.0);
        // Close 0.1 BTC at 55000
        tracker.update_position(pair.clone(), Side::Sell, 0.1, 55000.0, 5.5);

        // Should still have 0.1 BTC
        assert_eq!(tracker.account().positions.len(), 1);
        assert_eq!(tracker.account().positions[0].amount, 0.1);

        // Realized PnL: (55000 - 50000) * 0.1 - 5.5 = 50 - 5.5 = 44.5
        let expected_pnl = (55000.0 - 50000.0) * 0.1 - 5.5;
        assert!(
            (tracker.account().realized_pnl - expected_pnl).abs() < 0.01,
            "Expected PnL ~{}, got {}",
            expected_pnl,
            tracker.account().realized_pnl
        );
    }
}
