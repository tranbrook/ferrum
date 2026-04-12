//! Webhook handlers for external signals.

use ferrum_core::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub source: String,
    pub signal: String,
    pub pair: Option<String>,
    pub price: Option<f64>,
    pub confidence: Option<f64>,
    pub timestamp: i64,
}

/// Webhook handler
pub struct WebhookHandler {
    secret: String,
}

impl WebhookHandler {
    pub fn new(secret: String) -> Self { Self { secret } }

    pub fn validate(&self, payload: &str, signature: &str) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());
        expected == signature
    }

    pub fn parse(&self, body: &str) -> Result<WebhookPayload> {
        serde_json::from_str(body)
            .map_err(|e| ferrum_core::FerrumError::ConfigError(format!("Invalid webhook payload: {}", e)))
    }
}
