//! # Ferrum Exchange
//!
//! Exchange adapter implementations with pluggable trait-based architecture.

pub mod binance;
pub mod registry;

pub use registry::ExchangeRegistry;
