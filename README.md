# Ferrum 🦀

> **Rust Trading Agent Harness** — Kiến trúc LLM-First lấy cảm hứng từ [Hummingbot Condor](https://hummingbot.org/condor/)

Ferrum (tiếng Latin: Sắt) là một open-source trading agent harness viết bằng Rust, cho phép xây dựng, cấu hình và chạy các autonomous LLM-powered trading agents quan sát thị trường crypto, phân tích chiến lược bằng LLM, và thực thi giao dịch một cách deterministic.

## ✨ Tính năng chính

- 🧠 **LLM-First** — Sử dụng LLM (OpenAI, Anthropic, Groq) làm decision engine thay vì ML truyền thống
- 🔄 **OODA Loop** — Observe → Orient → Decide → Act, kết hợp AI reasoning với deterministic execution
- 🛡️ **4-Layer Risk Engine** — Validation layer bắt buộc giữa LLM và Exchange API
- 📦 **6 Executor Types** — Position (Triple Barrier), Order, Grid, Swap, LP, DCA
- 📝 **Agent Definition** — Định nghĩa agent bằng file `agent.md` (YAML + Markdown)
- 💾 **Session Persistence** — Journal, learnings (max 20 insights), session continuity
- 🔌 **12 Crate Workspace** — Modular, trait-based architecture
- 🐳 **Single Binary** — Build ra 1 static binary ~11MB

## 🏗️ Kiến trúc

```
ferrum/
├── crates/
│   ├── ferrum-core/       # Types, traits, events, config, errors
│   ├── ferrum-exchange/   # Exchange adapters (Binance, ...)
│   ├── ferrum-executors/  # Position, Order, Grid executors
│   ├── ferrum-positions/  # Position tracking + SQLite persistence
│   ├── ferrum-risk/       # 4-layer risk engine
│   ├── ferrum-llm/        # LLM integration (OpenAI, Anthropic, Groq)
│   ├── ferrum-agent/      # OODA loop + session management
│   ├── ferrum-routines/   # Technical indicators + alerts
│   ├── ferrum-api/        # REST API (Axum) + JWT auth
│   ├── ferrum-mcp/        # MCP protocol server
│   ├── ferrum-telegram/   # Telegram bot interface
│   └── ferrum-cli/        # CLI binary
├── agents/                 # Agent definitions
│   └── grid-market-maker/  # Sample agent
├── config/                 # Configuration files
├── Dockerfile              # Multi-stage build
└── docker-compose.yml      # Ferrum + Qdrant
```

## 🚀 Quick Start

```bash
# Build
cargo build --release

# Run API server
./target/release/ferrum serve --port 8080

# Run MCP server
./target/release/ferrum mcp --port 8081

# Run trading agent
./target/release/ferrum run --agent agents/grid-market-maker/agent.md

# Start Telegram bot
./target/release/ferrum telegram --token YOUR_BOT_TOKEN

# List agents
./target/release/ferrum list --dir agents/

# Docker
docker-compose up -d
```

## 📝 Định nghĩa Agent (agent.md)

```yaml
---
name: grid-market-maker
tick_interval_secs: 30
connectors:
  - binance
trading_pair: BTC-USDT
spread_percentage: 0.5
limits:
  max_position_size_quote: 1000
  max_daily_loss_quote: 50
  max_drawdown_pct: 10
  max_open_executors: 10
---

## Goal
Maintain a grid market making strategy on BTC-USDT

## Rules
- Place buy orders below mid price
- Place sell orders above mid price
- Never exceed 50 USDT daily loss limit
- Close all positions if drawdown exceeds 10%
```

## 🛡️ Risk Engine

**Nguyên tắc vàng: LLM proposes, deterministic code DISPOSES**

| Layer | Kiểm tra |
|-------|----------|
| Pre-tick | Daily loss, max drawdown, daily cost |
| Per-executor | Executor count, order size, position limit |
| Position | Leverage, exposure |
| Kill switch | Emergency stop |

`RiskLimits` (user-only) vs `AgentConfig` (agent có thể suggest, user approve)

## 🧪 Tests

```bash
cargo test        # 36 unit tests, all passing
cargo check       # 0 errors
cargo clippy      # Lint check
```

## 🔧 Crate Ecosystem

| Crate | Dependencies | Lines |
|-------|-------------|-------|
| ferrum-core | serde, thiserror, async-trait, tokio | ~500 |
| ferrum-exchange | reqwest, hmac, sha2 | ~350 |
| ferrum-executors | ferrum-core, ferrum-risk | ~450 |
| ferrum-positions | rusqlite, parking_lot | ~200 |
| ferrum-risk | parking_lot | ~250 |
| ferrum-llm | reqwest | ~250 |
| ferrum-agent | all above | ~350 |
| ferrum-routines | ferrum-core | ~250 |
| ferrum-api | axum, tower, jsonwebtoken | ~150 |
| ferrum-mcp | axum | ~150 |
| ferrum-telegram | teloxide | ~80 |
| ferrum-cli | clap, all crates | ~120 |

## 📋 Roadmap

- [x] Phase 1: Core types, traits, events
- [x] Phase 2: Exchange adapter (Binance)
- [x] Phase 3: Executor framework (Position, Order, Grid)
- [x] Phase 4: Risk engine (4-layer)
- [x] Phase 5: LLM integration + OODA agent
- [x] Phase 6: REST API + MCP + Telegram
- [x] Phase 7: CLI + Docker deployment
- [ ] WebSocket streaming for real-time orderbook
- [ ] Bybit, OKX, Hyperliquid adapters
- [ ] RAG pipeline (Qdrant + FinBERT)
- [ ] Backtesting engine
- [ ] Local LLM inference (candle-transformers)
- [ ] Web dashboard
- [ ] Multi-agent orchestration

## 📄 License

Apache License 2.0
