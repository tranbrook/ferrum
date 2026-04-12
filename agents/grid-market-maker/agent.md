---
name: grid-market-maker
tick_interval_secs: 30
connectors:
  - binance
trading_pair: BTC-USDT
spread_percentage: 0.5
grid_levels: 5
limits:
  max_position_size_quote: 1000
  max_single_order_quote: 100
  max_daily_loss_quote: 50
  max_open_executors: 10
  max_drawdown_pct: 10
  max_cost_per_day_usd: 10
---

## Goal
Maintain a grid market making strategy on BTC-USDT, capturing spread with controlled risk.

## Rules
- Place buy orders below mid price at configured spread levels
- Place sell orders above mid price at configured spread levels
- Cancel stale orders after 5 minutes if not filled
- Never exceed 50 USDT daily loss limit
- Close all positions if drawdown exceeds 10%
- Use trailing stops for all position executors
- Avoid trading during high volatility (spread > 0.1%)
