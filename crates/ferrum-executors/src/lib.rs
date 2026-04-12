//! # Ferrum Executors
//!
//! Self-contained trading operations with standardized P&L reporting.

pub mod position;
pub mod order;
pub mod grid;
pub mod triple_barrier;
pub mod factory;

pub use factory::ExecutorFactory;
