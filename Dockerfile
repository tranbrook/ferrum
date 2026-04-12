# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ferrum-core/Cargo.toml crates/ferrum-core/
COPY crates/ferrum-exchange/Cargo.toml crates/ferrum-exchange/
COPY crates/ferrum-executors/Cargo.toml crates/ferrum-executors/
COPY crates/ferrum-positions/Cargo.toml crates/ferrum-positions/
COPY crates/ferrum-risk/Cargo.toml crates/ferrum-risk/
COPY crates/ferrum-llm/Cargo.toml crates/ferrum-llm/
COPY crates/ferrum-agent/Cargo.toml crates/ferrum-agent/
COPY crates/ferrum-routines/Cargo.toml crates/ferrum-routines/
COPY crates/ferrum-api/Cargo.toml crates/ferrum-api/
COPY crates/ferrum-mcp/Cargo.toml crates/ferrum-mcp/
COPY crates/ferrum-telegram/Cargo.toml crates/ferrum-telegram/
COPY crates/ferrum-cli/Cargo.toml crates/ferrum-cli/

# Create dummy source files for dependency caching
RUN mkdir -p crates/ferrum-core/src && echo "" > crates/ferrum-core/src/lib.rs
RUN mkdir -p crates/ferrum-exchange/src && echo "" > crates/ferrum-exchange/src/lib.rs
RUN mkdir -p crates/ferrum-executors/src && echo "" > crates/ferrum-executors/src/lib.rs
RUN mkdir -p crates/ferrum-positions/src && echo "" > crates/ferrum-positions/src/lib.rs
RUN mkdir -p crates/ferrum-risk/src && echo "" > crates/ferrum-risk/src/lib.rs
RUN mkdir -p crates/ferrum-llm/src && echo "" > crates/ferrum-llm/src/lib.rs
RUN mkdir -p crates/ferrum-agent/src && echo "" > crates/ferrum-agent/src/lib.rs
RUN mkdir -p crates/ferrum-routines/src && echo "" > crates/ferrum-routines/src/lib.rs
RUN mkdir -p crates/ferrum-api/src && echo "" > crates/ferrum-api/src/lib.rs
RUN mkdir -p crates/ferrum-mcp/src && echo "" > crates/ferrum-mcp/src/lib.rs
RUN mkdir -p crates/ferrum-telegram/src && echo "" > crates/ferrum-telegram/src/lib.rs
RUN mkdir -p crates/ferrum-cli/src && echo "fn main() {}" > crates/ferrum-cli/src/main.rs

# Build dependencies only (cache layer)
RUN cargo build --release 2>/dev/null || true

# Copy actual source code
COPY . .

# Build final binary
RUN touch crates/*/src/*.rs && cargo build --release -p ferrum-cli

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates sqlite-libs

COPY --from=builder /app/target/release/ferrum /usr/local/bin/ferrum

# Default config
COPY config/ /app/config/

EXPOSE 8080 8081

ENTRYPOINT ["ferrum"]
CMD ["serve", "--port", "8080"]
