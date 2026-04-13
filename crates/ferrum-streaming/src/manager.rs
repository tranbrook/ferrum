//! Stream manager - orchestrates multiple exchange WebSocket connections.

use crate::traits::MarketDataStream;
use crate::binance::BinanceStream;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::events::FerrumEvent;
use ferrum_core::types::{Interval, TradingPair};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;

/// Manages multiple exchange WebSocket streams.
pub struct StreamManager {
    streams: HashMap<String, Arc<RwLock<Box<dyn MarketDataStream>>>>,
    event_tx: broadcast::Sender<FerrumEvent>,
}

impl StreamManager {
    /// Create a new stream manager.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(8192);
        Self {
            streams: HashMap::new(),
            event_tx,
        }
    }

    /// Register a Binance stream.
    pub fn register_binance(&mut self, testnet: bool) {
        let stream: Box<dyn MarketDataStream> = Box::new(BinanceStream::new(testnet));
        self.streams.insert(
            "binance".to_string(),
            Arc::new(RwLock::new(stream)),
        );
    }

    /// Register a custom stream adapter.
    pub fn register_stream(&mut self, name: &str, stream: Box<dyn MarketDataStream>) {
        self.streams.insert(name.to_string(), Arc::new(RwLock::new(stream)));
    }

    /// Connect all registered streams.
    pub async fn connect_all(&self) -> Vec<Result<()>> {
        let mut results = Vec::new();
        for (name, stream) in &self.streams {
            let mut guard = stream.write();
            match guard.connect().await {
                Ok(()) => {
                    tracing::info!("Stream connected: {}", name);
                    results.push(Ok(()));
                }
                Err(e) => {
                    tracing::error!("Stream connect failed {}: {}", name, e);
                    results.push(Err(e));
                }
            }
        }
        results
    }

    /// Disconnect all streams.
    pub async fn disconnect_all(&self) -> Vec<Result<()>> {
        let mut results = Vec::new();
        for (name, stream) in &self.streams {
            let mut guard = stream.write();
            match guard.disconnect().await {
                Ok(()) => {
                    tracing::info!("Stream disconnected: {}", name);
                    results.push(Ok(()));
                }
                Err(e) => {
                    tracing::error!("Stream disconnect failed {}: {}", name, e);
                    results.push(Err(e));
                }
            }
        }
        results
    }

    /// Subscribe to orderbook updates from a specific exchange.
    pub async fn subscribe_orderbook(
        &self,
        exchange: &str,
        pair: TradingPair,
    ) -> Result<broadcast::Receiver<FerrumEvent>> {
        let stream = self.streams.get(exchange).ok_or_else(|| {
            FerrumError::NotFound(format!("Stream not found: {}", exchange))
        })?;
        let mut guard = stream.write();
        guard.subscribe_orderbook(pair).await
    }

    /// Subscribe to trade updates from a specific exchange.
    pub async fn subscribe_trades(
        &self,
        exchange: &str,
        pair: TradingPair,
    ) -> Result<broadcast::Receiver<FerrumEvent>> {
        let stream = self.streams.get(exchange).ok_or_else(|| {
            FerrumError::NotFound(format!("Stream not found: {}", exchange))
        })?;
        let mut guard = stream.write();
        guard.subscribe_trades(pair).await
    }

    /// Subscribe to candle updates from a specific exchange.
    pub async fn subscribe_candles(
        &self,
        exchange: &str,
        pair: TradingPair,
        interval: Interval,
    ) -> Result<broadcast::Receiver<FerrumEvent>> {
        let stream = self.streams.get(exchange).ok_or_else(|| {
            FerrumError::NotFound(format!("Stream not found: {}", exchange))
        })?;
        let mut guard = stream.write();
        guard.subscribe_candles(pair, interval).await
    }

    /// Get a global event receiver (all exchanges).
    pub fn subscribe_all(&self) -> broadcast::Receiver<FerrumEvent> {
        self.event_tx.subscribe()
    }

    /// List registered streams.
    pub fn list_streams(&self) -> Vec<String> {
        self.streams.keys().cloned().collect()
    }

    /// Check if a specific exchange stream is connected.
    pub fn is_connected(&self, exchange: &str) -> bool {
        self.streams
            .get(exchange)
            .map(|s| s.read().is_connected())
            .unwrap_or(false)
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_manager_new() {
        let manager = StreamManager::new();
        assert!(manager.list_streams().is_empty());
    }

    #[test]
    fn test_register_binance() {
        let mut manager = StreamManager::new();
        manager.register_binance(true);
        assert!(manager.list_streams().contains(&"binance".to_string()));
    }
}
