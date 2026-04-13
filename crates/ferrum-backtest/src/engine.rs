//! Backtest engine - the core backtesting loop.

use crate::metrics::BacktestMetrics;
use crate::strategy::BacktestStrategy;
use crate::types::*;
use ferrum_core::error::Result;
use ferrum_core::types::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Backtesting engine.
pub struct BacktestEngine {
    config: BacktestConfig,
    strategy: Box<dyn BacktestStrategy>,
}

impl BacktestEngine {
    /// Create a new backtest engine.
    pub fn new(config: BacktestConfig, strategy: Box<dyn BacktestStrategy>) -> Self {
        Self { config, strategy }
    }

    /// Run the backtest against historical candle data.
    pub async fn run(&self, candles: Vec<Candle>) -> Result<BacktestResult> {
        let mut balance = self.config.initial_balance;
        let mut position: Option<BacktestTrade> = None;
        let mut trades: Vec<BacktestTrade> = Vec::new();
        let mut equity_curve: Vec<(i64, f64)> = Vec::with_capacity(candles.len());
        let mut orders_placed = 0;

        tracing::info!(
            "Starting backtest: {} on {} candles",
            self.strategy.name(),
            candles.len()
        );

        for (i, candle) in candles.iter().enumerate() {
            // Build market context — use a slice reference instead of cloning (O(n²) fix)
            let historical = &candles[..=i];
            let indicators = self.strategy.compute_indicators(historical);

            // Calculate current portfolio value (balance + unrealized PnL)
            let current_value = balance + position.as_ref().map_or(0.0, |p| {
                let unrealized = match p.side {
                    Side::Buy => (candle.close.0 - p.entry_price) * p.amount,
                    Side::Sell => (p.entry_price - candle.close.0) * p.amount,
                    _ => 0.0,
                };
                unrealized
            });

            equity_curve.push((candle.timestamp, current_value));

            let ctx = MarketContext {
                current_candle: candle.clone(),
                historical_candles: historical.to_vec(), // Strategy may need owned data
                balance,
                position: position.clone(),
                indicators: indicators.clone(),
            };

            // Get signal from strategy
            let signal = self.strategy.evaluate(&ctx).await?;

            match signal {
                TradeSignal::Buy => {
                    if position.is_none() {
                        let price = candle.close.0 * (1.0 + self.config.slippage);
                        let fee = balance * self.config.fee_rate;
                        let invest_amount = balance - fee;
                        let amount = invest_amount / price;

                        let trade = BacktestTrade {
                            id: Uuid::new_v4().to_string(),
                            pair: self.config.pair.clone(),
                            side: Side::Buy,
                            entry_price: price,
                            exit_price: None,
                            amount,
                            pnl: 0.0,
                            entry_fee: fee,
                            exit_fee: 0.0,
                            entry_time: candle.timestamp,
                            exit_time: None,
                            is_open: true,
                        };

                        balance = 0.0; // All in
                        position = Some(trade);
                        orders_placed += 1;
                    }
                }
                TradeSignal::Sell => {
                    if position.is_none() {
                        // Short selling: We borrow the asset and sell it.
                        // The balance is used as margin/collateral.
                        // When we close, we buy back at current price.
                        let price = candle.close.0 * (1.0 - self.config.slippage);
                        let fee = balance * self.config.fee_rate;
                        let margin = balance - fee;
                        let amount = margin / price;

                        let trade = BacktestTrade {
                            id: Uuid::new_v4().to_string(),
                            pair: self.config.pair.clone(),
                            side: Side::Sell,
                            entry_price: price,
                            exit_price: None,
                            amount,
                            pnl: 0.0,
                            entry_fee: fee,
                            exit_fee: 0.0,
                            entry_time: candle.timestamp,
                            exit_time: None,
                            is_open: true,
                        };

                        // Lock margin (balance set to 0; will be restored + PnL on close)
                        balance = 0.0;
                        position = Some(trade);
                        orders_placed += 1;
                    }
                }
                TradeSignal::ClosePosition => {
                    if let Some(mut open_trade) = position.take() {
                        let (exit_price, exit_fee, pnl) = Self::close_trade(
                            &open_trade, candle.close.0, self.config.slippage, self.config.fee_rate,
                        );

                        // Restore balance: margin back + profit (or - loss)
                        let proceeds = match open_trade.side {
                            Side::Buy => {
                                // Long close: we sell the asset, receive quote
                                exit_price * open_trade.amount - exit_fee
                            }
                            Side::Sell => {
                                // Short close: we buy back. Margin + profit.
                                let margin = open_trade.entry_price * open_trade.amount;
                                margin + pnl
                            }
                            _ => open_trade.entry_price * open_trade.amount,
                        };

                        balance = proceeds.max(0.0);

                        open_trade.exit_price = Some(exit_price);
                        open_trade.exit_fee = exit_fee;
                        open_trade.pnl = pnl;
                        open_trade.exit_time = Some(candle.timestamp);
                        open_trade.is_open = false;

                        trades.push(open_trade);
                        orders_placed += 1;
                    }
                }
                TradeSignal::Hold => {}
            }
        }

        // Close any remaining open position at last price
        if let Some(mut open_trade) = position.take() {
            let last_price = candles.last().map(|c| c.close.0).unwrap_or(open_trade.entry_price);
            let (exit_price, exit_fee, pnl) = Self::close_trade(
                &open_trade, last_price, self.config.slippage, self.config.fee_rate,
            );

            let proceeds = match open_trade.side {
                Side::Buy => exit_price * open_trade.amount - exit_fee,
                Side::Sell => {
                    let margin = open_trade.entry_price * open_trade.amount;
                    margin + pnl
                }
                _ => open_trade.entry_price * open_trade.amount,
            };

            balance = proceeds.max(0.0);
            open_trade.exit_price = Some(exit_price);
            open_trade.exit_fee = exit_fee;
            open_trade.pnl = pnl;
            open_trade.exit_time = candles.last().map(|c| c.timestamp);
            open_trade.is_open = false;
            trades.push(open_trade);
        }

        let final_value = balance;
        let start_time = candles.first().map(|c| c.timestamp).unwrap_or(0);
        let end_time = candles.last().map(|c| c.timestamp).unwrap_or(0);

        let metrics = BacktestMetrics::from_trades(
            &trades,
            self.config.initial_balance,
            final_value,
            start_time,
            end_time,
        );

        tracing::info!(
            "Backtest complete: {} trades, {:.2}% return, {:.2} Sharpe",
            metrics.total_trades,
            metrics.total_return,
            metrics.sharpe_ratio
        );

        Ok(BacktestResult {
            config: self.config.clone(),
            trades,
            metrics,
            final_value,
            equity_curve,
            orders_placed,
            candles_processed: candles.len(),
        })
    }

    /// Calculate exit price, fee, and PnL when closing a trade.
    fn close_trade(
        trade: &BacktestTrade,
        market_price: f64,
        slippage: f64,
        fee_rate: f64,
    ) -> (f64, f64, f64) {
        let exit_price = match trade.side {
            Side::Buy => market_price * (1.0 - slippage),  // Selling into the market
            Side::Sell => market_price * (1.0 + slippage), // Buying back from the market
            _ => market_price,
        };

        let exit_fee = exit_price * trade.amount * fee_rate;

        let pnl = match trade.side {
            Side::Buy => {
                (exit_price - trade.entry_price) * trade.amount - trade.entry_fee - exit_fee
            }
            Side::Sell => {
                (trade.entry_price - exit_price) * trade.amount - trade.entry_fee - exit_fee
            }
            _ => 0.0,
        };

        (exit_price, exit_fee, pnl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::SmaCrossoverStrategy;

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices.iter().enumerate().map(|(i, &p)| Candle {
            timestamp: (i as i64) * 3600000,
            open: Price(p),
            high: Price(p * 1.01),
            low: Price(p * 0.99),
            close: Price(p),
            volume: Quantity(100.0),
        }).collect()
    }

    #[tokio::test]
    async fn test_backtest_engine_run() {
        let config = BacktestConfig {
            initial_balance: 10000.0,
            fee_rate: 0.001,
            ..Default::default()
        };
        let strategy = Box::new(SmaCrossoverStrategy::new(3, 5));
        let engine = BacktestEngine::new(config, strategy);

        // Create trending price data
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64) * 0.5).collect();
        let candles = make_candles(&prices);

        let result = engine.run(candles).await.unwrap();
        assert!(result.candles_processed == 30);
        assert!(result.final_value > 0.0);
        assert!(!result.equity_curve.is_empty());
    }

    #[tokio::test]
    async fn test_backtest_losing_strategy() {
        let config = BacktestConfig {
            initial_balance: 10000.0,
            ..Default::default()
        };
        let strategy = Box::new(SmaCrossoverStrategy::new(2, 3));
        let engine = BacktestEngine::new(config, strategy);

        // Declining prices
        let prices: Vec<f64> = (0..20).map(|i| 100.0 - (i as f64) * 0.5).collect();
        let candles = make_candles(&prices);

        let result = engine.run(candles).await.unwrap();
        assert!(result.candles_processed == 20);
    }

    #[tokio::test]
    async fn test_close_trade_long_profit() {
        let trade = BacktestTrade {
            id: "test".to_string(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            entry_price: 50000.0,
            exit_price: None,
            amount: 0.1,
            pnl: 0.0,
            entry_fee: 5.0,
            exit_fee: 0.0,
            entry_time: 0,
            exit_time: None,
            is_open: true,
        };

        let (exit_price, exit_fee, pnl) = BacktestEngine::close_trade(&trade, 55000.0, 0.0, 0.001);

        assert_eq!(exit_price, 55000.0);
        assert!((exit_fee - 5.5).abs() < 0.01); // 55000 * 0.1 * 0.001 = 5.5
        // PnL = (55000 - 50000) * 0.1 - entry_fee(5) - exit_fee(5.5)
        //     = 500 - 5 - 5.5 = 489.5
        assert!((pnl - 489.5).abs() < 0.01, "Expected PnL ~489.5, got {}", pnl);
    }

    #[tokio::test]
    async fn test_close_trade_short_profit() {
        let trade = BacktestTrade {
            id: "test".to_string(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Sell,
            entry_price: 50000.0,
            exit_price: None,
            amount: 0.1,
            pnl: 0.0,
            entry_fee: 5.0,
            exit_fee: 0.0,
            entry_time: 0,
            exit_time: None,
            is_open: true,
        };

        let (exit_price, exit_fee, pnl) = BacktestEngine::close_trade(&trade, 45000.0, 0.0, 0.001);

        assert_eq!(exit_price, 45000.0);
        assert!((exit_fee - 4.5).abs() < 0.01); // 45000 * 0.1 * 0.001 = 4.5
        // PnL = (50000 - 45000) * 0.1 - entry_fee(5) - exit_fee(4.5)
        //     = 500 - 5 - 4.5 = 490.5
        assert!((pnl - 490.5).abs() < 0.01, "Expected PnL ~490.5, got {}", pnl);
    }
}
