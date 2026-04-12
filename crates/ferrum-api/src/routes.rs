//! API route handlers.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Application state shared across routes
#[derive(Clone)]
pub struct AppState {
    pub agents: Vec<AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub status: String,
    pub pair: String,
    pub pnl: f64,
}

/// Health check handler
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

/// List all agents
pub async fn list_agents(State(state): State<AppState>) -> Json<Vec<AgentInfo>> {
    Json(state.agents.clone())
}

/// Get agent details
pub async fn get_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<AgentInfo>, StatusCode> {
    state.agents.iter()
        .find(|a| a.name == name)
        .map(|a| Json(a.clone()))
        .ok_or(StatusCode::NOT_FOUND)
}

/// Create agent request
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub agent_md: String,
}

/// Create a new agent
pub async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(json!({ "name": req.name, "status": "created" })))
}

/// Build the API router
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/agents", get(list_agents).post(create_agent))
        .route("/api/v1/agents/:name", get(get_agent))
        .with_state(state)
}
