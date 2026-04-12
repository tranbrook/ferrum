//! # Ferrum API
//!
//! REST API server using Axum.

pub mod server;
pub mod routes;
pub mod auth;

pub use server::ApiServer;
