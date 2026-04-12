//! # Ferrum Core
//!
//! Core types, traits, events, and shared abstractions
//! for the Ferrum trading agent harness.

pub mod config;
pub mod error;
pub mod events;
pub mod traits;
pub mod types;

pub use config::*;
pub use error::*;
pub use events::*;
pub use traits::*;
pub use types::*;
