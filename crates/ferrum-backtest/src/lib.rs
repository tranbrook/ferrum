//! # Ferrum Backtest
//!
//! Backtesting engine for trading strategies.
//! Provides historical data replay, position simulation, and performance analytics.

pub mod engine;
pub mod metrics;
pub mod strategy;
pub mod types;

pub use engine::BacktestEngine;
pub use metrics::BacktestMetrics;
pub use strategy::BacktestStrategy;
pub use types::*;
