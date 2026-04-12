//! Core traits for the Ferrum trading system.

use crate::config::*;
use crate::error::*;
use crate::events::FerrumEvent;
use crate::types::*;
use async_trait::async_trait;
use tokio::sync::broadcast;

/// Exchange adapter trait - unified interface for all exchanges
#[async_trait]
pub trait ExchangeAdapter: Send + Sync {
    /// Unique name of this exchange adapter
    fn name(&self) -> &str;

    /// Connect and authenticate with the exchange
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect gracefully
    async fn disconnect(&mut self) -> Result<()>;

    /// Subscribe to order book updates
    async fn subscribe_orderbook(&self, pair: TradingPair) -> Result<broadcast::Receiver<FerrumEvent>>;

    /// Subscribe to candle updates
    async fn subscribe_candles(
        &self,
        pair: TradingPair,
        interval: Interval,
    ) -> Result<broadcast::Receiver<FerrumEvent>>;

    /// Get current order book snapshot
    async fn get_orderbook(&self, pair: &TradingPair) -> Result<OrderBook>;

    /// Get historical candles
    async fn get_candles(
        &self,
        pair: &TradingPair,
        interval: Interval,
        limit: usize,
    ) -> Result<Vec<Candle>>;

    /// Place an order on the exchange
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse>;

    /// Cancel an existing order
    async fn cancel_order(&self, pair: &TradingPair, order_id: &OrderId) -> Result<()>;

    /// Get balance for all assets
    async fn get_balances(&self) -> Result<Vec<Balance>>;

    /// Get current positions
    async fn get_positions(&self) -> Result<Vec<Position>>;

    /// Get server time for sync
    async fn get_server_time(&self) -> Result<i64>;
}

/// Executor trait - self-contained trading operations
#[async_trait]
pub trait Executor: Send + Sync {
    /// Unique identifier
    fn id(&self) -> &ExecutorId;

    /// Type of executor
    fn executor_type(&self) -> ExecutorType;

    /// Current status
    fn status(&self) -> &ExecutorStatus;

    /// Agent that controls this executor
    fn controller_id(&self) -> &str;

    /// Process a tick (market data update)
    async fn tick(&mut self, market: &MarketData) -> Result<Vec<FerrumEvent>>;

    /// Get current metrics
    fn metrics(&self) -> ExecutorMetrics;

    /// Whether to keep position on termination
    fn keep_position(&self) -> bool;

    /// Stop the executor gracefully
    async fn stop(&mut self) -> Result<()>;
}

/// Market data snapshot provided to executors on each tick
#[derive(Debug, Clone)]
pub struct MarketData {
    pub connector: String,
    pub pair: TradingPair,
    pub orderbook: Option<OrderBook>,
    pub latest_candles: Vec<Candle>,
    pub timestamp: i64,
}

/// Controller trait - decision engine
#[async_trait]
pub trait Controller: Send + Sync {
    /// Unique identifier
    fn id(&self) -> &str;

    /// Determine executor actions based on market context
    async fn determine_actions(&mut self, ctx: &MarketContext) -> Result<Vec<ExecutorAction>>;
}

/// Market context provided to controllers
#[derive(Debug, Clone)]
pub struct MarketContext {
    pub observation: MarketObservation,
    pub assessment: Option<MarketAssessment>,
    pub active_executors: Vec<ExecutorMetrics>,
    pub risk_state: RiskState,
}

/// Market observation from OODA Observe phase
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketObservation {
    pub orderbook: Option<OrderBook>,
    pub candles: Vec<Candle>,
    pub positions: Vec<Position>,
    pub active_executor_metrics: Vec<ExecutorMetrics>,
    pub risk_state: RiskState,
    pub sentiment_score: Option<f64>,
    pub timestamp: i64,
}

/// Market assessment from OODA Orient phase
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketAssessment {
    pub regime: MarketRegime,
    pub confidence: f64,
    pub rationale: String,
    pub key_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// Trading agent trait - OODA loop
#[async_trait]
pub trait TradingAgent: Send + Sync {
    /// Agent identifier
    fn agent_id(&self) -> &str;

    /// OODA: Observe - gather market data
    async fn observe(&self) -> Result<MarketObservation>;

    /// OODA: Orient - LLM reasoning
    async fn orient(&self, obs: &MarketObservation) -> Result<MarketAssessment>;

    /// OODA: Decide - generate actions
    async fn decide(&self, assessment: &MarketAssessment) -> Result<Vec<ExecutorAction>>;

    /// OODA: Act - execute with risk validation
    async fn act(&self, actions: Vec<ExecutorAction>) -> Result<Vec<ExecutorResult>>;
}

/// LLM client trait for provider-agnostic LLM calls
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a prompt and get a free-text response
    async fn complete(&self, prompt: &str) -> Result<String>;

    /// Send a prompt and get a structured JSON response as raw string
    async fn structured_complete_raw(&self, prompt: &str) -> Result<String>;

    /// Estimate cost of a prompt in USD
    fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64;
}

/// Extension trait for typed structured completion
#[async_trait]
pub trait LlmClientExt: LlmClient {
    async fn structured_complete<T: serde::de::DeserializeOwned + Send>(&self, prompt: &str) -> Result<T> {
        let raw = self.structured_complete_raw(prompt).await?;
        serde_json::from_str(&raw)
            .map_err(|e| FerrumError::LlmError(format!("Failed to parse LLM response: {}", e)))
    }
}

impl<T: LlmClient + ?Sized> LlmClientExt for T {}

/// Risk engine trait
pub trait RiskEngine: Send + Sync {
    /// Validate before tick execution
    fn validate_tick(&self, state: &RiskState, limits: &RiskLimits) -> std::result::Result<(), RiskBlock>;

    /// Validate individual executor action
    fn validate_executor_action(
        &self,
        action: &ExecutorAction,
        state: &RiskState,
        limits: &RiskLimits,
    ) -> std::result::Result<(), RiskBlock>;

    /// Compute current risk state
    fn compute_state(
        &self,
        daily_pnl: f64,
        exposure: f64,
        executor_count: u32,
        peak_equity: f64,
        current_equity: f64,
        daily_cost: f64,
        limits: &RiskLimits,
    ) -> RiskState;
}
