# Deployment Guide

## Local Development

### Prerequisites

- Rust 1.75+ (recommended: latest stable)
- C compiler (for rusqlite bundled SQLite)
- Git

### Setup

```bash
git clone https://github.com/tranbrook/vybrid.git
cd vybrid-rust
cp .env.example .env
# Edit .env with your Z.AI API key
cargo run
```

## Docker Deployment

### Build

```bash
docker build -t bonbo:latest .
```

### CLI Mode

```bash
docker compose run bonbo
```

### Telegram Bot Mode

```bash
# Set TELEGRAM_BOT_TOKEN in .env
docker compose up -d bonbo-telegram
```

### View Logs

```bash
docker compose logs -f bonbo-telegram
```

## GitHub Container Registry (GHCR)

Images are automatically published on release:

```bash
docker pull ghcr.io/tranbrook/vybrid:latest
docker pull ghcr.io/tranbrook/vybrid:1.4.1
```

## Production Checklist

- [ ] Set `ZAI_API_KEY` in `.env` or environment
- [ ] Set `TELEGRAM_BOT_TOKEN` for Telegram mode
- [ ] Configure `TELEGRAM_ALLOWED_USERS` to restrict access
- [ ] Set `BONBO_RATE_LIMIT_RPM` if needed (default: 20)
- [ ] Mount persistent volume for `~/.bonbo/`
- [ ] Set `RUST_LOG=info` for production, `RUST_LOG=debug` for troubleshooting
- [ ] Configure `restart: unless-stopped` in Docker Compose

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `ZAI_API_KEY` | — | **Required.** Z.AI API key |
| `SERPAPI_KEY` | — | Optional. SerpAPI for Google Search |
| `TELEGRAM_BOT_TOKEN` | — | Optional. Telegram bot token |
| `TELEGRAM_ALLOWED_USERS` | — | Optional. Comma-separated Telegram user IDs |
| `TELEGRAM_ALLOWED_CHATS` | — | Optional. Comma-separated chat IDs |
| `TELEGRAM_PROMPT_MODE` | `concise` | `concise` or `full` |
| `BONBO_RATE_LIMIT_RPM` | `20` | API requests per minute |
| `BONBO_ROOT` | `~/.bonbo` | Data directory |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

## Release Process

Releases are automated via GitHub Actions:

1. Update version in `Cargo.toml` and `CHANGELOG.md`
2. Commit: `git commit -m "chore: bump version to x.y.z"`
3. Tag: `git tag vx.y.z`
4. Push: `git push origin main --tags`
5. CI automatically:
   - Builds for Linux (x86_64), macOS (ARM64), Windows (x86_64)
   - Creates GitHub Release with binaries attached
   - Pushes Docker image to GHCR

### Manual Binary Build

```bash
# Linux
cargo build --release
# Binary at target/release/bonbo

# Cross-compile (install target first)
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

## Monitoring

### Health Check

The Telegram bot runs continuously. Monitor via:

```bash
docker compose ps
docker compose logs --tail=100 bonbo-telegram
```

### Database

Knowledge base stored at `$BONBO_ROOT/knowledge.db`:

```bash
sqlite3 ~/.bonbo/knowledge.db ".tables"
sqlite3 ~/.bonbo/knowledge.db "SELECT count(*) FROM knowledge;"
```

## Troubleshooting

### Build fails on rusqlite
Ensure you have a C compiler installed:
```bash
# Ubuntu/Debian
apt-get install build-essential

# macOS (Xcode Command Line Tools)
xcode-select --install
```

### Docker build slow
Docker layer caching should handle this. For a clean rebuild:
```bash
docker compose build --no-cache
```

### Telegram bot not responding
1. Check token is valid: `curl https://api.telegram.org/bot<TOKEN>/getMe`
2. Check logs: `docker compose logs bonbo-telegram`
3. Verify `TELEGRAM_ALLOWED_USERS` if set
