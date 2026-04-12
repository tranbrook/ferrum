//! MCP protocol server implementation.

use axum::{
    extract::Json,
    response::Json as JsonResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// MCP request
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    pub id: Option<i64>,
}

/// MCP response
#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<McpError>,
    pub id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct McpError {
    pub code: i64,
    pub message: String,
}

/// MCP server for LLM tool use
pub struct McpServer {
    port: u16,
}

impl McpServer {
    pub fn new(port: u16) -> Self { Self { port } }

    pub fn build_router(&self) -> Router {
        Router::new().route("/mcp", post(handle_mcp_request))
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.build_router();
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], self.port));
        tracing::info!("MCP server listening on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_mcp_request(Json(req): Json<McpRequest>) -> JsonResponse<McpResponse> {
    let result = match req.method.as_str() {
        "tools/list" => Some(json!({
            "tools": [
                { "name": "place_order", "description": "Place a trading order" },
                { "name": "cancel_order", "description": "Cancel an existing order" },
                { "name": "get_balance", "description": "Get account balances" },
                { "name": "get_positions", "description": "Get open positions" },
                { "name": "get_market_data", "description": "Get current market data" },
            ]
        })),
        "tools/call" => Some(json!({ "content": "Tool execution placeholder" })),
        "initialize" => Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "ferrum-mcp", "version": "0.1.0" }
        })),
        _ => None,
    };

    let error = if result.is_none() {
        Some(McpError { code: -32601, message: format!("Method not found: {}", req.method) })
    } else {
        None
    };

    JsonResponse(McpResponse {
        jsonrpc: "2.0".to_string(),
        result,
        error,
        id: req.id,
    })
}
