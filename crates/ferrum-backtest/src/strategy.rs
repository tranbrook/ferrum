//! Backtest strategy trait and built-in strategies.

use async_trait::async_trait;
use ferrum_core::error::Result;
use ferrum_core::types::*;
use crate::types::{MarketContext, TradeSignal};
use std::collections::HashMap;

/// Trait for a backtestable trading strategy.
#[async_trait]
pub trait BacktestStrategy: Send + Sync {
    /// Name of the strategy.
    fn name(&self) -> &str;

    /// Description of the strategy.
    fn description(&self) -> &str;

    /// Generate a trade signal based on market context.
    async fn evaluate(&self, ctx: &MarketContext) -> Result<TradeSignal>;

    /// Optional: compute indicators for the current context.
    fn compute_indicators(&self, candles: &[Candle]) -> HashMap<String, f64> {
        let _ = candles;
        HashMap::new()
    }
}

/// Simple SMA Crossover strategy.
pub struct SmaCrossoverStrategy {
    pub fast_period: usize,
    pub slow_period: usize,
}

impl SmaCrossoverStrategy {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self { fast_period, slow_period }
    }

    fn compute_sma(&self, candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period {
            return None;
        }
        let sum: f64 = candles.iter().rev().take(period)
            .map(|c| c.close.0)
            .sum();
        Some(sum / period as f64)
    }
}

#[async_trait]
impl BacktestStrategy for SmaCrossoverStrategy {
    fn name(&self) -> &str {
        "SMA Crossover"
    }

    fn description(&self) -> &str {
        "Buy when fast SMA crosses above slow SMA, sell on cross below"
    }

    async fn evaluate(&self, ctx: &MarketContext) -> Result<TradeSignal> {
        let fast_sma = self.compute_sma(&ctx.historical_candles, self.fast_period);
        let slow_sma = self.compute_sma(&ctx.historical_candles, self.slow_period);

        match (fast_sma, slow_sma) {
            (Some(fast), Some(slow)) => {
                // Check if we have a position
                if ctx.position.is_some() {
                    if fast < slow {
                        Ok(TradeSignal::ClosePosition)
                    } else {
                        Ok(TradeSignal::Hold)
                    }
                } else if fast > slow {
                    Ok(TradeSignal::Buy)
                } else {
                    Ok(TradeSignal::Hold)
                }
            }
            _ => Ok(TradeSignal::Hold),
        }
    }

    fn compute_indicators(&self, candles: &[Candle]) -> HashMap<String, f64> {
        let mut indicators = HashMap::new();
        if let Some(fast) = self.compute_sma(candles, self.fast_period) {
            indicators.insert("sma_fast".to_string(), fast);
        }
        if let Some(slow) = self.compute_sma(candles, self.slow_period) {
            indicators.insert("sma_slow".to_string(), slow);
        }
        indicators
    }
}

/// RSI Mean Reversion strategy.
pub struct RsiMeanReversionStrategy {
    pub period: usize,
    pub oversold: f64,
    pub overbought: f64,
}

impl RsiMeanReversionStrategy {
    pub fn new(period: usize, oversold: f64, overbought: f64) -> Self {
        Self { period, oversold, overbought }
    }

    fn compute_rsi(&self, candles: &[Candle]) -> Option<f64> {
        if candles.len() < self.period + 1 {
            return None;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in (candles.len() - self.period)..candles.len() {
            let change = candles[i].close.0 - candles[i - 1].close.0;
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / self.period as f64;
        let avg_loss = losses / self.period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

#[async_trait]
impl BacktestStrategy for RsiMeanReversionStrategy {
    fn name(&self) -> &str {
        "RSI Mean Reversion"
    }

    fn description(&self) -> &str {
        "Buy when RSI is oversold, sell when overbought"
    }

    async fn evaluate(&self, ctx: &MarketContext) -> Result<TradeSignal> {
        let rsi = self.compute_rsi(&ctx.historical_candles);

        match rsi {
            Some(rsi_val) => {
                if ctx.position.is_some() {
                    if rsi_val > self.overbought {
                        Ok(TradeSignal::ClosePosition)
                    } else {
                        Ok(TradeSignal::Hold)
                    }
                } else if rsi_val < self.oversold {
                    Ok(TradeSignal::Buy)
                } else {
                    Ok(TradeSignal::Hold)
                }
            }
            None => Ok(TradeSignal::Hold),
        }
    }

    fn compute_indicators(&self, candles: &[Candle]) -> HashMap<String, f64> {
        let mut indicators = HashMap::new();
        if let Some(rsi) = self.compute_rsi(candles) {
            indicators.insert("rsi".to_string(), rsi);
        }
        indicators
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_core::types::{Price, Quantity};

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
    async fn test_sma_crossover_buy_signal() {
        let strategy = SmaCrossoverStrategy::new(3, 5);
        // Create uptrend candles
        let prices = [100.0, 98.0, 96.0, 98.0, 100.0, 102.0, 104.0, 106.0];
        let candles = make_candles(&prices);

        let ctx = MarketContext {
            current_candle: candles.last().unwrap().clone(),
            historical_candles: candles.clone(),
            balance: 10000.0,
            position: None,
            indicators: HashMap::new(),
        };

        let signal = strategy.evaluate(&ctx).await.unwrap();
        assert_eq!(signal, TradeSignal::Buy);
    }

    #[tokio::test]
    async fn test_rsi_oversold_buy_signal() {
        let strategy = RsiMeanReversionStrategy::new(14, 30.0, 70.0);
        // Create declining candles (oversold)
        let mut prices = vec![];
        for i in 0..20 {
            prices.push(100.0 - (i as f64) * 2.0);
        }
        let candles = make_candles(&prices);

        let ctx = MarketContext {
            current_candle: candles.last().unwrap().clone(),
            historical_candles: candles.clone(),
            balance: 10000.0,
            position: None,
            indicators: HashMap::new(),
        };

        let signal = strategy.evaluate(&ctx).await.unwrap();
        // Should be buy (oversold)
        assert_eq!(signal, TradeSignal::Buy);
    }
}
