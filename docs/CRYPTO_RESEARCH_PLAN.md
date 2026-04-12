# 🚀 FULL CRYPTO RESEARCH PLATFORM - KẾ HOẠCH CHI TIẾT

## 📋 TỔNG QUAN DỰ ÁN

**Tên dự án:** Crypto Research Platform (CRP)  
**Ngôn ngữ chính:** Rust (backend) + TypeScript/React (frontend)  
**Mục tiêu:** Xây dựng nền tảng nghiên cứu crypto toàn diện, tự host, miễn phí — bao gồm market data, on-chain analysis, DeFi analytics, sentiment analysis, whale tracking, và technical analysis.

---

## 🏗️ KIẾN TRÚC HỆ THỐNG

```
┌─────────────────────────────────────────────────────────┐
│                    FRONTEND (React/Next.js)              │
│  Dashboard │ Charts │ Whale Tracker │ DeFi │ Sentiment  │
└──────────────────────────┬──────────────────────────────┘
                           │ REST API / WebSocket
┌──────────────────────────┴──────────────────────────────┐
│                  API GATEWAY (Rust/Axum)                 │
│            Auth │ Rate Limit │ Routing │ Caching         │
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────┐
│              DATA LAYER (Rust Microservices)             │
│                                                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐ │
│  │  Market   │ │  On-Chain│ │   DeFi   │ │ Sentiment  │ │
│  │  Data     │ │  Analyst │ │ Analyst  │ │ Analyst    │ │
│  │ Service   │ │ Service  │ │ Service  │ │ Service    │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬──────┘ │
│       │             │            │              │        │
│  ┌────┴─────┐ ┌────┴─────┐ ┌────┴─────┐ ┌────┴──────┐ │
│  │Technical │ │  Whale   │ │  Yield   │ │  Social   │ │
│  │Analysis  │ │ Tracker  │ │ Tracker  │ │ Scraper   │ │
│  │ Engine   │ │ Engine   │ │ Engine   │ │ Engine    │ │
│  └──────────┘ └──────────┘ └──────────┘ └───────────┘ │
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────┐
│                   DATA STORAGE                           │
│  TimescaleDB/SQLite │ Redis Cache │ Sled/Redb           │
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────┐
│              EXTERNAL DATA SOURCES                       │
│  CoinGecko │ DeFiLlama │ Etherscan │ Alternative.me     │
│  CoinMarketCap │ Birdeye │ DexScreener │ Blockchain RPC │
└─────────────────────────────────────────────────────────┘
```

---

## 📦 MODULE CHI TIẾT

### MODULE 1: CORE — Data Collector & Storage
**Priority: P0 (Foundation)**

#### 1.1 Market Data Service
- **API Sources (MIỄN PHÍ):**
  - CoinGecko API (Free tier: 30 calls/min)
  - CoinMarketCap API (Free: 10,000 calls/month)
  - CoinPaprika API (Free: 25,000 calls/month)
- **Data thu thập:**
  - Giá real-time (spot) cho 1000+ tokens
  - Market cap, volume 24h, circulating supply
  - Lịch sử giá (OHLCV) — daily, hourly, minute
  - Top gainers/losers
  - Trending coins
  - Exchange listings & trading pairs
- **Cron Jobs:**
  - Mỗi 30s: giá real-time top 100 coins
  - Mỗi 5 phút: giá extended top 500
  - Mỗi 1h: OHLCV candles
  - Mỗi 24h: full market snapshot, metadata

#### 1.2 Storage Engine
- **SQLite + TimescaleDB-like approach** (dùng SQLite với extension hoặcTimescaleDB)
- **Schema chính:**
  - `tokens` — id, symbol, name, coingecko_id, chains, logo
  - `price_feeds` — token_id, price, market_cap, volume, timestamp
  - `ohlcv_candles` — token_id, open, high, low, close, volume, timeframe, timestamp
  - `exchange_pairs` — exchange, pair, volume, trust_score

#### 1.3 Caching Layer
- Redis hoặc **sled** (embedded Rust DB) cho hot data
- Cache TTL: real-time = 30s, hourly = 5min, daily = 1h

---

### MODULE 2: ON-CHAIN ANALYSIS ENGINE
**Priority: P0**

#### 2.1 Blockchain Data Collector
- **RPC Sources:**
  - Ethereum: Ankr, Alchemy free tier, Public RPCs
  - Solana: Helius free, Solana RPC
  - BSC, Polygon, Arbitrum, Base — public RPCs
- **Data thu thập:**
  - Large transactions (whale alerts) > $100K
  - Token transfer volumes
  - Smart contract interactions
  - Gas tracker (ETH)
  - Active addresses count
  - New token deployments

#### 2.2 Whale Tracker
- Phát hiện wallet giao dịch lớn
- Phân loại: Exchange wallets, Whale wallets, Smart Money
- Track wallet activity history
- Alert system khi whale buy/sell

#### 2.3 Blockchain Explorer APIs
- **Etherscan** (Free: 100,000 calls/day)
- **Solscan/SolanaFM** API
- **BscScan** (Free tier)
- Dữ liệu: token holders, transaction history, contract verification

---

### MODULE 3: DeFi ANALYTICS
**Priority: P1**

#### 3.1 DeFiLlama Integration (MIỄN PHÍ)
- TVL per protocol, per chain
- Protocol comparison & rankings
- TVL history charts
- Fee & revenue data
- DEX volumes aggregator
- Yield pools & APY tracking

#### 3.2 DEX Analytics
- **DexScreener API** (Free) — DEX pairs, prices, liquidity
- **Birdeye API** (Free tier) — Solana DEX data
- DEX volume rankings
- New token launches & liquidity adds
- Price impact calculator

#### 3.3 Yield Opportunities
- Lending rates (Aave, Compound)
- Liquidity pool APYs
- Staking yields
- Vault strategies & auto-compounding
- Impermanent loss calculator

---

### MODULE 4: TECHNICAL ANALYSIS ENGINE
**Priority: P1**

#### 4.1 Indicator Calculations (Rust native)
- **Trend Indicators:**
  - SMA, EMA, WMA, DEMA, TEMA
  - MACD (12, 26, 9)
  - ADX, Parabolic SAR, Ichimoku Cloud
- **Momentum Indicators:**
  - RSI (14)
  - Stochastic Oscillator
  - Williams %R
  - CCI (Commodity Channel Index)
- **Volatility Indicators:**
  - Bollinger Bands
  - ATR (Average True Range)
  - Keltner Channels
- **Volume Indicators:**
  - OBV (On Balance Volume)
  - VWAP
  - Money Flow Index
- **Custom Signals:**
  - Buy/Sell signal aggregator
  - Signal strength scoring (0-100)

#### 4.2 Chart Pattern Recognition
- Support/Resistance detection
- Trend line auto-detection
- Chart patterns: Double top/bottom, Head & shoulders, Triangles
- Candlestick patterns: Doji, Hammer, Engulfing

---

### MODULE 5: SENTIMENT ANALYSIS
**Priority: P2**

#### 5.1 Market Sentiment
- **Alternative.me Fear & Greed Index** (Free API)
  - Current index value (0-100)
  - Historical data
  - Classification: Extreme Fear → Extreme Greed

#### 5.2 Social Media Scraping
- Twitter/X API (Free tier) — crypto influencers sentiment
- Reddit API — r/cryptocurrency, r/bitcoin sentiment
- Telegram group monitoring (optional)
- Sentiment scoring: Positive, Neutral, Negative

#### 5.3 News Aggregation
- Crypto news RSS feeds (CoinDesk, CoinTelegraph, etc.)
- News sentiment analysis
- Event detection & impact scoring

---

### MODULE 6: ALERT & NOTIFICATION SYSTEM
**Priority: P2**

#### 6.1 Alert Types
- Price alerts (above/below target)
- Whale movement alerts
- TVL change alerts (protocol)
- Fear & Greed extreme levels
- Technical signal alerts (RSI overbought/oversold)
- New token launch alerts
- Yield opportunity alerts (APY > threshold)

#### 6.2 Notification Channels
- Telegram Bot (primary)
- Discord Webhook
- Email (optional)
- In-app notifications

---

### MODULE 7: API & DASHBOARD
**Priority: P1**

#### 7.1 REST API (Rust/Axum)
```
GET  /api/v1/tokens              — List tokens with market data
GET  /api/v1/tokens/:id          — Token detail
GET  /api/v1/tokens/:id/price    — Price history
GET  /api/v1/tokens/:id/ta       — Technical analysis
GET  /api/v1/defi/tvl            — TVL overview
GET  /api/v1/defi/yields         — Yield opportunities
GET  /api/v1/defi/dex            — DEX analytics
GET  /api/v1/onchain/whales      — Whale tracker
GET  /api/v1/onchain/:chain      — Chain analytics
GET  /api/v1/sentiment           — Market sentiment
GET  /api/v1/alerts              — Alert management
POST /api/v1/alerts              — Create alert
WS   /ws/v1/live                 — Real-time data stream
```

#### 7.2 Frontend Dashboard
- **Tech:** Next.js + TailwindCSS + TradingView Lightweight Charts
- **Pages:**
  - 🏠 Dashboard — Market overview, trending, fear/greed
  - 📊 Token Analysis — Price chart + indicators + signals
  - 🐋 Whale Tracker — Real-time whale movements
  - 🏦 DeFi Hub — TVL, yields, DEX analytics
  - 📰 Sentiment — Social + News + Fear/Greed
  - ⚡ Alerts — Configure & manage alerts
  - 🔍 Screener — Filter tokens by criteria

---

## 🔧 TECH STACK CHI TIẾT

### Backend (Rust)
| Component | Technology |
|-----------|-----------|
| Web Framework | `axum` |
| Async Runtime | `tokio` |
| HTTP Client | `reqwest` |
| Serialization | `serde` + `serde_json` |
| Database | `sqlx` + SQLite |
| Cache | `sled` hoặc `moka` |
| WebSocket | `tokio-tungstenite` |
| Cron Scheduler | `tokio-cron-scheduler` |
| Technical Analysis | Custom Rust (ta library) |
| Logging | `tracing` |
| Error Handling | `anyhow` + `thiserror` |
| CLI | `clap` |
| Config | `config` + TOML |

### Frontend
| Component | Technology |
|-----------|-----------|
| Framework | Next.js 14+ (App Router) |
| UI Library | TailwindCSS + shadcn/ui |
| Charts | TradingView Lightweight Charts + Recharts |
| State | Zustand |
| Data Fetching | TanStack Query |
| WebSocket | native WebSocket API |

### Infrastructure
| Component | Technology |
|-----------|-----------|
| Database | SQLite (dev) / PostgreSQL (prod) |
| Cache | sled embedded hoặc Redis |
| Deployment | Docker + Docker Compose |
| Monitoring | Prometheus metrics (optional) |

---

## 📅 LỘ TRÌNH TRIỂN KHAI (6 PHASES)

### PHASE 1: Foundation (Tuần 1-2) ✅
- [ ] Project scaffolding (Cargo workspace)
- [ ] Database schema & migrations
- [ ] Config system (TOML + env)
- [ ] CoinGecko API client
- [ ] Price collector service
- [ ] Basic REST API (Axum)
- [ ] SQLite storage layer

### PHASE 2: Market Intelligence (Tuần 3-4) ✅
- [ ] CoinMarketCap + CoinPaprika clients
- [ ] OHLCV candle collector
- [ ] Technical Analysis engine (RSI, MACD, MA, BB)
- [ ] Signal generator
- [ ] Market screener logic
- [ ] Extended API endpoints

### PHASE 3: On-Chain & DeFi (Tuần 5-7) ✅
- [ ] Blockchain RPC clients (ETH, SOL, BSC)
- [ ] Etherscan/Blockchain explorer integration
- [ ] Whale transaction detector
- [ ] DeFiLlama API client
- [ ] TVL tracking & history
- [ ] DEX analytics (DexScreener)
- [ ] Yield pool tracker

### PHASE 4: Sentiment & Alerts (Tuần 8-9) ✅
- [ ] Fear & Greed Index integration
- [ ] Twitter/Reddit scraper
- [ ] News aggregator
- [ ] Sentiment scoring engine
- [ ] Alert rule engine
- [ ] Telegram Bot notifications
- [ ] Discord webhook integration

### PHASE 5: Frontend Dashboard (Tuần 10-13) ✅
- [ ] Next.js project setup
- [ ] Market overview dashboard
- [ ] Token analysis page with charts
- [ ] Whale tracker dashboard
- [ ] DeFi analytics dashboard
- [ ] Sentiment dashboard
- [ ] Alert management UI
- [ ] Real-time WebSocket updates
- [ ] Mobile responsive design

### PHASE 6: Polish & Deploy (Tuần 14-15) ✅
- [ ] Docker containerization
- [ ] Performance optimization
- [ ] API rate limiting & error handling
- [ ] Documentation & API docs
- [ ] Testing (unit + integration)
- [ ] CI/CD pipeline
- [ ] Production deployment guide

---

## 📂 CARGO WORKSPACE STRUCTURE

```
crypto-research/
├── Cargo.toml                    # Workspace root
├── config.toml                   # Configuration
├── docker-compose.yml
├── crates/
│   ├── crp-core/                 # Core types, error handling, config
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs
│   │       ├── error.rs
│   │       └── types.rs
│   │
│   ├── crp-db/                   # Database layer
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs
│   │       ├── models.rs
│   │       └── repository.rs
│   │
│   ├── crp-collector/            # Data collection services
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── coingecko.rs
│   │       ├── coinmarketcap.rs
│   │       ├── defillama.rs
│   │       ├── dexscreener.rs
│   │       ├── etherscan.rs
│   │       ├── blockchain_rpc.rs
│   │       └── scheduler.rs
│   │
│   ├── crp-ta/                   # Technical Analysis engine
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── indicators/
│   │       │   ├── mod.rs
│   │       │   ├── rsi.rs
│   │       │   ├── macd.rs
│   │       │   ├── bollinger.rs
│   │       │   ├── ma.rs
│   │       │   ├── adx.rs
│   │       │   └── stochastic.rs
│   │       ├── patterns.rs
│   │       └── signals.rs
│   │
│   ├── crp-onchain/              # On-chain analysis
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── whale_tracker.rs
│   │       ├── chain_analyzer.rs
│   │       └── token_analyzer.rs
│   │
│   ├── crp-sentiment/            # Sentiment analysis
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── fear_greed.rs
│   │       ├── social.rs
│   │       └── news.rs
│   │
│   ├── crp-alerts/               # Alert system
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs
│   │       ├── telegram.rs
│   │       └── discord.rs
│   │
│   └── crp-api/                  # API server (binary)
│       └── src/
│           ├── main.rs
│           ├── routes/
│           │   ├── mod.rs
│           │   ├── tokens.rs
│           │   ├── defi.rs
│           │   ├── onchain.rs
│           │   ├── sentiment.rs
│           │   ├── ta.rs
│           │   └── alerts.rs
│           ├── ws.rs
│           └── middleware.rs
│
├── frontend/                     # Next.js dashboard
│   ├── package.json
│   ├── src/
│   │   ├── app/
│   │   ├── components/
│   │   └── lib/
│   └── ...
│
└── migrations/                   # SQL migrations
    ├── 001_init.sql
    └── ...
```

---

## 🔑 DATA SOURCES SUMMARY (MIỄN PHÍ)

| Source | Data | Free Tier | API Type |
|--------|------|-----------|----------|
| CoinGecko | Prices, market, metadata | 30 req/min | REST |
| CoinMarketCap | Prices, rankings | 10K req/month | REST |
| CoinPaprika | Prices, history | 25K req/month | REST |
| DeFiLlama | TVL, yields, fees | Unlimited (free) | REST |
| DexScreener | DEX pairs, prices | Free | REST |
| Etherscan | ETH on-chain | 100K req/day | REST |
| BscScan | BSC on-chain | 100K req/day | REST |
| Alternative.me | Fear & Greed Index | Free | REST |
| Birdeye | Solana DEX data | Free tier | REST |
| Blockchain RPCs | Raw chain data | Free public RPCs | JSON-RPC |

---

## 💡 ĐIỂM MẠNH CỦA PLATFORM

1. **100% Miễn phí** — Tận dụng tối đa free API tiers
2. **Rust Performance** — Xử lý data cực nhanh, memory-safe
3. **Self-hosted** — Không phụ thuộc third-party platform
4. **Real-time** — WebSocket cho live data updates
5. **Modular** — Mỗi module độc lập, dễ mở rộng
6. **Alert System** — Thông báo Telegram khi có cơ hội
7. **Technical Analysis** — Tính toán indicators native Rust
8. **DeFi Coverage** — TVL, yields, DEX data toàn diện
