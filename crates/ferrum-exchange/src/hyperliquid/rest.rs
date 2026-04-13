//! Hyperliquid REST API adapter.
//! Hyperliquid uses a unique JSON-RPC style API.

use async_trait::async_trait;
use ferrum_core::config::ExchangeConfig;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::events::FerrumEvent;
use ferrum_core::traits::ExchangeAdapter;
use ferrum_core::types::*;
use tokio::sync::broadcast;
use super::types::*;

/// Hyperliquid exchange adapter.
pub struct HyperliquidAdapter {
    config: ExchangeConfig,
    client: reqwest::Client,
    base_url: String,
    event_tx: broadcast::Sender<FerrumEvent>,
}

impl HyperliquidAdapter {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://api.hyperliquid-testnet.xyz".to_string()
        } else {
            "https://api.hyperliquid.xyz".to_string()
        };
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            config,
            client: reqwest::Client::new(),
            base_url,
            event_tx,
        }
    }

    /// Post JSON-RPC style request to Hyperliquid info endpoint.
    async fn post_info(&self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/info", self.base_url);
        tracing::debug!("POST info to Hyperliquid");
        let resp = self.client.post(&url)
            .json(request)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    /// Post to exchange endpoint (requires EIP-712-style signing).
    async fn post_exchange(&self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/exchange", self.base_url);
        tracing::debug!("POST exchange to Hyperliquid");
        let resp = self.client.post(&url)
            .json(request)
            .send().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        let body: serde_json::Value = resp.json().await
            .map_err(|e| FerrumError::ExchangeError(e.to_string()))?;
        Ok(body)
    }

    fn check_response(&self, data: &serde_json::Value) -> Result<()> {
        if let Some(status) = data.get("status") {
            if status.as_str() == Some("err") {
                let msg = data.get("response")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(FerrumError::ExchangeError(format!("Hyperliquid error: {}", msg)));
            }
        }
        Ok(())
    }

    /// Resolve the asset index for a coin from Hyperliquid metadata.
    /// Falls back to querying the meta endpoint to find the correct index.
    async fn resolve_asset_index(&self, coin: &str) -> Result<i64> {
        let request = serde_json::json!({ "type": "meta" });
        let data = self.post_info(&request).await?;

        if let Some(universe) = data.get("universe").and_then(|v| v.as_array()) {
            for (idx, asset) in universe.iter().enumerate() {
                if asset.get("name").and_then(|v| v.as_str()) == Some(coin) {
                    return Ok(idx as i64);
                }
            }
        }

        // Fallback: return 0 with a warning (BTC is typically index 0)
        tracing::warn!("Could not resolve asset index for {}, defaulting to 0", coin);
        Ok(0)
    }
}

#[async_trait]
impl ExchangeAdapter for HyperliquidAdapter {
    fn name(&self) -> &str {
        "hyperliquid"
    }

    async fn connect(&mut self) -> Result<()> {
        let request = serde_json::json!({ "type": "metaAndAssetCtxs" });
        let resp = self.post_info(&request).await?;
        tracing::info!("Connected to Hyperliquid {}",
            if self.config.testnet { "testnet" } else { "mainnet" }
        );
        let _ = resp;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Disconnected from Hyperliquid");
        Ok(())
    }

    async fn subscribe_orderbook(&self, _pair: TradingPair) -> Result<broadcast::Receiver<FerrumEvent>> {
        Err(FerrumError::ExchangeError(
            "Hyperliquid WebSocket streaming not yet implemented. Use get_orderbook() for REST snapshots.".into()
        ))
    }

    async fn subscribe_candles(&self, _pair: TradingPair, _interval: Interval) -> Result<broadcast::Receiver<FerrumEvent>> {
        Err(FerrumError::ExchangeError(
            "Hyperliquid WebSocket streaming not yet implemented. Use get_candles() for REST snapshots.".into()
        ))
    }

    async fn get_orderbook(&self, pair: &TradingPair) -> Result<OrderBook> {
        let coin = pair_to_coin(pair);
        let request = serde_json::json!({
            "type": "l2Book",
            "coin": coin
        });
        let data = self.post_info(&request).await?;

        let bids = data.get("levels")
            .and_then(|v| v.as_array())
            .and_then(|levels| levels.first())
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let price: f64 = entry.get("px")?.as_str()?.parse().ok()?;
                    let qty: f64 = entry.get("sz")?.as_str()?.parse().ok()?;
                    Some(OrderBookLevel { price: Price(price), quantity: Quantity(qty) })
                }).collect()
            })
            .unwrap_or_default();

        let asks = data.get("levels")
            .and_then(|v| v.as_array())
            .and_then(|levels| levels.get(1))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let price: f64 = entry.get("px")?.as_str()?.parse().ok()?;
                    let qty: f64 = entry.get("sz")?.as_str()?.parse().ok()?;
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
        let coin = pair_to_coin(pair);
        let resolution = match interval {
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
        let request = serde_json::json!({
            "type": "candleSnapshot",
            "req": {
                "coin": coin,
                "resolution": resolution,
                "limit": limit
            }
        });
        let data = self.post_info(&request).await?;

        let candles = data.as_array()
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    Some(Candle {
                        timestamp: entry.get("t")?.as_i64()?,
                        open: Price(entry.get("o")?.as_str()?.parse().ok()?),
                        high: Price(entry.get("h")?.as_str()?.parse().ok()?),
                        low: Price(entry.get("l")?.as_str()?.parse().ok()?),
                        close: Price(entry.get("c")?.as_str()?.parse().ok()?),
                        volume: Quantity(entry.get("v")?.as_str()?.parse().ok()?),
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(candles)
    }

    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse> {
        let coin = pair_to_coin(&order.pair);
        let side_val = match order.side {
            Side::Buy => true,
            Side::Sell => false,
            Side::Range => return Err(FerrumError::OrderError("Cannot place Range order".into())),
        };
        let order_type = match order.order_type {
            OrderType::Market => "ioc",
            OrderType::Limit => "gtc",
            _ => return Err(FerrumError::OrderError("Hyperliquid: unsupported order type".into())),
        };

        let asset_index = self.resolve_asset_index(&coin).await?;

        let request = serde_json::json!({
            "action": {
                "type": "order",
                "orders": [{
                    "a": asset_index,
                    "b": side_val,
                    "p": order.price.map(|p| p.0.to_string()).unwrap_or("0".to_string()),
                    "s": order.amount.0.to_string(),
                    "r": order_type == "ioc",
                    "t": { "limit": { "tif": order_type } },
                    "c": order.client_order_id.as_ref().map(|id| id.0.clone()).unwrap_or_default(),
                }],
                "grouping": "na"
            }
        });

        let data = self.post_exchange(&request).await?;
        self.check_response(&data)?;

        let order_id = data.get("response")
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("statuses"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("resting"))
            .and_then(|v| v.get("oid"))
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
        let coin = pair_to_coin(pair);
        let asset_index = self.resolve_asset_index(&coin).await?;

        let request = serde_json::json!({
            "action": {
                "type": "cancel",
                "cancels": [{
                    "a": asset_index,
                    "oid": order_id.0.parse::<i64>().unwrap_or(0),
                }]
            }
        });
        let data = self.post_exchange(&request).await?;
        self.check_response(&data)?;
        Ok(())
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        let request = serde_json::json!({
            "type": "clearinghouseState",
            "user": self.config.api_key
        });
        let data = self.post_info(&request).await?;

        let balances = data.get("crossMarginSummary")
            .and_then(|v| v.get("assetPositions"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let asset = entry.get("coin")?.as_str()?.to_string();
                    let free: f64 = entry.get("hold")?.as_str()?.parse().ok()?;
                    let used: f64 = entry.get("marginUsed")?.as_str()?.parse().ok()?;
                    Some(Balance { asset, free: Quantity(free), used: Quantity(used) })
                }).collect()
            })
            .unwrap_or_else(|| {
                vec![Balance {
                    asset: "USDC".to_string(),
                    free: Quantity(data.get("withdrawable").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0)),
                    used: Quantity::zero(),
                }]
            });

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let request = serde_json::json!({
            "type": "clearinghouseState",
            "user": self.config.api_key
        });
        let data = self.post_info(&request).await?;

        let positions = data.get("assetPositions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    let position = entry.get("position")?;
                    let coin = position.get("coin")?.as_str()?;
                    let size: f64 = position.get("szi")?.as_str()?.parse().ok()?;
                    let entry_price: f64 = position.get("entryPx")?.as_str()?.parse().ok()?;
                    let unrealized_pnl: f64 = position.get("unrealizedPnl")?.as_str()?.parse().ok()?;

                    Some(Position {
                        id: format!("hl-{}", coin),
                        connector: "hyperliquid".to_string(),
                        pair: coin_to_pair(coin, "USDC"),
                        side: if size > 0.0 { Side::Buy } else { Side::Sell },
                        amount: Quantity(size.abs()),
                        entry_price: Price(entry_price),
                        unrealized_pnl,
                        realized_pnl: 0.0,
                        leverage: position.get("leverage").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()),
                        is_lp: false,
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(positions)
    }

    async fn get_server_time(&self) -> Result<i64> {
        let _ = self.post_info(&serde_json::json!({ "type": "metaAndAssetCtxs" })).await?;
        Ok(chrono::Utc::now().timestamp_millis())
    }
}
