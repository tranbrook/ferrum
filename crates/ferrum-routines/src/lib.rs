//! # Ferrum Routines
//!
//! Deterministic workflows: indicators, webhooks, reports, alerts.

pub mod indicators;
pub mod webhooks;
pub mod alerts;

pub use indicators::TechnicalIndicators;
pub use alerts::AlertManager;
