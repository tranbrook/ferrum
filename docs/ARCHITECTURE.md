# Architecture

BonBo is a Rust-based AI coding assistant with a modular architecture designed for reliability, safety, and extensibility.

## System Overview

```
┌──────────────────────────────────────────────────────────────┐
│                        BonBo v2.2.0                           │
├──────────┬──────────┬──────────┬─────────────────────────────┤
│  CLI     │ Telegram │  Prompt  │      Tools                  │
│  REPL    │   Bot    │ Builder  │  (20+ tools)                │
├──────────┴──────────┴──────────┴─────────────────────────────┤
│                   AI Processor (stream)                       │
├──────────────────────────────────────────────────────────────┤
│              Smart Router (failover + routing)                │
├──────────┬──────────┬──────────┬─────────────────────────────┤
│  Z.AI    │  OpenAI  │  Ollama  │  (extensible)               │
│  (GLM)   │ (GPT-4o) │ (Local)  │                             │
├──────────┴──────────┴──────────┴─────────────────────────────┤
│                                                              │
│  ┌──────────── Safety Layer (P1) ─────────────────────────┐  │
│  │  Compaction (5-stage) │ Permission (3 levels)          │  │
│  │  Bash Security        │ Prompt Cache Boundary          │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────── Quality Layer (P2) ────────────────────────┐  │
│  │  Tool Result Cache │ Verification Agent │ Hooks System │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────── Intelligence Layer (P3) ───────────────────┐  │
│  │  Subagent │ AutoDream │ Checkpoint/Undo │ Feature Flags │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  Config  │ Conversation │ Rate Limiter │   Metrics           │
├──────────────────────────────────────────────────────────────┤
│                   bonbo-km (SQLite)                          │
│  knowledge │ conversations │ feedback │ prompt_experiments   │
│  episodes  │ relations     │ tasks    │ tool_usage           │
└──────────────────────────────────────────────────────────────┘
```

## Module Breakdown

### Core (`src/`)

| Module | Responsibility |
|--------|---------------|
| `main.rs` | CLI REPL loop, command dispatch, session management |
| `ai_client.rs` | `AiClient` trait, `ModelConfig`, `Provider` enum, client factory |
| `ai_processor.rs` | Shared stream chunk processing (content, tool calls, reasoning) |
| `config.rs` | `.env` loading, API key resolution, multi-provider config |

### Client Providers (`src/client/`)

| Module | Responsibility |
|--------|---------------|
| `mod.rs` | Client module re-exports |
| `glm.rs` | Z.AI GLM API client with SSE streaming, retry with exponential backoff |
| `openai.rs` | OpenAI GPT-4o-mini/GPT-4 client with SSE streaming, tool calling |
| `ollama.rs` | Ollama local model client (OpenAI-compatible endpoint), no API key |
| `router.rs` | Smart Router — failover, load balancing, provider selection |
| `rate_limiter.rs` | Token bucket rate limiter (configurable RPM, 429 backoff) |

### Smart Router

The `SmartRouter` implements `AiClient` and transparently manages multiple providers:

| Strategy | Behavior |
|----------|----------|
| **Failover** (default) | Use primary, fall back on failure, auto-recover |
| **Fastest** | Route to provider with lowest latency |
| **Cheapest** | Route to lowest-cost provider (by priority) |
| **ComplexityBased** | Route by query complexity (planned) |

Key features:
- Health tracking per provider (consecutive failures, latency, success count)
- Auto-switch after 3 consecutive failures
- Recovery after provider reset
- Transparent — implements same `AiClient` trait as single providers

### 🛡️ Safety Layer — P1 (`src/`)

These modules protect BonBo from bricking its context, running unsafe commands, and wasting tokens.

| Module | Responsibility |
|--------|---------------|
| `compaction.rs` | 5-stage cascade: Snip → MicroCompact → Collapse → AutoCompact → HardTruncate. Model-specific token targets. Emergency compact on API `context_length_exceeded` errors. |
| `permission.rs` | 3-level permission system: ReadOnly → WorkspaceWrite → DangerFullAccess. 27+ tools classified by risk. DenialTracker — 3 consecutive / 20 total denials → fallback to prompting. |
| `bash_security.rs` | Validates shell commands before execution. Blocklist (rm -rf, mkfs, dd, etc.). Pattern-based dangerous command detection. Audit log of all bash executions. |
| `prompts/builder.rs` | Prompt Cache Boundary — splits system prompt into static (cacheable) + dynamic (per-session). Variable interpolation ({{cwd}}, {{date}}, {{git_branch}}). |
| `prompts/profile.rs` | Prompt profiles (full/concise/tutor) with TOML serialization. |
| `prompts/section.rs` | Section resolution (built-in, file-based, inline). |
| `prompts/defaults.rs` | Default section content (identity, tools, workflow, etc.). |
| `prompts/variables.rs` | Template variables (date, project, custom). |

#### 5-Stage Compaction Cascade

```
Conversation too large?
  │
  ├── Stage 1: Snip — Remove system/assistant markers, clean formatting
  │   └── Still too large? ↓
  ├── Stage 2: MicroCompact — Trim oldest messages to 100-char summaries
  │   └── Still too large? ↓
  ├── Stage 3: Collapse — Replace messages with "N messages collapsed"
  │   └── Still too large? ↓
  ├── Stage 4: AutoCompact — AI-powered summarization (placeholder)
  │   └── Still too large? ↓
  └── Stage 5: HardTruncate — Keep only most recent messages
```

#### Permission Levels

```
┌───────────────────┐
│ ReadOnly (0)      │ → read_file, grep, search, knowledge_get
├───────────────────┤
│ WorkspaceWrite (1)│ → + create_file, edit_file, shell commands
├───────────────────┤
│ DangerFullAccess  │ → + all tools, no restrictions
│ (2, default)      │
└───────────────────┘

DenialTracker: 3 consecutive OR 20 total denials → auto-fallback to prompting
```

### 🔍 Quality Layer — P2 (`src/`)

These modules improve reliability of tool execution and results.

| Module | Responsibility |
|--------|---------------|
| `tool_result_cache.rs` | Persists large tool outputs to `~/.bonbo/tool_results/`. Model sees preview (2000 chars) + disk path. Auto-cleanup after configurable hours. |
| `verification.rs` | Post-execution verification of tool results. Checks: empty output, error patterns ("not found", "command not found", "permission denied"), truncation warnings. Advisory only — does not block. |
| `hooks.rs` | PreToolUse / PostToolUse hook system. Hook decisions: Allow, Deny (with reason), Modify (change args), Defer (ask user). Supports wildcard `*` hooks. Built-in: bash logger, file write logger. |

#### Tool Result Persistence Flow

```
Tool execution completes
  │
  ├── Result ≤ max_tool_result_chars?
  │   └── Yes → Return inline
  │
  ├── tool_result_persistence enabled?
  │   └── Yes → Save to ~/.bonbo/tool_results/{tool}_{timestamp}.txt
  │            Return preview (2000 chars) + "use read_file(path) for full output"
  │
  └── Fallback → In-memory truncation with "[... truncated N chars]"
```

#### Hooks Execution Flow

```
execute_tool() called
  │
  ├── Pre-hooks (if enabled)
  │   ├── Tool-specific hooks first
  │   ├── Then wildcard (*) hooks
  │   ├── Decision: Allow → continue
  │   ├── Decision: Deny → return error
  │   ├── Decision: Modify → re-execute with new args
  │   └── Decision: Defer → auto-approve in full-access mode
  │
  ├── Execute tool
  ├── Verify result (if enabled)
  ├── Budget result (truncate/persist)
  │
  └── Post-hooks (if enabled)
      ├── Tool-specific hooks
      └── Wildcard hooks
```

### 🧠 Intelligence Layer — P3 (`src/`)

These modules add advanced capabilities for scaling and memory.

| Module | Responsibility |
|--------|---------------|
| `subagent.rs` | Fork agent context for isolated tasks. Config: max tool calls, allowed tools, file scope. State tracking (files read/modified, errors). Global registry with `register_subagent` / `finalize_subagent`. |
| `auto_dream.rs` | Session consolidation on exit. Heuristic-based extraction of key findings ("Key finding:", "Important:", "Decision:", "TODO:", "FIXME:"). Saves to Knowledge Base. Logs consolidation event. |
| `checkpoint.rs` | Auto-snapshot files before `create_file` / `edit_file`. Saved to `~/.bonbo/checkpoints/` with metadata. `/undo` restores most recent. `/checkpoints` lists all. Auto-cleanup configurable. |
| `feature_flags.rs` | 12 runtime feature flags. TOML config at `~/.bonbo/feature_flags.toml`. Env var overrides via `BONBO_FF_*=true/false`. Global lazy-initialized singleton. |

#### Feature Flags

| Flag | Default | Controls |
|------|---------|----------|
| `compaction_enabled` | ✅ true | 5-stage compaction |
| `permission_enabled` | ✅ true | Permission checks |
| `bash_validation_enabled` | ✅ true | Bash command validation |
| `prompt_cache_boundary` | ✅ true | Static/dynamic prompt split |
| `tool_result_persistence` | ✅ true | Large results → disk |
| `deferred_tools` | ⬜ false | Load only core tools |
| `hooks_enabled` | ⬜ false | Pre/post tool hooks |
| `checkpoints_enabled` | ✅ true | Auto-checkpoint before edits |
| `subagents_enabled` | ⬜ false | Subagent forking |
| `verification_agent` | ⬜ false | Tool result verification |
| `auto_dream` | ✅ true | Auto-memory consolidation |
| `reactive_compaction` | ✅ true | Emergency compact on API errors |

### CLI Commands (`src/commands/`)

| Module | Responsibility |
|--------|---------------|
| `model_cmd.rs` | `/model list`, `/model switch`, `/model status` commands |
| `menu_cmd.rs` | Interactive menu for API key setup |

### Tools (`src/tools/`)

| Tool | Description |
|------|-------------|
| `file_ops.rs` | Read, create, edit, append, multi-file operations |
| `shell.rs` | Bash/shell command execution (with security validation) |
| `web_search.rs` | SerpAPI Google search |
| `searxng.rs` | SearXNG privacy-respecting search |
| `smart_reader.rs` | Smart web reader (HTTP first, browser fallback) |
| `pinchtab.rs` | Browser automation (click, type, snapshot) |
| `knowledge.rs` | Knowledge base (add, search, relations, tasks) |
| `executor.rs` | Tool dispatch with hooks, verification, persistence, checkpoints |
| `definitions.rs` | Tool definitions (core, always, deferred classification) |

### Knowledge Management (`bonbo-km/`)

Standalone SQLite library with these tables:

| Table | Purpose |
|-------|---------|
| `knowledge` | Persistent knowledge entries with FTS5 search |
| `episodes` | Episodic memory log (events, queries, actions) |
| `relations` | Knowledge graph relations between entries |
| `tasks` | Persistent task tracking |
| `conversations` | Message history per session |
| `conversations_fts` | FTS5 full-text search over conversations |
| `feedback` | User feedback on AI responses |
| `tool_usage` | Tool execution tracking with timing |
| `model_usage` | Token consumption per request |
| `rate_limit_events` | 429 response logging |
| `prompt_experiments` | A/B prompt tracking with feedback correlation |

### Telegram Bot (`src/telegram/`)

| Module | Responsibility |
|--------|---------------|
| `bot.rs` | Teloxide bot setup, message routing |
| `handlers.rs` | Message handling, conversation management |
| `formatter.rs` | MarkdownV2 formatting, code block handling |
| `runtime_config.rs` | Runtime configuration for Telegram bot |

## Data Flow

### CLI Query Flow (with Safety Layers)

```
User Input → main.rs (REPL)
  → ConversationStore::add_user_message()
  → compact_if_needed()        ← P1: 5-stage compaction
  → SmartRouter::chat_stream()
    → Select provider (strategy-based)
    → Rate Limiter (wait if needed)
    → Provider API (SSE stream) — Z.AI / OpenAI / Ollama
    → On failure: auto-failover to next healthy provider
    → On context_length_exceeded: emergency_compact() + retry  ← P3: reactive
  → AiProcessor::process_stream()
    → Content chunks → stdout
    → Tool calls → ToolExecutor::execute()
      → Permission check          ← P1: permission level
      → Bash security validation  ← P1: command blocklist
      → Pre-hooks (if enabled)    ← P2: hooks system
      → Checkpoint file (if edit) ← P3: checkpoint/undo
      → Execute tool
      → Verify result (if enabled)← P2: verification
      → Persist/truncate result   ← P2: tool result cache
      → Post-hooks (if enabled)   ← P2: hooks system
      → Tool result → ConversationStore::add_tool_result()
      → Recursive AI call with tool results
    → Final response → ConversationStore::add_assistant_message()
  → Metrics recording
  → Feedback prompt
```

### Session Exit Flow (with AutoDream)

```
/quit or /exit
  │
  ├── AutoDream consolidation (if enabled)
  │   ├── Extract findings from session
  │   ├── Save to Knowledge Base (max 10)
  │   └── Log consolidation event to episodes
  │
  ├── Cleanup old tool results (>168h)
  ├── Cleanup old checkpoints (>168h)
  ├── Display session metrics
  └── Exit
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ZAI_API_KEY` | — | Z.AI API key (primary provider) |
| `OPENAI_API_KEY` | — | OpenAI API key |
| `OPENAI_MODEL` | `gpt-4o-mini` | OpenAI model to use |
| `OPENAI_BASE_URL` | `https://api.openai.com/v1` | OpenAI-compatible base URL |
| `OLLAMA_URL` | — | Ollama server URL |
| `OLLAMA_MODEL` | `llama3.2` | Ollama model to use |
| `BONBO_ROUTING_STRATEGY` | `failover` | Routing: failover/fastest/cheapest/complexity |
| `SERPAPI_KEY` | — | SerpAPI key for Google search |
| `PINCHTAB_URL` | — | PinchTab browser URL |
| `BONBO_PERMISSION` | `dangerfullaccess` | Permission level: readonly/workspacereadwrite/dangerfullaccess |
| `BONBO_FF_*` | varies | Feature flag overrides (see Feature Flags table) |

### File-based Configuration

| File | Purpose |
|------|---------|
| `.env` | API keys, provider config |
| `~/.bonbo/feature_flags.toml` | Feature flag settings |
| `~/.bonbo/prompts/sections/` | Custom prompt section overrides |
| `~/.bonbo/tool_results/` | Persisted large tool outputs |
| `~/.bonbo/checkpoints/` | File edit snapshots |
| `~/.bonbo/knowledge.db` | Knowledge base (SQLite) |

## Design Decisions

1. **Multi-provider architecture** — Smart Router transparently manages providers
2. **Failover-first** — Automatic recovery, never leave user without a working provider
3. **SQLite over external DB** — Zero-config, single-file, ACID, fast FTS5
4. **Token bucket rate limiting** — Smooth request pacing, respects 429 headers
5. **SSE streaming** — Real-time response display, better UX
6. **5-stage compaction** — Graceful degradation, never lose the conversation
7. **Default-deny permissions** — Safety-first with backward-compatible defaults
8. **Prompt cache boundary** — Static/dynamic split maximizes cache hits
9. **Persistent knowledge** — Survives across sessions, builds institutional memory
10. **Ollama for offline** — Zero-cost local fallback, no API key needed
11. **Feature flags for gradual rollout** — Toggle subsystems without redeployment
12. **Hooks for extensibility** — Allow/Deny/Modify/Defer without modifying core
