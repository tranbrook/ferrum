//! Triple Barrier exit condition logic.

use ferrum_core::types::*;
use ferrum_core::config::TripleBarrierConfig;

/// Result of checking triple barrier conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarrierResult {
    /// No barrier triggered, continue monitoring
    Active,
    /// Take profit hit
    TakeProfit,
    /// Stop loss hit
    StopLoss,
    /// Time limit exceeded
    TimeLimit,
    /// Trailing stop triggered
    TrailingStop,
}

/// State tracker for trailing stop
#[derive(Debug, Clone)]
pub struct TrailingStopState {
    pub activation_price: f64,
    pub trailing_delta: f64,
    pub highest_price: Option<f64>,
    pub lowest_price: Option<f64>,
    pub activated: bool,
}

impl TrailingStopState {
    pub fn new(activation_price: f64, trailing_delta: f64) -> Self {
        Self {
            activation_price,
            trailing_delta,
            highest_price: None,
            lowest_price: None,
            activated: false,
        }
    }

    pub fn update(&mut self, current_price: f64, side: Side) {
        match side {
            Side::Buy => {
                if let Some(highest) = self.highest_price {
                    if current_price > highest {
                        self.highest_price = Some(current_price);
                    }
                } else {
                    self.highest_price = Some(current_price);
                }
                if !self.activated && current_price >= self.activation_price {
                    self.activated = true;
                }
            }
            Side::Sell => {
                if let Some(lowest) = self.lowest_price {
                    if current_price < lowest {
                        self.lowest_price = Some(current_price);
                    }
                } else {
                    self.lowest_price = Some(current_price);
                }
                if !self.activated && current_price <= self.activation_price {
                    self.activated = true;
                }
            }
            Side::Range => {}
        }
    }

    pub fn is_triggered(&self, current_price: f64, side: Side) -> bool {
        if !self.activated {
            return false;
        }
        match side {
            Side::Buy => {
                if let Some(highest) = self.highest_price {
                    current_price <= highest - self.trailing_delta
                } else {
                    false
                }
            }
            Side::Sell => {
                if let Some(lowest) = self.lowest_price {
                    current_price >= lowest + self.trailing_delta
                } else {
                    false
                }
            }
            Side::Range => false,
        }
    }
}

/// Check triple barrier conditions against current price
pub fn check_triple_barrier(
    config: &TripleBarrierConfig,
    entry_price: f64,
    current_price: f64,
    side: Side,
    elapsed_seconds: u64,
    trailing_state: &mut Option<TrailingStopState>,
) -> BarrierResult {
    // Check time limit first (highest priority)
    if let Some(time_limit) = config.time_limit {
        if elapsed_seconds >= time_limit {
            return BarrierResult::TimeLimit;
        }
    }

    let pnl_pct = match side {
        Side::Buy => (current_price - entry_price) / entry_price,
        Side::Sell => (entry_price - current_price) / entry_price,
        Side::Range => 0.0,
    };

    // Check take profit
    if let Some(tp) = config.take_profit {
        if pnl_pct >= tp {
            return BarrierResult::TakeProfit;
        }
    }

    // Check stop loss
    if let Some(sl) = config.stop_loss {
        if pnl_pct <= -sl {
            return BarrierResult::StopLoss;
        }
    }

    // Check trailing stop
    if let Some(ts) = trailing_state {
        ts.update(current_price, side);
        if ts.is_triggered(current_price, side) {
            return BarrierResult::TrailingStop;
        }
    } else if config.trailing_stop_activation.is_some() && config.trailing_stop_delta.is_some() {
        let activation = config.trailing_stop_activation.unwrap();
        let delta = config.trailing_stop_delta.unwrap();
        let mut state = TrailingStopState::new(
            entry_price * (1.0 + activation),
            entry_price * delta,
        );
        state.update(current_price, side);
        *trailing_state = Some(state);
        if let Some(ref ts) = *trailing_state {
            if ts.is_triggered(current_price, side) {
                return BarrierResult::TrailingStop;
            }
        }
    }

    BarrierResult::Active
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_profit_buy() {
        let config = TripleBarrierConfig {
            take_profit: Some(0.02),
            stop_loss: Some(0.01),
            time_limit: None,
            trailing_stop_activation: None,
            trailing_stop_delta: None,
        };
        let result = check_triple_barrier(&config, 100.0, 102.5, Side::Buy, 0, &mut None);
        assert_eq!(result, BarrierResult::TakeProfit);
    }

    #[test]
    fn test_stop_loss_buy() {
        let config = TripleBarrierConfig {
            take_profit: Some(0.02),
            stop_loss: Some(0.01),
            time_limit: None,
            trailing_stop_activation: None,
            trailing_stop_delta: None,
        };
        let result = check_triple_barrier(&config, 100.0, 98.5, Side::Buy, 0, &mut None);
        assert_eq!(result, BarrierResult::StopLoss);
    }

    #[test]
    fn test_take_profit_sell() {
        let config = TripleBarrierConfig {
            take_profit: Some(0.02),
            stop_loss: Some(0.01),
            time_limit: None,
            trailing_stop_activation: None,
            trailing_stop_delta: None,
        };
        let result = check_triple_barrier(&config, 100.0, 97.5, Side::Sell, 0, &mut None);
        assert_eq!(result, BarrierResult::TakeProfit);
    }

    #[test]
    fn test_time_limit() {
        let config = TripleBarrierConfig {
            take_profit: Some(0.1),
            stop_loss: Some(0.1),
            time_limit: Some(3600),
            trailing_stop_activation: None,
            trailing_stop_delta: None,
        };
        let result = check_triple_barrier(&config, 100.0, 100.5, Side::Buy, 3601, &mut None);
        assert_eq!(result, BarrierResult::TimeLimit);
    }

    #[test]
    fn test_no_barrier_triggered() {
        let config = TripleBarrierConfig {
            take_profit: Some(0.02),
            stop_loss: Some(0.01),
            time_limit: Some(3600),
            trailing_stop_activation: None,
            trailing_stop_delta: None,
        };
        let result = check_triple_barrier(&config, 100.0, 100.5, Side::Buy, 100, &mut None);
        assert_eq!(result, BarrierResult::Active);
    }

    #[test]
    fn test_trailing_stop() {
        let config = TripleBarrierConfig {
            take_profit: Some(0.1),
            stop_loss: Some(0.05),
            time_limit: None,
            trailing_stop_activation: Some(0.02),
            trailing_stop_delta: Some(0.01),
        };
        let mut state = None;
        // Price rises, activating trailing stop
        check_triple_barrier(&config, 100.0, 103.0, Side::Buy, 0, &mut state);
        // Price drops, triggering trailing stop
        let result = check_triple_barrier(&config, 100.0, 101.5, Side::Buy, 0, &mut state);
        assert_eq!(result, BarrierResult::TrailingStop);
    }
}
