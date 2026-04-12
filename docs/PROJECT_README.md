# Ferrum - Rust Trading Agent Harness

## Overview
Ferrum (Latin: Iron) is an open-source Rust trading agent harness inspired by Hummingbot Condor. It enables users to build, configure, and run autonomous LLM-powered trading agents that observe crypto markets, reason about strategy using LLMs, and execute trades deterministically across multiple exchanges.

## Architecture
- **12-crate Cargo workspace** with trait-based abstractions
- **OODA Loop**: Observe → Orient (LLM) → Decide (LLM) → Act (deterministic)
- **Deterministic Risk Engine**: 4-layer validation between LLM and exchange
- **Event-driven**: tokio broadcast channels for real-time event routing

## Crates
| Crate | Purpose |
|-------|---------|
| ferrum-core | Domain types, traits, events, config |
| ferrum-exchange | Exchange adapters (Binance first) |
| ferrum-executors | Trading operations (Position, Order, Grid) |
| ferrum-positions | Position tracking + SQLite persistence |
| ferrum-risk | 4-layer risk engine |
| ferrum-llm | LLM integration (OpenAI, Anthropic, Groq) |
| ferrum-agent | OODA loop + session management |
| ferrum-routines | Technical indicators + alerts |
| ferrum-api | REST API (Axum) |
| ferrum-mcp | MCP protocol server |
| ferrum-telegram | Telegram bot |
| ferrum-cli | CLI binary |

## Quick Start
```bash
# Build
cargo build --release

# Run API server
ferrum serve --port 8080

# Run MCP server
ferrum mcp --port 8081

# Run a trading agent
ferrum run --agent agents/grid-market-maker/agent.md

# Start Telegram bot
ferrum telegram --token YOUR_BOT_TOKEN

# List available agents
ferrum list --dir agents/
```

## Agent Definition (agent.md)
```yaml
---
name: grid-market-maker
tick_interval_secs: 30
connectors:
  - binance
trading_pair: BTC-USDT
limits:
  max_position_size_quote: 1000
  max_daily_loss_quote: 50
  max_drawdown_pct: 10
---

## Goal
Maintain a grid market making strategy on BTC-USDT

## Rules
- Place buy orders below mid price
- Place sell orders above mid price
- Never exceed 50 USDT daily loss limit
```

## Test Status
- 36 unit tests passing
- 0 compilation errors
