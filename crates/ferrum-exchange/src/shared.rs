//! Shared HTTP client and utilities for exchange adapters.
//!
//! Reduces code duplication across exchange adapters by providing:
//! - Common HTTP request methods with structured logging
//! - URL sanitization to prevent API key leaks in logs
//! - A trait for exchange-specific request signing

use ferrum_core::error::{FerrumError, Result};
use reqwest::header::HeaderMap;
use std::collections::HashSet;

/// Shared HTTP client for exchange adapters.
pub struct ExchangeHttpClient {
    client: reqwest::Client,
    base_url: String,
    /// Keys to redact from logged URLs.
    sensitive_params: HashSet<String>,
}

impl ExchangeHttpClient {
    /// Create a new HTTP client for an exchange.
    pub fn new(base_url: String) -> Self {
        let mut sensitive_params = HashSet::new();
        sensitive_params.insert("apiKey".to_string());
        sensitive_params.insert("api_key".to_string());
        sensitive_params.insert("signature".to_string());
        sensitive_params.insert("timestamp".to_string());
        Self {
            client: reqwest::Client::new(),
            base_url,
            sensitive_params,
        }
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// GET request with no auth.
    pub async fn public_get(&self, endpoint: &str, params: &str) -> Result<serde_json::Value> {
        let url = self.build_url(endpoint, params);
        self.log_request("GET", &url);

        let resp = self.client.get(&url).send().await
            .map_err(|e| FerrumError::ExchangeError(format!("HTTP GET failed: {}", e)))?;

        self.parse_response(resp).await
    }

    /// GET request with custom headers (for signed requests).
    pub async fn signed_get(
        &self,
        endpoint: &str,
        params: &str,
        headers: HeaderMap,
    ) -> Result<serde_json::Value> {
        let url = self.build_url(endpoint, params);
        self.log_request("GET (signed)", &url);

        let resp = self.client.get(&url).headers(headers).send().await
            .map_err(|e| FerrumError::ExchangeError(format!("Signed GET failed: {}", e)))?;

        self.parse_response(resp).await
    }

    /// POST request with custom headers and body.
    pub async fn signed_post(
        &self,
        endpoint: &str,
        body: &str,
        headers: HeaderMap,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        self.log_request("POST (signed)", &url);

        let resp = self.client.post(&url)
            .headers(headers)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send().await
            .map_err(|e| FerrumError::ExchangeError(format!("Signed POST failed: {}", e)))?;

        self.parse_response(resp).await
    }

    /// POST JSON-RPC style request (for Hyperliquid-style APIs).
    pub async fn json_post(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        self.log_request("POST (json)", &url);

        let resp = self.client.post(&url)
            .json(body)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(format!("JSON POST failed: {}", e)))?;

        self.parse_response(resp).await
    }

    /// Build full URL from endpoint and query params.
    fn build_url(&self, endpoint: &str, params: &str) -> String {
        if params.is_empty() {
            format!("{}{}", self.base_url, endpoint)
        } else {
            format!("{}{}?{}", self.base_url, endpoint, params)
        }
    }

    /// Log a request with sanitized URL (no API keys in logs).
    fn log_request(&self, method: &str, url: &str) {
        let sanitized = self.sanitize_url(url);
        tracing::debug!("{} {}", method, sanitized);
    }

    /// Remove sensitive query parameters from URL for safe logging.
    fn sanitize_url(&self, url: &str) -> String {
        if let Some((base, query)) = url.split_once('?') {
            let sanitized_params: Vec<String> = query.split('&')
                .map(|param| {
                    if let Some((key, _)) = param.split_once('=') {
                        if self.sensitive_params.contains(key) {
                            format!("{}=***", key)
                        } else {
                            param.to_string()
                        }
                    } else {
                        param.to_string()
                    }
                })
                .collect();
            format!("{}?{}", base, sanitized_params.join("&"))
        } else {
            url.to_string()
        }
    }

    /// Parse HTTP response into JSON.
    async fn parse_response(&self, resp: reqwest::Response) -> Result<serde_json::Value> {
        let status = resp.status();
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(
                format!("Failed to parse response (status {}): {}", status, e)
            ))?;
        Ok(body)
    }
}

/// Trait for exchange-specific response error checking.
pub trait ExchangeResponseChecker {
    /// Check if the response indicates an error and return it.
    fn check_response(&self, data: &serde_json::Value) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_url_with_api_key() {
        let client = ExchangeHttpClient::new("https://api.binance.com".to_string());
        let url = "https://api.binance.com/api/v3/order?symbol=BTCUSDT&apiKey=mySecretKey123&timestamp=12345&signature=abc";
        let sanitized = client.sanitize_url(url);
        assert!(!sanitized.contains("mySecretKey123"));
        assert!(!sanitized.contains("abc"));
        assert!(sanitized.contains("apiKey=***"));
        assert!(sanitized.contains("signature=***"));
        assert!(sanitized.contains("symbol=BTCUSDT"));
    }

    #[test]
    fn test_sanitize_url_no_params() {
        let client = ExchangeHttpClient::new("https://api.binance.com".to_string());
        let url = "https://api.binance.com/api/v3/time";
        let sanitized = client.sanitize_url(url);
        assert_eq!(sanitized, url);
    }

    #[test]
    fn test_build_url() {
        let client = ExchangeHttpClient::new("https://api.binance.com".to_string());
        assert_eq!(client.build_url("/api/v3/time", ""), "https://api.binance.com/api/v3/time");
        assert_eq!(client.build_url("/api/v3/time", "a=1"), "https://api.binance.com/api/v3/time?a=1");
    }
}
