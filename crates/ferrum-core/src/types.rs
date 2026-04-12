//! Domain types for the Ferrum trading system.
//!
//! All monetary values use newtype wrappers for type safety.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type-safe price wrapper (f64 inner, always positive)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Price(pub f64);

impl Price {
    pub fn zero() -> Self { Self(0.0) }
    pub fn is_positive(&self) -> bool { self.0 > 0.0 }
    pub fn inner(&self) -> f64 { self.0 }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:.8}", self.0) }
}

/// Type-safe quantity wrapper
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quantity(pub f64);

impl Quantity {
    pub fn zero() -> Self { Self(0.0) }
    pub fn is_positive(&self) -> bool { self.0 > 0.0 }
    pub fn inner(&self) -> f64 { self.0 }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:.8}", self.0) }
}

/// Trading pair in BASE-QUOTE format
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradingPair {
    pub base: String,
    pub quote: String,
}

impl TradingPair {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self { base: base.into(), quote: quote.into() }
    }

    pub fn from_dash(s: &str) -> Result<Self, crate::FerrumError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(crate::FerrumError::InvalidTradingPair(s.to_string()));
        }
        Ok(Self { base: parts[0].to_string(), quote: parts[1].to_string() })
    }

    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

impl fmt::Display for TradingPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}-{}", self.base, self.quote) }
}

impl FromStr for TradingPair {
    type Err = crate::FerrumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_dash(s) }
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
    Range, // For LP positions
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
    StopMarket,
    TakeProfitMarket,
}

/// Executor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutorType {
    Swap,
    Order,
    Lp,
    Position,
    Grid,
    Dca,
}

/// Executor status with close type on termination
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutorStatus {
    Created,
    Active,
    Terminated(CloseType),
}

/// How an executor was terminated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloseType {
    TakeProfit,
    StopLoss,
    TimeLimit,
    TrailingStop,
    EarlyStop,
}

/// Unique executor identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutorId(pub String);

impl ExecutorId {
    pub fn new() -> Self { Self(uuid::Uuid::new_v4().to_string()) }
    pub fn inner(&self) -> &str { &self.0 }
}

impl Default for ExecutorId {
    fn default() -> Self { Self::new() }
}

/// Unique order identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub String);

impl OrderId {
    pub fn new() -> Self { Self(uuid::Uuid::new_v4().to_string()) }
}

/// Candle interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Interval {
    M1,
    M3,
    M5,
    M15,
    M30,
    H1,
    H4,
    D1,
    W1,
}

impl Interval {
    pub fn to_seconds(&self) -> u64 {
        match self {
            Self::M1 => 60,
            Self::M3 => 180,
            Self::M5 => 300,
            Self::M15 => 900,
            Self::M30 => 1800,
            Self::H1 => 3600,
            Self::H4 => 14400,
            Self::D1 => 86400,
            Self::W1 => 604800,
        }
    }
}

/// OHLCV Candle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: i64,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
}

/// Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Price,
    pub quantity: Quantity,
}

/// Order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub pair: TradingPair,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: i64,
}

impl OrderBook {
    pub fn mid_price(&self) -> Option<Price> {
        let best_bid = self.bids.first()?.price.0;
        let best_ask = self.asks.first()?.price.0;
        Some(Price((best_bid + best_ask) / 2.0))
    }

    pub fn spread(&self) -> Option<f64> {
        let best_bid = self.bids.first()?.price.0;
        let best_ask = self.asks.first()?.price.0;
        Some(best_ask - best_bid)
    }

    pub fn spread_pct(&self) -> Option<f64> {
        let mid = self.mid_price()?.0;
        let spread = self.spread()?;
        Some(spread / mid * 100.0)
    }
}

/// Balance for a single asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: Quantity,
    pub used: Quantity,
}

impl Balance {
    pub fn total(&self) -> Quantity { Quantity(self.free.0 + self.used.0) }
}

/// Position (spot, LP, perp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub connector: String,
    pub pair: TradingPair,
    pub side: Side,
    pub amount: Quantity,
    pub entry_price: Price,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub leverage: Option<u32>,
    pub is_lp: bool,
}

/// Standardized executor metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorMetrics {
    pub executor_id: ExecutorId,
    pub controller_id: String,
    pub executor_type: ExecutorType,
    pub net_pnl_quote: f64,
    pub fees_paid_quote: f64,
    pub fees_earned_quote: f64,
    pub value_quote: f64,
    pub volume_quote: f64,
    pub duration_seconds: u64,
    pub close_type: Option<CloseType>,
}

/// Order request to exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub pair: TradingPair,
    pub side: Side,
    pub order_type: OrderType,
    pub amount: Quantity,
    pub price: Option<Price>,
    pub stop_price: Option<Price>,
    pub leverage: Option<u32>,
    pub client_order_id: Option<OrderId>,
}

/// Order response from exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: OrderId,
    pub client_order_id: Option<OrderId>,
    pub status: OrderStatus,
    pub filled_quantity: Quantity,
    pub avg_fill_price: Option<Price>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

/// Executor action that agent produces
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecutorAction {
    Create {
        executor_type: ExecutorType,
        connector: String,
        pair: TradingPair,
        side: Side,
        amount: Quantity,
        #[serde(flatten)]
        params: serde_json::Value,
    },
    Stop {
        executor_id: ExecutorId,
    },
    Modify {
        executor_id: ExecutorId,
        #[serde(flatten)]
        params: serde_json::Value,
    },
}

impl ExecutorAction {
    pub fn amount(&self) -> f64 {
        match self {
            Self::Create { amount, .. } => amount.0,
            Self::Stop { .. } => 0.0,
            Self::Modify { .. } => 0.0,
        }
    }
}

/// Result of executor action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum ExecutorResult {
    Success { executor_id: ExecutorId },
    Blocked { reason: String },
    Failed { error: String },
}

/// Market regime classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketRegime {
    TrendingUp,
    TrendingDown,
    Ranging,
    Volatile,
    Crisis,
}

/// Trading signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_pair_from_dash() {
        let pair = TradingPair::from_dash("BTC-USDT").unwrap();
        assert_eq!(pair.base, "BTC");
        assert_eq!(pair.quote, "USDT");
        assert_eq!(pair.to_string(), "BTC-USDT");
    }

    #[test]
    fn test_trading_pair_invalid() {
        assert!(TradingPair::from_dash("INVALID").is_err());
    }

    #[test]
    fn test_order_book_mid_price() {
        let ob = OrderBook {
            pair: TradingPair::new("BTC", "USDT"),
            bids: vec![OrderBookLevel { price: Price(9999.0), quantity: Quantity(1.0) }],
            asks: vec![OrderBookLevel { price: Price(10001.0), quantity: Quantity(1.0) }],
            timestamp: 0,
        };
        assert_eq!(ob.mid_price().unwrap(), Price(10000.0));
        assert_eq!(ob.spread().unwrap(), 2.0);
    }

    #[test]
    fn test_interval_seconds() {
        assert_eq!(Interval::M5.to_seconds(), 300);
        assert_eq!(Interval::H1.to_seconds(), 3600);
        assert_eq!(Interval::D1.to_seconds(), 86400);
    }

    #[test]
    fn test_executor_action_amount() {
        let action = ExecutorAction::Create {
            executor_type: ExecutorType::Position,
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            amount: Quantity(1.5),
            params: serde_json::Value::Null,
        };
        assert_eq!(action.amount(), 1.5);
    }

    #[test]
    fn test_balance_total() {
        let b = Balance { asset: "BTC".into(), free: Quantity(1.0), used: Quantity(0.5) };
        assert_eq!(b.total(), Quantity(1.5));
    }
}
