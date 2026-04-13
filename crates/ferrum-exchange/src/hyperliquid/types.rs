//! Hyperliquid-specific type conversions.

use ferrum_core::types::*;

/// Convert TradingPair to Hyperliquid symbol (BTC).
/// Hyperliquid uses base currency only for perps, or COIN for spot.
pub fn pair_to_coin(pair: &TradingPair) -> String {
    pair.base.clone()
}

/// Convert Hyperliquid coin back to TradingPair.
pub fn coin_to_pair(coin: &str, quote: &str) -> TradingPair {
    TradingPair::new(coin, quote)
}
