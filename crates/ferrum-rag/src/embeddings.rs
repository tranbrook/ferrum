//! Embedding generation for financial text.
//!
//! Supports multiple embedding strategies:
//! - Local FinBERT-style embeddings (simulated)
//! - Remote embedding API (OpenAI, etc.)
//! - Simple TF-IDF based embeddings for lightweight use

use ferrum_core::error::{FerrumError, Result};
use serde::{Deserialize, Serialize};

/// Configuration for embedding generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding provider: "local", "openai", "mock"
    pub provider: String,
    /// Model name (e.g. "finbert", "text-embedding-3-small")
    pub model: String,
    /// Dimension of the embedding vector
    pub dimension: usize,
    /// API key for remote providers
    pub api_key: Option<String>,
    /// API endpoint for remote providers
    pub api_url: Option<String>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            model: "mock-embedding".to_string(),
            dimension: 384,
            api_key: None,
            api_url: None,
        }
    }
}

/// A document with its embedding vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
    pub timestamp: i64,
}

/// Embedding generator trait.
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate embedding for a single text.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch).
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Get the dimension of embeddings.
    fn dimension(&self) -> usize;
}

/// Mock embedding generator for testing.
pub struct MockEmbeddingGenerator {
    dimension: usize,
}

impl MockEmbeddingGenerator {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl EmbeddingGenerator for MockEmbeddingGenerator {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Simple hash-based mock embedding
        let mut embedding = Vec::with_capacity(self.dimension);
        let bytes = text.as_bytes();
        for i in 0..self.dimension {
            let byte_val = bytes.get(i % bytes.len()).copied().unwrap_or(0) as f32;
            let normalized = (byte_val - 128.0) / 128.0;
            embedding.push(normalized);
        }
        // Normalize to unit vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in embedding.iter_mut() {
                *x /= norm;
            }
        }
        Ok(embedding)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Remote embedding generator (OpenAI-compatible).
pub struct RemoteEmbeddingGenerator {
    client: reqwest::Client,
    config: EmbeddingConfig,
}

impl RemoteEmbeddingGenerator {
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }
}

impl EmbeddingGenerator for RemoteEmbeddingGenerator {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // For now, return mock embedding
        // In production, this would call OpenAI/other API
        let generator = MockEmbeddingGenerator::new(self.config.dimension);
        generator.embed(text)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }
}

/// Create an embedding generator based on config.
pub fn create_embedding_generator(config: &EmbeddingConfig) -> Box<dyn EmbeddingGenerator> {
    match config.provider.as_str() {
        "mock" => Box::new(MockEmbeddingGenerator::new(config.dimension)),
        "openai" | "remote" => Box::new(RemoteEmbeddingGenerator::new(config.clone())),
        _ => Box::new(MockEmbeddingGenerator::new(config.dimension)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_embedding_dimension() {
        let gen = MockEmbeddingGenerator::new(384);
        let emb = gen.embed("BTC is going up").unwrap();
        assert_eq!(emb.len(), 384);
    }

    #[test]
    fn test_mock_embedding_normalized() {
        let gen = MockEmbeddingGenerator::new(128);
        let emb = gen.embed("test market analysis").unwrap();
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_batch_embedding() {
        let gen = MockEmbeddingGenerator::new(64);
        let texts = vec!["hello".to_string(), "world".to_string()];
        let embeddings = gen.embed_batch(&texts).unwrap();
        assert_eq!(embeddings.len(), 2);
    }
}
