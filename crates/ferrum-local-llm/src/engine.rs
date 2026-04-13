//! Local LLM inference engine.

use crate::config::LocalLlmConfig;
use crate::provider::LocalLlmProvider;
use ferrum_core::traits::LlmClient;
use std::sync::Arc;

/// Local LLM engine managing model loading and inference.
pub struct LocalLlmEngine {
    config: LocalLlmConfig,
    provider: Arc<dyn LlmClient>,
}

impl LocalLlmEngine {
    /// Create a new local LLM engine.
    pub fn new(config: LocalLlmConfig) -> Self {
        let provider = Arc::new(LocalLlmProvider::new(config.clone()));
        Self { config, provider }
    }

    /// Get a reference to the provider.
    pub fn provider(&self) -> Arc<dyn LlmClient> {
        self.provider.clone()
    }

    /// Check if the local model is available.
    pub fn is_available(&self) -> bool {
        // Check if model file exists if specified
        if let Some(ref path) = self.config.model_path {
            std::path::Path::new(path).exists()
        } else {
            true // Mock provider always available
        }
    }

    /// Quick inference for trading signals.
    pub async fn quick_inference(&self, prompt: &str) -> ferrum_core::error::Result<String> {
        self.provider.complete(prompt).await
    }

    /// Full analysis inference.
    pub async fn analysis_inference(&self, prompt: &str) -> ferrum_core::error::Result<String> {
        self.provider.complete(prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_engine() {
        let config = LocalLlmConfig::default();
        let engine = LocalLlmEngine::new(config);
        let response = engine.quick_inference("test").await.unwrap();
        assert!(!response.is_empty());
    }
}
