//! # Ferrum Dashboard
//!
//! Web dashboard for monitoring trading bots, positions,
//! and agent activity. Built with Axum.

pub mod api;
pub mod handlers;
pub mod server;

pub use server::DashboardServer;
