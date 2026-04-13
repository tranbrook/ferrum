//! RAG Pipeline - ties together embeddings, vector store, and retrieval.

use crate::embeddings::{create_embedding_generator, EmbeddingConfig, EmbeddingGenerator};
use crate::qdrant::{QdrantClient, QdrantConfig, VectorPoint};
use crate::store::{KnowledgeDocument, KnowledgeQuery, KnowledgeSearchResult};
use ferrum_core::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// RAG Pipeline for trading intelligence.
pub struct RagPipeline {
    embedding_generator: Box<dyn EmbeddingGenerator>,
    vector_client: Arc<QdrantClient>,
    document_store: RwLock<HashMap<String, KnowledgeDocument>>,
}

impl RagPipeline {
    /// Create a new RAG pipeline with default configuration.
    pub fn new() -> Self {
        let embedding_config = EmbeddingConfig::default();
        let qdrant_config = QdrantConfig::default();
        Self::with_config(embedding_config, qdrant_config)
    }

    /// Create a new RAG pipeline with custom configuration.
    pub fn with_config(embedding_config: EmbeddingConfig, qdrant_config: QdrantConfig) -> Self {
        let embedding_generator = create_embedding_generator(&embedding_config);
        let vector_client = Arc::new(QdrantClient::new(qdrant_config));
        Self {
            embedding_generator,
            vector_client,
            document_store: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize the pipeline (ensure Qdrant collection exists).
    pub async fn initialize(&self) -> Result<()> {
        match self.vector_client.ensure_collection().await {
            Ok(()) => tracing::info!("RAG pipeline initialized"),
            Err(e) => tracing::warn!("Qdrant unavailable, using in-memory store: {}", e),
        }
        Ok(())
    }

    /// Index a knowledge document.
    pub async fn index_document(&self, doc: KnowledgeDocument) -> Result<()> {
        // Generate embedding for document content
        let text = format!("{} {}", doc.title, doc.content);
        let embedding = self.embedding_generator.embed(&text)?;

        // Create vector point
        let mut payload = HashMap::new();
        payload.insert("title".to_string(), serde_json::Value::String(doc.title.clone()));
        payload.insert("category".to_string(), serde_json::to_value(doc.category).unwrap_or_default());
        payload.insert("tags".to_string(), serde_json::to_value(&doc.tags).unwrap_or_default());
        payload.insert("source".to_string(), serde_json::Value::String(doc.source.clone()));
        payload.insert("timestamp".to_string(), serde_json::Value::Number(doc.timestamp.into()));

        let point = VectorPoint {
            id: doc.id.clone(),
            vector: embedding,
            payload,
        };

        // Store document locally
        self.document_store.write().insert(doc.id.clone(), doc.clone());

        // Upsert to vector store
        self.vector_client.upsert(vec![point]).await?;

        tracing::debug!("Indexed document: {}", doc.id);
        Ok(())
    }

    /// Index multiple documents.
    pub async fn index_documents(&self, docs: Vec<KnowledgeDocument>) -> Result<()> {
        for doc in docs {
            self.index_document(doc).await?;
        }
        Ok(())
    }

    /// Search for relevant documents.
    pub async fn search(&self, query: KnowledgeQuery) -> Result<Vec<KnowledgeSearchResult>> {
        // Generate query embedding
        let query_embedding = self.embedding_generator.embed(&query.query)?;

        // Search vector store
        let results = self.vector_client.search(query_embedding, query.limit).await?;

        // Convert to knowledge search results
        let store = self.document_store.read();
        let search_results: Vec<KnowledgeSearchResult> = results
            .into_iter()
            .filter(|r| r.score >= query.min_score)
            .filter_map(|r| {
                let doc = store.get(&r.id)?;

                // Filter by category if specified
                if let Some(ref cat) = query.category {
                    if doc.category != *cat {
                        return None;
                    }
                }

                Some(KnowledgeSearchResult {
                    document: doc.clone(),
                    score: r.score,
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Generate a RAG-augmented prompt for LLM.
    pub async fn augment_prompt(&self, prompt: &str, context_limit: usize) -> Result<String> {
        let query = KnowledgeQuery {
            query: prompt.to_string(),
            limit: context_limit,
            min_score: 0.3,
            ..Default::default()
        };

        let results = self.search(query).await?;

        if results.is_empty() {
            return Ok(prompt.to_string());
        }

        let mut context = String::from("Relevant context from knowledge base:\n\n");
        for (i, result) in results.iter().enumerate() {
            context.push_str(&format!(
                "[{}] {} (score: {:.2})\n{}\n\n",
                i + 1,
                result.document.title,
                result.score,
                result.document.content
            ));
        }
        context.push_str(&format!("Based on the above context, answer:\n{}", prompt));

        Ok(context)
    }

    /// Get the number of indexed documents.
    pub fn document_count(&self) -> usize {
        self.document_store.read().len()
    }
}

impl Default for RagPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::DocumentCategory;

    #[tokio::test]
    async fn test_rag_pipeline_index_and_search() {
        let pipeline = RagPipeline::new();
        pipeline.initialize().await.unwrap();

        let doc = KnowledgeDocument {
            id: "test-1".to_string(),
            title: "BTC Bullish Pattern".to_string(),
            content: "Bitcoin showing cup and handle pattern on 4H chart".to_string(),
            category: DocumentCategory::MarketAnalysis,
            tags: vec!["btc".to_string()],
            source: "analyst".to_string(),
            timestamp: 1234567890,
        };

        pipeline.index_document(doc).await.unwrap();
        assert_eq!(pipeline.document_count(), 1);

        let query = KnowledgeQuery {
            query: "BTC bullish".to_string(),
            limit: 5,
            min_score: 0.0,
            ..Default::default()
        };

        let results = pipeline.search(query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_augment_prompt() {
        let pipeline = RagPipeline::new();
        pipeline.initialize().await.unwrap();

        let doc = KnowledgeDocument {
            id: "test-2".to_string(),
            title: "ETH Analysis".to_string(),
            content: "Ethereum support at 3000".to_string(),
            category: DocumentCategory::MarketAnalysis,
            tags: vec!["eth".to_string()],
            source: "agent".to_string(),
            timestamp: 1234567890,
        };

        pipeline.index_document(doc).await.unwrap();

        let augmented = pipeline.augment_prompt("What is ETH support?", 5).await.unwrap();
        assert!(augmented.contains("ETH Analysis"));
        assert!(augmented.contains("What is ETH support?"));
    }
}
