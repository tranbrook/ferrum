//! Prompt templates for LLM trading agent.

use ferrum_core::config::RiskState;
use ferrum_core::traits::{MarketAssessment, MarketObservation};
use ferrum_core::types::*;

/// Builder for trading agent prompts
pub struct PromptBuilder;

impl PromptBuilder {
    /// Build the OBSERVE stage prompt (not LLM-driven, purely data)
    pub fn build_observe_context(obs: &MarketObservation) -> String {
        let mut ctx = format!("Market Observation for {}\n", obs.timestamp);
        if let Some(ob) = &obs.orderbook {
            ctx.push_str(&format!("Mid Price: {}\n", ob.mid_price().unwrap_or(Price(0.0))));
            ctx.push_str(&format!("Spread: {:.4}%\n", ob.spread_pct().unwrap_or(0.0)));
        }
        if let Some(candle) = obs.candles.last() {
            ctx.push_str(&format!("Latest Close: {} Volume: {}\n", candle.close, candle.volume));
        }
        if let Some(sentiment) = obs.sentiment_score {
            ctx.push_str(&format!("Sentiment Score: {:.2}\n", sentiment));
        }
        ctx.push_str(&format!("Active Executors: {}\n", obs.active_executor_metrics.len()));
        ctx.push_str(&format!("Risk State: daily_pnl={:.2}, exposure={:.2}, drawdown={:.2}%\n",
            obs.risk_state.daily_pnl, obs.risk_state.total_exposure, obs.risk_state.drawdown_pct));
        ctx
    }

    /// Build the ORIENT stage prompt for regime detection
    pub fn build_orient_prompt(obs: &MarketObservation, learnings: &[String]) -> String {
        let mut prompt = String::new();

        prompt.push_str("Analyze the current market conditions and classify the market regime.\n\n");
        prompt.push_str("## Market Data\n");
        prompt.push_str(&Self::build_observe_context(obs));

        // Add candle context
        if obs.candles.len() >= 2 {
            let latest = &obs.candles[obs.candles.len() - 1];
            let prev = &obs.candles[obs.candles.len() - 2];
            let change_pct = (latest.close.0 - prev.close.0) / prev.close.0 * 100.0;
            prompt.push_str(&format!("\nPrice Change: {:.4}%\n", change_pct));
        }

        // Add learnings context
        if !learnings.is_empty() {
            prompt.push_str("\n## Past Learnings\n");
            for (i, learning) in learnings.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, learning));
            }
        }

        prompt.push_str("\n## Response Format\n");
        prompt.push_str("Respond with JSON:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"regime\": \"TRENDING_UP|TRENDING_DOWN|RANGING|VOLATILE|CRISIS\",\n");
        prompt.push_str("  \"confidence\": 0.0-1.0,\n");
        prompt.push_str("  \"rationale\": \"string (max 200 words)\",\n");
        prompt.push_str("  \"key_factors\": [\"list of key factors\"],\n");
        prompt.push_str("  \"recommended_actions\": [\"list of recommended actions\"]\n");
        prompt.push_str("}\n");

        prompt
    }

    /// Build the DECIDE stage prompt for signal generation
    pub fn build_decide_prompt(
        assessment: &MarketAssessment,
        obs: &MarketObservation,
        limits: &RiskState,
        risk_limits: &ferrum_core::config::RiskLimits,
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("Based on the market assessment, generate trading actions.\n\n");
        prompt.push_str(&format!("## Current Regime: {:?} (confidence: {:.2})\n", assessment.regime, assessment.confidence));
        prompt.push_str(&format!("Rationale: {}\n\n", assessment.rationale));

        prompt.push_str("## Risk Constraints\n");
        prompt.push_str(&format!("- Daily P&L: {:.2}\n", limits.daily_pnl));
        prompt.push_str(&format!("- Max Position Size: {:.2}\n", risk_limits.max_position_size_quote));
        prompt.push_str(&format!("- Max Single Order: {:.2}\n", risk_limits.max_single_order_quote));
        prompt.push_str(&format!("- Open Executors: {}/{}\n", limits.executor_count, risk_limits.max_open_executors));
        prompt.push_str(&format!("- Max Drawdown: {:.2}%\n\n", risk_limits.max_drawdown_pct));

        prompt.push_str("## Active Positions\n");
        for pos in &obs.positions {
            prompt.push_str(&format!("- {:?} {} {} @ {} (PnL: {:.2})\n",
                pos.side, pos.amount, pos.pair, pos.entry_price, pos.unrealized_pnl));
        }

        prompt.push_str("\n## Response Format\n");
        prompt.push_str("Respond with a JSON array of actions:\n");
        prompt.push_str("[{\n");
        prompt.push_str("  \"type\": \"create\",\n");
        prompt.push_str("  \"executor_type\": \"position|order|grid\",\n");
        prompt.push_str("  \"connector\": \"binance\",\n");
        prompt.push_str("  \"pair\": {\"base\": \"BTC\", \"quote\": \"USDT\"},\n");
        prompt.push_str("  \"side\": \"BUY\"|\"SELL\",\n");
        prompt.push_str("  \"amount\": 0.0,\n");
        prompt.push_str("  \"triple_barrier\": {\"take_profit\": 0.02, \"stop_loss\": 0.01, \"time_limit\": 3600}\n");
        prompt.push_str("}]\n");
        prompt.push_str("\nOr an empty array [] if no action is recommended.\n");

        prompt
    }
}
