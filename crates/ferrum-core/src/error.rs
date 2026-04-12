//! Error types for the Ferrum trading system.

use thiserror::Error;

/// Unified error type for Ferrum
#[derive(Error, Debug)]
pub enum FerrumError {
    #[error("Invalid trading pair: {0}")]
    InvalidTradingPair(String),

    #[error("Exchange error: {0}")]
    ExchangeError(String),

    #[error("Order error: {0}")]
    OrderError(String),

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Agent error: {0}")]
    AgentError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Internal(String),
}

/// Risk-specific block reasons
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RiskBlock {
    #[error("Daily loss limit exceeded")]
    DailyLossExceeded,

    #[error("Max drawdown exceeded")]
    MaxDrawdownExceeded,

    #[error("Daily cost limit exceeded")]
    CostLimitExceeded,

    #[error("Too many open executors")]
    TooManyExecutors,

    #[error("Single order size exceeded")]
    OrderSizeExceeded,

    #[error("Position size limit exceeded")]
    PositionLimitExceeded,

    #[error("Agent is blocked: {0}")]
    AgentBlocked(String),
}

pub type Result<T> = std::result::Result<T, FerrumError>;
