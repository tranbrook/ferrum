//! Qdrant vector database client.
//!
//! Provides storage and similarity search over document embeddings
//! using Qdrant vector database.

use ferrum_core::error::{FerrumError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Qdrant client configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub dimension: usize,
    pub api_key: Option<String>,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            collection_name: "ferrum_knowledge".to_string(),
            dimension: 384,
            api_key: None,
        }
    }
}

/// A point (document) in the vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, serde_json::Value>,
}

/// Search result from Qdrant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: HashMap<String, serde_json::Value>,
}

/// Qdrant vector store client.
pub struct QdrantClient {
    config: QdrantConfig,
    client: reqwest::Client,
    /// In-memory fallback when Qdrant is not available.
    memory_store: parking_lot::RwLock<HashMap<String, VectorPoint>>,
}

impl QdrantClient {
    pub fn new(config: QdrantConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            memory_store: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Get the Qdrant REST API base URL.
    fn base_url(&self) -> String {
        format!("{}/collections/{}", self.config.url, self.config.collection_name)
    }

    /// Ensure the collection exists (create if needed).
    pub async fn ensure_collection(&self) -> Result<()> {
        let url = format!("{}/collections/{}", self.config.url, self.config.collection_name);
        let resp = self.client.get(&url).send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                tracing::info!("Qdrant collection '{}' exists", self.config.collection_name);
                return Ok(());
            }
            _ => {
                // Try to create collection
                let create_url = format!("{}/collections", self.config.url);
                let body = serde_json::json!({
                    "create_collection": {
                        "collection_name": self.config.collection_name,
                        "vectors": {
                            "size": self.config.dimension,
                            "distance": "Cosine"
                        }
                    }
                });
                let resp = self.client.put(&create_url)
                    .json(&body)
                    .send().await
                    .map_err(|e| FerrumError::DatabaseError(format!("Qdrant error: {}", e)))?;

                if resp.status().is_success() {
                    tracing::info!("Created Qdrant collection '{}'", self.config.collection_name);
                    Ok(())
                } else {
                    tracing::warn!("Qdrant unavailable, using in-memory store");
                    Ok(()) // Fall back to memory
                }
            }
        }
    }

    /// Upsert points into the collection.
    pub async fn upsert(&self, points: Vec<VectorPoint>) -> Result<()> {
        // Always store in memory as fallback
        {
            let mut store = self.memory_store.write();
            for point in &points {
                store.insert(point.id.clone(), point.clone());
            }
        }

        // Try to upsert to Qdrant
        let url = format!("{}/points", self.base_url());
        let points_json: Vec<serde_json::Value> = points.iter().map(|p| {
            serde_json::json!({
                "id": p.id,
                "vector": p.vector,
                "payload": p.payload,
            })
        }).collect();

        let body = serde_json::json!({ "points": points_json });

        let _ = self.client.put(&url)
            .json(&body)
            .send().await;

        Ok(())
    }

    /// Search for similar vectors.
    pub async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        // Try Qdrant first
        let url = format!("{}/points/search", self.base_url());
        let body = serde_json::json!({
            "vector": vector,
            "limit": limit,
            "with_payload": true,
        });

        if let Ok(resp) = self.client.post(&url).json(&body).send().await {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(results) = data.get("result").and_then(|v| v.as_array()) {
                    let search_results: Vec<SearchResult> = results.iter()
                        .filter_map(|r| {
                            let id = r.get("id")?.as_str()?.to_string();
                            let score = r.get("score")?.as_f64()? as f32;
                            let payload = r.get("payload")
                                .and_then(|v| serde_json::from_value(v.clone()).ok())
                                .unwrap_or_default();
                            Some(SearchResult { id, score, payload })
                        })
                        .collect();
                    if !search_results.is_empty() {
                        return Ok(search_results);
                    }
                }
            }
        }

        // Fall back to in-memory cosine similarity search
        self.memory_search(&vector, limit)
    }

    /// In-memory cosine similarity search (fallback).
    fn memory_search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let store = self.memory_store.read();
        let mut scored: Vec<SearchResult> = store.values()
            .map(|point| {
                let score = cosine_similarity(query, &point.vector);
                SearchResult {
                    id: point.id.clone(),
                    score,
                    payload: point.payload.clone(),
                }
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        Ok(scored)
    }

    /// Delete points by IDs.
    pub async fn delete(&self, ids: &[String]) -> Result<()> {
        {
            let mut store = self.memory_store.write();
            for id in ids {
                store.remove(id);
            }
        }
        Ok(())
    }
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 0.001);
    }

    #[test]
    fn test_memory_search() {
        let config = QdrantConfig::default();
        let client = QdrantClient::new(config);

        // Insert some points
        let points = vec![
            VectorPoint {
                id: "doc1".to_string(),
                vector: vec![1.0, 0.0, 0.0],
                payload: HashMap::new(),
            },
            VectorPoint {
                id: "doc2".to_string(),
                vector: vec![0.0, 1.0, 0.0],
                payload: HashMap::new(),
            },
        ];

        // Manually insert into memory store
        {
            let mut store = client.memory_store.write();
            for p in &points {
                store.insert(p.id.clone(), p.clone());
            }
        }

        let results = client.memory_search(&[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1");
    }
}
