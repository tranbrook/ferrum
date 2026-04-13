//! Orchestrator message types.

use ferrum_core::types::TradingPair;
use serde::{Deserialize, Serialize};

/// Unique agent identifier.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Message types for inter-agent communication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    MarketAnalysis,
    RiskAssessment,
    TradeSignal,
    ExecutionReport,
    PortfolioUpdate,
    ResearchResult,
    Alert,
    Command,
    Stop,
}

/// Agent role in the trading team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Analyst,
    RiskManager,
    Executor,
    PortfolioManager,
    Researcher,
}

impl AgentRole {
    /// Message types this role is interested in.
    pub fn subscribes_to(&self) -> Vec<MessageType> {
        match self {
            AgentRole::Analyst => vec![MessageType::ResearchResult, MessageType::Alert],
            AgentRole::RiskManager => vec![MessageType::TradeSignal, MessageType::PortfolioUpdate, MessageType::MarketAnalysis],
            AgentRole::Executor => vec![MessageType::TradeSignal, MessageType::RiskAssessment],
            AgentRole::PortfolioManager => vec![MessageType::ExecutionReport, MessageType::MarketAnalysis],
            AgentRole::Researcher => vec![MessageType::Command, MessageType::Alert],
        }
    }
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::Analyst => write!(f, "analyst"),
            AgentRole::RiskManager => write!(f, "risk_manager"),
            AgentRole::Executor => write!(f, "executor"),
            AgentRole::PortfolioManager => write!(f, "portfolio_manager"),
            AgentRole::Researcher => write!(f, "researcher"),
        }
    }
}

/// Orchestrator message for inter-agent communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorMessage {
    pub id: String,
    pub from: AgentId,
    pub to: Option<AgentId>,
    pub message_type: MessageType,
    pub content: String,
    pub priority: Priority,
    pub timestamp: i64,
    pub pair: Option<TradingPair>,
    pub metadata: serde_json::Value,
}

impl OrchestratorMessage {
    pub fn new(from: AgentId, to: Option<AgentId>, message_type: MessageType, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            message_type,
            content,
            priority: Priority::Normal,
            timestamp: chrono::Utc::now().timestamp_millis(),
            pair: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn broadcast(from: AgentId, message_type: MessageType, content: String) -> Self {
        Self::new(from, None, message_type, content)
    }

    pub fn with_pair(mut self, pair: TradingPair) -> Self {
        self.pair = Some(pair);
        self
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = OrchestratorMessage::new(
            AgentId("analyst".to_string()),
            Some(AgentId("executor".to_string())),
            MessageType::TradeSignal,
            "Buy BTC".to_string(),
        );
        assert_eq!(msg.from.0, "analyst");
    }
}
