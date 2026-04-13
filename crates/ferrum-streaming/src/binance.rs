//! Binance WebSocket streaming adapter.
//!
//! Connects to Binance's combined streams endpoint for real-time
//! orderbook, trade, and kline data.

use async_trait::async_trait;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::events::FerrumEvent;
use ferrum_core::types::*;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use crate::traits::MarketDataStream;

/// Binance WebSocket stream adapter.
pub struct BinanceStream {
    base_url: String,
    connected: bool,
    event_tx: broadcast::Sender<FerrumEvent>,
    subscriptions: Vec<String>,
}

impl BinanceStream {
    /// Create a new Binance stream adapter.
    pub fn new(testnet: bool) -> Self {
        let base_url = if testnet {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:9443/ws".to_string()
        };
        let (event_tx, _) = broadcast::channel(4096);
        Self {
            base_url,
            connected: false,
            event_tx,
            subscriptions: Vec::new(),
        }
    }

    /// Convert a TradingPair to lowercase Binance stream symbol.
    fn pair_to_stream_symbol(pair: &TradingPair) -> String {
        format!("{}{}", pair.base.to_lowercase(), pair.quote.to_lowercase())
    }

    /// Build combined stream URL from subscriptions.
    fn build_stream_url(&self) -> String {
        if self.subscriptions.is_empty() {
            self.base_url.clone()
        } else {
            format!(
                "wss://stream.binance.com:9443/stream?streams={}",
                self.subscriptions.join("/")
            )
        }
    }

    /// Parse a Binance WebSocket message into FerrumEvent(s).
    fn parse_message(&self, msg: &str) -> Vec<FerrumEvent> {
        let mut events = Vec::new();
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(msg) {
            // Combined stream format has "stream" and "data" fields
            let stream_name = data
                .get("stream")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let payload = data.get("data").unwrap_or(&data);

            if stream_name.contains("@depth") || stream_name.contains("@bookTicker") {
                if let Some(event) = self.parse_orderbook(payload) {
                    events.push(event);
                }
            } else if stream_name.contains("@trade") {
                if let Some(event) = self.parse_trade(payload) {
                    events.push(event);
                }
            } else if stream_name.contains("@kline") {
                if let Some(event) = self.parse_kline(payload) {
                    events.push(event);
                }
            }
        }
        events
    }

    fn parse_orderbook(&self, data: &serde_json::Value) -> Option<FerrumEvent> {
        // Partial book depth or book ticker
        let bids = data
            .get("b")
            .or_else(|| data.get("bids"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let price: f64 = entry.get(0)?.as_str()?.parse().ok()?;
                        let qty: f64 = entry.get(1)?.as_str()?.parse().ok()?;
                        Some(OrderBookLevel {
                            price: Price(price),
                            quantity: Quantity(qty),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let asks = data
            .get("a")
            .or_else(|| data.get("asks"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let price: f64 = entry.get(0)?.as_str()?.parse().ok()?;
                        let qty: f64 = entry.get(1)?.as_str()?.parse().ok()?;
                        Some(OrderBookLevel {
                            price: Price(price),
                            quantity: Quantity(qty),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract pair from symbol
        let symbol = data
            .get("s")
            .or_else(|| data.get("symbol"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let pair = self.symbol_to_pair(symbol);

        Some(FerrumEvent::OrderBookUpdate {
            connector: "binance".to_string(),
            pair,
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    fn parse_trade(&self, data: &serde_json::Value) -> Option<FerrumEvent> {
        let symbol = data.get("s")?.as_str()?;
        let price: f64 = data.get("p")?.as_str()?.parse().ok()?;
        let qty: f64 = data.get("q")?.as_str()?.parse().ok()?;
        let is_buyer_maker = data.get("m")?.as_bool()?;
        let timestamp = data.get("T")?.as_i64()?;

        Some(FerrumEvent::TradeUpdate {
            connector: "binance".to_string(),
            pair: self.symbol_to_pair(symbol),
            price: Price(price),
            quantity: Quantity(qty),
            side: if is_buyer_maker {
                Side::Sell
            } else {
                Side::Buy
            },
            timestamp,
        })
    }

    fn parse_kline(&self, data: &serde_json::Value) -> Option<FerrumEvent> {
        let kline = data.get("k")?;
        let interval_str = kline.get("i")?.as_str()?;
        let interval = self.parse_interval(interval_str)?;
        let symbol = kline.get("s")?.as_str()?;

        let candle = Candle {
            timestamp: kline.get("t")?.as_i64()?,
            open: Price(kline.get("o")?.as_str()?.parse().ok()?),
            high: Price(kline.get("h")?.as_str()?.parse().ok()?),
            low: Price(kline.get("l")?.as_str()?.parse().ok()?),
            close: Price(kline.get("c")?.as_str()?.parse().ok()?),
            volume: Quantity(kline.get("v")?.as_str()?.parse().ok()?),
        };

        Some(FerrumEvent::CandleUpdate {
            connector: "binance".to_string(),
            pair: self.symbol_to_pair(symbol),
            interval,
            candle,
        })
    }

    fn symbol_to_pair(&self, symbol: &str) -> TradingPair {
        let quotes = ["USDC", "USDT", "BTC", "ETH", "BNB", "BUSD"];
        for quote in &quotes {
            if symbol.ends_with(quote) {
                let base = symbol.trim_end_matches(quote);
                if !base.is_empty() {
                    return TradingPair::new(base, *quote);
                }
            }
        }
        TradingPair::new(symbol, "UNKNOWN")
    }

    fn parse_interval(&self, s: &str) -> Option<Interval> {
        match s {
            "1m" => Some(Interval::M1),
            "3m" => Some(Interval::M3),
            "5m" => Some(Interval::M5),
            "15m" => Some(Interval::M15),
            "30m" => Some(Interval::M30),
            "1h" => Some(Interval::H1),
            "4h" => Some(Interval::H4),
            "1d" => Some(Interval::D1),
            "1w" => Some(Interval::W1),
            _ => None,
        }
    }
}

#[async_trait]
impl MarketDataStream for BinanceStream {
    fn exchange_name(&self) -> &str {
        "binance"
    }

    async fn connect(&mut self) -> Result<()> {
        let url = self.build_stream_url();
        tracing::info!("Connecting to Binance WebSocket: {}", url);

        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                self.connected = true;
                let (_, mut read) = ws_stream.split();
                let event_tx = self.event_tx.clone();

                // Spawn reader task
                tokio::spawn(async move {
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                // Parse and broadcast
                                let stream = BinanceStream::new(false);
                                for event in stream.parse_message(&text) {
                                    let _ = event_tx.send(event);
                                }
                            }
                            Ok(Message::Ping(data)) => {
                                tracing::trace!("Received ping: {:?}", data);
                            }
                            Ok(Message::Close(_)) => {
                                tracing::warn!("Binance WebSocket connection closed");
                                break;
                            }
                            Err(e) => {
                                tracing::error!("Binance WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                });

                tracing::info!("Connected to Binance WebSocket");
                Ok(())
            }
            Err(e) => {
                self.connected = false;
                Err(FerrumError::WebSocketError(format!(
                    "Failed to connect to Binance WebSocket: {}",
                    e
                )))
            }
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.subscriptions.clear();
        tracing::info!("Disconnected from Binance WebSocket");
        Ok(())
    }

    async fn subscribe_orderbook(
        &mut self,
        pair: TradingPair,
    ) -> Result<broadcast::Receiver<FerrumEvent>> {
        let symbol = Self::pair_to_stream_symbol(&pair);
        let stream_name = format!("{}@depth20@100ms", symbol);
        self.subscriptions.push(stream_name);
        tracing::info!("Subscribed to Binance orderbook: {}", pair);
        Ok(self.event_tx.subscribe())
    }

    async fn subscribe_trades(
        &mut self,
        pair: TradingPair,
    ) -> Result<broadcast::Receiver<FerrumEvent>> {
        let symbol = Self::pair_to_stream_symbol(&pair);
        let stream_name = format!("{}@trade", symbol);
        self.subscriptions.push(stream_name);
        tracing::info!("Subscribed to Binance trades: {}", pair);
        Ok(self.event_tx.subscribe())
    }

    async fn subscribe_candles(
        &mut self,
        pair: TradingPair,
        interval: Interval,
    ) -> Result<broadcast::Receiver<FerrumEvent>> {
        let symbol = Self::pair_to_stream_symbol(&pair);
        let interval_str = match interval {
            Interval::M1 => "1m",
            Interval::M3 => "3m",
            Interval::M5 => "5m",
            Interval::M15 => "15m",
            Interval::M30 => "30m",
            Interval::H1 => "1h",
            Interval::H4 => "4h",
            Interval::D1 => "1d",
            Interval::W1 => "1w",
        };
        let stream_name = format!("{}@kline_{}", symbol, interval_str);
        self.subscriptions.push(stream_name);
        tracing::info!("Subscribed to Binance candles: {} {:?}", pair, interval);
        Ok(self.event_tx.subscribe())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pair_to_stream_symbol() {
        let pair = TradingPair::new("BTC", "USDT");
        assert_eq!(BinanceStream::pair_to_stream_symbol(&pair), "btcusdt");
    }

    #[test]
    fn test_parse_orderbook_message() {
        let stream = BinanceStream::new(false);
        let msg = r#"{
            "stream": "btcusdt@depth20@100ms",
            "data": {
                "s": "BTCUSDT",
                "b": [["50000.00", "1.500"], ["49999.00", "2.000"]],
                "a": [["50001.00", "0.800"], ["50002.00", "1.200"]]
            }
        }"#;
        let events = stream.parse_message(msg);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_trade_message() {
        let stream = BinanceStream::new(false);
        let msg = r#"{
            "stream": "btcusdt@trade",
            "data": {
                "e": "trade",
                "s": "BTCUSDT",
                "p": "50000.00",
                "q": "0.100",
                "m": false,
                "T": 1234567890000
            }
        }"#;
        let events = stream.parse_message(msg);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_kline_message() {
        let stream = BinanceStream::new(false);
        let msg = r#"{
            "stream": "btcusdt@kline_1m",
            "data": {
                "k": {
                    "t": 1234567890000,
                    "s": "BTCUSDT",
                    "i": "1m",
                    "o": "50000.00",
                    "h": "50100.00",
                    "l": "49900.00",
                    "c": "50050.00",
                    "v": "10.5"
                }
            }
        }"#;
        let events = stream.parse_message(msg);
        assert_eq!(events.len(), 1);
    }
}
