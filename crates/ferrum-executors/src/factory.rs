//! Executor factory - creates executors from actions.

use ferrum_core::config::TripleBarrierConfig;
use ferrum_core::error::{FerrumError, Result};
use ferrum_core::traits::Executor;
use ferrum_core::types::*;
use crate::position::{PositionExecutor, PositionExecutorConfig};
use crate::order::{OrderExecutor, OrderExecutorConfig};
use crate::grid::{GridExecutor, GridExecutorConfig};

/// Factory for creating executors from actions
pub struct ExecutorFactory;

impl ExecutorFactory {
    pub fn create_from_action(action: &ExecutorAction, controller_id: &str) -> Result<Box<dyn Executor>> {
        match action {
            ExecutorAction::Create {
                executor_type,
                connector,
                pair,
                side,
                amount,
                params,
            } => match executor_type {
                ExecutorType::Position => {
                    let tb: TripleBarrierConfig = params.get("triple_barrier")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let config = PositionExecutorConfig {
                        controller_id: controller_id.into(),
                        connector: connector.clone(),
                        pair: pair.clone(),
                        side: *side,
                        amount: *amount,
                        entry_price: params.get("entry_price")
                            .and_then(|v| v.as_f64())
                            .map(Price),
                        leverage: params.get("leverage").and_then(|v| v.as_u64()).map(|v| v as u32),
                        triple_barrier: tb,
                        keep_position: params.get("keep_position")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    };
                    Ok(Box::new(PositionExecutor::new(config)))
                }
                ExecutorType::Order => {
                    let config = OrderExecutorConfig {
                        controller_id: controller_id.into(),
                        connector: connector.clone(),
                        pair: pair.clone(),
                        side: *side,
                        amount: *amount,
                        price: params.get("price").and_then(|v| v.as_f64()).map(Price),
                        order_type: params.get("order_type")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or(OrderType::Market),
                    };
                    Ok(Box::new(OrderExecutor::new(config)))
                }
                ExecutorType::Grid => {
                    let grid_config = ferrum_core::config::GridConfig {
                        start_price: params.get("start_price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        end_price: params.get("end_price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        levels: params.get("levels").and_then(|v| v.as_u64()).unwrap_or(5) as u32,
                        total_amount_quote: amount.0,
                    };
                    let config = GridExecutorConfig {
                        controller_id: controller_id.into(),
                        connector: connector.clone(),
                        pair: pair.clone(),
                        side: *side,
                        grid: grid_config,
                        keep_position: params.get("keep_position")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                    };
                    Ok(Box::new(GridExecutor::new(config)))
                }
                ExecutorType::Swap => Err(FerrumError::Internal("SwapExecutor not yet implemented".into())),
                ExecutorType::Lp => Err(FerrumError::Internal("LPExecutor not yet implemented".into())),
                ExecutorType::Dca => Err(FerrumError::Internal("DCAExecutor not yet implemented".into())),
            },
            _ => Err(FerrumError::Internal("Cannot create executor from non-Create action".into())),
        }
    }
}
