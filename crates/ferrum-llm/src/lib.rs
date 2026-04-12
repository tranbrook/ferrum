//! # Ferrum LLM
//!
//! LLM integration layer supporting OpenAI, Anthropic, Groq, and local models.

pub mod client;
pub mod prompt;

pub use client::OpenAiClient;
pub use prompt::PromptBuilder;
