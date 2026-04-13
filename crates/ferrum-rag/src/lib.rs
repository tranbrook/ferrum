//! # Ferrum RAG
//!
//! Retrieval-Augmented Generation pipeline for trading intelligence.
//! Uses Qdrant vector database + FinBERT-style embeddings for semantic
//! search over market analysis, research, and historical patterns.

pub mod embeddings;
pub mod pipeline;
pub mod qdrant;
pub mod store;

pub use pipeline::RagPipeline;
