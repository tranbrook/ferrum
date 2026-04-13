//! Bybit REST API adapter (V5).

use async_trait::async_trait;
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

/// Bybit V5 exchange adapter.
pub struct BybitAdapter {
    config: ExchangeConfig,
    client: reqwest::Client,
    base_url: String,
    event_tx: broadcast::Sender<FerrumEvent>,
}

impl BybitAdapter {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://api-testnet.bybit.com".to_string()
        } else {
            "https://api.bybit.com".to_string()
        };
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            config,
            client: reqwest::Client::new(),
            base_url,
            event_tx,
        }
    }

    /// Generate Bybit V5 signature.
    /// Signature = HMAC_SHA256(apiKey + recvWindow + timestamp + queryString/StringBody)
    fn sign(&self, timestamp: i64, recv_window: &str, params: &str) -> String {
        let payload = format!(
            "{}{}{}{}",
            self.config.api_key,
            recv_window,
            timestamp,
            params
        );
        let mut mac =
            HmacSha256::new_from_slice(self.config.api_secret.as_bytes())
                .expect("HMAC key valid");
        mac.update(payload.as_bytes());
        hex::encode(mac.finalize().into_bytes())
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
        let timestamp = chrono::Utc::now().timestamp_millis();
        let recv_window = "5000";
        let sign = self.sign(timestamp, recv_window, params);
        let url = if params.is_empty() {
            format!("{}{}", self.base_url, endpoint)
        } else {
            format!("{}{}?{}", self.base_url, endpoint, params)
        };
        tracing::debug!("GET (signed) {}", endpoint);
        let resp = self.client.get(&url)
            .header("X-BAPI-API-KEY", &self.config.api_key)
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .header("X-BAPI-SIGN", sign)
            .header("X-BAPI-RECV-WINDOW", recv_window)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    async fn signed_post(&self, endpoint: &str, body: &str) -> Result<serde_json::Value> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let recv_window = "5000";
        let sign = self.sign(timestamp, recv_window, body);
        tracing::debug!("POST (signed) {}", endpoint);
        let resp = self.client.post(format!("{}{}", self.base_url, endpoint))
            .header("X-BAPI-API-KEY", &self.config.api_key)
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .header("X-BAPI-SIGN", sign)
            .header("X-BAPI-RECV-WINDOW", recv_window)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let resp_body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(resp_body)
    }

    fn check_response(&self, data: &serde_json::Value) -> Result<()> {
        let ret_code = data.get("retCode").and_then(|v| v.as_i64()).unwrap_or(0);
        if ret_code != 0 {
            let msg = data.get("retMsg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            return Err(FerrumError::ExchangeError(format!("Bybit error {}: {}", ret_code, msg)));
        }
        Ok(())
    }
}

#[async_trait]
impl ExchangeAdapter for BybitAdapter {
    fn name(&self) -> &str {
        "bybit"
    }

    async fn connect(&mut self) -> Result<()> {
        let resp = self.public_get("/v5/market/time", "").await?;
        self.check_response(&resp)?;
        tracing::info!("Connected to Bybit {}",
            if self.config.testnet { "testnet" } else { "mainnet" }
        );
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnected from Bybit");
        Ok(())
    }

    async fn subscribe_orderbook(&self, _pair: TradingPair) -> Result<broadcast::Receiver<FerrumEvent>> {
        // WebSocket streaming not yet implemented
        Err(FerrumError::ExchangeError(
            "Bybit WebSocket streaming not yet implemented. Use get_orderbook() for REST snapshots.".into()
        ))
    }

    async fn subscribe_candles(&self, _pair: TradingPair, _interval: Interval) -> Result<broadcast::Receiver<FerrumEvent>> {
        Err(FerrumError::ExchangeError(
            "Bybit WebSocket streaming not yet implemented. Use get_candles() for REST snapshots.".into()
        ))
    }

    async fn get_orderbook(&self, pair: &TradingPair) -> Result<OrderBook> {
        let symbol = pair_to_symbol(pair);
        let params = format!("category=spot&symbol={}&limit=20", symbol);
        let data = self.public_get("/v5/market/orderbook", &params).await?;
        self.check_response(&data)?;

        let result = data.get("result").ok_or_else(|| {
            FerrumError::ExchangeError("No result in Bybit response".into())
        })?;

        // Bybit V5 orderbook format: bids/asks are arrays of ["price", "size"]
        // Reference: https://bybit-exchange.github.io/docs/v5/market/orderbook
        let bids = result.get("b")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let arr = entry.as_array()?;
                    // Index 0 = price, Index 1 = size
                    let price: f64 = arr.first()?.as_str()?.parse().ok()?;
                    let qty: f64 = arr.get(1)?.as_str()?.parse().ok()?;
                    Some(OrderBookLevel { price: Price(price), quantity: Quantity(qty) })
                }).collect()
            })
            .unwrap_or_default();

        let asks = result.get("a")
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
        let symbol = pair_to_symbol(pair);
        let interval_str = match interval {
            Interval::M1 => "1",
            Interval::M3 => "3",
            Interval::M5 => "5",
            Interval::M15 => "15",
            Interval::M30 => "30",
            Interval::H1 => "60",
            Interval::H4 => "240",
            Interval::D1 => "D",
            Interval::W1 => "W",
        };
        let params = format!("category=spot&symbol={}&interval={}&limit={}", symbol, interval_str, limit);
        let data = self.public_get("/v5/market/kline", &params).await?;
        self.check_response(&data)?;

        let result = data.get("result").ok_or_else(|| {
            FerrumError::ExchangeError("No result in Bybit response".into())
        })?;

        let candles = result.get("list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let arr = entry.as_array()?;
                    if arr.len() < 7 { return None; }
                    // Bybit V5 kline: [startTime, open, high, low, close, volume, turnover]
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
        let symbol = pair_to_symbol(&order.pair);
        let side = match order.side {
            Side::Buy => "Buy",
            Side::Sell => "Sell",
            Side::Range => return Err(FerrumError::OrderError("Cannot place Range order".into())),
        };
        let order_type = match order.order_type {
            OrderType::Market => "Market",
            OrderType::Limit => "Limit",
            OrderType::StopMarket => "StopMarket",
            OrderType::TakeProfitMarket => "TakeProfitMarket",
        };

        let mut body = serde_json::json!({
            "category": "spot",
            "symbol": symbol,
            "side": side,
            "orderType": order_type,
            "qty": order.amount.0.to_string(),
        });

        if let Some(price) = order.price {
            body["price"] = serde_json::Value::String(price.0.to_string());
        }
        if let Some(ref id) = order.client_order_id {
            body["orderLinkId"] = serde_json::Value::String(id.0.clone());
        }

        let data = self.signed_post("/v5/order/create", &body.to_string()).await?;
        self.check_response(&data)?;

        let result = data.get("result").ok_or_else(|| {
            FerrumError::OrderError("No result in Bybit order response".into())
        })?;

        Ok(OrderResponse {
            order_id: OrderId(result.get("orderId").and_then(|v| v.as_str()).unwrap_or_default().to_string()),
            client_order_id: order.client_order_id,
            status: OrderStatus::New,
            filled_quantity: Quantity::zero(),
            avg_fill_price: None,
        })
    }

    async fn cancel_order(&self, pair: &TradingPair, order_id: &OrderId) -> Result<()> {
        let symbol = pair_to_symbol(pair);
        let body = serde_json::json!({
            "category": "spot",
            "symbol": symbol,
            "orderId": order_id.0,
        });
        let data = self.signed_post("/v5/order/cancel", &body.to_string()).await?;
        self.check_response(&data)?;
        Ok(())
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        let data = self.signed_get("/v5/account/wallet-balance", "accountType=UNIFIED").await?;
        self.check_response(&data)?;

        let result = data.get("result").ok_or_else(|| {
            FerrumError::ExchangeError("No result in Bybit balance response".into())
        })?;

        let balances = result.get("list")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|account| account.get("coin"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let asset = entry.get("coin")?.as_str()?.to_string();
                    let free = entry.get("availableToWithdraw")?.as_str()?.parse().ok()?;
                    let used = entry.get("locked")?.as_str()?.parse().ok()?;
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
        let data = self.public_get("/v5/market/time", "").await?;
        self.check_response(&data)?;
        data.get("result")
            .and_then(|v| v.get("time"))
            .and_then(|v| v.as_f64())
            .map(|v| v as i64)
            .ok_or_else(|| FerrumError::ExchangeError("No time in response".into()))
    }
}
