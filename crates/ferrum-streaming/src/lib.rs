//! # Ferrum Streaming
//!
//! Real-time WebSocket streaming for market data (orderbook, trades, candles).
//! Supports Binance, Bybit, OKX, and Hyperliquid WebSocket feeds.

pub mod binance;
pub mod manager;
pub mod traits;

pub use manager::StreamManager;
pub use traits::MarketDataStream;
