//! MCP tool definitions for trading operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool definition
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Get all available MCP tools
pub fn get_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "place_order".into(),
            description: "Place a trading order on an exchange".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "connector": { "type": "string", "description": "Exchange name" },
                    "pair": { "type": "string", "description": "Trading pair e.g. BTC-USDT" },
                    "side": { "type": "string", "enum": ["BUY", "SELL"] },
                    "amount": { "type": "number", "description": "Order quantity" },
                    "order_type": { "type": "string", "enum": ["MARKET", "LIMIT"] },
                    "price": { "type": "number", "description": "Limit price (required for LIMIT orders)" }
                },
                "required": ["connector", "pair", "side", "amount", "order_type"]
            }),
        },
        ToolDefinition {
            name: "cancel_order".into(),
            description: "Cancel an existing order".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "connector": { "type": "string" },
                    "pair": { "type": "string" },
                    "order_id": { "type": "string" }
                },
                "required": ["connector", "pair", "order_id"]
            }),
        },
        ToolDefinition {
            name: "get_balance".into(),
            description: "Get account balance for an exchange".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "connector": { "type": "string" }
                },
                "required": ["connector"]
            }),
        },
        ToolDefinition {
            name: "get_positions".into(),
            description: "Get all open positions".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "connector": { "type": "string" }
                }
            }),
        },
        ToolDefinition {
            name: "get_market_data".into(),
            description: "Get current market data for a trading pair".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "connector": { "type": "string" },
                    "pair": { "type": "string" }
                },
                "required": ["connector", "pair"]
            }),
        },
    ]
}
