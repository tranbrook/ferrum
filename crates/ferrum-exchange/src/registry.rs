//! Exchange adapter registry.

use ferrum_core::config::ExchangeConfig;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::traits::ExchangeAdapter;
use crate::binance::BinanceAdapter;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Registry for managing exchange adapters
pub struct ExchangeRegistry {
    adapters: HashMap<String, Arc<RwLock<Box<dyn ExchangeAdapter>>>>,
}

impl ExchangeRegistry {
    pub fn new() -> Self {
        Self { adapters: HashMap::new() }
    }

    /// Register a new exchange adapter from config
    pub fn register(&mut self, config: ExchangeConfig) -> Result<()> {
        let adapter: Box<dyn ExchangeAdapter> = match config.name.as_str() {
            "binance" => Box::new(BinanceAdapter::new(config.clone())),
            other => return Err(FerrumError::ExchangeError(format!("Unknown exchange: {}", other))),
        };
        self.adapters.insert(config.name.clone(), Arc::new(RwLock::new(adapter)));
        Ok(())
    }

    /// Get a reference to an exchange adapter
    pub fn get(&self, name: &str) -> Option<Arc<RwLock<Box<dyn ExchangeAdapter>>>> {
        self.adapters.get(name).cloned()
    }

    /// List registered exchanges
    pub fn list_exchanges(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }

    /// Connect all registered exchanges
    pub async fn connect_all(&self) -> Vec<Result<()>> {
        let mut results = Vec::new();
        for (name, adapter) in &self.adapters {
            let mut guard = adapter.write();
            match guard.connect().await {
                Ok(()) => {
                    tracing::info!("Connected to {}", name);
                    results.push(Ok(()));
                }
                Err(e) => {
                    tracing::error!("Failed to connect to {}: {:?}", name, e);
                    results.push(Err(e));
                }
            }
        }
        results
    }
}

impl Default for ExchangeRegistry {
    fn default() -> Self { Self::new() }
}
