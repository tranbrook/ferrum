//! OKX REST API adapter (V5).

use async_trait::async_trait;
use base64::Engine;
use ferrum_core::config::ExchangeConfig;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::events::FerrumEvent;
use ferrum_core::traits::ExchangeAdapter;
use ferrum_core::types::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::broadcast;
use super::types::*;

type HmacSha256 = Hmac<Sha256>;

/// OKX V5 exchange adapter.
pub struct OkxAdapter {
    config: ExchangeConfig,
    client: reqwest::Client,
    base_url: String,
    event_tx: broadcast::Sender<FerrumEvent>,
}

impl OkxAdapter {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://www.okx.com".to_string() // OKX uses same URL with x-simulated-trading header
        } else {
            "https://www.okx.com".to_string()
        };
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            config,
            client: reqwest::Client::new(),
            base_url,
            event_tx,
        }
    }

    /// OKX V5 signature: HMAC SHA256 of timestamp + method + requestPath + body
    /// For GET: timestamp + "GET" + /api/v5/...?param=value + ""
    /// For POST: timestamp + "POST" + /api/v5/... + jsonBody
    /// Reference: https://www.okx.com/docs-v5/en/#rest-api-authentication-signature
    fn sign(&self, timestamp: &str, method: &str, request_path: &str, body: &str) -> String {
        let message = format!("{}{}{}{}", timestamp, method, request_path, body);
        let mut mac = HmacSha256::new_from_slice(self.config.api_secret.as_bytes())
            .expect("HMAC key valid");
        mac.update(message.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    fn passphrase(&self) -> String {
        self.config.passphrase.clone().unwrap_or_default()
    }

    /// Build signed headers for a request.
    /// `request_path` must be the full path including query string for GET,
    /// and just the endpoint path for POST.
    fn signed_headers(&self, method: &str, request_path: &str, body: &str) -> reqwest::header::HeaderMap {
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let sign = self.sign(&timestamp, method, request_path, body);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("OK-ACCESS-KEY", self.config.api_key.parse().unwrap());
        headers.insert("OK-ACCESS-SIGN", sign.parse().unwrap());
        headers.insert("OK-ACCESS-TIMESTAMP", timestamp.parse().unwrap());
        headers.insert("OK-ACCESS-PASSPHRASE", self.passphrase().parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        if self.config.testnet {
            headers.insert("x-simulated-trading", "1".parse().unwrap());
        }

        headers
    }

    async fn public_get(&self, endpoint: &str, params: &str) -> Result<serde_json::Value> {
        let url = if params.is_empty() {
            format!("{}{}", self.base_url, endpoint)
        } else {
            format!("{}{}?{}", self.base_url, endpoint, params)
        };
        tracing::debug!("GET {}", endpoint);
        let resp = self.client.get(&url).send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    async fn signed_get(&self, endpoint: &str, params: &str) -> Result<serde_json::Value> {
        // For GET requests, the requestPath used in signature includes query params
        let request_path = if params.is_empty() {
            endpoint.to_string()
        } else {
            format!("{}?{}", endpoint, params)
        };
        let url = format!("{}{}", self.base_url, request_path);
        let headers = self.signed_headers("GET", &request_path, "");
        tracing::debug!("GET (signed) {}", endpoint);
        let resp = self.client.get(&url).headers(headers).send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    async fn signed_post(&self, endpoint: &str, body: &str) -> Result<serde_json::Value> {
        // For POST requests, the requestPath is just the endpoint (no query params)
        let url = format!("{}{}", self.base_url, endpoint);
        let headers = self.signed_headers("POST", endpoint, body);
        tracing::debug!("POST (signed) {}", endpoint);
        let resp = self.client.post(&url).headers(headers).body(body.to_string()).send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let resp_body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(resp_body)
    }

    fn check_response(&self, data: &serde_json::Value) -> Result<()> {
        let code = data.get("code").and_then(|v| v.as_str()).unwrap_or("0");
        if code != "0" {
            let msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            return Err(FerrumError::ExchangeError(format!("OKX error {}: {}", code, msg)));
        }
        Ok(())
    }
}

#[async_trait]
impl ExchangeAdapter for OkxAdapter {
    fn name(&self) -> &str {
        "okx"
    }

    async fn connect(&mut self) -> Result<()> {
        let resp = self.public_get("/api/v5/public/time", "").await?;
        self.check_response(&resp)?;
        tracing::info!("Connected to OKX {}",
            if self.config.testnet { "testnet" } else { "mainnet" }
        );
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnected from OKX");
        Ok(())
    }

    async fn subscribe_orderbook(&self, _pair: TradingPair) -> Result<broadcast::Receiver<FerrumEvent>> {
        Err(FerrumError::ExchangeError(
            "OKX WebSocket streaming not yet implemented. Use get_orderbook() for REST snapshots.".into()
        ))
    }

    async fn subscribe_candles(&self, _pair: TradingPair, _interval: Interval) -> Result<broadcast::Receiver<FerrumEvent>> {
        Err(FerrumError::ExchangeError(
            "OKX WebSocket streaming not yet implemented. Use get_candles() for REST snapshots.".into()
        ))
    }

    async fn get_orderbook(&self, pair: &TradingPair) -> Result<OrderBook> {
        let inst_id = pair_to_inst_id(pair);
        let params = format!("instId={}&sz=20", inst_id);
        let data = self.public_get("/api/v5/market/books", &params).await?;
        self.check_response(&data)?;

        let bids = data.get("data")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|book| book.get("bids"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let arr = entry.as_array()?;
                    // OKX format: ["price", "size", "numOrders", ...]
                    let price: f64 = arr.first()?.as_str()?.parse().ok()?;
                    let qty: f64 = arr.get(1)?.as_str()?.parse().ok()?;
                    Some(OrderBookLevel { price: Price(price), quantity: Quantity(qty) })
                }).collect()
            })
            .unwrap_or_default();

        let asks = data.get("data")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|book| book.get("asks"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let arr = entry.as_array()?;
                    let price: f64 = arr.first()?.as_str()?.parse().ok()?;
                    let qty: f64 = arr.get(1)?.as_str()?.parse().ok()?;
                    Some(OrderBookLevel { price: Price(price), quantity: Quantity(qty) })
                }).collect()
            })
            .unwrap_or_default();

        Ok(OrderBook {
            pair: pair.clone(),
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    async fn get_candles(&self, pair: &TradingPair, interval: Interval, limit: usize) -> Result<Vec<Candle>> {
        let inst_id = pair_to_inst_id(pair);
        let bar = match interval {
            Interval::M1 => "1m",
            Interval::M3 => "3m",
            Interval::M5 => "5m",
            Interval::M15 => "15m",
            Interval::M30 => "30m",
            Interval::H1 => "1H",
            Interval::H4 => "4H",
            Interval::D1 => "1D",
            Interval::W1 => "1W",
        };
        let params = format!("instId={}&bar={}&limit={}", inst_id, bar, limit);
        let data = self.public_get("/api/v5/market/candles", &params).await?;
        self.check_response(&data)?;

        let candles = data.get("data")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let arr = entry.as_array()?;
                    if arr.len() < 7 { return None; }
                    // OKX candle: [ts, o, h, l, c, vol, volCcy, volCcyQuote, confirm]
                    Some(Candle {
                        timestamp: arr[0].as_str()?.parse().ok()?,
                        open: Price(arr[1].as_str()?.parse().ok()?),
                        high: Price(arr[2].as_str()?.parse().ok()?),
                        low: Price(arr[3].as_str()?.parse().ok()?),
                        close: Price(arr[4].as_str()?.parse().ok()?),
                        volume: Quantity(arr[5].as_str()?.parse().ok()?),
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(candles)
    }

    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse> {
        let inst_id = pair_to_inst_id(&order.pair);
        let side = match order.side {
            Side::Buy => "buy",
            Side::Sell => "sell",
            Side::Range => return Err(FerrumError::OrderError("Cannot place Range order".into())),
        };
        let ord_type = match order.order_type {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::StopMarket => "stop",
            OrderType::TakeProfitMarket => "move_order_stop",
        };

        let mut body = serde_json::json!({
            "instId": inst_id,
            "tdMode": "cash",
            "side": side,
            "ordType": ord_type,
            "sz": order.amount.0.to_string(),
        });

        if let Some(price) = order.price {
            body["px"] = serde_json::Value::String(price.0.to_string());
        }
        if let Some(ref id) = order.client_order_id {
            body["clOrdId"] = serde_json::Value::String(id.0.clone());
        }

        let data = self.signed_post("/api/v5/trade/order", &body.to_string()).await?;
        self.check_response(&data)?;

        let order_id = data.get("data")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("ordId"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(OrderResponse {
            order_id: OrderId(order_id),
            client_order_id: order.client_order_id,
            status: OrderStatus::New,
            filled_quantity: Quantity::zero(),
            avg_fill_price: None,
        })
    }

    async fn cancel_order(&self, pair: &TradingPair, order_id: &OrderId) -> Result<()> {
        let inst_id = pair_to_inst_id(pair);
        let body = serde_json::json!({
            "instId": inst_id,
            "ordId": order_id.0,
        });
        let data = self.signed_post("/api/v5/trade/cancel-order", &body.to_string()).await?;
        self.check_response(&data)?;
        Ok(())
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        let data = self.signed_get("/api/v5/account/balance", "").await?;
        self.check_response(&data)?;

        let balances = data.get("data")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("details"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let asset = entry.get("ccy")?.as_str()?.to_string();
                    let free: f64 = entry.get("availBal")?.as_str()?.parse().ok()?;
                    let used: f64 = entry.get("frozenBal")?.as_str()?.parse().ok()?;
                    Some(Balance { asset, free: Quantity(free), used: Quantity(used) })
                }).collect()
            })
            .unwrap_or_default();

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        Ok(vec![])
    }

    async fn get_server_time(&self) -> Result<i64> {
        let data = self.public_get("/api/v5/public/time", "").await?;
        self.check_response(&data)?;
        data.get("data")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|v| v as i64)
            .ok_or_else(|| FerrumError::ExchangeError("No time in response".into()))
    }
}
