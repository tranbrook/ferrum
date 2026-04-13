//! Local LLM configuration.

use serde::{Deserialize, Serialize};

/// Configuration for local LLM inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLlmConfig {
    /// Backend to use: "candle", "llama-cpp", "mock"
    pub backend: String,
    /// Path to the model file (GGUF, safetensors, etc.)
    pub model_path: Option<String>,
    /// HuggingFace model ID for download
    pub model_id: Option<String>,
    /// Maximum context length.
    pub max_context_length: usize,
    /// Number of GPU layers to offload (0 = CPU only).
    pub gpu_layers: usize,
    /// Number of threads.
    pub threads: usize,
    /// Temperature for sampling.
    pub temperature: f64,
    /// Top-p for sampling.
    pub top_p: f64,
    /// Maximum tokens to generate.
    pub max_tokens: usize,
    /// Repeat penalty.
    pub repeat_penalty: f64,
}

impl Default for LocalLlmConfig {
    fn default() -> Self {
        Self {
            backend: "mock".to_string(),
            model_path: None,
            model_id: None,
            max_context_length: 4096,
            gpu_layers: 0,
            threads: 4,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 1024,
            repeat_penalty: 1.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LocalLlmConfig::default();
        assert_eq!(config.backend, "mock");
        assert_eq!(config.max_context_length, 4096);
    }
}
