//! Streaming traits for market data feeds.

use async_trait::async_trait;
use ferrum_core::error::Result;
use ferrum_core::events::FerrumEvent;
use ferrum_core::types::{Interval, TradingPair};
use tokio::sync::broadcast;

/// Trait for exchange WebSocket streaming adapters.
#[async_trait]
pub trait MarketDataStream: Send + Sync {
    /// Name of the exchange this stream connects to.
    fn exchange_name(&self) -> &str;

    /// Connect to the WebSocket endpoint.
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the WebSocket endpoint.
    async fn disconnect(&mut self) -> Result<()>;

    /// Subscribe to real-time orderbook updates for a trading pair.
    async fn subscribe_orderbook(
        &mut self,
        pair: TradingPair,
    ) -> Result<broadcast::Receiver<FerrumEvent>>;

    /// Subscribe to real-time trade updates for a trading pair.
    async fn subscribe_trades(
        &mut self,
        pair: TradingPair,
    ) -> Result<broadcast::Receiver<FerrumEvent>>;

    /// Subscribe to real-time candle/kline updates for a trading pair.
    async fn subscribe_candles(
        &mut self,
        pair: TradingPair,
        interval: Interval,
    ) -> Result<broadcast::Receiver<FerrumEvent>>;

    /// Check if the WebSocket connection is alive.
    fn is_connected(&self) -> bool;
}
