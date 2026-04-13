# Ferrum - AI-Powered Crypto Trading Bot

## Overview
Ferrum is a comprehensive, modular crypto trading bot written in Rust. It features a multi-agent architecture with LLM-powered intelligence, supporting multiple exchanges with a unified adapter pattern.

## Architecture
```
ferrum/
├── crates/
│   ├── ferrum-core/         # Core types, traits, config, events
│   ├── ferrum-config/       # YAML config loading
│   ├── ferrum-exchange/     # Exchange adapters (Binance, Bybit, OKX, Hyperliquid)
│   ├── ferrum-llm/          # LLM client (OpenAI, Groq, Anthropic)
│   ├── ferrum-local-llm/    # Local LLM inference
│   ├── ferrum-rag/          # RAG pipeline (Qdrant + embeddings)
│   ├── ferrum-agent/        # OODA loop trading agent
│   ├── ferrum-orchestrator/ # Multi-agent coordination
│   ├── ferrum-executors/    # Trade execution layer
│   ├── ferrum-risk/         # Risk management engine
│   ├── ferrum-positions/    # Position tracking (SQLite)
│   ├── ferrum-backtest/     # Backtesting engine + strategies
│   ├── ferrum-paper/        # Paper trading simulation
│   ├── ferrum-routines/     # Indicators and strategy routines
│   ├── ferrum-streaming/    # WebSocket streaming
│   ├── ferrum-dashboard/    # Web dashboard (Axum)
│   ├── ferrum-api/          # REST API
│   ├── ferrum-mcp/          # MCP integration
│   ├── ferrum-telegram/     # Telegram bot
│   ├── ferrum-cli/          # CLI interface
│   └── ferrum-server/       # Main binary
```

## Key Features
- **Multi-Exchange**: Binance, Bybit, OKX, Hyperliquid via unified trait
- **AI Intelligence**: RAG pipeline with vector search, local LLM inference
- **Multi-Agent**: Orchestrated team of specialized agents (Analyst, Risk, Executor, Portfolio, Research)
- **Backtesting**: Historical replay with SMA Crossover and RSI strategies
- **Paper Trading**: Simulated execution with slippage and fee modeling
- **Risk Management**: Configurable limits, validation engine
- **Web Dashboard**: Real-time monitoring via Axum REST API

## Build & Test
```bash
cargo build
cargo test        # 77 tests, all passing
cargo check       # 0 errors
```

## Stats
- **19 crates** in workspace
- **9,111 lines** of Rust code
- **77 unit tests** passing
- **4 exchange adapters**


---

## Session Update - 2026-04-13 11:25
- **Session Started**: 2026-04-13 11:25
- **Context Status**: Verified and up-to-date

*Context automatically updated for new development session*
