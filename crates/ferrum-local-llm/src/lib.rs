//! # Ferrum Local LLM
//!
//! Local LLM inference using Candle/llama.cpp for privacy-preserving
//! on-device model execution.

pub mod config;
pub mod engine;
pub mod provider;

pub use config::LocalLlmConfig;
pub use engine::LocalLlmEngine;
pub use provider::LocalLlmProvider;
