# Ferrum Bugfix Sprint - Activity Log

## 2026-04-13 - Bugfix Sprint

### P0 Critical Fixes
- **#2 Fixed double fee deduction in paper trading** (`ferrum-paper/src/engine.rs`)
  - BUY: Fee deducted from total cost (fill_price * amount + fee)
  - SELL: Fee deducted from proceeds (fill_price * amount - fee)
  - Fee only tracked in position for accounting, not double-deducted
  - Added round-trip PnL test verifying 489.5 USDT profit

- **#3 Fixed short selling in backtest** (`ferrum-backtest/src/engine.rs`)
  - Short now properly locks margin (balance → 0)
  - Close returns margin + PnL (margin + (entry - exit) * amount - fees)
  - Extracted `close_trade()` helper for consistent long/short PnL calculation
  - Added unit tests for long profit (489.5) and short profit (490.5)

- **#4 Fixed PnL accumulation in position tracker** (`ferrum-paper/src/tracker.rs`)
  - Fee deducted once at position level, not re-deducted at account level
  - Fixed partial close logic (use min(amount, pos.amount))
  - Added reversal handling (over-close opens opposite position)
  - Added tests for: partial close, add-to-position, average entry price

- **#12 Fixed Dashboard data race** (`ferrum-dashboard/src/server.rs`)
  - Changed `Arc<DashboardState>` → `Arc<RwLock<DashboardState>>`
  - Handlers acquire read lock before accessing state
  - Added SharedDashboardState type alias
  - Added concurrent read/write test

### P1 High Priority Fixes
- **#5 Added API key sanitization** (`ferrum-exchange/src/shared.rs`)
  - New ExchangeHttpClient with URL sanitization
  - Replaces sensitive params (apiKey, signature, timestamp) with "***"
  - Added tests for sanitization

- **#6 Fixed Bybit orderbook** (`ferrum-exchange/src/bybit/rest.rs`)
  - Corrected field mapping: arr[0] = price, arr[1] = size
  - Added proper error messages for unimplemented WebSocket streaming

- **#9 Created shared exchange HTTP client** (`ferrum-exchange/src/shared.rs`)
  - ExchangeHttpClient with public_get, signed_get, signed_post, json_post
  - ExchangeResponseChecker trait for per-exchange error handling
  - DRY foundation for future adapter refactoring

- **#10 Fixed O(n²) backtest** (`ferrum-backtest/src/engine.rs`)
  - Changed `candles[..=i].to_vec()` → `&candles[..=i]`
  - compute_indicators receives slice, only to_vec when strategy needs owned

### P2 Medium Priority Fixes
- **#7 Added Price/Quantity validation** (`ferrum-core/src/types.rs`)
  - `Price::checked(f64)` and `Quantity::checked(f64)` with NaN/Inf/negative checks
  - `Price::unchecked(f64)` for hot paths where value is known good
  - Added `InvalidInput` error variant to FerrumError

- **#15 Fixed subscribe placeholders** (all exchange adapters)
  - Changed from silently returning empty channels to returning proper errors
  - Message: "WebSocket streaming not yet implemented. Use get_xxx() for REST snapshots."

- **#16 Fixed Hyperliquid asset index** (`ferrum-exchange/src/hyperliquid/rest.rs`)
  - Added `resolve_asset_index()` that queries /info meta endpoint
  - Looks up coin name in universe array to get correct index
  - Falls back to 0 with warning if not found

### P3 Low Priority Fixes
- **#13 Fixed OKX signature** (`ferrum-exchange/src/okx/rest.rs`)
  - GET: requestPath includes query params (endpoint + "?" + params)
  - POST: requestPath is endpoint only, body passed separately
  - Consistent with OKX V5 authentication documentation

- **#17 Fixed Sharpe annualization** (`ferrum-backtest/src/metrics.rs`)
  - Changed from hardcoded sqrt(252) to sqrt(trades_per_year)
  - trades_per_year = total_trades / duration_years
  - More accurate for strategies with varying trade frequency

- **#19 Added Send+Sync static assert** (`ferrum-orchestrator/src/coordinator.rs`)
  - Compile-time verification that Orchestrator is Send + Sync

### Final Stats
- **88 tests**, all passing
- **0 compilation errors**
- **9,682 lines** of Rust code
