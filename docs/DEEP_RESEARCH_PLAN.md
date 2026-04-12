# 📋 Deep Research Feature — Kế hoạch thực hiện

## Tổng quan
Tính năng Deep Research cho phép người dùng yêu cầu BonBo nghiên cứu chuyên sâu một vấn đề.
Nhiều subagent chạy **song song**, mỗi agent tập trung vào một góc độ khác nhau, sau đó một **Synthesis Agent**
tổng hợp kết quả thành báo cáo cuối cùng.

---

## Phase 0: Foundation — Core Types & Config
- [ ] Tạo module `src/research/mod.rs` — module root, re-exports
- [ ] Tạo `src/research/types.rs` — ResearchTask, ResearchResult, AgentFinding, SynthesisReport
- [ ] Tạo `src/research/config.rs` — ResearchConfig (max_agents, depth, timeout, model_override)
- [ ] Tạo `src/research/errors.rs` — ResearchError enum với thiserror
- [ ] Thêm feature flag `deep_research_enabled` vào `src/feature_flags.rs`

## Phase 1: Research Planner — Decompose Problem
- [ ] Tạo `src/research/planner.rs` — Research Planner
  - [ ] Hàm `decompose_research(topic, depth) -> Vec<ResearchSubtask>`
  - [ ] Gọi AI để phân tích vấn đề → chia thành 3-6 góc độ nghiên cứu
  - [ ] Mỗi subtask có: focus_area, search_queries[], perspective, expected_output
  - [ ] Prompt template cho planner
- [ ] Tạo `src/research/prompts.rs` — Prompt templates cho planner/synthesis
- [ ] Unit test cho planner với mock AI client

## Phase 2: Research Agent — Individual Worker
- [ ] Tạo `src/research/agent.rs` — ResearchAgent struct
  - [ ] Mỗi agent nhận 1 ResearchSubtask
  - [ ] Chu trình: web_search → đọc kết quả → phân tích → search tiếp (iterative)
  - [ ] Max iterations configurable (mặc định 3 vòng)
  - [ ] Ghi nhận sources, key findings, confidence level
  - [ ] Stream progress về parent qua mpsc channel
- [ ] Tạo `src/research/source.rs` — Source tracking (URL, title, snippet, relevance_score)

## Phase 3: Orchestrator — Fan-out/Fan-in Pattern
- [ ] Tạo `src/research/orchestrator.rs` — ResearchOrchestrator
  - [ ] `async fn run_research(task: ResearchTask) -> Result<SynthesisReport>`
  - [ ] Phase 1: Planner phân chia → Vec<ResearchSubtask>
  - [ ] Phase 2: Spawn N agents song song qua `tokio::spawn` + `FuturesUnordered`
  - [ ] Phase 3: Thu thập kết quả từ tất cả agents
  - [ ] Phase 4: Synthesis Agent tổng hợp báo cáo
  - [ ] Progress tracking qua `tokio::sync::watch` channel
  - [ ] Timeout per-agent và total timeout
  - [ ] Graceful cancellation qua CancellationToken

## Phase 4: Synthesis Agent — Tổng hợp báo cáo
- [ ] Tạo `src/research/synthesis.rs` — Synthesis Agent
  - [ ] Nhận Vec<AgentFinding> từ tất cả research agents
  - [ ] Gọi AI để tổng hợp: loại bỏ trùng lặp, tìm patterns, đánh giá conflicting info
  - [ ] Output: SynthesisReport { executive_summary, detailed_findings, sources, recommendations, confidence }
  - [ ] Format báo cáo: Markdown với sections rõ ràng
  - [ ] Bonus: So sánh quan điểm giữa các agents

## Phase 5: Tool Integration — CLI & Telegram
- [ ] Thêm tool definition `deep_research` vào `src/tools/definitions.rs`
- [ ] Thêm tool execution branch trong `src/tools/executor.rs`
- [ ] Progress UI trong CLI — hiển thị agent nào đang chạy, tìm thấy gì
- [ ] Telegram integration — gửi progress updates + báo cáo cuối
- [ ] Lưu research report vào Knowledge Base qua `knowledge_add`

## Phase 6: Persistence & History
- [ ] Tạo `src/research/storage.rs` — SQLite persistence
  - [ ] Table `research_sessions` — id, topic, created_at, status, report_path
  - [ ] Table `research_findings` — session_id, agent_idx, focus_area, finding, sources
  - [ ] CRUD operations
- [ ] CLI command: `/research history` — xem các phiên research trước
- [ ] CLI command: `/research show <id>` — xem lại báo cáo

## Phase 7: Polish & Testing
- [ ] Integration test: end-to-end research flow với mock AI
- [ ] Integration test: concurrent agent execution
- [ ] Integration test: timeout và cancellation
- [ ] Benchmarks: so sánh sequential vs parallel research
- [ ] Documentation: ARCHITECTURE.md update
- [ ] Error handling edge cases

---

## Kiến trúc tổng thể

```
User Query: "So sánh Rust vs Go cho backend services"
        │
        ▼
┌─────────────────────┐
│  Research Planner   │ ← AI decompose problem
│  (1 AI call)        │
└────────┬────────────┘
         │
    ┌────┴────┬──────────┬──────────┐
    ▼         ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│Agent 1 │ │Agent 2 │ │Agent 3 │ │Agent 4 │   ← PARALLEL
│Perf    │ │Eco-sys │ │Deploy  │ │Cost    │     (tokio::spawn)
│bench   │ │libs    │ │ops     │ │licens  │
└───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘
    │          │          │          │
    └────┬─────┴────┬─────┴──────────┘
         ▼          ▼
┌─────────────────────────┐
│  Synthesis Agent        │ ← AI tổng hợp
│  (1-2 AI calls)         │
└──────────┬──────────────┘
           ▼
┌─────────────────────────┐
│  SynthesisReport        │
│  - Executive Summary    │
│  - Detailed Findings    │
│  - Sources & Citations  │
│  - Recommendations      │
│  - Confidence Score     │
└─────────────────────────┘
```

## Module Structure

```
src/research/
├── mod.rs           # Module root, public API
├── types.rs         # Core types (ResearchTask, AgentFinding, SynthesisReport)
├── config.rs        # Configuration (ResearchConfig)
├── errors.rs        # Error types (ResearchError)
├── planner.rs       # Problem decomposition via AI
├── agent.rs         # Individual research agent (iterative search+analyze)
├── orchestrator.rs  # Fan-out/fan-in coordinator
├── synthesis.rs     # Report synthesis from multiple findings
├── source.rs        # Source tracking & citation management
├── prompts.rs       # Prompt templates
└── storage.rs       # SQLite persistence for research sessions
```

## Dependencies mới cần thêm
- `tokio-util` (đã có) — cho CancellationToken
- Không cần thêm crate nào mới! Tận dụng:
  - `tokio::spawn` + `FuturesUnordered` cho parallelism
  - `tokio::sync::mpsc` cho progress reporting
  - `serde`/`serde_json` cho serialization
  - `reqwest` + existing web_search cho searching
  - `bonbo-km` cho embedding-based relevance

## Ước tính effort
- Phase 0-1: 2-3 giờ (Foundation + Planner)
- Phase 2: 2 giờ (Research Agent)
- Phase 3: 3-4 giờ (Orchestrator — core complexity)
- Phase 4: 2 giờ (Synthesis)
- Phase 5: 2-3 giờ (Tool integration + UI)
- Phase 6: 1-2 giờ (Persistence)
- Phase 7: 2 giờ (Testing & Polish)
- **Tổng: ~14-18 giờ development**
