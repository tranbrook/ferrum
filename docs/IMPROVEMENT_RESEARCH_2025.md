# BonBo (vybrid-rust) — Đề xuất Cải tiến Dựa trên Nghiên cứu Kiến thức Mới 2025-2026

> **Ngày phân tích**: 2026-04-07  
> **Phiên bản hiện tại**: v1.5.1 | ~16,000 dòng Rust code | 54 source files  
> **Nguồn tham khảo**: Rust 2024 Edition, Rig.rs, MCP Protocol, Tree-sitter Code Indexing, Ratatui TUI, và các xu hướng AI coding assistant mới nhất

---

## 📋 Tóm tắt Điều tra Dự án Hiện tại

### Điểm mạnh
- ✅ Kiến trúc modular tốt: Smart Router, multi-provider, failover
- ✅ Knowledge Management với SQLite + FTS5
- ✅ 30+ tools cho AI agent (file ops, web search, browser automation, knowledge)
- ✅ Streaming SSE với byte-level buffering UTF-8
- ✅ Retry với exponential backoff + jitter
- ✅ Rate limiting với token bucket
- ✅ Telegram bot integration
- ✅ Prompt Builder system (profile/section/variable)
- ✅ Metrics & analytics (model usage, tool usage, feedback)
- ✅ 284 tests passing
- ✅ CI/CD pipelines (GitHub Actions)
- ✅ Docker support

### Điểm yếu / Cơ hội cải tiến
- ❌ Edition 2021 — chưa upgrade lên Rust 2024
- ❌ Chưa có MCP (Model Context Protocol) support
- ❌ Không có RAG / Code Indexing (Tree-sitter)
- ❌ Token counting chỉ dùng char_len/4 — không chính xác
- ❌ Không có cấu trúc CLI argument parsing (chỉ dùng raw args)
- ❌ UI chỉ dùng println! — không có TUI dashboard
- ❌ Không có tool call parallelization
- ❌ Anthropic (Claude) provider chưa implement
- ❌ Không có on-device/local inference option (ngoài Ollama)
- ❌ Không có config validation / health-check command
- ❌ Knowledge base thiếu embedding/semantic search
- ❌ Không có conversation summarization
- ❌ Chưa có workspace-level project awareness

---

## 🏗️ PHẦN 1: NÂNG CẤP NỀN TẢNG RUST

### 1.1 Upgrade lên Rust 2024 Edition ⭐⭐⭐

**Lý do**: Rust 2024 (stabilized in 1.85) mang đến:
- `async fn` trong traits — loại bỏ cần `async_trait` macro
- `for<'a>` lifetime capture rules mới — rõ ràng hơn
- `gen` blocks ( nightly) — iterator generators
- Or-patterns trong `match`
- Array `IntoIterator` — `for x in [1,2,3]` hoạt động trực tiếp
- Cải tiến `impl Trait` lifetime capture

**Thay đổi cụ thể**:
```toml
# Cargo.toml
edition = "2024"
```

```rust
// TRƯỚC: cần async_trait crate
#[async_trait]
pub trait AiClient: Send + Sync {
    async fn chat_stream(&self, ...) -> Result<BoxStream<StreamChunk>>;
}

// SAU: native async fn in trait (Rust 2024)
pub trait AiClient: Send + Sync {
    async fn chat_stream(&self, ...) -> Result<BoxStream<StreamChunk>>;
}
```

**Files cần sửa**: `ai_client.rs`, `Cargo.toml` (remove `async-trait` dependency)

**Impact**: Giảm dependency, code idiomatic hơn, compile time nhanh hơn

---

### 1.2 Thay thế `once_cell` bằng `std::sync::LazyLock` ⭐⭐

**Lý do**: `LazyLock` đã stable trong Rust 1.80+. Project đã dùng nó trong `definitions.rs` nhưng vẫn còn `once_cell` dependency.

```rust
// TRƯỚC
use once_cell::sync::Lazy;
static BUILDER: Lazy<Mutex<PromptBuilder>> = Lazy::new(|| ...);

// SAU
use std::sync::LazyLock;
static BUILDER: LazyLock<Mutex<PromptBuilder>> = LazyLock::new(|| ...);
```

**Impact**: Loại bỏ 1 dependency

---

### 1.3 CLI Argument Parsing với `clap` ⭐⭐⭐

**Lý do**: Hiện tại dùng raw `std::env::args()` matching, rất hạn chế.

```rust
// Cargo.toml
clap = { version = "4", features = ["derive"] }

// src/cli.rs
#[derive(Parser)]
#[command(name = "bonbo", version, about = "AI Coding Assistant")]
struct Cli {
    /// Run in Telegram bot mode
    #[arg(short, long)]
    telegram: bool,

    /// Set log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Load a specific session
    #[arg(long)]
    session: Option<String>,

    /// Use a specific prompt profile
    #[arg(long)]
    profile: Option<String>,

    /// Run a single command and exit
    #[arg(long)]
    command: Option<String>,

    /// Show version and config info
    #[arg(long)]
    info: bool,
}
```

**Impact**: User experience tốt hơn, auto-generated help, tab completion, proper parsing

---

## 🧠 PHẦN 2: AI CAPABILITY IMPROVEMENTS

### 2.1 MCP (Model Context Protocol) Server Support ⭐⭐⭐⭐⭐

**Lý do**: MCP đã trở thành industry standard (2025) cho AI tool integration. Tất cả AI coding assistants lớn đều hỗ trợ MCP.

```rust
// Cargo.toml
rmcp = "0.1"  // Rust MCP SDK (rust-mcp-stack)

// src/mcp/server.rs — expose BonBo's tools as MCP server
pub struct BonboMcpServer {
    km: Arc<KmState>,
    router: Arc<SmartRouter>,
}

impl McpServer for BonboMcpServer {
    async fn list_tools(&self) -> Vec<ToolDefinition> {
        // Convert BonBo tool definitions to MCP format
        get_all_tools().into_iter().map(|t| ToolDefinition {
            name: t.function.name,
            description: t.function.description,
            input_schema: t.function.parameters,
        }).collect()
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value> {
        let result = execute_tool(name, &args.to_string(), None).await?;
        Ok(Value::String(result))
    }
}
```

**Impact**: BonBo có thể hoạt động như MCP server cho Claude Code, Cursor, v.v. Hoặc BonBo có thể call MCP servers khác.

---

### 2.2 Tree-sitter Code Indexing & RAG ⭐⭐⭐⭐⭐

**Lý do**: Paper "Codebase-Memory" (2026) cho thấy Tree-sitter knowledge graph giảm 56% token usage so với grep-based exploration.

```rust
// Cargo.toml
tree-sitter = "0.24"
tree-sitter-rust = "0.23"

// src/indexing/code_index.rs
pub struct CodeIndex {
    /// AST-based symbol table: file → functions, structs, traits, impls
    symbols: HashMap<PathBuf, Vec<Symbol>>,
    /// Dependency graph: module → imports
    imports: HashMap<PathBuf, Vec<Import>>,
    /// Function call graph
    call_graph: HashMap<String, Vec<String>>,
}

pub struct Symbol {
    name: String,
    kind: SymbolKind, // Function, Struct, Trait, Enum, Impl, Const, Type
    file: PathBuf,
    line_start: usize,
    line_end: usize,
    doc_comment: Option<String>,
    visibility: Visibility,
    signature: Option<String>, // fn foo(x: i32) -> Result<String>
}

impl CodeIndex {
    /// Build index for a project directory using Tree-sitter
    pub fn index_project(root: &Path) -> Result<Self> { ... }

    /// Find relevant code for a query — replaces random grep
    pub fn query(&self, query: &str, limit: usize) -> Vec<CodeContext> { ... }

    /// Generate a project overview (for system prompt)
    pub fn project_summary(&self) -> String { ... }
}
```

**Impact**: Giảm token usage, context chất lượng hơn, hiểu cấu trúc code thay vì chỉ text matching

---

### 2.3 Token Counting Chính xác với `tiktoken-rs` ⭐⭐⭐

**Lý do**: Hiện tại dùng `text.len() / 4` — sai lệch lớn với code Rust, tiếng Việt, emoji.

```rust
// Cargo.toml
tiktoken-rs = "0.5"

// src/tokenizer.rs
use tiktoken_rs::cl100k_base;

pub struct TokenCounter {
    encoder: CoreBPE,
}

impl TokenCounter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            encoder: cl100k_base()?,
        })
    }

    pub fn count_tokens(&self, text: &str) -> usize {
        self.encoder.encode_with_special_tokens(text).len()
    }

    pub fn count_messages_tokens(&self, messages: &[Message]) -> usize {
        messages.iter()
            .map(|m| {
                let mut count = 4; // message overhead
                if let Some(ref content) = m.content {
                    count += self.count_tokens(content);
                }
                count += self.count_tokens(&m.role);
                count
            })
            .sum()
    }
}
```

**Impact**: Trim conversation chính xác hơn, tránh vượt quá context window, ước lượng chi phí chính xác

---

### 2.4 Parallel Tool Execution ⭐⭐⭐⭐

**Lý do**: Khi AI gọi nhiều tool calls (ví dụ: đọc 5 files), hiện tại thực hiện sequential. Parallel execution giảm latency đáng kể.

```rust
// src/main.rs — trong process_ai_response
if !response.tool_calls.is_empty() {
    ui::print_tool_execution(response.tool_calls.len());

    // THAY VÌ: sequential
    // for tool_call in &response.tool_calls { ... }

    // MỚI: parallel execution
    let results: Vec<(String, Result<String>)> = futures::future::join_all(
        response.tool_calls.iter().map(|tc| async {
            let name = tc.function.name.clone();
            let result = execute_tool(&name, &tc.function.arguments, Some(session_id)).await;
            (name, result)
        })
    ).await;

    for (tool_call, (name, result)) in response.tool_calls.iter().zip(results) {
        // ... handle results
    }
}
```

**Impact**: Giảm thời gian xử lý tool calls từ O(n*t) xuống O(max(t))

---

### 2.5 Conversation Summarization ⭐⭐⭐

**Lý do**: Khi conversation quá dài, thay vì cắt (trim), hãy tóm tắt.

```rust
// src/conversation/summarizer.rs
pub struct ConversationSummarizer {
    client: Arc<dyn AiClient>,
}

impl ConversationSummarizer {
    /// Summarize old messages and replace with a compact version.
    pub async fn summarize(&self, messages: &[Message]) -> Result<String> {
        let summary_prompt = format!(
            "Summarize the following conversation in a compact but complete way. \
             Preserve all important code decisions, file paths, and technical details:\n\n\
             {}",
            messages.iter()
                .filter(|m| m.role != "system")
                .map(|m| format!("[{}]: {}", m.role, m.content.as_deref().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Use a cheap/fast model for summarization
        let response = self.client.chat(
            vec![Message {
                role: "user".to_string(),
                content: Some(summary_prompt),
                tool_calls: None,
                tool_call_id: None,
            }],
            None,
        ).await?;

        Ok(response.content.unwrap_or_default())
    }
}
```

**Impact**: Giữ context quan trọng, giảm token usage, session dài vẫn hiệu quả

---

## 🖥️ PHẦN 3: UI/UX IMPROVEMENTS

### 3.1 Ratatui TUI Dashboard ⭐⭐⭐⭐

**Lý do**: Ratatui là TUI framework #1 cho Rust (3200+ crates, 19.5k stars). Thay thế println! bằng interactive dashboard.

```rust
// Cargo.toml
ratatui = "0.29"
crossterm = "0.28"

// src/tui/app.rs
pub enum AppMode {
    Chat,
    Tools,
    Metrics,
    Knowledge,
    Sessions,
}

pub struct App {
    mode: AppMode,
    input: String,
    messages: Vec<ChatMessage>,
    scroll_offset: u16,
    show_sidebar: bool,
}

// Features:
// - Split pane: chat (left) + tool results (right)
// - Bottom input bar with autocomplete
// - Ctrl+T: toggle tools panel
// - Ctrl+M: metrics dashboard
// - Ctrl+K: knowledge search popup
// - Syntax-highlighted code blocks
```

**Impact**: Professional UI, user-friendly hơn nhiều, competitive với Claude Code/Cursor

---

### 3.2 Syntax Highlighting Cải tiến cho Tool Output ⭐⭐

**Lý do**: Đã có `syntect` nhưng chưa dùng cho tool output.

```rust
// Hiện tại: raw text output
// Tương lai: syntax-highlighted output cho code blocks
```

**Impact**: Output đọc được tốt hơn

---

## 🔌 PHẦN 4: PROVIDER EXPANSION

### 4.1 Anthropic Claude Provider ⭐⭐⭐⭐

**Lý do**: Claude là model mạnh nhất cho coding. Đã có `Provider::Anthropic` enum nhưng chưa implement.

```rust
// src/client/anthropic.rs
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
    retry: RetryConfig,
}

// Anthropic API khác OpenAI:
// - Header: x-api-key thay vì Authorization: Bearer
// - Format: messages với content blocks
// - Tool use format khác
// - Streaming: SSE với event types (message_start, content_block_start, etc.)
```

**Impact**: Tiếp cận model mạnh nhất cho coding tasks

---

### 4.2 Google Gemini Provider ⭐⭐⭐

```rust
// src/client/gemini.rs
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,  // gemini-2.0-flash, gemini-2.5-pro
}

// Gemini có context window 1M+ tokens — rất phù hợp cho codebase analysis
```

**Impact**: Thêm lựa chọn model với context window cực lớn

---

## 🗄️ PHẦN 5: KNOWLEDGE BASE NÂNG CẤP

### 5.1 Semantic Search với Embeddings ⭐⭐⭐⭐

**Lý do**: FTS5 BM25 chỉ tìm exact keyword matches. Semantic search hiểu meaning.

```rust
// Cargo.toml
fastembed = "4"  // Local embedding inference (Rust, no Python)

// src/knowledge/embeddings.rs
pub struct SemanticSearch {
    model: TextEmbedding,  // fastembed
    dimension: usize,
}

impl SemanticSearch {
    pub async fn embed(&self, text: &str) -> Vec<f32> {
        self.model.embed(vec![text], None).await.unwrap()[0].clone()
    }

    pub async fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_embedding = self.embed(query).await;
        // Cosine similarity search against stored embeddings
        self.cosine_search(&query_embedding, limit)
    }
}
```

**Impact**: Tìm kiếm intelligent hơn — "how to handle errors" sẽ tìm thấy `anyhow` và `thiserror` usage

---

### 5.2 Knowledge Base Export/Import Formats ⭐⭐

```rust
// Export to multiple formats
pub fn export_json(&self, path: &str) -> Result<()>;
pub fn export_markdown(&self, dir: &str) -> Result<usize>; // ✅ đã có
pub fn export_orgmode(&self, dir: &str) -> Result<usize>;
pub fn export_obsidian(&self, vault: &str) -> Result<usize>; // Obsidian vault format
```

---

## ⚡ PHẦN 6: PERFORMANCE & RELIABILITY

### 6.1 Connection Pooling & HTTP/2 ⭐⭐⭐

```rust
// src/client/glm.rs — cải tiến Client builder
let client = Client::builder()
    .http2_prior_knowledge()        // HTTP/2 cho low latency
    .pool_max_idle_per_host(4)      // Tăng pool size
    .pool_idle_timeout(Duration::from_secs(120))
    .tcp_keepalive(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .timeout(Duration::from_secs(300))
    .build()?;
```

---

### 6.2 Graceful Shutdown ⭐⭐⭐

**Lý do**: Hiện tại Ctrl+C có thể cắt giữa tool execution.

```rust
// src/shutdown.rs
use tokio::signal;
use tokio_util::sync::CancellationToken;

pub struct ShutdownGuard {
    token: CancellationToken,
}

impl ShutdownGuard {
    pub fn new() -> Self {
        let token = CancellationToken::new();
        let cloned = token.clone();

        tokio::spawn(async move {
            signal::ctrl_c().await.ok();
            tracing::info!("Shutdown signal received, finishing current operation...");
            cloned.cancel();
        });

        Self { token }
    }

    pub fn cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
}
```

---

### 6.3 Schema Migration với `rusqlite_migration` ⭐⭐

**Lý do**: Hiện tại dùng `include_str!("schema.sql")` — không có migration path.

```rust
// bonbo-km/Cargo.toml
rusqlite_migration = "1"

// bonbo-km/src/migrations.rs
use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up("CREATE TABLE knowledge (...)"),
        M::up("ALTER TABLE knowledge ADD COLUMN embedding BLOB"),
        // Future migrations
    ])
}
```

**Impact**: Upgrade database schema an toàn mà không mất data

---

## 📊 PHẦN 7: ANALYTICS & INSIGHTS

### 7.1 Cost Tracking Dashboard ⭐⭐⭐

```rust
// src/analytics/cost.rs
pub struct CostTracker {
    pricing: HashMap<String, ModelPricing>,
}

struct ModelPricing {
    prompt_per_1m: f64,    // USD per 1M tokens
    completion_per_1m: f64,
}

impl CostTracker {
    pub fn estimate_session_cost(&self, session_id: &str) -> Result<CostReport> {
        // Query model_usage table and calculate costs
    }

    pub fn daily_summary(&self) -> Result<DailyReport> { ... }
}
```

---

### 7.2 Auto-Context: Smart File Suggestions ⭐⭐⭐⭐

**Lý do**: Khi user hỏi về code, tự động suggest relevant files.

```rust
// src/context/suggestor.rs
pub struct ContextSuggestor {
    code_index: CodeIndex,
    km: Arc<KmState>,
}

impl ContextSuggestor {
    /// Suggest relevant files based on user query
    pub fn suggest(&self, query: &str, max_files: usize) -> Vec<PathBuf> {
        // 1. Check knowledge base for project structure
        // 2. Use code index for symbol search
        // 3. Rank by relevance
        // 4. Return top N
    }
}
```

**Impact**: AI có context tốt hơn mà không cần user /add files thủ công

---

## 📦 PHẦN 8: PACKAGING & DISTRIBUTION

### 8.1 Completion Scripts ⭐⭐

```bash
# Auto-generate shell completion
bonbo --generate-completion bash > /etc/bash_completion.d/bonbo
bonbo --generate-completion zsh > ~/.zfunc/_bonbo
bonbo --generate-completion fish > ~/.config/fish/completions/bonbo.fish
```

---

### 8.2 Plugin System ⭐⭐⭐⭐

```rust
// src/plugin/mod.rs
pub trait BonboPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn tools(&self) -> Vec<Tool>;
    fn execute(&self, tool: &str, args: &str) -> Pin<Box<dyn Future<Output = Result<String>>>>;
}

// Dynamic loading
pub struct PluginManager {
    plugins: Vec<Box<dyn BonboPlugin>>,
}

// Users can add plugins:
// ~/.bonbo/plugins/weather.rs → compiled to .so → loaded at runtime
// Or Lua/WASM plugins for safety
```

---

## 🎯 PRIORITIZED ROADMAP

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| 🔴 P0 | **MCP Server Support** (2.1) | 2 weeks | Game-changer |
| 🔴 P0 | **Token Counting** (2.3) | 3 days | Accuracy |
| 🔴 P0 | **Parallel Tool Execution** (2.4) | 3 days | Speed |
| 🟠 P1 | **clap CLI Parsing** (1.3) | 2 days | UX |
| 🟠 P1 | **Anthropic Provider** (4.1) | 1 week | Coverage |
| 🟠 P1 | **Conversation Summarization** (2.5) | 4 days | Token savings |
| 🟠 P1 | **Rust 2024 Upgrade** (1.1) | 1 day | Modern |
| 🟡 P2 | **Tree-sitter Code Indexing** (2.2) | 3 weeks | Intelligence |
| 🟡 P2 | **Ratatui TUI** (3.1) | 3 weeks | UX |
| 🟡 P2 | **Semantic Search** (5.1) | 2 weeks | Search quality |
| 🟡 P2 | **Graceful Shutdown** (6.2) | 2 days | Reliability |
| 🟢 P3 | **Google Gemini** (4.2) | 1 week | Coverage |
| 🟢 P3 | **Cost Tracking** (7.1) | 3 days | Analytics |
| 🟢 P3 | **Plugin System** (8.2) | 4 weeks | Extensibility |
| 🟢 P3 | **Auto-Context** (7.2) | 2 weeks | UX |

---

## 📚 Tài liệu Tham khảo

1. **Rust 2024 Edition** — https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/
2. **MCP Specification** — https://modelcontextprotocol.io/specification/2025-11-25
3. **Rust MCP SDK** — https://github.com/rust-mcp-stack/rust-mcp-schema
4. **Rig.rs LLM Framework** — https://rig.rs/ — Agent orchestration in Rust
5. **Codebase-Memory Paper** — Tree-sitter Knowledge Graphs (arXiv 2026)
6. **Ratatui TUI** — https://ratatui.rs/
7. **fastembed** — Local embedding inference in Rust
8. **tiktoken-rs** — Accurate token counting for LLM models
9. **rusqlite_migration** — Schema migration for SQLite in Rust
10. **Crane (candle-rs)** — On-device LLM inference in pure Rust

---

*Báo cáo được tạo bởi BonBo AI Assistant dựa trên phân tích source code và nghiên cứu web.*
