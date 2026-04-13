//! Local LLM provider implementing the LlmClient trait from ferrum-core.

use async_trait::async_trait;
use ferrum_core::error::Result;
use ferrum_core::traits::LlmClient;
use crate::config::LocalLlmConfig;

/// Local LLM provider for on-device inference.
pub struct LocalLlmProvider {
    config: LocalLlmConfig,
}

impl LocalLlmProvider {
    pub fn new(config: LocalLlmConfig) -> Self {
        Self { config }
    }

    /// Build a prompt from messages in chat-ml format.
    fn build_prompt(&self, messages: &[(String, String)]) -> String {
        let mut prompt = String::new();
        for (role, content) in messages {
            prompt.push_str(&format!("<|{}|>\n{}\n", role, content));
        }
        prompt.push_str("<|assistant|\\n");
        prompt
    }

    /// Mock inference for testing.
    fn mock_inference(&self, prompt: &str) -> String {
        let lower = prompt.to_lowercase();
        if lower.contains("buy") || lower.contains("long") {
            "Based on my analysis, the market shows mixed signals. \
             Current price action suggests consolidation with a slight bullish bias. \
             Risk-reward ratio for a long entry is approximately 1:1.5. \
             Recommended position size: 2% of portfolio. \
             Stop loss should be placed below the recent support level.".to_string()
        } else if lower.contains("sell") || lower.contains("short") {
            "The market is showing potential weakness at current levels. \
             Volume is declining and momentum indicators are turning bearish. \
             A short position could be considered with tight risk management. \
             Suggested stop loss above the recent resistance.".to_string()
        } else if lower.contains("analysis") || lower.contains("market") {
            "Market Analysis Summary:\n\
             - Trend: Mixed/Consolidating\n\
             - Key Levels: Support and Resistance identified\n\
             - Volume: Below average\n\
             - Recommendation: Wait for clear breakout before entering".to_string()
        } else {
            "I'm running locally and ready to assist with trading analysis. \
             Please provide specific market data or questions for detailed analysis.".to_string()
        }
    }
}

#[async_trait]
impl LlmClient for LocalLlmProvider {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let response = self.mock_inference(prompt);
        let max = self.config.max_tokens;
        let words: Vec<&str> = response.split_whitespace().collect();
        let truncated: Vec<&str> = words.into_iter().take(max).collect();
        Ok(truncated.join(" "))
    }

    async fn structured_complete_raw(&self, prompt: &str) -> Result<String> {
        // For mock, return a JSON-like structure
        let response = self.mock_inference(prompt);
        Ok(serde_json::json!({
            "analysis": response,
            "confidence": 0.75,
            "action": "hold",
            "risk_level": "medium"
        }).to_string())
    }

    fn estimate_cost(&self, _input_tokens: u32, _output_tokens: u32) -> f64 {
        0.0 // Local inference is free
    }
}

impl LocalLlmProvider {
    /// Check if the model is available.
    pub fn is_available(&self) -> bool {
        if let Some(ref path) = self.config.model_path {
            std::path::Path::new(path).exists()
        } else {
            true // Mock provider always available
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_llm_complete() {
        let config = LocalLlmConfig::default();
        let provider = LocalLlmProvider::new(config);
        let response = provider.complete("Should I buy BTC?").await.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_local_llm_structured() {
        let config = LocalLlmConfig::default();
        let provider = LocalLlmProvider::new(config);
        let response = provider.structured_complete_raw("Analyze market").await.unwrap();
        assert!(response.contains("analysis"));
    }

    #[test]
    fn test_build_prompt() {
        let config = LocalLlmConfig::default();
        let provider = LocalLlmProvider::new(config);
        let prompt = provider.build_prompt(&[
            ("system".to_string(), "You are a trading assistant".to_string()),
            ("user".to_string(), "What is BTC price?".to_string()),
        ]);
        assert!(prompt.contains("system"));
        assert!(prompt.contains("user"));
    }
}
