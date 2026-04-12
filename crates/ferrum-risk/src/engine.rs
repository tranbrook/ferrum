//! Risk Engine implementation - 4-layer deterministic validation.

use ferrum_core::*;
use ferrum_core::config::*;
use ferrum_core::error::RiskBlock;
use ferrum_core::traits::RiskEngine;
use parking_lot::RwLock;
use std::sync::Arc;

/// Concrete risk engine implementation
pub struct FerrumRiskEngine {
    state: Arc<RwLock<RiskState>>,
}

impl FerrumRiskEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RiskState::default())),
        }
    }

    pub fn current_state(&self) -> RiskState {
        self.state.read().clone()
    }

    pub fn update_state(&self, new_state: RiskState) {
        *self.state.write() = new_state;
    }

    pub fn update_daily_pnl(&self, pnl: f64) {
        self.state.write().daily_pnl = pnl;
    }

    pub fn update_exposure(&self, exposure: f64) {
        self.state.write().total_exposure = exposure;
    }

    pub fn increment_executor_count(&self) {
        self.state.write().executor_count += 1;
    }

    pub fn decrement_executor_count(&self) {
        let mut state = self.state.write();
        if state.executor_count > 0 {
            state.executor_count -= 1;
        }
    }

    pub fn update_cost(&self, cost: f64) {
        self.state.write().daily_cost = cost;
    }
}

impl Default for FerrumRiskEngine {
    fn default() -> Self { Self::new() }
}

impl RiskEngine for FerrumRiskEngine {
    fn validate_tick(&self, state: &RiskState, limits: &RiskLimits) -> std::result::Result<(), RiskBlock> {
        if state.daily_pnl < -limits.max_daily_loss_quote {
            return Err(RiskBlock::DailyLossExceeded);
        }
        if state.drawdown_pct > limits.max_drawdown_pct {
            return Err(RiskBlock::MaxDrawdownExceeded);
        }
        if state.daily_cost > limits.max_cost_per_day_usd {
            return Err(RiskBlock::CostLimitExceeded);
        }
        if state.is_blocked {
            return Err(RiskBlock::AgentBlocked(
                state.block_reason.clone().unwrap_or_default()
            ));
        }
        Ok(())
    }

    fn validate_executor_action(
        &self,
        action: &ExecutorAction,
        state: &RiskState,
        limits: &RiskLimits,
    ) -> std::result::Result<(), RiskBlock> {
        match action {
            ExecutorAction::Create { amount, .. } => {
                if state.executor_count >= limits.max_open_executors {
                    return Err(RiskBlock::TooManyExecutors);
                }
                if amount.0 > limits.max_single_order_quote {
                    return Err(RiskBlock::OrderSizeExceeded);
                }
                if state.total_exposure + amount.0 > limits.max_position_size_quote {
                    return Err(RiskBlock::PositionLimitExceeded);
                }
            }
            ExecutorAction::Stop { .. } | ExecutorAction::Modify { .. } => {}
        }
        Ok(())
    }

    fn compute_state(
        &self,
        daily_pnl: f64,
        exposure: f64,
        executor_count: u32,
        peak_equity: f64,
        current_equity: f64,
        daily_cost: f64,
        limits: &RiskLimits,
    ) -> RiskState {
        let drawdown_pct = if peak_equity > 0.0 {
            ((peak_equity - current_equity) / peak_equity) * 100.0
        } else {
            0.0
        };

        let is_blocked = daily_pnl < -limits.max_daily_loss_quote
            || drawdown_pct > limits.max_drawdown_pct
            || daily_cost > limits.max_cost_per_day_usd;

        let block_reason = if is_blocked {
            let mut reasons = Vec::new();
            if daily_pnl < -limits.max_daily_loss_quote {
                reasons.push("daily_loss_exceeded".to_string());
            }
            if drawdown_pct > limits.max_drawdown_pct {
                reasons.push("max_drawdown_exceeded".to_string());
            }
            if daily_cost > limits.max_cost_per_day_usd {
                reasons.push("cost_limit_exceeded".to_string());
            }
            Some(reasons.join(", "))
        } else {
            None
        };

        RiskState {
            daily_pnl,
            total_exposure: exposure,
            executor_count,
            drawdown_pct,
            daily_cost,
            is_blocked,
            block_reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_core::types::*;

    #[test]
    fn test_validate_tick_ok() {
        let engine = FerrumRiskEngine::new();
        let state = RiskState::default();
        let limits = RiskLimits::default();
        assert!(engine.validate_tick(&state, &limits).is_ok());
    }

    #[test]
    fn test_validate_tick_daily_loss_exceeded() {
        let engine = FerrumRiskEngine::new();
        let state = RiskState {
            daily_pnl: -100.0,
            ..Default::default()
        };
        let limits = RiskLimits { max_daily_loss_quote: 50.0, ..Default::default() };
        assert_eq!(engine.validate_tick(&state, &limits), Err(RiskBlock::DailyLossExceeded));
    }

    #[test]
    fn test_validate_tick_drawdown_exceeded() {
        let engine = FerrumRiskEngine::new();
        let state = RiskState {
            drawdown_pct: 15.0,
            ..Default::default()
        };
        let limits = RiskLimits { max_drawdown_pct: 10.0, ..Default::default() };
        assert_eq!(engine.validate_tick(&state, &limits), Err(RiskBlock::MaxDrawdownExceeded));
    }

    #[test]
    fn test_validate_executor_too_many() {
        let engine = FerrumRiskEngine::new();
        let action = ExecutorAction::Create {
            executor_type: ExecutorType::Position,
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            amount: Quantity(50.0),
            params: serde_json::Value::Null,
        };
        let state = RiskState { executor_count: 10, ..Default::default() };
        let limits = RiskLimits { max_open_executors: 10, ..Default::default() };
        assert_eq!(engine.validate_executor_action(&action, &state, &limits), Err(RiskBlock::TooManyExecutors));
    }

    #[test]
    fn test_validate_executor_order_size() {
        let engine = FerrumRiskEngine::new();
        let action = ExecutorAction::Create {
            executor_type: ExecutorType::Position,
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            amount: Quantity(500.0),
            params: serde_json::Value::Null,
        };
        let state = RiskState::default();
        let limits = RiskLimits { max_single_order_quote: 100.0, ..Default::default() };
        assert_eq!(engine.validate_executor_action(&action, &state, &limits), Err(RiskBlock::OrderSizeExceeded));
    }

    #[test]
    fn test_validate_executor_position_limit() {
        let engine = FerrumRiskEngine::new();
        let action = ExecutorAction::Create {
            executor_type: ExecutorType::Position,
            connector: "binance".into(),
            pair: TradingPair::new("BTC", "USDT"),
            side: Side::Buy,
            amount: Quantity(50.0), // Below max_single_order_quote
            params: serde_json::Value::Null,
        };
        let state = RiskState { total_exposure: 980.0, ..Default::default() };
        let limits = RiskLimits {
            max_position_size_quote: 1000.0,
            max_single_order_quote: 100.0,
            ..Default::default()
        };
        // 980 + 50 = 1030 > 1000, so position limit exceeded
        assert_eq!(engine.validate_executor_action(&action, &state, &limits), Err(RiskBlock::PositionLimitExceeded));
    }

    #[test]
    fn test_compute_state_normal() {
        let engine = FerrumRiskEngine::new();
        let limits = RiskLimits::default();
        let state = engine.compute_state(10.0, 500.0, 3, 1000.0, 1010.0, 1.0, &limits);
        assert!(!state.is_blocked);
        assert_eq!(state.daily_pnl, 10.0);
    }

    #[test]
    fn test_compute_state_blocked() {
        let engine = FerrumRiskEngine::new();
        let limits = RiskLimits::default();
        let state = engine.compute_state(-60.0, 500.0, 3, 1000.0, 940.0, 1.0, &limits);
        assert!(state.is_blocked);
    }
}
