//! # Ferrum Exchange
//!
//! Exchange adapter implementations with pluggable trait-based architecture.
//! Supports Binance, Bybit, OKX, and Hyperliquid.

pub mod binance;
pub mod bybit;
pub mod hyperliquid;
pub mod okx;
pub mod registry;
pub mod shared;

pub use registry::ExchangeRegistry;
pub use shared::ExchangeHttpClient;
