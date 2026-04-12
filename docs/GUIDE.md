# 📖 Hướng Dẫn Sử Dụng Ferrum

> **Ferrum** — Rust Trading Agent Harness | LLM-First Crypto Trading System

---

## 📋 Mục Lục

1. [Cài đặt](#1-cài-đặt)
2. [Cấu hình](#2-cấu-hình)
3. [Tạo Agent](#3-tạo-agent)
4. [Chạy Agent](#4-chạy-agent)
5. [API Server](#5-api-server)
6. [MCP Server](#6-mcp-server)
7. [Telegram Bot](#7-telegram-bot)
8. [Risk Engine](#8-risk-engine)
9. [Executors](#9-executors)
10. [Technical Indicators](#10-technical-indicators)
11. [Docker Deployment](#11-docker-deployment)
12. [Troubleshooting](#12-troubleshooting)

---

## 1. Cài đặt

### Yêu cầu hệ thống

- **Rust** >= 1.75 (`rustup install stable`)
- **OpenSSL** (cho TLS)
- **SQLite3** (bundled, tự compile)
- **Git**

### Build từ source

```bash
# Clone repository
cd /home/tranbrook/ferrum

# Build debug (nhanh, có debug info)
cargo build

# Build release (chậm hơn, binary nhỏ + nhanh)
cargo build --release

# Binary nằm tại:
# ./target/debug/ferrum    (debug)
# ./target/release/ferrum  (release, 11MB)
```

### Chạy tests

```bash
# Chạy tất cả 36 unit tests
cargo test

# Chạy tests cho 1 crate cụ thể
cargo test -p ferrum-risk
cargo test -p ferrum-core
cargo test -p ferrum-executors

# Xem output chi tiết
cargo test -- --nocapture

# Lint check
cargo clippy
```

---

## 2. Cấu hình

### File cấu hình chính: `config/ferrum.toml`

```toml
# Ferrum Configuration

database_path = "ferrum.db"    # SQLite database path
log_level = "info"              # trace, debug, info, warn, error
api_port = 8080                 # REST API port

# Exchange credentials
[[exchanges]]
name = "binance"
api_key = "YOUR_BINANCE_API_KEY"
api_secret = "YOUR_BINANCE_API_SECRET"
testnet = true                  # ⚠️ LUÔN dùng testnet khi test!

# Thêm exchange khác (khi có adapter)
# [[exchanges]]
# name = "bybit"
# api_key = "YOUR_BYBIT_API_KEY"
# api_secret = "YOUR_BYBIT_API_SECRET"
# testnet = true
```

### Environment Variables

```bash
# Cách tốt nhất: dùng file .env
export OPENAI_API_KEY="sk-..."
export BINANCE_API_KEY="..."
export BINANCE_API_SECRET="..."
export TELEGRAM_BOT_TOKEN="123456:ABC..."

# Hoặc set khi chạy
OPENAI_API_KEY="sk-..." ./target/release/ferrum run --agent agents/grid-market-maker/agent.md
```

### Log Levels

```bash
# Debug mode (nhiều log chi tiết)
RUST_LOG=debug ./target/release/ferrum serve

# Chỉ log từ crate cụ thể
RUST_LOG=ferrum_agent=debug,ferrum_risk=trace ./target/release/ferrum serve

# JSON format log (cho production)
RUST_LOG=info ./target/release/ferrum serve 2>&1 | jq
```

---

## 3. Tạo Agent

### Cấu trúc thư mục Agent

```
agents/
└── my-strategy/
    ├── agent.md       # BẮT BUỘC: Định nghĩa agent
    ├── config.yml     # TUỲ CHỌN: Runtime config
    └── learnings.md   # TỰ ĐỘNG: Cross-session learnings
```

### File `agent.md` — Định nghĩa Agent

```yaml
---
# ═══════════════════════════════════════════
# PHẦN YAML FRONMATTER (giữa --- và ---)
# ═══════════════════════════════════════════

name: my-strategy              # Tên agent (unique)
tick_interval_secs: 60         # Chu kỳ OODA loop (giây)
connectors:                    # Exchange kết nối
  - binance
trading_pair: BTC-USDT         # Cặp giao dịch

# Configs: Agent CÓ THỂ suggest, User approve
spread_percentage: 0.5
grid_levels: 5
leverage: 1

# ═══════════════════════════════════════════
# LIMITS: Agent KHÔNG THỂ vượt qua
# Đây là guardrails bảo vệ vốn
# ═══════════════════════════════════════════
limits:
  max_position_size_quote: 1000     # Max tổng vị thế (USDT)
  max_single_order_quote: 100       # Max 1 lệnh (USDT)
  max_daily_loss_quote: 50          # Max loss 1 ngày (USDT)
  max_open_executors: 10            # Max số executor mở
  max_drawdown_pct: 10.0            # Max drawdown (%)
  max_cost_per_day_usd: 10.0        # Max phí giao dịch/ngày
---

## Goal
Mô tả mục tiêu chiến lược của agent bằng ngôn ngữ tự nhiên.
LLM sẽ đọc phần này để hiểu agent cần làm gì.

## Rules
- Quy tắc 1: Mua khi RSI < 30
- Quy tắc 2: Bán khi RSI > 70
- Quy tắc 3: Luôn đặt stop-loss 1%
- Quy tắc 4: Không giao dịch khi spread > 0.1%
- Quy tắc 5: Giảm 50% position khi thị trường volatile
```

### Agent mẫu: Grid Market Maker

```bash
# Xem agent mẫu có sẵn
cat agents/grid-market-maker/agent.md

# Copy làm template
cp -r agents/grid-market-maker agents/my-new-strategy
# Sửa agents/my-new-strategy/agent.md theo ý muốn
```

### Liệt kê Agents

```bash
./target/release/ferrum list
# Output:
# 📋 grid-market-maker

# Chỉ định thư mục khác
./target/release/ferrum list --dir /path/to/agents
```

---

## 4. Chạy Agent

### Lệnh cơ bản

```bash
# Chạy agent (paper trading mode - KHÔNG đặt lệnh thật)
./target/release/ferrum run --agent agents/grid-market-maker/agent.md --paper

# Chạy agent (LIVE mode - ⚠️ CẨN THẬN!)
./target/release/ferrum run --agent agents/grid-market-maker/agent.md
```

### OODA Loop hoạt động như thế nào?

```
Mỗi tick (mặc định 30-60 giây):

┌─────────────────────────────────────────────────────┐
│  Phase 0: RISK PRE-CHECK (Deterministic)            │
│  ├── daily_pnl < -max_daily_loss? → BLOCK tick      │
│  ├── drawdown_pct > max_drawdown? → BLOCK tick      │
│  └── daily_cost > max_cost? → BLOCK tick             │
├─────────────────────────────────────────────────────┤
│  Phase 1: OBSERVE (Data Gathering)                   │
│  ├── Lấy OrderBook từ Binance                        │
│  ├── Lấy 100 candles gần nhất (5m)                   │
│  ├── Lấy positions hiện tại                          │
│  └── Lấy risk state hiện tại                         │
├─────────────────────────────────────────────────────┤
│  Phase 2: ORIENT (LLM Reasoning)                    │
│  ├── Gửi market data + learnings → OpenAI            │
│  ├── LLM phân tích & phân loại regime                │
│  │   (TRENDING_UP/DOWN, RANGING, VOLATILE, CRISIS)  │
│  └── Output: MarketAssessment {regime, confidence}   │
├─────────────────────────────────────────────────────┤
│  Phase 3: DECIDE (LLM Signal Generation)            │
│  ├── Gửi assessment + risk limits → OpenAI           │
│  ├── LLM quyết định hành động                        │
│  │   (BUY/SELL/HOLD + entry/SL/TP)                  │
│  └── Output: Vec<ExecutorAction>                     │
├─────────────────────────────────────────────────────┤
│  Phase 4: ACT (Deterministic Validation + Execute)  │
│  ├── Kiểm tra MỖI action qua Risk Engine             │
│  │   ├── executor_count < max?                       │
│  │   ├── order_amount < max_single?                  │
│  │   └── exposure + new < max_position?              │
│  ├── Nếu PASS → Tạo Executor + Đặt lệnh              │
│  └── Nếu FAIL → Log warning, SKIP action             │
├─────────────────────────────────────────────────────┤
│  Phase 5: EXECUTOR TICK (Lifecycle)                  │
│  ├── Cập nhật giá mới                                │
│  ├── Kiểm tra Triple Barrier (TP/SL/Time/Trailing)   │
│  ├── Nếu triggered → Đóng position                   │
│  └── Clean up terminated executors                   │
└─────────────────────────────────────────────────────┘
```

### Xem log chi tiết khi chạy

```bash
# Trace level (tất cả log)
RUST_LOG=trace ./target/release/ferrum run --agent agents/grid-market-maker/agent.md --paper

# Chỉ log từ agent
RUST_LOG=ferrum_agent=debug ./target/release/ferrum run --agent agents/grid-market-maker/agent.md
```

---

## 5. API Server

### Khởi động

```bash
# Mặc định port 8080
./target/release/ferrum serve

# Port tùy chỉnh
./target/release/ferrum serve --port 9090
```

### Endpoints

```bash
# Health check
curl http://localhost:8080/health
# {"status":"ok","version":"0.1.0"}

# Liệt kê agents
curl http://localhost:8080/api/v1/agents

# Chi tiết agent
curl http://localhost:8080/api/v1/agents/grid-market-maker

# Tạo agent mới
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-agent",
    "agent_md": "---\nname: my-agent\ntick_interval_secs: 60\n---\n\n## Goal\nTest"
  }'
```

---

## 6. MCP Server

MCP (Model Context Protocol) cho phép Claude, GPT, hoặc LLM khác điều khiển Ferrum.

### Khởi động

```bash
./target/release/ferrum mcp --port 8081
```

### MCP Tools có sẵn

| Tool | Mô tả | Parameters |
|------|--------|-----------|
| `place_order` | Đặt lệnh giao dịch | connector, pair, side, amount, order_type, price? |
| `cancel_order` | Hủy lệnh | connector, pair, order_id |
| `get_balance` | Xem số dư | connector |
| `get_positions` | Xem vị thế | connector |
| `get_market_data` | Xem dữ liệu thị trường | connector, pair |

### Ví dụ sử dụng từ Claude/GPT

```json
// initialize
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "id": 1
}

// list tools
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 2
}

// call tool
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_market_data",
    "arguments": { "connector": "binance", "pair": "BTC-USDT" }
  },
  "id": 3
}
```

---

## 7. Telegram Bot

### Khởi động

```bash
# Cần Telegram Bot Token từ @BotFather
./target/release/ferrum telegram --token "123456:ABC-DEF..."
```

### Lệnh Telegram

| Lệnh | Mô tả | Ví dụ |
|------|--------|-------|
| `/help` | Hiển thị trợ giúp | `/help` |
| `/list` | Liệt kê agents | `/list` |
| `/start <name>` | Khởi động agent | `/start grid-market-maker` |
| `/stop <name>` | Dừng agent | `/stop grid-market-maker` |
| `/status <name>` | Xem trạng thái agent | `/status grid-market-maker` |
| `/portfolio` | Xem tóm tắt portfolio | `/portfolio` |
| `/positions` | Xem vị thế đang mở | `/positions` |

---

## 8. Risk Engine

### 4 Layers bảo vệ

```
Layer 1: STRATEGY-LEVEL (Pre-tick)
┌─────────────────────────────────────────┐
│ ✅ daily_pnl < -max_daily_loss_quote    │
│ ✅ drawdown_pct < max_drawdown_pct      │
│ ✅ daily_cost < max_cost_per_day_usd    │
│ → Nếu VI PHẠM → SKIP toàn bộ tick      │
└─────────────────────────────────────────┘

Layer 2: EXECUTOR-LEVEL (Per-action)
┌─────────────────────────────────────────┐
│ ✅ executor_count < max_open_executors  │
│ ✅ order_amount < max_single_order_quote│
│ ✅ total_exposure + new < max_position  │
│ → Nếu VI PHẠM → BLOCK action đó        │
└─────────────────────────────────────────┘

Layer 3: POSITION-LEVEL
┌─────────────────────────────────────────┐
│ Triple Barrier:                          │
│ ├── Take Profit: +2% (default)          │
│ ├── Stop Loss: -1% (default)            │
│ ├── Time Limit: 1 hour (default)        │
│ └── Trailing Stop: configurable         │
└─────────────────────────────────────────┘

Layer 4: KILL SWITCH
┌─────────────────────────────────────────┐
│ Nếu drawdown > threshold → DỪNG TẤT CẢ │
│ Không cần LLM quyết định                │
│ → Hoàn toàn deterministic               │
└─────────────────────────────────────────┘
```

### Configs vs Limits

| Loại | Ai kiểm soát | Ví dụ |
|------|-------------|-------|
| **Configs** | Agent suggest → User approve | trading_pair, spread, grid_levels |
| **Limits** | **User-only**, agent KHÔNG thể sửa | max_daily_loss, max_drawdown, max_position |

> ⚠️ **Nguyên tắc vàng: LLM proposes, deterministic code DISPOSES**

---

## 9. Executors

### 3 Executors đã implement

#### 9.1 PositionExecutor (Directional Trades)

```yaml
# Trong agent.md → params
executor_type: position
triple_barrier:
  take_profit: 0.02      # +2% chốt lời
  stop_loss: 0.01        # -1% cắt lỗ
  time_limit: 3600       # 1 giờ tối đa
  trailing_stop_activation: 0.015  # Kích hoạt trailing khi +1.5%
  trailing_stop_delta: 0.005       # Trailing 0.5%
```

**Lifecycle:**
```
CREATED → ACTIVE → TERMINATED
                     ├── TakeProfit  (đạt TP)
                     ├── StopLoss    (đạt SL)
                     ├── TimeLimit   (hết giờ)
                     ├── TrailingStop (trailing triggered)
                     └── EarlyStop   (dừng thủ công)
```

#### 9.2 OrderExecutor (Simple Orders)

```yaml
executor_type: order
order_type: MARKET  # hoặc LIMIT
side: BUY
amount: 0.01
price: 100000       # Chỉ cho LIMIT
```

#### 9.3 GridExecutor (Range Trading)

```yaml
executor_type: grid
start_price: 95000
end_price: 105000
levels: 10
total_amount_quote: 1000
# Tự chia thành 10 lệnh giữa 95000-105000
```

---

## 10. Technical Indicators

Ferrum có sẵn 7 technical indicators trong `ferrum-routines`:

| Indicator | Hàm | Parameters |
|-----------|-----|-----------|
| **SMA** | `sma(candles, period)` | period: số nến |
| **EMA** | `ema(candles, period)` | period: số nến |
| **RSI** | `rsi(candles, period)` | period: thường dùng 14 |
| **MACD** | `macd(candles)` | Trả về (macd, signal, histogram) |
| **Bollinger** | `bollinger_bands(candles, period, std_dev)` | period: 20, std_dev: 2.0 |
| **VWAP** | `vwap(candles)` | Volume Weighted Average Price |
| **ATR** | `atr(candles, period)` | Average True Range |

---

## 11. Docker Deployment

### Build & Run

```bash
# Build Docker image
docker build -t ferrum:latest .

# Chạy container
docker run -d \
  --name ferrum \
  -p 8080:8080 \
  -p 8081:8081 \
  -e RUST_LOG=info \
  -e OPENAI_API_KEY="sk-..." \
  -e BINANCE_API_KEY="..." \
  -e BINANCE_API_SECRET="..." \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/agents:/app/agents \
  ferrum:latest

# Hoặc dùng docker-compose (Ferrum + Qdrant)
docker-compose up -d
```

### docker-compose.yml

```yaml
services:
  ferrum:
    build: .
    ports:
      - "8080:8080"    # REST API
      - "8081:8081"    # MCP
    environment:
      - RUST_LOG=info
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - BINANCE_API_KEY=${BINANCE_API_KEY}
      - BINANCE_API_SECRET=${BINANCE_API_SECRET}
    volumes:
      - ./data:/app/data
      - ./agents:/app/agents

  qdrant:              # Vector DB cho RAG
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
```

---

## 12. Troubleshooting

### Lỗi thường gặp

#### `LlmError: API error 401`
```bash
# Kiểm tra API key
echo $OPENAI_API_KEY

# Set lại
export OPENAI_API_KEY="sk-..."
```

#### `ExchangeError: Invalid signature`
```bash
# Kiểm tra API credentials
# Đảm bảo testnet=true khi test!
cat config/ferrum.toml | grep testnet
```

#### `RiskLimitExceeded: Daily loss exceeded`
```bash
# Agent bị block vì vượt giới hạn
# Kiểm tra risk state qua API
curl http://localhost:8080/api/v1/agents/grid-market-maker

# Tăng limits nếu cần (trong agent.md)
limits:
  max_daily_loss_quote: 100  # Tăng từ 50 lên 100
```

#### Build lỗi OpenSSL
```bash
# Ubuntu/Debian
sudo apt install pkg-config libssl-dev

# macOS
brew install openssl

# Alpine (cho Docker)
apk add openssl-dev
```

---

## 🗺️ Workflow tổng quan

```
1. Tạo agent.md → Định nghĩa chiến lược
2. Cấu hình limits → Bảo vệ vốn
3. Chạy --paper → Test không rủi ro
4. Quan sát log → Kiểm tra quyết định LLM
5. Chạy live → Giao dịch thật (cẩn thận!)
6. Xem session journal → Học & tối ưu
7. Agent tự học → learnings.md cập nhật
```

---

> **⚠️ DISCLAIMER**: Ferrum là phần mềm giao dịch. Giao dịch tiền điện tử có rủi ro cao. Luôn bắt đầu với **paper trading** và **testnet**. Không bao giờ giao dịch số tiền bạn không thể mất.
