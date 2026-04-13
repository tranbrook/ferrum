//! Dashboard API types.

use ferrum_core::types::*;
use serde::{Deserialize, Serialize};

/// Dashboard status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub agents_active: usize,
    pub exchanges_connected: usize,
    pub total_pnl: f64,
    pub open_positions: usize,
    pub timestamp: i64,
}

/// Position summary for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSummary {
    pub pair: String,
    pub side: String,
    pub amount: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub pnl_percentage: f64,
}

/// Agent status for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub id: String,
    pub role: String,
    pub status: String,
    pub messages_processed: usize,
    pub events_processed: usize,
    pub last_activity: Option<i64>,
}

/// Trade history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistoryEntry {
    pub id: String,
    pub pair: String,
    pub side: String,
    pub amount: f64,
    pub price: f64,
    pub pnl: f64,
    pub timestamp: i64,
}

/// Balance summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSummary {
    pub asset: String,
    pub free: f64,
    pub used: f64,
    pub total: f64,
    pub usd_value: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_status_serialization() {
        let status = DashboardStatus {
            version: "0.1.0".to_string(),
            uptime_seconds: 3600,
            agents_active: 3,
            exchanges_connected: 2,
            total_pnl: 150.0,
            open_positions: 1,
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("0.1.0"));
    }
}
