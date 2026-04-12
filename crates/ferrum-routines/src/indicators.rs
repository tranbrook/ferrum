//! Technical indicator calculations.

use ferrum_core::types::{Candle, Price};

/// Technical indicator calculations
pub struct TechnicalIndicators;

impl TechnicalIndicators {
    /// Simple Moving Average
    pub fn sma(candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period { return None; }
        let sum: f64 = candles.iter().rev().take(period)
            .map(|c| c.close.0)
            .sum();
        Some(sum / period as f64)
    }

    /// Exponential Moving Average
    pub fn ema(candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period { return None; }
        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = candles[0].close.0;
        for candle in &candles[1..] {
            ema = (candle.close.0 - ema) * multiplier + ema;
        }
        Some(ema)
    }

    /// Relative Strength Index
    pub fn rsi(candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period + 1 { return None; }
        let mut gains = 0.0;
        let mut losses = 0.0;
        let prices: Vec<f64> = candles.iter().map(|c| c.close.0).collect();
        for i in (prices.len() - period)..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 { gains += change; } else { losses += change.abs(); }
        }
        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;
        if avg_loss == 0.0 { return Some(100.0); }
        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    /// MACD (Moving Average Convergence Divergence)
    pub fn macd(candles: &[Candle]) -> Option<(f64, f64, f64)> {
        let ema12 = Self::ema(candles, 12)?;
        let ema26 = Self::ema(candles, 26)?;
        let macd_line = ema12 - ema26;
        // Simplified signal line
        let signal = macd_line * 0.15; // EMA9 approximation
        let histogram = macd_line - signal;
        Some((macd_line, signal, histogram))
    }

    /// Bollinger Bands (upper, middle, lower)
    pub fn bollinger_bands(candles: &[Candle], period: usize, std_dev: f64) -> Option<(f64, f64, f64)> {
        if candles.len() < period { return None; }
        let closes: Vec<f64> = candles.iter().rev().take(period).map(|c| c.close.0).collect();
        let mean = closes.iter().sum::<f64>() / closes.len() as f64;
        let variance = closes.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / closes.len() as f64;
        let std = variance.sqrt();
        Some((mean + std_dev * std, mean, mean - std_dev * std))
    }

    /// VWAP (Volume Weighted Average Price)
    pub fn vwap(candles: &[Candle]) -> Option<f64> {
        if candles.is_empty() { return None; }
        let total_volume_price: f64 = candles.iter()
            .map(|c| (c.high.0 + c.low.0 + c.close.0) / 3.0 * c.volume.0)
            .sum();
        let total_volume: f64 = candles.iter().map(|c| c.volume.0).sum();
        if total_volume == 0.0 { return None; }
        Some(total_volume_price / total_volume)
    }

    /// Average True Range
    pub fn atr(candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period + 1 { return None; }
        let mut trs = Vec::new();
        for i in 1..candles.len() {
            let high_low = candles[i].high.0 - candles[i].low.0;
            let high_close = (candles[i].high.0 - candles[i - 1].close.0).abs();
            let low_close = (candles[i].low.0 - candles[i - 1].close.0).abs();
            trs.push(high_low.max(high_close).max(low_close));
        }
        let recent: Vec<f64> = trs.into_iter().rev().take(period).collect();
        Some(recent.iter().sum::<f64>() / recent.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_core::types::Quantity;

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices.iter().enumerate().map(|(i, &p)| Candle {
            timestamp: i as i64 * 300,
            open: Price(p - 0.5),
            high: Price(p + 1.0),
            low: Price(p - 1.0),
            close: Price(p),
            volume: Quantity(100.0),
        }).collect()
    }

    #[test]
    fn test_sma() {
        let candles = make_candles(&[100.0, 101.0, 102.0, 103.0, 104.0]);
        let sma = TechnicalIndicators::sma(&candles, 3).unwrap();
        assert!((sma - 103.0).abs() < 0.01);
    }

    #[test]
    fn test_rsi() {
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let candles = make_candles(&prices);
        let rsi = TechnicalIndicators::rsi(&candles, 14).unwrap();
        assert!(rsi > 50.0); // Uptrend
    }

    #[test]
    fn test_bollinger_bands() {
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.1)).collect();
        let candles = make_candles(&prices);
        let (upper, mid, lower) = TechnicalIndicators::bollinger_bands(&candles, 20, 2.0).unwrap();
        assert!(upper > mid);
        assert!(mid > lower);
    }

    #[test]
    fn test_vwap() {
        let candles = make_candles(&[100.0, 101.0, 102.0]);
        let vwap = TechnicalIndicators::vwap(&candles).unwrap();
        assert!(vwap > 100.0 && vwap < 103.0);
    }
}
