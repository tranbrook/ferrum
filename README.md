# 🔥 Ferrum — AI-Powered Crypto Trading Bot

**Rust** workspace với kiến trúc multi-agent, RAG pipeline, backtesting engine, và hỗ trợ đa sàn giao dịch.

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-88%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

---

## ✨ Tính năng chính

- 🔄 **Đa sàn giao dịch**: Binance, Bybit, OKX, Hyperliquid — cùng một API
- 🤖 **Multi-Agent**: 5 vai trò chuyên biệt phối hợp (Analyst, Risk, Executor, Portfolio, Research)
- 🧠 **AI Intelligence**: RAG pipeline + LLM (OpenAI/Groq/Local)
- 📊 **Backtesting**: SMA Crossover, RSI + metrics (Sharpe, Sortino, Max Drawdown)
- 📝 **Paper Trading**: Mô phỏng真实 với slippage + fees
- 🛡️ **Risk Management**: 4-layer validation (daily loss, drawdown, cost, manual block)
- 📈 **Web Dashboard**: REST API real-time monitoring
- 📱 **Telegram Bot**: Điều khiển qua chat
- 🔌 **MCP Server**: Tích hợp với ChatGPT/Claude

---

## 🚀 Quick Start

```bash
# Clone
git clone https://github.com/tranbrook/ferrum.git
cd ferrum

# Build
cargo build --release

# Chạy tests (88 tests)
cargo test

# Xem help
./target/release/ferrum --help
```

## 📖 Hướng dẫn sử dụng

Xem hướng dẫn đầy đủ tại: **[docs/USAGE_GUIDE.md](docs/USAGE_GUIDE.md)**

---

## 🏗️ Architecture

```
ferrum/  (19 crates, 9,682 LOC)
├── ferrum-core          # Types, traits, config, events, errors
├── ferrum-exchange      # Binance | Bybit | OKX | Hyperliquid adapters
├── ferrum-llm           # OpenAI / Groq / Anthropic client
├── ferrum-local-llm     # On-device LLM inference
├── ferrum-rag           # RAG pipeline (Qdrant + embeddings)
├── ferrum-agent         # OODA loop trading agent
├── ferrum-orchestrator  # Multi-agent message routing
├── ferrum-executors     # Trade execution layer
├── ferrum-risk          # Risk management engine
├── ferrum-positions     # Position tracking (SQLite)
├── ferrum-backtest      # Strategy backtesting + metrics
├── ferrum-paper         # Paper trading simulation
├── ferrum-routines      # Indicators + strategy routines
├── ferrum-streaming     # WebSocket streaming
├── ferrum-dashboard     # Web dashboard (Axum)
├── ferrum-api           # REST API server
├── ferrum-mcp           # MCP integration
├── ferrum-telegram      # Telegram bot
└── ferrum-cli           # Command-line interface
```

---

## 📖 Usage

### CLI Commands

```bash
# Xem help
ferrum --help

# Chạy API server + Dashboard
ferrum serve --port 8080

# Chạy trading agent (paper mode)
ferrum run --agent agents/my-strategy/agent.md --paper

# Chạy Telegram bot
ferrum telegram --token "YOUR_TOKEN"

# Chạy MCP server
ferrum mcp --port 8081

# Liệt kê agents
ferrum list
```

### Định nghĩa Agent

Tạo file `agents/my-strategy/agent.md`:

```markdown
---
name: "BTC Grid Trader"
pair: "BTC-USDT"
tick_interval: 60
connectors: ["binance"]
---

# Goal
Generate profit with grid orders on BTC/USDT.

# Rules
- Close all if daily loss > $50
- Max $500 total exposure
- Use 1% portfolio per order
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/status` | System status |
| GET | `/api/positions` | Open positions |
| GET | `/api/agents` | Agent status |
| GET | `/api/balances` | Account balances |
| GET | `/api/trades` | Trade history |
| POST | `/api/start` | Start trading |
| POST | `/api/stop` | Stop trading |

📖 **Full guide**: [docs/USAGE_GUIDE.md](docs/USAGE_GUIDE.md)

---

## 🧪 Testing

```bash
# 88 tests, all passing
cargo test

# Individual crates
cargo test -p ferrum-backtest
cargo test -p ferrum-paper
cargo test -p ferrum-exchange

# With logs
RUST_LOG=debug cargo test -p ferrum-agent
```

---

## ⚠️ Disclaimer

**Ferrum là phần mềm giáo dục và nghiên cứu.** Trading cryptocurrency có rủi ro cao. LUÔN:

1. ✅ Test trên **testnet** trước
2. ✅ Chạy **paper trading** ít nhất 2 tuần
3. ✅ Bắt đầu với số tiền **rất nhỏ**
4. ❌ Không bao giờ invest hơn số bạn có thể mất

---

## 📄 License

MIT License — xem [LICENSE](LICENSE)
