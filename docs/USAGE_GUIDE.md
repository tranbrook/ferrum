# 📘 Hướng dẫn sử dụng Ferrum Trading Bot

## Mục lục

1. [Cài đặt](#1-cài-đặt)
2. [Cấu hình](#2-cấu-hình)
3. [CLI - Dòng lệnh](#3-cli---dòng-lệnh)
4. [Chạy Trading Agent](#4-chạy-trading-agent)
5. [Paper Trading](#5-paper-trading)
6. [Backtesting](#6-backtesting)
7. [Dashboard Web](#7-dashboard-web)
8. [Telegram Bot](#8-telegram-bot)
9. [REST API](#9-rest-api)
10. [MCP Server](#10-mcp-server)
11. [Đa sàn giao dịch](#11-đa-sàn-giao-dịch)
12. [Quản lý rủi ro](#12-quản-lý-rủi-ro)
13. [RAG Pipeline](#13-rag-pipeline)
14. [Local LLM](#14-local-llm)
15. [Multi-Agent Orchestration](#15-multi-agent-orchestration)

---

## 1. Cài đặt

### Yêu cầu
- **Rust** 1.75+ (`rustup install stable`)
- **SQLite3** (bundled, không cần cài riêng)
- **Qdrant** (tùy chọn, cho RAG pipeline)

### Build từ source

```bash
# Clone repo
git clone https://github.com/tranbrook/ferrum.git
cd ferrum

# Build release (tối ưu)
cargo build --release

# Binary ở: target/release/ferrum
sudo cp target/release/ferrum /usr/local/bin/

# Hoặc chạy trực tiếp
cargo run --release -- <command>
```

### Chạy tests

```bash
# Chạy tất cả 88 tests
cargo test

# Chạy tests cho 1 crate cụ thể
cargo test -p ferrum-backtest
cargo test -p ferrum-paper
```

---

## 2. Cấu hình

### File cấu hình chính: `ferrum.toml`

Tạo file `ferrum.toml` ở thư mục chạy:

```toml
[general]
log_level = "info"

[exchanges.binance]
api_key = "YOUR_BINANCE_API_KEY"
api_secret = "YOUR_BINANCE_API_SECRET"
testnet = true    # ← LUÔN dùng testnet khi mới bắt đầu!

[exchanges.bybit]
api_key = "YOUR_BYBIT_API_KEY"
api_secret = "YOUR_BYBIT_API_SECRET"
testnet = true

[exchanges.okx]
api_key = "YOUR_OKX_API_KEY"
api_secret = "YOUR_OKX_API_SECRET"
passphrase = "YOUR_OKX_PASSPHRASE"
testnet = true

[exchanges.hyperliquid]
api_key = "YOUR_HYPERLIQUID_WALLET_ADDRESS"
testnet = true

[risk]
max_position_size_quote = 1000.0    # Tối đa $1000/position
max_single_order_quote = 100.0      # Tối đa $100/order
max_daily_loss_quote = 50.0         # Dừng nếu lỗ quá $50/ngày
max_drawdown_pct = 10.0             # Dừng nếu drawdown > 10%
max_cost_per_day_usd = 10.0         # Giới hạn phí $10/ngày

[llm]
provider = "openai"                 # openai | groq | anthropic | local
api_key = "YOUR_OPENAI_API_KEY"
model = "gpt-4o-mini"
base_url = "https://api.openai.com/v1"

[dashboard]
host = "0.0.0.0"
port = 8080
```

### Cấu hình qua biến môi trường

```bash
export FERRUM_BINANCE_API_KEY="your_key"
export FERRUM_BINANCE_API_SECRET="your_secret"
export FERRUM_LLM_API_KEY="your_openai_key"
export RUST_LOG="ferrum=debug"      # Chi tiết log
```

---

## 3. CLI - Dòng lệnh

```bash
# Xem help
ferrum --help

# Xem các lệnh con
ferrum

# Output:
# Ferrum - Rust Trading Agent Harness v0.1.0
#
# Commands:
#   serve     Start the API server
#   mcp       Start the MCP server
#   run       Run a trading agent
#   telegram  Start Telegram bot
#   list      List available agents
```

### Các flag chung

| Flag | Mô tả | Mặc định |
|------|--------|-----------|
| `-c, --config` | Đường dẫn file cấu hình | `ferrum.toml` |
| `-l, --log-level` | Mức log | `info` |

```bash
ferrum -c my-config.toml -l debug serve --port 9090
```

---

## 4. Chạy Trading Agent

### Định nghĩa Agent bằng file `agent.md`

Tạo thư mục `agents/` và file `agents/my-agent/agent.md`:

```markdown
---
name: "BTC Grid Trader"
pair: "BTC-USDT"
tick_interval: 60
connectors: ["binance"]
---

# Goal
Generate consistent profit by placing grid orders on BTC/USDT with 0.5% spread.

# Rules
- Only trade during high volume periods (volume > 24h average)
- Close all positions if daily loss exceeds $50
- Never exceed $500 total exposure
- Use 1% of portfolio per order

# Strategy
- Place buy orders at -0.5%, -1%, -1.5% from current price
- Place sell orders at +0.5%, +1%, +1.5% from current price
- Rebalance grid every tick
```

### Chạy agent

```bash
# Chạy agent thật (CẨN THẬN!)
ferrum run --agent agents/my-agent/agent.md

# Chạy ở chế độ paper trading (AN TOÀN - dùng dữ liệu thật nhưng tiền giả)
ferrum run --agent agents/my-agent/agent.md --paper

# Xem log chi tiết
RUST_LOG=debug ferrum run --agent agents/my-agent/agent.md --paper
```

### Liệt kê agents

```bash
ferrum list --dir agents/
# Output:
# 📋 my-agent
# 📋 eth-momentum
# 📋 sol-scalper
```

---

## 5. Paper Trading

Paper trading cho phép test chiến lược với **dữ liệu thị trường thật** nhưng **tiền giả**. Rất hữu ích để validate chiến lược trước khi dùng tiền thật.

### Tạo Paper Trading Engine

```rust
use ferrum_paper::{PaperTradingEngine, PaperTradingConfig, SlippageModel};
use ferrum_core::types::*;

#[tokio::main]
async fn main() {
    // Cấu hình: 10,000 USDT ban đầu, phí 0.1%
    let config = PaperTradingConfig {
        initial_balances: vec![
            ("USDT".to_string(), 10000.0),
            ("BTC".to_string(), 0.0),
        ],
        fee_rate: 0.001,                          // 0.1% phí
        slippage_model: SlippageModel::Fixed(0.0005), // 0.05% trượt giá
        allow_shorting: false,
        max_position_fraction: 0.25,               // Tối đa 25% portfolio/position
    };

    let mut engine = PaperTradingEngine::new(config);

    // Mua 0.1 BTC ở giá 50,000
    let pair = TradingPair::new("BTC", "USDT");
    let order = engine.submit_market_order(
        pair.clone(),
        Side::Buy,
        0.1,       // Số lượng
        50000.0,   // Giá thị trường hiện tại
    ).unwrap();

    println!("Order: {:?}", order.status);        // Filled
    println!("Fill price: {:?}", order.avg_fill_price); // ~50025 (với slippage)

    // Bán khi giá lên 55,000
    let sell_order = engine.submit_market_order(
        pair.clone(),
        Side::Sell,
        0.1,
        55000.0,
    ).unwrap();

    // Xem PnL
    let account = engine.account_summary();
    println!("Realized PnL: ${:.2}", account.realized_pnl);
    println!("Total fees:    ${:.2}", account.total_fees);
    println!("Trades:        {}", account.trade_count);
}
```

### Slippage Models

```rust
// Không trượt giá (lý tưởng)
SlippageModel::None

// Trượt giá cố định 0.05%
SlippageModel::Fixed(0.0005)

// Trượt giá theo khối lượng (sàn nhỏ)
SlippageModel::VolumeBased {
    base_slippage: 0.0002,   // Base 0.02%
    volume_factor: 0.000001, // Tăng theo giá trị order
}
```

---

## 6. Backtesting

Backtesting cho phép chạy chiến lược trên **dữ liệu lịch sử** để đánh giá hiệu quả.

### Sử dụng

```rust
use ferrum_backtest::{
    BacktestEngine, BacktestConfig, BacktestStrategy,
    strategy::{SmaCrossoverStrategy, RsiMeanReversionStrategy},
};
use ferrum_core::types::*;

#[tokio::main]
async fn main() {
    // Cấu hình backtest
    let config = BacktestConfig {
        pair: TradingPair::new("BTC", "USDT"),
        connector: "binance".to_string(),
        start_time: 1700000000000,  // Unix ms
        end_time: 1710000000000,
        interval: Interval::H1,
        initial_balance: 10000.0,
        fee_rate: 0.001,            // 0.1% phí
        slippage: 0.0005,           // 0.05% trượt giá
        leverage: 1.0,
    };

    // Chọn chiến lược
    let strategy = Box::new(SmaCrossoverStrategy::new(7, 25)); // Fast 7, Slow 25

    // Hoặc RSI
    // let strategy = Box::new(RsiMeanReversionStrategy::new(14, 30.0, 70.0));

    let engine = BacktestEngine::new(config, strategy);

    // Candles dữ liệu lịch sử (fetch từ exchange hoặc file)
    let candles = fetch_historical_candles().await;

    // Chạy backtest
    let result = engine.run(candles).await.unwrap();

    // Xem kết quả
    let m = &result.metrics;
    println!("═══ Backtest Results ═══");
    println!("Total Return:    {:.2}%", m.total_return);
    println!("Annual Return:   {:.2}%", m.annualized_return);
    println!("Max Drawdown:    {:.2}%", m.max_drawdown);
    println!("Sharpe Ratio:    {:.2}", m.sharpe_ratio);
    println!("Sortino Ratio:   {:.2}", m.sortino_ratio);
    println!("Win Rate:        {:.1}%", m.win_rate);
    println!("Total Trades:    {}", m.total_trades);
    println!("Profit Factor:   {:.2}", m.profit_factor);
    println!("Avg Win:         ${:.2}", m.avg_win);
    println!("Avg Loss:        ${:.2}", m.avg_loss);
    println!("Total Fees:      ${:.2}", m.total_fees);
    println!("Best Trade:      ${:.2}", m.best_trade);
    println!("Worst Trade:     ${:.2}", m.worst_trade);
    println!("Final Value:     ${:.2}", result.final_value);
}
```

### Tạo chiến lược tùy chỉnh

```rust
use async_trait::async_trait;
use ferrum_backtest::{BacktestStrategy, TradeSignal, types::MarketContext};
use ferrum_core::error::Result;
use std::collections::HashMap;

pub struct MyStrategy {
    pub threshold: f64,
}

#[async_trait]
impl BacktestStrategy for MyStrategy {
    fn name(&self) -> &str { "My Custom Strategy" }
    fn description(&self) -> &str { "Mô tả chiến lược" }

    async fn evaluate(&self, ctx: &MarketContext) -> Result<TradeSignal> {
        // Lấy indicator đã tính
        let rsi = ctx.indicators.get("rsi").copied().unwrap_or(50.0);

        if ctx.position.is_some() {
            // Đang có position
            if rsi > 70.0 {
                Ok(TradeSignal::ClosePosition)
            } else {
                Ok(TradeSignal::Hold)
            }
        } else {
            // Chưa có position
            if rsi < 30.0 {
                Ok(TradeSignal::Buy)
            } else {
                Ok(TradeSignal::Hold)
            }
        }
    }

    fn compute_indicators(&self, candles: &[Candle]) -> HashMap<String, f64> {
        // Tính indicators tùy chỉnh
        let mut map = HashMap::new();
        // ... thêm RSI, MACD, Bollinger, v.v.
        map.insert("rsi".to_string(), 45.0);
        map
    }
}
```

---

## 7. Dashboard Web

Dashboard cung cấp giao diện web real-time để theo dõi bot.

### Khởi động

```bash
# Cách 1: CLI
ferrum serve --port 8080

# Cách 2: Programmatically
```

```rust
use ferrum_dashboard::{DashboardServer, DashboardConfig};

#[tokio::main]
async fn main() {
    let server = DashboardServer::new(DashboardConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
    });

    // Cập nhật state từ agent thread
    let state = server.state();
    {
        let mut s = state.write();
        s.agents_active = 3;
        s.total_pnl = 150.0;
    }

    server.start().await.unwrap();
}
```

### API Endpoints

| Method | Endpoint | Mô tả |
|--------|----------|--------|
| `GET` | `/api/status` | Trạng thái hệ thống |
| `GET` | `/api/positions` | Danh sách positions đang mở |
| `GET` | `/api/agents` | Trạng thái các agents |
| `GET` | `/api/balances` | Số dư tài khoản |
| `GET` | `/api/trades` | Lịch sử giao dịch gần đây |
| `POST` | `/api/start` | Bắt đầu trading |
| `POST` | `/api/stop` | Dừng trading |

### Ví dụ response

```bash
curl http://localhost:8080/api/status
```

```json
{
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "agents_active": 3,
  "exchanges_connected": 2,
  "total_pnl": 150.5,
  "open_positions": 1,
  "timestamp": 1713000000000
}
```

---

## 8. Telegram Bot

Điều khiển bot qua Telegram - tiện lợi khi không có máy tính.

### Khởi động

```bash
ferrum telegram --token "YOUR_TELEGRAM_BOT_TOKEN"
```

### Programmatically

```rust
use ferrum_telegram::TelegramBot;

#[tokio::main]
async fn main() {
    let bot = TelegramBot::new("YOUR_TOKEN".to_string());
    bot.run().await;
}
```

### Lệnh Telegram (dự kiến)

```
/start       - Bắt đầu bot
/status      - Xem trạng thái bot
/balance     - Xem số dư
/positions   - Xem positions đang mở
/pnl         - Xem PnL hôm nay
/stop        - Dừng tất cả agents
/risk        - Xem risk state
```

---

## 9. REST API

REST API server cho phép tích hợp Ferrum vào hệ thống khác.

```bash
ferrum serve --port 8080
```

### Authentication

```bash
# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "secret"}'

# → {"token": "eyJhbG..."}
```

### Sử dụng token

```bash
curl http://localhost:8080/api/status \
  -H "Authorization: Bearer eyJhbG..."
```

---

## 10. MCP Server

Model Context Protocol server cho phép LLM (ChatGPT, Claude) tương tác với Ferrum.

```bash
ferrum mcp --port 8081
```

### Công cụ MCP

| Tool | Mô tả |
|------|--------|
| `get_market_data` | Lấy giá thị trường |
| `place_order` | Đặt lệnh giao dịch |
| `get_positions` | Xem positions |
| `get_balance` | Xem số dư |
| `run_backtest` | Chạy backtest |

---

## 11. Đa sàn giao dịch

Ferrum hỗ trợ 4 sàn giao dịch chính với cùng một API:

### Sàn được hỗ trợ

| Sàn | API Version | Features |
|-----|------------|----------|
| **Binance** | V3 REST | Spot, Margin |
| **Bybit** | V5 REST | Spot, Linear |
| **OKX** | V5 REST | Spot, Swap |
| **Hyperliquid** | JSON-RPC | Perpetuals |

### Sử dụng Exchange Adapter

```rust
use ferrum_exchange::ExchangeRegistry;
use ferrum_core::config::ExchangeConfig;
use ferrum_core::types::*;

#[tokio::main]
async fn main() {
    // Tạo registry
    let mut registry = ExchangeRegistry::new();

    // Đăng ký Binance
    registry.register("binance", ExchangeConfig {
        name: "binance".to_string(),
        api_key: "your_key".to_string(),
        api_secret: "your_secret".to_string(),
        passphrase: None,
        testnet: true,
    });

    // Đăng ký Bybit
    registry.register("bybit", ExchangeConfig {
        name: "bybit".to_string(),
        api_key: "your_key".to_string(),
        api_secret: "your_secret".to_string(),
        passphrase: None,
        testnet: true,
    });

    // Kết nối
    let binance = registry.get("binance").unwrap();
    let mut adapter = binance.write().await;
    adapter.connect().await.unwrap();

    // Lấy order book
    let pair = TradingPair::new("BTC", "USDT");
    let orderbook = adapter.get_orderbook(&pair).await.unwrap();
    println!("Best bid: {:?}", orderbook.bids.first());
    println!("Best ask: {:?}", orderbook.asks.first());

    // Lấy candles
    let candles = adapter.get_candles(&pair, Interval::H1, 100).await.unwrap();
    println!("Got {} candles", candles.len());

    // Đặt lệnh
    let order = adapter.place_order(OrderRequest {
        pair: pair.clone(),
        side: Side::Buy,
        order_type: OrderType::Limit,
        amount: Quantity(0.001),
        price: Some(Price(50000.0)),
        client_order_id: None,
    }).await.unwrap();
    println!("Order ID: {:?}", order.order_id);

    // Xem số dư
    let balances = adapter.get_balances().await.unwrap();
    for b in &balances {
        println!("{}: free={}, used={}", b.asset, b.free, b.used);
    }
}
```

---

## 12. Quản lý rủi ro

Ferrum có hệ thống quản lý rủi ro 4 tầng:

### 4 Lớp kiểm tra

```
┌─────────────────────────────────────────┐
│  Layer 1: Daily Loss Check              │  ← Tổng lỗ ngày > max?
├─────────────────────────────────────────┤
│  Layer 2: Drawdown Check                │  ← Drawdown > % limit?
├─────────────────────────────────────────┤
│  Layer 3: Cost Check                    │  ← Phí vượt ngân sách?
├─────────────────────────────────────────┤
│  Layer 4: Manual Block                  │  ← User chặn thủ công?
└─────────────────────────────────────────┘
```

### Cấu hình

```toml
[risk]
max_position_size_quote = 1000.0    # $1000 tối đa / position
max_single_order_quote = 100.0      # $100 tối đa / order
max_daily_loss_quote = 50.0         # Dừng nếu lỗ > $50/ngày
max_open_executors = 10             # Tối đa 10 executors mở
max_drawdown_pct = 10.0             # Dừng nếu drawdown > 10%
max_cost_per_day_usd = 10.0         # Tối đa $10 phí/ngày
```

### Sử dụng Risk Engine

```rust
use ferrum_risk::RiskEngine;
use ferrum_core::config::{RiskLimits, RiskState};

let limits = RiskLimits::default();
let state = RiskState::default();
let mut engine = RiskEngine::new(limits, state);

// Kiểm tra trước khi đặt lệnh
let can_trade = engine.validate_order(50.0); // $50 order
if can_trade.is_ok() {
    // Đặt lệnh
    engine.record_trade(50.0, 5.0); // cost, pnl
} else {
    println!("Risk blocked: {}", can_trade.unwrap_err());
}
```

---

## 13. RAG Pipeline

RAG (Retrieval-Augmented Generation) cho phép bot tìm kiếm kiến thức trading từ vector database.

### Cài đặt Qdrant (tùy chọn)

```bash
# Docker
docker run -p 6333:6333 qdrant/qdrant

# Hoặc dùng in-memory store (mặc định)
```

### Sử dụng

```rust
use ferrum_rag::RagPipeline;
use ferrum_rag::store::KnowledgeDocument;

#[tokio::main]
async fn main() {
    let pipeline = RagPipeline::new(
        "http://localhost:6333",  // Qdrant URL (hoặc in-memory nếu không chạy)
        "ferrum_knowledge",       // Collection name
        384,                      // Embedding dimension
    );

    // Khởi tạo
    pipeline.initialize().await.unwrap();

    // Thêm kiến thức
    pipeline.add_document(KnowledgeDocument {
        id: "btc_analysis_1".to_string(),
        title: "Bitcoin Support/Resistance".to_string(),
        content: "BTC has major support at 60000 and resistance at 73000...".to_string(),
        source: "technical_analysis".to_string(),
        created_at: chrono::Utc::now().timestamp_millis(),
        metadata: Default::default(),
    }).await.unwrap();

    // Tìm kiếm
    let results = pipeline.search("Where is BTC support?", 5).await.unwrap();
    for doc in &results {
        println!("{} (score: {:.3}): {}", doc.title, 0.95, &doc.content[..100]);
    }

    // Tạo RAG-augmented prompt cho LLM
    let prompt = pipeline.augment_prompt("Should I buy BTC now?", &results);
    // → "Context: BTC has major support at 60000...
    //    Question: Should I buy BTC now?"
}
```

---

## 14. Local LLM

Chạy LLM cục bộ (không cần Internet) cho privacy và zero-cost.

### Sử dụng

```rust
use ferrum_local_llm::{LocalLlmEngine, LocalLlmConfig};
use ferrum_core::traits::LlmClient;

#[tokio::main]
async fn main() {
    let config = LocalLlmConfig {
        backend: "mock".to_string(),     // "mock" | "candle" | "llama-cpp"
        model_path: None,                 // Đường dẫn model file
        max_context_length: 4096,
        gpu_layers: 0,                    // 0 = CPU only
        threads: 4,
        temperature: 0.7,
        top_p: 0.9,
        max_tokens: 1024,
        repeat_penalty: 1.1,
    };

    let engine = LocalLlmEngine::new(config);

    // Quick inference
    let response = engine.quick_inference("Should I buy BTC?").await.unwrap();
    println!("{}", response);

    // Full analysis
    let analysis = engine.analysis_inference(
        "Analyze BTC/USDT: price=65000, RSI=35, volume declining"
    ).await.unwrap();
    println!("{}", analysis);
}
```

### Kết nối LLM Cloud

```rust
use ferrum_llm::LlmClientImpl;
use ferrum_core::traits::LlmClient;

let llm = LlmClientImpl::new(
    "https://api.openai.com/v1",
    "sk-...",
    "gpt-4o-mini",
);

let response = llm.complete("Analyze market trend for BTC").await.unwrap();
```

---

## 15. Multi-Agent Orchestration

Hệ thống multi-agent cho phép nhiều agent chuyên biệt phối hợp với nhau.

### Các vai trò của Agent

```
┌──────────────────────────────────────────────────┐
│                  ORCHESTRATOR                     │
│            (routes messages)                      │
├──────────┬──────────┬──────────┬─────────────────┤
│ ANALYST  │   RISK   │ EXECUTOR │ PORTFOLIO MGR   │
│          │ MANAGER  │          │                  │
│ Phân tích│ Quản lý  │ Thực thi │ Theo dõi        │
│ thị trường│ rủi ro   │ lệnh     │ danh mục        │
└──────────┴──────────┴──────────┴─────────────────┘
```

### Sử dụng

```rust
use ferrum_orchestrator::{
    Orchestrator, OrchestratorMessage,
    message::{AgentRole, MessageType, AgentId},
    coordinator::AgentDescriptor,
};

#[tokio::main]
async fn main() {
    let mut orch = Orchestrator::new();

    // Đăng ký agents
    let analyst_rx = orch.register_agent(AgentDescriptor {
        id: "analyst-1".to_string(),
        role: AgentRole::Analyst,
        config: serde_json::Value::Null,
    });

    let risk_rx = orch.register_agent(AgentDescriptor {
        id: "risk-1".to_string(),
        role: AgentRole::RiskManager,
        config: serde_json::Value::Null,
    });

    let executor_rx = orch.register_agent(AgentDescriptor {
        id: "executor-1".to_string(),
        role: AgentRole::Executor,
        config: serde_json::Value::Null,
    });

    // Khởi động
    orch.start().await.unwrap();

    // Gửi tín hiệu trading
    let signal = OrchestratorMessage::broadcast(
        AgentId("analyst-1".to_string()),
        MessageType::MarketAnalysis,
        "BTC showing bullish divergence on 4H".to_string(),
    );
    orch.broadcast(signal).await;

    // Gửi lệnh trực tiếp
    let order = OrchestratorMessage::new(
        AgentId("risk-1".to_string()),
        Some(AgentId("executor-1".to_string())),
        MessageType::TradeSignal,
        "BUY BTC 0.01 at 50000".to_string(),
    );
    orch.send_to(AgentId("executor-1".to_string()), order).await.unwrap();

    // Dừng
    orch.stop().await.unwrap();
}
```

---

## Kiến trúc tổng quan

```
ferrum/
├── CLI (ferrum-cli)          ← Điểm vào chính
│   ├── serve                 ← API + Dashboard server
│   ├── run                   ← Chạy trading agent
│   ├── telegram              ← Telegram bot
│   ├── mcp                   ← MCP server cho LLM
│   └── list                  ← Liệt kê agents
│
├── Core (ferrum-core)        ← Types, traits, config, events
├── Exchange (ferrum-exchange) ← Binance, Bybit, OKX, Hyperliquid
├── LLM (ferrum-llm)          ← OpenAI/Groq/Anthropic client
├── RAG (ferrum-rag)           ← Knowledge retrieval pipeline
├── Agent (ferrum-agent)       ← OODA loop trading agent
├── Orchestrator               ← Multi-agent coordination
├── Risk (ferrum-risk)         ← 4-layer risk management
├── Backtest (ferrum-backtest) ← Strategy backtesting
├── Paper (ferrum-paper)       ← Paper trading simulation
└── Dashboard (ferrum-dashboard) ← Web monitoring
```

## Workflow khuyến nghị

```
1. Backtest → Test chiến lược trên dữ liệu lịch sử
2. Paper Trade → Chạy với data thật, tiền giả (1-2 tuần)
3. Live Trade → Bắt đầu với số tiền nhỏ
4. Monitor → Theo dõi qua Dashboard / Telegram
5. Iterate → Cải thiện chiến lược dựa trên kết quả
```

⚠️ **LUÔN dùng testnet và paper trading trước khi trade thật!**
