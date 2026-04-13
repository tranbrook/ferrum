//! Backtest performance metrics.

use serde::{Deserialize, Serialize};

/// Comprehensive backtest performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    /// Total return (percentage).
    pub total_return: f64,
    /// Annualized return (percentage).
    pub annualized_return: f64,
    /// Maximum drawdown (percentage, negative).
    pub max_drawdown: f64,
    /// Sharpe ratio (annualized, assuming risk-free rate = 0).
    pub sharpe_ratio: f64,
    /// Sortino ratio (annualized).
    pub sortino_ratio: f64,
    /// Win rate (percentage).
    pub win_rate: f64,
    /// Total number of trades.
    pub total_trades: usize,
    /// Number of winning trades.
    pub winning_trades: usize,
    /// Number of losing trades.
    pub losing_trades: usize,
    /// Average win (absolute PnL).
    pub avg_win: f64,
    /// Average loss (absolute PnL).
    pub avg_loss: f64,
    /// Profit factor (gross profit / gross loss).
    pub profit_factor: f64,
    /// Average trade duration (in milliseconds).
    pub avg_trade_duration: f64,
    /// Total fees paid.
    pub total_fees: f64,
    /// Best trade (PnL).
    pub best_trade: f64,
    /// Worst trade (PnL).
    pub worst_trade: f64,
}

impl BacktestMetrics {
    /// Calculate metrics from a list of trade results.
    pub fn from_trades(
        trades: &[crate::types::BacktestTrade],
        initial_balance: f64,
        final_value: f64,
        start_time: i64,
        end_time: i64,
    ) -> Self {
        let closed_trades: Vec<_> = trades.iter().filter(|t| !t.is_open).collect();
        let total_trades = closed_trades.len();
        let winning: Vec<_> = closed_trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losing: Vec<_> = closed_trades.iter().filter(|t| t.pnl <= 0.0).collect();

        let total_return = if initial_balance > 0.0 {
            ((final_value - initial_balance) / initial_balance) * 100.0
        } else {
            0.0
        };

        let duration_years = if end_time > start_time {
            (end_time - start_time) as f64 / (365.25 * 24.0 * 3600.0 * 1000.0)
        } else {
            1.0
        };

        let annualized_return = if duration_years > 0.0 && initial_balance > 0.0 && final_value > 0.0 {
            ((final_value / initial_balance).powf(1.0 / duration_years) - 1.0) * 100.0
        } else {
            0.0
        };

        let win_rate = if total_trades > 0 {
            (winning.len() as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let avg_win = if !winning.is_empty() {
            winning.iter().map(|t| t.pnl).sum::<f64>() / winning.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losing.is_empty() {
            losing.iter().map(|t| t.pnl).sum::<f64>() / losing.len() as f64
        } else {
            0.0
        };

        let gross_profit: f64 = winning.iter().map(|t| t.pnl).sum();
        let gross_loss: f64 = losing.iter().map(|t| t.pnl.abs()).sum();
        let profit_factor = if gross_loss > 0.0 { gross_profit / gross_loss } else { f64::INFINITY };

        let total_fees: f64 = closed_trades.iter().map(|t| t.entry_fee + t.exit_fee).sum();

        let avg_trade_duration = if total_trades > 0 {
            closed_trades.iter()
                .filter_map(|t| t.exit_time.map(|et| (et - t.entry_time) as f64))
                .sum::<f64>() / total_trades as f64
        } else {
            0.0
        };

        let best_trade = closed_trades.iter().map(|t| t.pnl).fold(0.0_f64, f64::max);
        let worst_trade = closed_trades.iter().map(|t| t.pnl).fold(0.0_f64, f64::min);

        let max_drawdown = Self::calculate_max_drawdown(trades, initial_balance);

        // Sharpe/Sortino: Use per-trade returns, annualized by trades-per-year
        let returns: Vec<f64> = closed_trades.iter()
            .map(|t| if initial_balance > 0.0 { t.pnl / initial_balance } else { 0.0 })
            .collect();

        // Calculate annualization factor based on trades-per-year
        let trades_per_year = if duration_years > 0.0 {
            total_trades as f64 / duration_years
        } else {
            252.0 // Default: assume ~1 trade per trading day
        };
        let annualization_factor = trades_per_year.sqrt().max(1.0);

        let sharpe_ratio = Self::calculate_sharpe(&returns, annualization_factor);
        let sortino_ratio = Self::calculate_sortino(&returns, annualization_factor);

        Self {
            total_return,
            annualized_return,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            win_rate,
            total_trades,
            winning_trades: winning.len(),
            losing_trades: losing.len(),
            avg_win,
            avg_loss,
            profit_factor,
            avg_trade_duration,
            total_fees,
            best_trade,
            worst_trade,
        }
    }

    fn calculate_max_drawdown(trades: &[crate::types::BacktestTrade], initial_balance: f64) -> f64 {
        let mut peak = initial_balance;
        let mut max_dd = 0.0;
        let mut equity = initial_balance;

        for trade in trades {
            if !trade.is_open {
                equity += trade.pnl;
                if equity > peak {
                    peak = equity;
                }
                let dd = if peak > 0.0 {
                    -((equity - peak) / peak) * 100.0
                } else {
                    0.0
                };
                if dd < max_dd {
                    max_dd = dd;
                }
            }
        }

        max_dd
    }

    /// Calculate annualized Sharpe ratio.
    /// `annualization_factor` = sqrt(trades_per_year)
    fn calculate_sharpe(returns: &[f64], annualization_factor: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
            0.0
        } else {
            (mean / std_dev) * annualization_factor
        }
    }

    /// Calculate annualized Sortino ratio.
    /// `annualization_factor` = sqrt(trades_per_year)
    fn calculate_sortino(returns: &[f64], annualization_factor: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let downside_variance: f64 = returns.iter()
            .filter(|&&r| r < 0.0)
            .map(|r| r.powi(2))
            .sum::<f64>() / returns.len() as f64;
        let downside_dev = downside_variance.sqrt();
        if downside_dev == 0.0 {
            0.0
        } else {
            (mean / downside_dev) * annualization_factor
        }
    }
}

impl Default for BacktestMetrics {
    fn default() -> Self {
        Self {
            total_return: 0.0,
            annualized_return: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            win_rate: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            avg_win: 0.0,
            avg_loss: 0.0,
            profit_factor: 0.0,
            avg_trade_duration: 0.0,
            total_fees: 0.0,
            best_trade: 0.0,
            worst_trade: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_metrics() {
        let metrics = BacktestMetrics::default();
        assert_eq!(metrics.total_return, 0.0);
        assert_eq!(metrics.total_trades, 0);
    }

    #[test]
    fn test_sharpe_annualization_factor() {
        // With 252 trades/year, factor = sqrt(252) ≈ 15.87
        let returns = vec![0.01, -0.005, 0.02, -0.003, 0.015];
        let factor = 252_f64.sqrt();
        let sharpe = BacktestMetrics::calculate_sharpe(&returns, factor);
        // Sharpe should be non-zero for varied returns
        assert_ne!(sharpe, 0.0);
    }
}
