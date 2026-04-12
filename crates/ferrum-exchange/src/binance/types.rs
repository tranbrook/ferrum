//! Binance-specific type conversions.

use ferrum_core::types::*;
use ferrum_core::error::Result;

/// Convert Binance symbol (BTCUSDT) to TradingPair
pub fn symbol_to_pair(symbol: &str) -> Option<TradingPair> {
    // Common quote currencies, longest first
    let quotes = ["USDC", "USDT", "BTC", "ETH", "BNB", "BUSD"];
    for quote in &quotes {
        if symbol.ends_with(quote) {
            let base = symbol.trim_end_matches(quote);
            if !base.is_empty() {
                return Some(TradingPair::new(base, *quote));
            }
        }
    }
    None
}

/// Convert TradingPair to Binance symbol
pub fn pair_to_symbol(pair: &TradingPair) -> String {
    format!("{base}{quote}", base = pair.base, quote = pair.quote)
}

/// Parse Binance kline data into Candle
pub fn parse_kline(data: &[serde_json::Value]) -> Option<Candle> {
    Some(Candle {
        timestamp: data.get(0)?.as_i64()?,
        open: Price(data.get(1)?.as_str()?.parse().ok()?),
        high: Price(data.get(2)?.as_str()?.parse().ok()?),
        low: Price(data.get(3)?.as_str()?.parse().ok()?),
        close: Price(data.get(4)?.as_str()?.parse().ok()?),
        volume: Quantity(data.get(5)?.as_str()?.parse().ok()?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_to_pair() {
        let pair = symbol_to_pair("BTCUSDT").unwrap();
        assert_eq!(pair.base, "BTC");
        assert_eq!(pair.quote, "USDT");
    }

    #[test]
    fn test_pair_to_symbol() {
        let pair = TradingPair::new("ETH", "USDT");
        assert_eq!(pair_to_symbol(&pair), "ETHUSDT");
    }
}
