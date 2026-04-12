# BonBo AutoResearch — Implementation Plan

> Adapt Karpathy's AutoResearch pattern for BonBo AI Coding Agent.
> Human writes strategy in markdown → Agent runs autonomous research loop overnight → Human reviews validated results in the morning.

---

## 📖 Background

**Karpathy's AutoResearch** (26K⭐ GitHub, March 2026): AI agent tự chạy 100+ ML experiments qua đêm trên 1 GPU, chỉ giữ lại improvements.

**Core Pattern** — Không giới hạn ML:
```
Define METRIC → Define CONSTRAINTS → Write program.md → Set agent FREE → Sleep → Review results
```

### Karpathy's 4 Levels of AI-Assisted Development
```
Level 1: Vibe Coding        — Human prompts, AI writes
Level 2: Agentic Engineering — Human orchestrates agents (BonBo hiện tại)
Level 3: Independent Research — Human sets direction, agent runs autonomously ← AutoResearch
Level 4: Agent Swarm         — Distributed research community
```

---

## 🎯 Vision: BonBo AutoResearch

### Mục tiêu
Biến BonBo từ **Level 2** (Agentic Engineering) → **Level 3** (Independent Research), trong khi giữ nguyên Rust performance + Knowledge Management advantages.

### Not Just ML — Universal Research

| Domain | Metric | Constraint | Agent edits |
|---|---|---|---|
| Code Optimization | Benchmark time | Tests pass | src/**/*.rs |
| Refactoring | Lines of code | Tests pass, same behavior | src/**/*.rs |
| Prompt Engineering | Task success rate | Cost ceiling | prompts/**/*.md |
| Security | Vulnerability count | Tests pass | Cargo.toml, src/ |
| Binary Size | Binary size (MB) | Tests pass | Cargo.toml, src/ |
| Documentation | Coverage % | Accuracy | docs/**/*.md |
| ML Training | val_bpb / loss | Time budget | train.py |

---

## 🏗️ Architecture

### 3 Components (mirrors Karpathy's 3-file design)

```
┌──────────────────────────────────────────────────────────┐
│                  BONBO AUTORESEARCH                       │
│                                                           │
│  1. program.md    ← HUMAN writes (strategy)              │
│     "Focus on reducing P99 latency of the router"        │
│     "Try different attention patterns"                    │
│     "Keep code simple, reject ugly optimizations"        │
│                                                           │
│  2. autoresearch.rs ← AGENT executes (tactics)           │
│     Ratchet loop: propose → commit → measure → keep/revert│
│     NEVER STOP until max_iterations or Ctrl+C             │
│                                                           │
│  3. metric_command ← SYSTEM measures (ground truth)      │
│     Shell command → parse metric → compare with best      │
│     e.g., "cargo bench 2>&1 | grep 'mean:'"              │
│     e.g., "uv run train.py > run.log && grep val_bpb"    │
└──────────────────────────────────────────────────────────┘
```

### The Ratchet Loop

```
┌──────────────────────────────────────────────────────────────┐
│                    RATCHET LOOP                               │
│                                                               │
│  1. 🧠 AI proposes idea → edits files in scope               │
│  2. 📦 git commit -m "autoresearch: <description>"           │
│  3. 🚀 Run metric_command (shell)                            │
│  4. 📊 Parse metric value from output                        │
│  5. ✅ Improved? → KEEP (update best_metric)                 │
│     ❌ Worse?    → git reset --hard HEAD~1 (DISCARD)         │
│     💥 Crash?    → log error, revert, continue               │
│  6. 📝 Log to results.tsv                                    │
│  7. 💾 Auto-save findings to Knowledge Base                  │
│  8. 🔁 Loop to 1 (NEVER STOP until max_iterations)          │
└──────────────────────────────────────────────────────────────┘
```

### Data Structures

```rust
// src/autoresearch/config.rs
pub struct AutoResearchConfig {
    pub topic: String,                    // "Reduce router P99 latency"
    pub metric_command: String,           // "cargo bench 2>&1 | grep 'mean:'"
    pub metric_direction: MetricDirection, // LowerIsBetter or HigherIsBetter
    pub file_scope: Vec<String>,          // ["src/client/router.rs", "src/client/*.rs"]
    pub immutable_files: Vec<String>,     // ["src/tools/definitions.rs"] — agent CANNOT edit
    pub time_budget_secs: u64,            // Per-iteration: 300 (5 min)
    pub max_iterations: usize,            // 0 = infinite (until Ctrl+C)
    pub branch_prefix: String,            // "autoresearch/<tag>"
    pub save_to_knowledge: bool,          // Auto-save findings to KB
    pub simplicity_weight: f64,           // 0.0 = ignore, 1.0 = equal to metric
}

pub enum MetricDirection {
    LowerIsBetter,  // val_bpb, latency, binary size, vuln count
    HigherIsBetter, // accuracy, throughput, coverage %
}

// src/autoresearch/result.rs
pub struct IterationResult {
    pub iteration: usize,
    pub commit: String,           // 7-char git hash
    pub metric_value: f64,        // e.g., 0.9697
    pub memory_or_resource: f64,  // e.g., VRAM MB, binary size MB
    pub status: IterationStatus,  // Keep, Discard, Crash
    pub description: String,      // "increase LR to 0.04"
    pub duration_secs: u64,
    pub lines_changed: i32,       // +10 or -5 (simplicity tracking)
}

pub enum IterationStatus { Keep, Discard, Crash }

// src/autoresearch/report.rs
pub struct ResearchReport {
    pub topic: String,
    pub total_iterations: usize,
    pub improvements: usize,
    pub crashes: usize,
    pub baseline_metric: f64,
    pub best_metric: f64,
    pub improvement_pct: f64,
    pub results: Vec<IterationResult>,
    pub duration_total: Duration,
    pub knowledge_id: Option<i64>,  // Auto-saved to KB
}
```

---

## 📋 Implementation Phases

### Phase 1: Foundation (2-3 days) — Core Ratchet Loop

**Files to create/modify:**
- `src/autoresearch/mod.rs` — Module entry
- `src/autoresearch/config.rs` — Config structs
- `src/autoresearch/engine.rs` — Core ratchet loop
- `src/autoresearch/git_ops.rs` — Git operations (commit, reset, branch)
- `src/autoresearch/metric.rs` — Metric extraction (shell → parse f64)
- `src/autoresearch/result.rs` — Result logging to TSV
- `src/tools/definitions.rs` — Add `deep_research` tool definition
- `src/tools/executor.rs` — Dispatch to autoresearch engine

**Tasks:**
- [ ] Create `src/autoresearch/` module with config structs
- [ ] Implement `git_ops.rs`: branch creation, commit, reset --hard, short hash
- [ ] Implement `metric.rs`: run shell command, parse metric value
- [ ] Implement ratchet logic: compare metric → keep/revert
- [ ] Implement results.tsv logging (TSV format, untracked)
- [ ] Add `autoresearch` tool to definitions.rs
- [ ] Add dispatch in executor.rs
- [ ] Test with simple metric: "wc -l src/main.rs" (lower is better)

### Phase 2: Autonomous Loop (2-3 days) — NEVER STOP

**Tasks:**
- [ ] Implement infinite loop with iteration budget (0 = infinite)
- [ ] Crash recovery: tail log → classify error → revert → continue
- [ ] Simplicity criterion: track lines_changed, penalize complexity
- [ ] Context management: redirect output to log file (don't flood context)
- [ ] Progress display: iteration count, best metric, improvement %, ETA
- [ ] Graceful shutdown: SIGINT/Ctrl+C → save state → print summary
- [ ] Timeout per iteration: kill if exceeds time_budget_secs
- [ ] program.md parsing: read strategy, inject into AI prompt

### Phase 3: Knowledge Integration (1-2 days)

**Tasks:**
- [ ] Auto-save findings to knowledge base via `knowledge_add`
- [ ] Generate structured research report (markdown)
- [ ] Cross-session continuity: save/load state from KB
- [ ] Meta-research: agent can rewrite its own program.md based on results
- [ ] Research history: `knowledge_search("autoresearch")` shows all past runs

### Phase 4: Multi-Domain Presets (1-2 days)

**Tasks:**
- [ ] Preset: Code Optimization — metric = benchmark time
- [ ] Preset: Refactoring — metric = line count + tests pass
- [ ] Preset: Prompt Engineering — metric = task success rate
- [ ] Preset: Security Audit — metric = vulnerability count
- [ ] Preset: Binary Size — metric = stripped binary size
- [ ] Preset: ML Training (Karpathy-compatible) — metric = val_bpb
- [ ] Parallel research: spawn subagents for independent experiments

---

## 🎯 Example Usage

### Example 1: Code Optimization
```
User: "/autoresearch --topic 'Optimize router latency' \
       --metric 'cargo bench -- router_mean 2>&1 | grep mean' \
       --direction lower \
       --scope 'src/client/router.rs,src/client/rate_limiter.rs' \
       --budget 300 \
       --iterations 50"

BonBo: 🔬 AutoResearch started on branch autoresearch/apr12-router
       Branch: autoresearch/apr12-router
       Metric: router mean latency (lower is better)
       Baseline: 2.34ms
       Running 50 iterations (5 min each)...
       
       [1/50] Try: replace HashMap with FxHashMap → 2.34ms → KEEP ✅
       [2/50] Try: add LRU cache to route lookup → 2.01ms → KEEP ✅  
       [3/50] Try: inline hot path → crash → DISCARD ❌
       [4/50] Try: pre-compute routing table → 1.87ms → KEEP ✅
       ...
       
       📊 Results: 50 iterations, 12 improvements, 3 crashes
       Baseline: 2.34ms → Best: 1.52ms (35% improvement)
       Report saved to knowledge base (ID: 82)
```

### Example 2: ML Training (Karpathy-compatible)
```
User: "/autoresearch --topic 'Reduce val_bpb' \
       --metric 'grep val_bpb run.log' \
       --direction lower \
       --scope 'train.py' \
       --immutable 'prepare.py' \
       --budget 300 \
       --iterations 0"  # 0 = infinite

BonBo: 🔬 AutoResearch started (Karpathy-compatible mode)
       Branch: autoresearch/apr12-ml
       Baseline: 0.997900
       Running until interrupted...
       
       [1] Baseline → 0.9979
       [2] Add QKnorm scaler → 0.9932 → KEEP ✅
       [3] Increase LR to 0.04 → 0.9891 → KEEP ✅
       ...
```

### Example 3: Prompt Engineering
```
User: "/autoresearch --topic 'Optimize coding prompts' \
       --metric './scripts/eval_prompts.sh 2>&1 | grep score' \
       --direction higher \
       --scope 'prompts/**/*.md' \
       --budget 120 \
       --iterations 20"
```

---

## 🆚 BonBo AutoResearch vs Karpathy's Original

| Feature | Karpathy AutoResearch | BonBo AutoResearch |
|---|---|---|
| **Language** | Python | Rust |
| **Domain** | ML training only | Universal (any measurable metric) |
| **Knowledge** | Git history + TSV only | Git + TSV + Knowledge Base |
| **Cross-session** | Manual | Auto-persist in KB |
| **Meta-research** | Community proposal | Phase 3: agent rewrites program.md |
| **Parallel** | SkyPilot (manual) | Subagent swarm (built-in) |
| **Integration** | Standalone | Integrated in coding agent |
| **Search** | No web search | web_search for papers/solutions |
| **SpecKit** | No | Phase 4: integrate research into SDD |
| **Smart Router** | Fixed model | Complexity-based routing (cheap model for iterations) |

---

## 🔮 Future Vision: Level 4 — Agent Swarm

```
┌─────────────────────────────────────────────────────────────┐
│              BONBO RESEARCH SWARM (Level 4)                   │
│                                                               │
│  Subagent A: "Try Rust optimizations"  ──┐                   │
│  Subagent B: "Try algorithm changes"   ──┤                   │
│  Subagent C: "Try dependency swaps"    ──┼──→ Shared         │
│  Subagent D: "Try architecture tweaks" ──┤    results.tsv    │
│  Subagent E: "Search papers for ideas"  ──┘                  │
│                                                               │
│  Meta-Agent: Reviews all results, rewrites program.md         │
│  Coordinator: Merges best improvements from each agent        │
│                                                               │
│  Like SETI@home for software research                         │
└─────────────────────────────────────────────────────────────┘
```
