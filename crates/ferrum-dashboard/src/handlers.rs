//! HTTP handlers for the dashboard.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use crate::api::*;
use crate::server::{DashboardState, SharedDashboardState};
use std::sync::Arc;

/// Build the dashboard router.
pub fn dashboard_router(state: SharedDashboardState) -> Router {
    Router::new()
        .route("/api/status", get(status_handler))
        .route("/api/positions", get(positions_handler))
        .route("/api/agents", get(agents_handler))
        .route("/api/balances", get(balances_handler))
        .route("/api/trades", get(trades_handler))
        .route("/api/start", post(start_handler))
        .route("/api/stop", post(stop_handler))
        .with_state(state)
}

/// GET /api/status
async fn status_handler(State(state): State<SharedDashboardState>) -> Json<DashboardStatus> {
    let state = state.read();
    Json(DashboardStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        agents_active: state.agents_active,
        exchanges_connected: state.exchanges_connected,
        total_pnl: state.total_pnl,
        open_positions: state.open_positions.len(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}

/// GET /api/positions
async fn positions_handler(State(state): State<SharedDashboardState>) -> Json<Vec<PositionSummary>> {
    let state = state.read();
    let positions: Vec<PositionSummary> = state.open_positions.iter().map(|p| PositionSummary {
        pair: p.pair.clone(),
        side: p.side.clone(),
        amount: p.amount,
        entry_price: p.entry_price,
        current_price: p.current_price,
        unrealized_pnl: p.unrealized_pnl,
        pnl_percentage: if p.entry_price > 0.0 && p.amount > 0.0 {
            (p.unrealized_pnl / (p.entry_price * p.amount)) * 100.0
        } else {
            0.0
        },
    }).collect();
    Json(positions)
}

/// GET /api/agents
async fn agents_handler(State(state): State<SharedDashboardState>) -> Json<Vec<AgentStatus>> {
    let state = state.read();
    Json(state.agents.clone())
}

/// GET /api/balances
async fn balances_handler(State(state): State<SharedDashboardState>) -> Json<Vec<BalanceSummary>> {
    let state = state.read();
    Json(state.balances.clone())
}

/// GET /api/trades
async fn trades_handler(State(state): State<SharedDashboardState>) -> Json<Vec<TradeHistoryEntry>> {
    let state = state.read();
    Json(state.recent_trades.clone())
}

/// POST /api/start
async fn start_handler(State(state): State<SharedDashboardState>) -> StatusCode {
    tracing::info!("Start command received");
    StatusCode::OK
}

/// POST /api/stop
async fn stop_handler(State(state): State<SharedDashboardState>) -> StatusCode {
    tracing::info!("Stop command received");
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::time::Instant;

    #[test]
    fn test_router_creation() {
        let state = Arc::new(RwLock::new(DashboardState::default()));
        let _router = dashboard_router(state);
    }
}
