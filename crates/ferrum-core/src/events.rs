//! Event types for the event-driven architecture.

use crate::types::*;
use serde::{Deserialize, Serialize};

/// Core events flowing through the system event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum FerrumEvent {
    // Market Data Events
    OrderBookUpdate {
        connector: String,
        pair: TradingPair,
        bids: Vec<OrderBookLevel>,
        asks: Vec<OrderBookLevel>,
        timestamp: i64,
    },
    CandleUpdate {
        connector: String,
        pair: TradingPair,
        interval: Interval,
        candle: Candle,
    },
    TradeUpdate {
        connector: String,
        pair: TradingPair,
        price: Price,
        quantity: Quantity,
        side: Side,
        timestamp: i64,
    },

    // Executor Events
    ExecutorCreated {
        executor_id: ExecutorId,
        controller_id: String,
        executor_type: ExecutorType,
    },
    ExecutorActivated {
        executor_id: ExecutorId,
    },
    ExecutorTerminated {
        executor_id: ExecutorId,
        close_type: CloseType,
        pnl: f64,
    },

    // Order Events
    OrderPlaced {
        order_id: OrderId,
        executor_id: Option<ExecutorId>,
        pair: TradingPair,
        side: Side,
        amount: Quantity,
    },
    OrderFilled {
        order_id: OrderId,
        filled_quantity: Quantity,
        fill_price: Price,
    },
    OrderCanceled {
        order_id: OrderId,
    },
    OrderRejected {
        order_id: OrderId,
        reason: String,
    },

    // Agent Events
    AgentTickStarted {
        agent_id: String,
        session_id: String,
    },
    AgentTickCompleted {
        agent_id: String,
        session_id: String,
        actions_taken: u32,
    },
    AgentBlocked {
        agent_id: String,
        reason: String,
    },

    // Risk Events
    RiskLimitTriggered {
        agent_id: String,
        limit: String,
        current_value: f64,
        limit_value: f64,
    },
    KillSwitchActivated {
        agent_id: String,
        reason: String,
    },
}
