//! # Ferrum Paper Trading
//!
//! Paper trading engine for strategy validation without real money.
//! Simulates order execution, slippage, and fees using real market data.

pub mod engine;
pub mod tracker;
pub mod types;

pub use engine::PaperTradingEngine;
pub use tracker::PositionTracker;
pub use types::*;
