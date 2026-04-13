# Ferrum Bugfix Sprint - TODO

## P0 - Critical (Sai số tiền, Crash)
- [x] #2 Fix double fee deduction in paper trading engine
- [x] #3 Fix short selling logic in backtest engine
- [x] #4 Fix PnL accumulation error in position tracker
- [x] #12 Fix Dashboard data race (add RwLock)

## P1 - High (Bảo mật, Dữ liệu)
- [x] #5 Sanitize API keys from logs (ExchangeHttpClient)
- [x] #6 Fix Bybit orderbook field mapping (index 0=price, 1=size)
- [x] #9 Extract shared exchange HTTP client (DRY - shared.rs)
- [x] #10 Fix O(n²) backtest performance (use slice reference)

## P2 - Medium (Validation, Precision)
- [x] #7 Add Price/Quantity validation constructors (checked/unchecked)
- [x] #15 Mark unimplemented subscribe methods with proper errors
- [x] #16 Fix Hyperliquid hardcoded asset index (resolve_asset_index)

## P3 - Low (Architecture, Metrics)
- [x] #13 Fix OKX signature endpoint handling (requestPath includes query for GET)
- [x] #17 Fix Sharpe annualization (use trades-per-year factor)
- [x] #19 Add Send+Sync bounds to orchestrator (static assert)

## Final Verification
- [x] cargo check - 0 errors, 51 warnings
- [x] cargo test - 88 tests, ALL PASSING
- [x] 9,682 lines of Rust code
