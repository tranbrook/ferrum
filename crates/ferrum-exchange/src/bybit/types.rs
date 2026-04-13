//! Bybit-specific type conversions.

use ferrum_core::types::*;

/// Convert TradingPair to Bybit symbol (BTCUSDT).
pub fn pair_to_symbol(pair: &TradingPair) -> String {
    format!("{}{}", pair.base, pair.quote)
}

/// Convert Bybit symbol to TradingPair.
pub fn symbol_to_pair(symbol: &str) -> Option<TradingPair> {
    let quotes = ["USDC", "USDT", "BTC", "ETH", "USD"];
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
