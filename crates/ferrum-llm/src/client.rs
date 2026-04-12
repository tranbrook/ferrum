//! OpenAI-compatible LLM client.

use async_trait::async_trait;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::traits::LlmClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: u32,
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI-compatible LLM client
pub struct OpenAiClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
    temperature: f64,
}

impl OpenAiClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model,
            max_tokens: 4096,
            temperature: 0.1, // Low temperature for trading decisions
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub fn with_temperature(mut self, temp: f64) -> Self {
        self.temperature = temp;
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn groq(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.groq.com/openai/v1".to_string(),
            model: "llama-3.1-70b-versatile".to_string(),
            max_tokens: 4096,
            temperature: 0.1,
        }
    }

    pub fn anthropic(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            temperature: 0.1,
        }
    }

    async fn chat_internal(&self, system_prompt: &str, user_prompt: &str, json_mode: bool) -> Result<ChatResponse> {
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system_prompt.into(),
            },
            ChatMessage {
                role: "user".into(),
                content: user_prompt.into(),
            },
        ];

        let mut request = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            response_format: None,
        };

        if json_mode {
            request.response_format = Some(ResponseFormat {
                r#type: "json_object".into(),
            });
        }

        let resp = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| FerrumError::LlmError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(FerrumError::LlmError(format!("API error {}: {}", status, body)));
        }

        resp.json::<ChatResponse>()
            .await
            .map_err(|e| FerrumError::LlmError(e.to_string()))
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let response = self.chat_internal(
            "You are an expert cryptocurrency trading analyst. Provide clear, concise analysis.",
            prompt,
            false,
        ).await?;

        response.choices.first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| FerrumError::LlmError("No response from LLM".into()))
    }

    async fn structured_complete_raw(&self, prompt: &str) -> Result<String> {
        let system = "You are an expert cryptocurrency trading analyst. \
            Always respond with valid JSON matching the requested schema. \
            Do not include any text outside the JSON object.";

        let response = self.chat_internal(system, prompt, true).await?;

        response.choices.first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| FerrumError::LlmError("No response from LLM".into()))
    }

    fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        // GPT-4o pricing approximation
        let input_cost = (input_tokens as f64 / 1_000_000.0) * 2.50;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * 10.00;
        input_cost + output_cost
    }
}
