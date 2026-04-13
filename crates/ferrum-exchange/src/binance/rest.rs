//! Binance REST API adapter.

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

/// Binance exchange adapter
pub struct BinanceAdapter {
    config: ExchangeConfig,
    client: reqwest::Client,
    base_url: String,
    event_tx: broadcast::Sender<FerrumEvent>,
}

impl BinanceAdapter {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://testnet.binance.vision".to_string()
        } else {
            "https://api.binance.com".to_string()
        };
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            config,
            client: reqwest::Client::new(),
            base_url,
            event_tx,
        }
    }

    fn sign_request(&self, query: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.config.api_secret.as_bytes())
            .expect("HMAC key length is valid");
        mac.update(query.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn signed_url(&self, endpoint: &str, params: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = if params.is_empty() {
            format!("timestamp={}", timestamp)
        } else {
            format!("{}&timestamp={}", params, timestamp)
        };
        let signature = self.sign_request(&query);
        format!("{}/api/v3/{}?{}&signature={}", self.base_url, endpoint, query, signature)
    }

    async fn public_get(&self, endpoint: &str, params: &str) -> Result<serde_json::Value> {
        let url = if params.is_empty() {
            format!("{}/api/v3/{}", self.base_url, endpoint)
        } else {
            format!("{}/api/v3/{}?{}", self.base_url, endpoint, params)
        };
        let resp = self.client.get(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    async fn signed_get(&self, endpoint: &str, params: &str) -> Result<serde_json::Value> {
        let url = self.signed_url(endpoint, params);
        let resp = self.client.get(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    async fn signed_post(&self, endpoint: &str, params: &str) -> Result<serde_json::Value> {
        let url = self.signed_url(endpoint, params);
        let resp = self.client.post(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    async fn signed_delete(&self, endpoint: &str, params: &str) -> Result<serde_json::Value> {
        let url = self.signed_url(endpoint, params);
        let resp = self.client.delete(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }
}

#[async_trait]
impl ExchangeAdapter for BinanceAdapter {
    fn name(&self) -> &str { "binance" }

    async fn connect(&mut self) -> Result<()> {
        // Test connectivity
        let resp = self.public_get("ping", "").await?;
        if resp.as_object().is_some() || resp == serde_json::Value::Null {
            tracing::info!("Connected to Binance {}", if self.config.testnet { "testnet" } else { "mainnet" });
            Ok(())
        } else {
            Err(FerrumError::ExchangeError("Failed to connect to Binance".into()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnected from Binance");
        Ok(())
    }

    async fn subscribe_orderbook(&self, _pair: TradingPair) -> Result<broadcast::Receiver<FerrumEvent>> {
        Err(FerrumError::ExchangeError(
            "Binance WebSocket streaming not yet implemented. Use get_orderbook() for REST snapshots.".into()
        ))
    }

    async fn subscribe_candles(&self, _pair: TradingPair, _interval: Interval) -> Result<broadcast::Receiver<FerrumEvent>> {
        Err(FerrumError::ExchangeError(
            "Binance WebSocket streaming not yet implemented. Use get_candles() for REST snapshots.".into()
        ))
    }

    async fn get_orderbook(&self, pair: &TradingPair) -> Result<OrderBook> {
        let symbol = pair_to_symbol(pair);
        let params = format!("symbol={}&limit=20", symbol);
        let data = self.public_get("depth", &params).await?;

        let bids = data.get("bids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let price: f64 = entry.get(0)?.as_str()?.parse().ok()?;
                    let qty: f64 = entry.get(1)?.as_str()?.parse().ok()?;
                    Some(OrderBookLevel { price: Price(price), quantity: Quantity(qty) })
                }).collect()
            })
            .unwrap_or_default();

        let asks = data.get("asks")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let price: f64 = entry.get(0)?.as_str()?.parse().ok()?;
                    let qty: f64 = entry.get(1)?.as_str()?.parse().ok()?;
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
            Interval::M1 => "1m", Interval::M3 => "3m", Interval::M5 => "5m",
            Interval::M15 => "15m", Interval::M30 => "30m", Interval::H1 => "1h",
            Interval::H4 => "4h", Interval::D1 => "1d", Interval::W1 => "1w",
        };
        let params = format!("symbol={}&interval={}&limit={}", symbol, interval_str, limit);
        let data = self.public_get("klines", &params).await?;

        let candles = data.as_array()
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let arr = entry.as_array()?;
                    parse_kline(arr)
                }).collect()
            })
            .unwrap_or_default();

        Ok(candles)
    }

    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse> {
        let symbol = pair_to_symbol(&order.pair);
        let side = match order.side {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
            Side::Range => return Err(FerrumError::OrderError("Cannot place Range order".into())),
        };
        let order_type = match order.order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit => "LIMIT",
            OrderType::StopMarket => "STOP_MARKET",
            OrderType::TakeProfitMarket => "TAKE_PROFIT_MARKET",
        };

        let mut params = format!(
            "symbol={}&side={}&type={}&quantity={}",
            symbol, side, order_type, order.amount.0
        );

        if let Some(price) = order.price {
            params.push_str(&format!("&price={}&timeInForce=GTC", price.0));
        }
        if let Some(ref id) = order.client_order_id {
            params.push_str(&format!("&newClientOrderId={}", id.0));
        }

        let data = self.signed_post("order", &params).await?;

        if let Some(code) = data.get("code") {
            let msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            return Err(FerrumError::OrderError(format!("Binance error {}: {}", code, msg)));
        }

        Ok(OrderResponse {
            order_id: OrderId(data.get("orderId").and_then(|v| v.as_i64()).map(|v| v.to_string()).unwrap_or_default()),
            client_order_id: order.client_order_id,
            status: OrderStatus::New,
            filled_quantity: Quantity::zero(),
            avg_fill_price: None,
        })
    }

    async fn cancel_order(&self, pair: &TradingPair, order_id: &OrderId) -> Result<()> {
        let symbol = pair_to_symbol(pair);
        let params = format!("symbol={}&orderId={}", symbol, order_id.0);
        let data = self.signed_delete("order", &params).await?;
        if data.get("code").is_some() {
            let msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown");
            return Err(FerrumError::OrderError(msg.to_string()));
        }
        Ok(())
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        let data = self.signed_get("account", "").await?;
        let balances = data.get("balances")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let asset = entry.get("asset")?.as_str()?.to_string();
                    let free = entry.get("free")?.as_str()?.parse().ok()?;
                    let used = entry.get("locked")?.as_str()?.parse().ok()?;
                    Some(Balance { asset, free: Quantity(free), used: Quantity(used) })
                }).collect()
            })
            .unwrap_or_default();
        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        // Spot positions derived from balances
        Ok(vec![])
    }

    async fn get_server_time(&self) -> Result<i64> {
        let data = self.public_get("time", "").await?;
        data.get("serverTime")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| FerrumError::ExchangeError("No serverTime in response".into()))
    }
}
