//! OKX-specific type conversions.

use ferrum_core::types::*;

/// Convert TradingPair to OKX instrument ID (BTC-USDT).
pub fn pair_to_inst_id(pair: &TradingPair) -> String {
    format!("{}-{}", pair.base, pair.quote)
}

/// Convert OKX instrument ID to TradingPair.
pub fn inst_id_to_pair(inst_id: &str) -> Option<TradingPair> {
    let parts: Vec<&str> = inst_id.split('-').collect();
    if parts.len() >= 2 {
        Some(TradingPair::new(parts[0], parts[1]))
    } else {
        None
    }
}
