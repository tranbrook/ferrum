//! Dashboard server.

use crate::api::*;
use crate::handlers::dashboard_router;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;

/// Thread-safe shared dashboard state.
/// Uses RwLock to allow concurrent reads from multiple HTTP handlers
/// while ensuring safe writes from agent threads.
#[derive(Debug)]
pub struct DashboardState {
    pub start_time: Instant,
    pub agents_active: usize,
    pub exchanges_connected: usize,
    pub total_pnl: f64,
    pub open_positions: Vec<OpenPosition>,
    pub agents: Vec<AgentStatus>,
    pub balances: Vec<BalanceSummary>,
    pub recent_trades: Vec<TradeHistoryEntry>,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            agents_active: 0,
            exchanges_connected: 0,
            total_pnl: 0.0,
            open_positions: Vec::new(),
            agents: Vec::new(),
            balances: Vec::new(),
            recent_trades: Vec::new(),
        }
    }
}

/// Thread-safe wrapper for DashboardState.
pub type SharedDashboardState = Arc<RwLock<DashboardState>>;

/// Simplified open position for dashboard display.
#[derive(Debug, Clone)]
pub struct OpenPosition {
    pub pair: String,
    pub side: String,
    pub amount: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
}

/// Dashboard server configuration.
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub host: String,
    pub port: u16,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

/// The dashboard web server.
pub struct DashboardServer {
    config: DashboardConfig,
    state: SharedDashboardState,
}

impl DashboardServer {
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(DashboardState::default())),
        }
    }

    /// Get a handle to the shared state for external updates.
    /// Thread-safe: wrap in RwLock for concurrent access.
    pub fn state(&self) -> SharedDashboardState {
        self.state.clone()
    }

    /// Start the dashboard server.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let app = dashboard_router(self.state.clone());

        tracing::info!("Dashboard listening on http://{}", addr);

        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_dashboard_server_creation() {
        let server = DashboardServer::new(DashboardConfig::default());
        let state = server.state();
        let guard = state.read();
        assert_eq!(guard.agents_active, 0);
    }

    #[test]
    fn test_dashboard_state_concurrent_access() {
        let state = Arc::new(RwLock::new(DashboardState::default()));

        // Simulate concurrent writes and reads
        {
            let mut w = state.write();
            w.agents_active = 3;
            w.total_pnl = 150.0;
        }

        {
            let r = state.read();
            assert_eq!(r.agents_active, 3);
            assert!((r.total_pnl - 150.0).abs() < 0.01);
        }
    }
}
