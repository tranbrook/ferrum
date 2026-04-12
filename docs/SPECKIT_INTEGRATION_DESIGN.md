# 🏗️ THIẾT KẾ TÍCH HỢP SPEC KIT VÀO BONBO AI CODING AGENT

> **Phiên bản**: v1.0  
> **Ngày**: 2025-04-04  
> **Tác giả**: BonBo Agent  
> **Status**: Design Document

---

## 0. EXECUTIVE SUMMARY

### Vấn đề
BonBo hiện tại (vybrid-rust v1.5.1) hoạt động theo模式 **"vibe coding"** — user nhập prompt, agent sinh code, hy vọng đúng. Điều này dẫn đến:
- ❌ Không có **structured planning** trước khi code
- ❌ **Consistency thấp** giữa các sessions (quên context)
- ❌ Không có **quality gates** giữa các phases
- ❌ **Rework cao** — phải sửa nhiều lần vì hiểu sai yêu cầu

### Giải pháp
Tích hợp **Spec-Driven Development (SDD) workflow** từ GitHub Spec Kit vào BonBo, biến BonBo thành **SpecKit-powered coding agent** với:
- ✅ Structured 6-phase workflow: Constitution → Specify → Clarify → Plan → Tasks → Implement
- ✅ Constitution-as-code (lưu trong SQLite knowledge base)
- ✅ Template-driven quality (Markdown templates ép LLM tuân thủ quy trình)
- ✅ Phase gates với checklists
- ✅ Feature numbering + artifact co-location
- ✅ TDD enforcement

### Kết quả mong đợi
- 🎯 **90% giảm rework** từ việc clarify requirements trước
- 🎯 **Consistency across sessions** nhờ constitution persistence
- 🎯 **Predictable quality** nhờ template-driven workflow
- 🎯 **Faster delivery** nhờ structured task breakdown

---

## 1. KIẾN TRÚC TỔNG THỂ

### 1.1 So sánh: BonBo hiện tại vs BonBo + SpecKit

```
━━━ BONBO HIỆN TẠI (Vibe Coding) ━━━

User: "Build me a photo album app"
  │
  ▼
Agent: Nhập prompt → Sinh code → Hy vọng đúng
  │       (no planning, no specs)
  ▼
Output: Code (có thể đúng, có thể sai)

━━━ BONBO + SPEC KIT (SDD Workflow) ━━━

User: "Build me a photo album app"
  │
  ▼
Phase 0: Load Constitution ← SQLite knowledge base
  │
  ▼
Phase 1: SPECIFY (WHAT & WHY)
  │  → User stories, acceptance criteria
  │  → Template-driven, checklist validated
  ▼
Phase 2: CLARIFY
  │  → [NEEDS CLARIFICATION] markers
  │  → User confirms/adjusts
  ▼
Phase 3: PLAN (HOW)
  │  → Tech stack, architecture, data models
  │  → Constitutional compliance check
  ▼
Phase 4: TASKS
  │  → Actionable task breakdown
  │  → Dependencies, parallel markers
  ▼
Phase 5: IMPLEMENT
  │  → TDD: tests first, then code
  │  → Incremental, per-phase delivery
  ▼
Output: Verified code + Full spec documentation
```

### 1.2 Component Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                     BONBO AGENT (Rust)                        │
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐ │
│  │ SDD Engine  │  │  Templates   │  │  Knowledge Base     │ │
│  │ (NEW)       │  │  (NEW)       │  │  (EXISTING SQLite)  │ │
│  │             │  │              │  │                     │ │
│  │ Phase mgr  │  │ spec.md.tmpl │  │ constitution       │ │
│  │ Gate checker│  │ plan.md.tmpl │  │ specs              │ │
│  │ Numbering  │  │ tasks.md.tmpl│  │ plans              │ │
│  │ Clarify mgr│  │ clarify.tmpl │  │ tasks              │ │
│  └──────┬──────┘  └──────┬───────┘  │ episodes          │ │
│         │                │           │ relations         │ │
│         └────────┬───────┘           └─────────┬─────────┘ │
│                  │                             │           │
│         ┌────────▼─────────────────────────────▼─────────┐ │
│         │           EXISTING BONBO INFRASTRUCTURE        │ │
│         │                                                │ │
│         │  Tools: file_ops, shell, grep, knowledge,     │ │
│         │         web_search, project, smart_reader...   │ │
│         │                                                │ │
│         │  System: AiClient, SmartRouter, Streaming,    │ │
│         │         Conversation, Metrics, Telegram Bot    │ │
│         └────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. CHI TIẾT TỪNG COMPONENT MỚI

### 2.1 SDD Engine (`src/sdd/mod.rs`) — Component mới chính

```rust
// src/sdd/mod.rs — SpecKit Engine cho BonBo

pub mod constitution;
pub mod phases;
pub mod templates;
pub mod numbering;
pub mod gates;
pub mod clarify;

/// SDD Phase enum — 6 phases của SpecKit workflow
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SddPhase {
    Constitution,  // Phase 0: Establish principles
    Specify,       // Phase 1: WHAT & WHY
    Clarify,       // Phase 2: Clarify ambiguities
    Plan,          // Phase 3: HOW (tech stack, architecture)
    Tasks,         // Phase 4: Break down into tasks
    Implement,     // Phase 5: Execute tasks
}

/// SDD Session State — persisted in SQLite knowledge base
pub struct SddSession {
    pub project_name: String,
    pub current_phase: SddPhase,
    pub current_feature: Option<FeatureNumber>,
    pub constitution: Option<Constitution>,
    pub active_specs: Vec<FeatureSpec>,
}

/// Feature Number — auto-incremented per project
#[derive(Debug, Clone)]
pub struct FeatureNumber {
    pub number: u32,      // 001, 002, 003...
    pub slug: String,     // "photo-albums"
    pub branch: String,   // "001-photo-albums"
}

/// Constitution — project's immutable principles
pub struct Constitution {
    pub principles: Vec<Principle>,
    pub architecture_rules: Vec<String>,
    pub testing_standards: Vec<String>,
    pub quality_gates: Vec<Gate>,
}

/// Phase Gate — checkpoint before moving to next phase
pub struct Gate {
    pub phase: SddPhase,
    pub checklist: Vec<ChecklistItem>,
    pub status: GateStatus,
}

pub enum GateStatus {
    Pending,
    InReview,
    Passed,
    Failed(String),  // reason
}
```

### 2.2 Constitution Layer (`src/sdd/constitution.rs`)

```rust
// src/sdd/constitution.rs — Constitution-as-Code

use bonbo_km::KnowledgeManager;

/// Load constitution from SQLite knowledge base
pub fn load_constitution(km: &KnowledgeManager) -> Result<Option<Constitution>> {
    // Tìm trong knowledge base category="constitution"
    let entries = km.list_by_category("constitution", 10)?;
    // Parse thành Constitution struct
    ...
}

/// Save constitution to knowledge base (auto-loaded mỗi session)
pub fn save_constitution(km: &KnowledgeManager, constitution: &Constitution) -> Result<()> {
    km.add(KnowledgeEntry {
        title: format!("Constitution: {}", constitution.project_name),
        content: constitution.to_markdown(),
        category: "constitution".into(),
        tags: vec!["constitution".into(), "sdd".into()],
        ..Default::default()
    })?;
    Ok(())
}

/// Validate that a plan complies with constitution
pub fn validate_constitutional_compliance(
    constitution: &Constitution,
    plan: &ImplementationPlan,
) -> Vec<ComplianceIssue> {
    let mut issues = Vec::new();
    
    for principle in &constitution.principles {
        if !principle.is_satisfied_by(plan) {
            issues.push(ComplianceIssue {
                principle: principle.name.clone(),
                violation: principle.describe_violation(plan),
                severity: principle.severity,
            });
        }
    }
    
    issues
}
```

### 2.3 Template System (`src/sdd/templates.rs`)

```rust
// src/sdd/templates.rs — Template-driven quality

/// Spec Template — định nghĩa WHAT & WHY (không bàn HOW)
pub const SPEC_TEMPLATE: &str = r#"
# Feature Specification: {feature_name}

## Overview
> **Feature Number**: {feature_number}
> **Status**: Draft
> **Created**: {date}

## Motivation (WHY)
{motivation}

## User Stories
### US-{story_number}: {story_title}
**As a** {role},
**I want** {capability},
**So that** {benefit}.

**Acceptance Criteria:**
- [ ] {criterion_1}
- [ ] {criterion_2}
- [ ] {criterion_3}

## Out of Scope
{out_of_scope}

## Clarifications Needed
<!-- Mark anything ambiguous with [NEEDS CLARIFICATION: question] -->
{clarifications}

## Review Checklist
- [ ] No [NEEDS CLARIFICATION] markers remain
- [ ] All user stories have acceptance criteria
- [ ] Requirements are testable and unambiguous
- [ ] No implementation details (tech stack, APIs, code)
- [ ] Success criteria are measurable
"#;

/// Plan Template — định nghĩa HOW
pub const PLAN_TEMPLATE: &str = r#"
# Implementation Plan: {feature_name}

## Constitutional Compliance
- [ ] All principles from constitution.md are satisfied
- [ ] Architecture follows established patterns
- [ ] Testing strategy meets standards

## Tech Stack
{tech_stack}

## Architecture
{architecture}

## Data Model
{data_model}

## API Contracts
{contracts}

## Implementation Phases
### Phase 1: {phase_name}
- Tasks: {task_count}
- Estimated complexity: {complexity}

## Simplicity Gate (from Constitution)
- [ ] Using ≤3 major dependencies?
- [ ] No speculative features?
- [ ] Using framework directly, no unnecessary wrappers?
"#;

/// Tasks Template — task breakdown
pub const TASKS_TEMPLATE: &str = r#"
# Task Breakdown: {feature_name}

## Feature: {feature_number} — {feature_name}

### Phase 1: {phase_name}
- [ ] T1.1: {task_description} [TEST-FIRST]
- [ ] T1.2: {task_description}
- [ ] T1.3: {task_description} [P] (parallelizable)

### Phase 2: {phase_name}
- [ ] T2.1: {task_description} [depends: T1.1]
- [ ] T2.2: {task_description} [P]

## Checkpoint
- [ ] All Phase 1 tasks completed
- [ ] Tests passing
- [ ] No regressions
"#;

/// Clarification Template
pub const CLARIFY_TEMPLATE: &str = r#"
# Clarification: {feature_name}

## Ambiguities Found
{ambiguities}

## Questions for User
{questions}

## Resolved
{resolved}

## Still Needs Clarification
{still_unclear}
"#;
```

### 2.4 Phase Manager (`src/sdd/phases.rs`)

```rust
// src/sdd/phases.rs — Phase workflow management

use crate::sdd::{SddPhase, SddSession, Gate, GateStatus};

/// Main SDD workflow controller
pub struct SddEngine {
    session: SddSession,
    km: KnowledgeManager,
}

impl SddEngine {
    /// Start new SDD session — load constitution
    pub fn start_session(project_name: &str) -> Result<Self> {
        let km = KnowledgeManager::new()?;
        let constitution = load_constitution(&km)?;
        
        Ok(Self {
            session: SddSession {
                project_name: project_name.into(),
                current_phase: SddPhase::Constitution,
                current_feature: None,
                constitution,
                active_specs: Vec::new(),
            },
            km,
        })
    }
    
    /// Execute current phase and advance
    pub fn execute_phase(&mut self, input: &str) -> Result<PhaseOutput> {
        match self.session.current_phase {
            SddPhase::Constitution => self.execute_constitution(input),
            SddPhase::Specify     => self.execute_specify(input),
            SddPhase::Clarify     => self.execute_clarify(input),
            SddPhase::Plan        => self.execute_plan(input),
            SddPhase::Tasks       => self.execute_tasks(input),
            SddPhase::Implement   => self.execute_implement(input),
        }
    }
    
    /// Phase 0: Create/Load Constitution
    fn execute_constitution(&mut self, input: &str) -> Result<PhaseOutput> {
        let constitution = generate_constitution(input)?;
        
        // Save to knowledge base
        save_constitution(&self.km, &constitution)?;
        
        // Auto-advance to Specify
        self.session.current_phase = SddPhase::Specify;
        
        Ok(PhaseOutput {
            artifacts: vec![("constitution.md", constitution.to_markdown())],
            gate_status: GateStatus::Passed,
            next_action: "Use /speckit.specify to define WHAT you want to build".into(),
        })
    }
    
    /// Phase 1: Generate Feature Specification
    fn execute_specify(&mut self, input: &str) -> Result<PhaseOutput> {
        // Auto-number feature
        let feature_num = self.next_feature_number(input)?;
        self.session.current_feature = Some(feature_num.clone());
        
        // Generate spec from template
        let spec = generate_spec(input, &feature_num, &self.session.constitution)?;
        
        // Save to knowledge base + create file
        self.save_feature_artifact(&feature_num, "spec.md", &spec)?;
        
        // Advance to Clarify
        self.session.current_phase = SddPhase::Clarify;
        
        Ok(PhaseOutput {
            artifacts: vec![
                ("spec.md", spec),
            ],
            gate_status: GateStatus::Passed,
            next_action: "Review spec, then /speckit.clarify to resolve ambiguities".into(),
        })
    }
    
    // ... tương tự cho Clarify, Plan, Tasks, Implement
}
```

### 2.5 Knowledge Base Extensions

```sql
-- Thêm vào bonbo-km/src/schema.sql

-- SDD Feature tracking
CREATE TABLE IF NOT EXISTS sdd_features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project TEXT NOT NULL,
    feature_number INTEGER NOT NULL,     -- 001, 002, 003
    slug TEXT NOT NULL,                  -- "photo-albums"
    branch TEXT,                         -- "001-photo-albums"
    current_phase TEXT NOT NULL,         -- "specify", "plan", "tasks", "implement"
    spec_content TEXT,
    plan_content TEXT,
    tasks_content TEXT,
    constitution_id INTEGER,             -- FK to knowledge table
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    UNIQUE(project, feature_number)
);

-- SDD Phase gates
CREATE TABLE IF NOT EXISTS sdd_gates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL REFERENCES sdd_features(id),
    phase TEXT NOT NULL,
    checklist TEXT NOT NULL,             -- JSON array of checklist items
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, passed, failed
    reviewed_at TEXT,
    UNIQUE(feature_id, phase)
);

-- SDD Clarifications
CREATE TABLE IF NOT EXISTS sdd_clarifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL REFERENCES sdd_features(id),
    question TEXT NOT NULL,
    answer TEXT,
    status TEXT NOT NULL DEFAULT 'open',  -- open, resolved, skipped
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);
```

---

## 3. TÍCH HỢP VÀO BONBO WORKFLOW

### 3.1 Cách hoạt động thực tế

```
━━━ KỊCH BẢN 1: GREENFIELD PROJECT ━━━

User:> /speckit.init my-crypto-app
BonBo: ✅ SDD Session initialized for "my-crypto-app"
       📁 Created .speckit/ directory structure
       📋 Constitution not found — let's create one!

User:> /speckit.constitution Rust-first, SQLite for data, 
       Axum for API, testing mandatory, max 3 dependencies

BonBo: 📜 Constitution generated and saved!
       
       ═══ CONSTITUTION: my-crypto-app ═══
       Article I: Rust-First Development
       Article II: SQLite for Data Persistence
       Article III: Axum Web Framework
       Article IV: Mandatory Testing (TDD)
       Article V: Minimal Dependencies (≤3 major)
       
       ✅ All gates passed. Ready to specify.
       
       Next: /speckit.specify <what you want to build>

User:> /speckit.specify A crypto portfolio tracker that 
       shows real-time prices, tracks holdings, and sends 
       alerts when prices change significantly

BonBo: 📋 Feature Spec #001 generated!
       
       ═══ SPEC: 001-crypto-portfolio-tracker ═══
       
       ## User Stories
       US-1: As a crypto investor, I want to see real-time
             prices of my holdings...
       US-2: As a user, I want to add/remove coins from my
             portfolio...
       US-3: As a user, I want price alerts when change >5%
       
       ## ⚠️ Needs Clarification
       [NEEDS CLARIFICATION: Which price API? CoinGecko?]
       [NEEDS CLARIFICATION: Alert via what channel?]
       [NEEDS CLARIFICATION: "Significant" = what %?]
       
       Next: /speckit.clarify to resolve these

User:> CoinGecko API, Telegram alerts, 5% threshold

BonBo: ✅ Clarifications resolved!
       ✅ All [NEEDS CLARIFICATION] markers cleared.
       ✅ Spec review checklist: 5/5 items passed.
       
       Next: /speckit.plan

User:> /speckit.plan Axum backend, SQLite with rusqlite,
       CoinGecko API via reqwest, teloxide for Telegram bot

BonBo: 📐 Implementation Plan generated!
       
       ═══ PLAN: 001-crypto-portfolio-tracker ═══
       
       ## Constitutional Compliance
       ✅ Rust-First: All code in Rust
       ✅ SQLite: Using rusqlite (bundled)
       ✅ Axum: Web framework confirmed
       ✅ TDD: Tests before implementation
       ✅ ≤3 deps: rusqlite + reqwest + teloxide ✓
       
       ## Architecture
       [diagram here]
       
       ## Data Model
       CREATE TABLE holdings (...)
       CREATE TABLE price_history (...)
       CREATE TABLE alerts (...)
       
       Next: /speckit.tasks

BonBo: 📋 12 tasks generated across 4 phases!
       
       Phase 1: Foundation (4 tasks)
       - [ ] T1.1: Setup Cargo project [TEST-FIRST]
       - [ ] T1.2: SQLite schema + migrations [TEST-FIRST]
       - [ ] T1.3: CoinGecko API client [TEST-FIRST]
       - [ ] T1.4: Config management
       
       Phase 2: Core Features (4 tasks) [depends: Phase 1]
       - [ ] T2.1: Portfolio CRUD API [TEST-FIRST]
       - [ ] T2.2: Price fetcher cron [TEST-FIRST]
       - [ ] T2.3: Alert engine
       - [ ] T2.4: Telegram notifier
       
       Phase 3: API & Integration (2 tasks) [depends: Phase 2]
       - [ ] T3.1: REST API endpoints [TEST-FIRST]
       - [ ] T3.2: WebSocket for real-time prices
       
       Phase 4: Polish (2 tasks)
       - [ ] T4.1: Error handling hardening
       - [ ] T4.2: Docker setup
       
       Next: /speckit.implement

User:> /speckit.implement

BonBo: 🔨 Starting implementation...
       
       ▶ Phase 1: Foundation
       ▶ T1.1: Setup Cargo project [TEST-FIRST]
         ✅ Created tests/project_setup_test.rs
         ✅ Created src/main.rs, src/lib.rs
         ✅ Tests passing (3/3)
         
       ▶ T1.2: SQLite schema + migrations [TEST-FIRST]
         ✅ Created tests/db_test.rs
         ✅ Created src/db/schema.sql
         ✅ Tests passing (5/5)
         
       ... continuing through all tasks ...
```

### 3.2 Integration vào existing tools

```rust
// src/tools/definitions.rs — Thêm SDD tools

// Tool 1: /speckit.constitution
ToolDefinition {
    name: "speckit_constitution",
    description: "Create or update project constitution (immutable principles)",
    parameters: json!({
        "type": "object",
        "properties": {
            "principles": {
                "type": "string",
                "description": "Project principles: tech stack, testing, quality standards"
            }
        },
        "required": ["principles"]
    }),
}

// Tool 2: /speckit.specify  
ToolDefinition {
    name: "speckit_specify",
    description: "Create feature specification (WHAT & WHY, no tech stack)",
    parameters: json!({
        "type": "object",
        "properties": {
            "description": {
                "type": "string",
                "description": "What to build and why (user stories, experiences, outcomes)"
            }
        },
        "required": ["description"]
    }),
}

// Tool 3: /speckit.clarify
ToolDefinition {
    name: "speckit_clarify", 
    description: "Clarify ambiguities in spec before planning",
    parameters: json!({
        "type": "object",
        "properties": {
            "answers": {
                "type": "string",
                "description": "Answers to clarification questions"
            }
        }
    }),
}

// Tool 4: /speckit.plan
ToolDefinition {
    name: "speckit_plan",
    description: "Create technical implementation plan (HOW)",
    parameters: json!({
        "type": "object",
        "properties": {
            "tech_stack": {
                "type": "string",
                "description": "Tech stack, architecture, constraints"
            }
        },
        "required": ["tech_stack"]
    }),
}

// Tool 5: /speckit.tasks
ToolDefinition {
    name: "speckit_tasks",
    description: "Break down plan into actionable tasks",
    parameters: json!({
        "type": "object",
        "properties": {}
    }),
}

// Tool 6: /speckit.implement
ToolDefinition {
    name: "speckit_implement",
    description: "Execute all tasks from task breakdown (TDD)",
    parameters: json!({
        "type": "object",
        "properties": {
            "phase_filter": {
                "type": "string",
                "description": "Optional: implement only specific phase"
            }
        }
    }),
}

// Tool 7: /speckit.analyze
ToolDefinition {
    name: "speckit_analyze",
    description: "Cross-artifact consistency analysis",
    parameters: json!({
        "type": "object",
        "properties": {}
    }),
}

// Tool 8: /speckit.status
ToolDefinition {
    name: "speckit_status",
    description: "Show current SDD workflow status",
    parameters: json!({
        "type": "object",
        "properties": {}
    }),
}
```

---

## 4. DIRECTORY STRUCTURE MỚI

```
vybrid-rust/
├── src/
│   ├── main.rs                    # Updated: add SDD command dispatch
│   ├── sdd/                       # 🆕 SpecKit Engine
│   │   ├── mod.rs                 # SddEngine, SddPhase, types
│   │   ├── constitution.rs        # Constitution CRUD + validation
│   │   ├── phases.rs              # Phase execution logic
│   │   ├── templates.rs           # All SDD templates
│   │   ├── numbering.rs           # Feature auto-numbering
│   │   ├── gates.rs               # Phase gate checker
│   │   ├── clarify.rs             # Clarification manager
│   │   └── artifacts.rs           # File artifact management
│   ├── tools/
│   │   ├── definitions.rs         # Updated: add 8 SDD tools
│   │   ├── executor.rs            # Updated: dispatch SDD tools
│   │   └── speckit.rs             # 🆕 SDD tool implementations
│   └── ...
├── bonbo-km/
│   └── src/
│       └── schema.sql             # Updated: add sdd_* tables
├── templates/                     # 🆕 SDD Templates
│   ├── constitution.md
│   ├── spec.md
│   ├── plan.md
│   ├── tasks.md
│   └── clarify.md
└── ...
```

---

## 5. SYSTEM PROMPT ENHANCEMENT

### 5.1 SDD Context Injection vào System Prompt

```rust
// src/prompts.rs — Updated system prompt với SDD awareness

const SDD_SYSTEM_SECTION: &str = r#"

## 🏗️ Spec-Driven Development (SDD) Mode

You are operating in SDD mode, powered by GitHub Spec Kit methodology.

### Mandatory Workflow
When a user asks you to build something, ALWAYS follow this sequence:

1. **Constitution Check**: Load project constitution from knowledge base
   - If not found, prompt user to create one with /speckit.constitution
   
2. **Specify (WHAT & WHY)**: NEVER jump to code. First create a spec:
   - Use /speckit.specify to define user stories & acceptance criteria
   - Mark ALL ambiguities with [NEEDS CLARIFICATION: question]
   - Focus on WHAT users need, not HOW to build it
   
3. **Clarify**: Resolve all [NEEDS CLARIFICATION] markers
   - Do NOT proceed to planning until ALL markers are resolved
   - Use /speckit.clarify to track resolution
   
4. **Plan (HOW)**: Only after spec is complete:
   - Use /speckit.plan for tech stack, architecture, data models
   - Validate against constitution (compliance check)
   - Generate contracts, research notes
   
5. **Tasks**: Break plan into actionable items:
   - Use /speckit.tasks for structured task breakdown
   - Mark test tasks [TEST-FIRST] before implementation
   - Mark parallelizable tasks [P]
   
6. **Implement**: Execute tasks in order:
   - Use /speckit.implement
   - TDD: Write tests FIRST, then make them pass
   - Validate each task with checkpoint before next

### Quality Gates
Before advancing phases, verify:
- ✅ No [NEEDS CLARIFICATION] markers remain
- ✅ Constitutional compliance confirmed
- ✅ All acceptance criteria are testable
- ✅ Review checklist items checked off

### For Small Tasks (bug fixes, tweaks)
SDD mode can be lightweight:
- Quick spec: "Fix bug where X happens when Y"
- Skip formal plan/tasks if single fix
- Still validate fix against constitution
"#;
```

---

## 6. IMPLEMENTATION ROADMAP

### Phase 1: Foundation (1 tuần)
```
□ 1.1 Tạo src/sdd/mod.rs với types & SddEngine skeleton
□ 1.2 Tạo src/sdd/constitution.rs — CRUD qua knowledge base
□ 1.3 Tạo src/sdd/templates.rs — 5 templates (spec, plan, tasks, clarify, constitution)
□ 1.4 Thêm sdd_* tables vào bonbo-km/src/schema.sql
□ 1.5 Viết unit tests cho templates & constitution
```

### Phase 2: Tool Integration (1 tuần)
```
□ 2.1 Thêm 8 SDD tool definitions vào definitions.rs
□ 2.2 Tạo src/tools/speckit.rs — tool implementation dispatch
□ 2.3 Update executor.rs để dispatch SDD tools
□ 2.4 Update system prompt với SDD_SYSTEM_SECTION
□ 2.5 Test tool dispatch end-to-end
```

### Phase 3: Phase Logic (1 tuần)
```
□ 3.1 Implement phases.rs — 6 phase handlers
□ 3.2 Implement numbering.rs — auto feature numbering
□ 3.3 Implement gates.rs — phase gate validation
□ 3.4 Implement clarify.rs — clarification tracking
□ 3.5 Implement artifacts.rs — file creation & management
```

### Phase 4: Polish (1 tuần)
```
□ 4.1 CLI commands: /speckit.* slash command parsing
□ 4.2 Telegram bot SDD support
□ 4.3 Cross-artifact consistency analysis (/speckit.analyze)
□ 4.4 Status dashboard (/speckit.status)
□ 4.5 Integration tests cho full SDD workflow
```

---

## 7. SO SÁNH: BONBO TRƯỚC vs SAU SPEC KIT

| Khía cạnh | BonBo trước SpecKit | BonBo sau SpecKit |
|-----------|--------------------|--------------------|
| **Planning** | Không có | Constitution → Spec → Plan → Tasks |
| **Consistency** | Mỗi session khác nhau | Constitution đảm bảo consistency |
| **Quality** | Phụ thuộc prompt | Template-driven + Phase gates |
| **Clarity** | Guess ambiguities | [NEEDS CLARIFICATION] markers |
| **Testing** | Optional | TDD mandatory (tests first) |
| **Traceability** | Không có | Feature numbered artifacts |
| **Rework rate** | Cao (guessing) | Thấp (clarified upfront) |
| **Documentation** | Manual | Auto-generated với specs |
| **Onboarding** | Đọc code | Đọc spec (higher level) |

---

## 8. RISKS & MITIGATIONS

| Risk | Impact | Mitigation |
|------|--------|------------|
| Overhead for small tasks | Developers bỏ qua SDD | Lightweight mode cho bugs, mandatory cho features |
| Template rigidity | Không fit mọi use case | Customizable templates + override mechanism |
| Knowledge base bloat | SQLite quá lớn | Auto-cleanup old specs, archive completed features |
| LLM ignores workflow | Agent skip phases | Gate enforcement trong code, không cho skip |
| Learning curve | Users unfamiliar with SDD | Interactive tutorial mode, /speckit.status helper |

---

*Tài liệu thiết kế này là blueprint cho việc tích hợp SpecKit SDD workflow vào BonBo coding agent.*
