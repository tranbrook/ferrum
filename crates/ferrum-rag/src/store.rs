//! Knowledge store for trading documents.

use ferrum_core::error::Result;
use serde::{Deserialize, Serialize};

/// A knowledge document to be indexed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: DocumentCategory,
    pub tags: Vec<String>,
    pub source: String,
    pub timestamp: i64,
}

/// Category of knowledge document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentCategory {
    MarketAnalysis,
    Research,
    TradingStrategy,
    RiskAnalysis,
    HistoricalPattern,
    News,
    AgentLearnings,
    General,
}

/// Search query for knowledge documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub query: String,
    pub category: Option<DocumentCategory>,
    pub limit: usize,
    pub min_score: f32,
}

impl Default for KnowledgeQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            category: None,
            limit: 10,
            min_score: 0.5,
        }
    }
}

/// Search result with document and relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSearchResult {
    pub document: KnowledgeDocument,
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_document_creation() {
        let doc = KnowledgeDocument {
            id: "test-1".to_string(),
            title: "BTC Analysis".to_string(),
            content: "Bitcoin is showing bullish patterns".to_string(),
            category: DocumentCategory::MarketAnalysis,
            tags: vec!["btc".to_string(), "bullish".to_string()],
            source: "agent".to_string(),
            timestamp: 1234567890,
        };
        assert_eq!(doc.id, "test-1");
        assert_eq!(doc.category, DocumentCategory::MarketAnalysis);
    }
}
