# 🧠 ĐỀ XUẤT KIẾN TRÚC: MODULE QUẢN LÝ TRI THỨC CHO AI AGENT BONBO

> **Ngày:** Tháng 7, 2025  
> **Mục tiêu:** Thiết kế module Knowledge Management (KM) cho AI agent BonBo  
> **Kết luận chính:** 🏆 **SQLite làm lõi + Markdown làm giao diện người đọc = Kiến trúc Hybrid tối ưu**

---

## 📋 MỤC LỤC

1. [Phân tích yêu cầu AI Agent BonBo](#1-phân-tích-yêu-cầu)
2. [Câu trả lời nhanh: Dùng phương pháp nào?](#2-câu-trả-lời-nhanh)
3. [Kiến trúc 3 lớp bộ nhớ cho BonBo](#3-kiến-trúc-3-lớp-bộ-nhớ)
4. [Schema SQLite chi tiết](#4-schema-sqlite-chi-tiết)
5. [Tại sao KHÔNG chỉ dùng Obsidian/Markdown?](#5-tại-sao-không-chỉ-dùng-obsidian)
6. [Tại sao KHÔNG chỉ dùng SQLite?](#6-tại-sao-không-chỉ-dùng-sqlite)
7. [Kiến trúc Hybrid đề xuất](#7-kiến-trúc-hybrid-đề-xuất)
8. [Ví dụ triển khai code](#8-ví-d-triển-khai-code)
9. [So sánh 3 phương án](#9-so-sánh-3-phương-án)
10. [Lộ trình triển khai](#10-lộ-trình-triển-khai)

---

## 1. PHÂN TÍCH YÊU CẦU AI AGENT BONBO

### BonBo cần gì từ module Knowledge Management?

```
🤖 AI Agent BonBo — Yêu cầu Knowledge Management:

┌─────────────────────────────────────────────────────────────┐
│ 1. 📝 GHI NHỚ NGỮ CẢNH (Episodic Memory)                    │
│    → Nhớ các cuộc hội thoại, task đã làm, quyết định         │
│    → "Hôm qua tôi đã refactor module X, thay đổi gì?"        │
│                                                               │
│ 2. 📚 TRI THỨC DỰ ÁN (Semantic Memory)                      │
│    → Lưu trữ kiến thức về codebase, kiến trúc, quyết định    │
│    → "Tại sao dùng Rust thay vì Go cho module này?"          │
│    → "Cấu trúc thư mục dự án ABC như thế nào?"               │
│                                                               │
│ 3. ⚡ KỸ NĂNG THỰC HÀNH (Procedural Memory)                  │
│    → Workflow đã học: deploy, test, debug pattern             │
│    → "Khi gặp lỗi Rust borrow checker, làm gì?"              │
│                                                               │
│ 4. 🔍 TÌM KIẾM THÔNG MINH (Smart Retrieval)                 │
│    → Tìm kiếm ngữ nghĩa (không chỉ keyword)                  │
│    → "Tìm tất cả code liên quan đến authentication"          │
│                                                               │
│ 5. 🔗 LIÊN KẾT TRI THỨC (Knowledge Graph)                   │
│    → Module A phụ thuộc Module B → Tác động thay đổi?        │
│    → "Ai maintain module X? Lần cuối sửa khi nào?"           │
│                                                               │
│ 6. 📊 DỮ LIỆU CẤU TRÚC (Structured Data)                    │
│    → Tasks, activity logs, metrics, project metadata         │
│    → "Thống kê số task hoàn thành tuần này"                   │
│                                                               │
│ 7. 👤 QUAN SÁT ĐỌC ĐƯỢC (Human-Readable Observability)      │
│    → Người dùng có thể xem BonBo đang nhớ gì                 │
│    → "Cho tôi xem knowledge base của bạn về dự án X"         │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. CÂU TRẢ LỜI NHANH

### 🏆 ĐÁP ÁN: SQLITE LÀM LÕI + MARKDOWN LÀM GIAO DIỆN

```
┌──────────────────────────────────────────────────┐
│         KIẾN TRÚC ĐỀ XUẤT CHO BONBO             │
│                                                   │
│   📁 Markdown Layer (Human-Readable)             │
│   ├── docs/, tasks/, README — người đọc          │
│   │   ↕ Đồng bộ 2 chiều                          │
│   🗄️ SQLite Layer (Machine-Optimized)            │
│   ├── Knowledge, Memory, Tasks, Embeddings       │
│   │   ↕ Query                                     │
│   🧠 Embedding Layer (Vector Search)             │
│   └── sqlite-vec + FTS5 (Hybrid Search)          │
└──────────────────────────────────────────────────┘
```

**Tại sao?**

| Yêu cầu BonBo | Giải pháp |
|---|---|
| Ghi nhớ ngữ cảnh cuộc hội thoại | ✅ **SQLite** — Episodic Memory table |
| Lưu tri thức dự án | ✅ **SQLite** — Semantic Memory table |
| Tìm kiếm ngữ nghĩa (semantic) | ✅ **SQLite + sqlite-vec** — Vector search |
| Tìm kiếm full-text (keyword) | ✅ **SQLite FTS5** — Full-text search |
| Dữ liệu có cấu trúc (tasks, logs) | ✅ **SQLite** — Relational tables |
| Liên kết tri thức (knowledge graph) | ✅ **SQLite** — Relations table + recursive CTE |
| Người dùng đọc được | ✅ **Markdown files** — Synced from SQLite |
| ACID, transaction, rollback | ✅ **SQLite WAL mode** |
| Hiệu suất cao, không server | ✅ **SQLite** — Embedded, zero-config |
| Miễn phí, local-first | ✅ Cả hai đều miễn phí |

---

## 3. KIẾN TRÚC 3 LỚP BỘ NHỚ CHO BONBO

### Theo nghiên cứu Cognitive Science, AI Agent cần 3 loại bộ nhớ:

```
┌─────────────────────────────────────────────────────────────────┐
│                    BONBO MEMORY ARCHITECTURE                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  💬 LAYER 1: WORKING MEMORY (Bộ nhớ làm việc)                   │
│  ├── Context window hiện tại (session)                           │
│  ├── Tạm thời, mất khi kết thúc session                         │
│  ├── Dung lượng: ~128K tokens                                    │
│  └── Lưu trữ: In-memory (RAM)                                    │
│       ↓ Consolidate khi kết thúc session                         │
│                                                                  │
│  🧠 LAYER 2: LONG-TERM MEMORY (Bộ nhớ dài hạn)                  │
│  ├── 📚 Semantic Memory — Tri thức, facts, kiến thức            │
│  │   "Dự án ABC dùng Rust + Tokio + PostgreSQL"                  │
│  │   "Module auth nằm ở src/auth/, dùng JWT"                     │
│  │                                                                │
│  ├── 📅 Episodic Memory — Sự kiện, interactions                 │
│  │   "15/07/2025: User yêu cầu thêm feature login"               │
│  │   "16/07/2025: Hoàn thành refactor module X"                  │
│  │                                                                │
│  ├── ⚡ Procedural Memory — Kỹ năng, workflows                  │
│  │   "Workflow deploy: test → build → deploy → verify"           │
│  │   "Khi gặp lỗi E0597 Rust → check lifetime annotations"       │
│  │                                                                │
│  └── 💾 Lưu trữ: SQLite (bonbo_knowledge.db)                     │
│       ↕                                                           │
│  📄 LAYER 3: HUMAN INTERFACE (Giao diện người đọc)               │
│  ├── Markdown files — Người dùng có thể xem/sửa                  │
│  ├── tasks/todo.md, docs/activity.md, docs/*.md                  │
│  └── Đồng bộ 2 chiều với SQLite                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. SCHEMA SQLITE CHI TIẾT

```sql
-- ============================================================
-- BONBO KNOWLEDGE BASE SCHEMA
-- Engine: SQLite with WAL mode + FTS5 + sqlite-vec
-- ============================================================

PRAGMA journal_mode=WAL;        -- Concurrent reads + safe writes
PRAGMA foreign_keys=ON;         -- Enforce relationships
PRAGMA busy_timeout=5000;       -- Wait up to 5s on lock

-- ============================================================
-- 1. SEMANTIC MEMORY — Tri thức, facts, kiến thức dự án
-- ============================================================
CREATE TABLE knowledge (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,                  -- Tiêu đề
    content     TEXT NOT NULL,                  -- Nội dung (Markdown)
    category    TEXT NOT NULL,                  -- 'architecture', 'decision', 'pattern', 'api', 'config'
    project     TEXT,                           -- Tên dự án
    tags        TEXT,                           -- JSON array: ["rust", "auth", "jwt"]
    embedding   BLOB,                           -- Vector embedding (768-dim float32)
    confidence  REAL DEFAULT 1.0,               -- Độ tin cậy (0.0 - 1.0)
    source      TEXT DEFAULT 'agent',           -- 'agent', 'user', 'documentation', 'web'
    source_url  TEXT,                           -- URL nguồn (nếu có)
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    accessed_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- Lần truy cập cuối
    access_count INTEGER DEFAULT 0              -- Số lần truy cập
);

-- Full-text search cho knowledge
CREATE VIRTUAL TABLE knowledge_fts USING fts5(
    title, content, tags, category,
    content=knowledge, content_rowid=id
);

-- Index cho truy vấn phổ biến
CREATE INDEX idx_knowledge_category ON knowledge(category);
CREATE INDEX idx_knowledge_project ON knowledge(project);
CREATE INDEX idx_knowledge_confidence ON knowledge(confidence DESC);
CREATE INDEX idx_knowledge_accessed ON knowledge(accessed_at DESC);

-- ============================================================
-- 2. EPISODIC MEMORY — Sự kiện, interactions, conversations
-- ============================================================
CREATE TABLE episodes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL,                  -- ID phiên làm việc
    timestamp   DATETIME DEFAULT CURRENT_TIMESTAMP,
    event_type  TEXT NOT NULL,                  -- 'user_query', 'agent_action', 'task_complete', 'error', 'decision'
    actor       TEXT NOT NULL DEFAULT 'bonbo',  -- 'bonbo' hoặc 'user'
    content     TEXT NOT NULL,                  -- Nội dung sự kiện
    summary     TEXT,                           -- Tóm tắt (do LLM generate)
    embedding   BLOB,                           -- Vector embedding
    metadata    TEXT,                           -- JSON: {tool: 'bash', exit_code: 0, duration_ms: 1500}
    project     TEXT                            -- Dự án liên quan
);

CREATE INDEX idx_episodes_session ON episodes(session_id);
CREATE INDEX idx_episodes_type ON episodes(event_type);
CREATE INDEX idx_episodes_timestamp ON episodes(timestamp DESC);
CREATE INDEX idx_episodes_project ON episodes(project);

-- ============================================================
-- 3. PROCEDURAL MEMORY — Kỹ năng, workflows, patterns
-- ============================================================
CREATE TABLE procedures (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,           -- Tên workflow: "deploy_to_production"
    description TEXT NOT NULL,                  -- Mô tả
    steps       TEXT NOT NULL,                  -- JSON array of steps
    triggers    TEXT,                           -- Khi nào dùng: ["deploy", "release"]
    prerequisites TEXT,                         -- JSON: yêu cầu trước khi chạy
    success_rate REAL DEFAULT 0.0,              -- Tỷ lệ thành công (học từ experience)
    last_used   DATETIME,                       -- Lần sử dụng cuối
    use_count   INTEGER DEFAULT 0               -- Số lần sử dụng
);

-- ============================================================
-- 4. KNOWLEDGE GRAPH — Liên kết tri thức
-- ============================================================
CREATE TABLE relations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type TEXT NOT NULL,                  -- 'knowledge', 'episode', 'procedure'
    source_id   INTEGER NOT NULL,
    target_type TEXT NOT NULL,
    target_id   INTEGER NOT NULL,
    relation    TEXT NOT NULL,                  -- 'depends_on', 'related_to', 'caused', 'solves', 'supersedes'
    weight      REAL DEFAULT 1.0,              -- Độ mạnh liên kết
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (source_id) REFERENCES knowledge(id) ON DELETE CASCADE
);

CREATE INDEX idx_relations_source ON relations(source_type, source_id);
CREATE INDEX idx_relations_target ON relations(target_type, target_id);
CREATE INDEX idx_relations_type ON relations(relation);

-- ============================================================
-- 5. TASKS — Quản lý công việc
-- ============================================================
CREATE TABLE tasks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,
    description TEXT,
    status      TEXT DEFAULT 'pending',         -- 'pending', 'in_progress', 'completed', 'blocked'
    priority    TEXT DEFAULT 'medium',          -- 'low', 'medium', 'high', 'critical'
    project     TEXT,
    assignee    TEXT DEFAULT 'bonbo',
    due_date    DATETIME,
    completed_at DATETIME,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    embedding   BLOB                            -- Vector để tìm task tương tự
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_project ON tasks(project);
CREATE INDEX idx_tasks_priority ON tasks(priority);

-- ============================================================
-- 6. FILE SYNC — Đồng bộ với Markdown files
-- ============================================================
CREATE TABLE file_sync (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path   TEXT NOT NULL UNIQUE,           -- Đường dẫn file markdown
    db_table    TEXT NOT NULL,                  -- 'knowledge', 'tasks', 'episodes'
    db_id       INTEGER NOT NULL,               -- ID trong table
    checksum    TEXT,                           -- MD5 checksum để detect changes
    last_synced DATETIME DEFAULT CURRENT_TIMESTAMP,
    sync_direction TEXT DEFAULT 'db_to_file'    -- 'db_to_file', 'file_to_db', 'bidirectional'
);

-- ============================================================
-- 7. USER PREFERENCES — Cá nhân hóa
-- ============================================================
CREATE TABLE preferences (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,                  -- JSON value
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TRIGGERS — Tự động cập nhật
-- ============================================================

-- Auto-update FTS khi knowledge thay đổi
CREATE TRIGGER knowledge_ai AFTER INSERT ON knowledge BEGIN
    INSERT INTO knowledge_fts(rowid, title, content, tags, category)
    VALUES (new.id, new.title, new.content, new.tags, new.category);
END;

CREATE TRIGGER knowledge_au AFTER UPDATE ON knowledge BEGIN
    INSERT INTO knowledge_fts(knowledge_fts, rowid, title, content, tags, category)
    VALUES ('delete', old.id, old.title, old.content, old.tags, old.category);
    INSERT INTO knowledge_fts(rowid, title, content, tags, category)
    VALUES (new.id, new.title, new.content, new.tags, new.category);
END;

CREATE TRIGGER knowledge_ad AFTER DELETE ON knowledge BEGIN
    INSERT INTO knowledge_fts(knowledge_fts, rowid, title, content, tags, category)
    VALUES ('delete', old.id, old.title, old.content, old.tags, old.category);
END;

-- Auto-update timestamp
CREATE TRIGGER knowledge_update_time AFTER UPDATE ON knowledge BEGIN
    UPDATE knowledge SET updated_at = CURRENT_TIMESTAMP WHERE id = new.id;
END;
```

---

## 5. TẠI SAO KHÔNG CHỈ DÙNG OBSIDIAN/MARKDOWN?

### ❌ Markdown thuần không đủ cho AI Agent

| Vấn đề | Giải thích |
|---|---|
| **Không truy vấn được** | "Tìm tất cả kiến thức liên quan đến Rust auth với confidence > 0.8" — **Không thể** |
| **Không có vector search** | Tìm kiếm ngữ nghĩa (semantic search) — **Không thể** |
| **Không có relations** | "Module nào phụ thuộc Module A?" — **Chỉ backlinks, rất chậm** |
| **Không có transaction** | Thêm knowledge + relation atomically — **Không thể** |
| **Không có aggregation** | "Thống kê số task hoàn thành tuần này" — **Không thể** |
| **Chậm với dữ liệu lớn** | 10,000+ notes — **Graph view chậm, indexing chậm** |
| **Không có embedding** | Lưu vector embedding cho semantic search — **Không hỗ trợ** |

### ❌ Obsidian là công cụ cho CON NGƯỜI, không phải cho AI Agent

```
Obsidian = UI/UX cho con người suy nghĩ
SQLite   = Engine cho máy tính truy vấn

AI Agent BonBo cần:
  → Truy vấn nhanh (sub-ms)
  → Semantic search (vector)
  → Relations + Graph traversal
  → Structured data + Aggregation
  → ACID transactions
  → Embedding storage
  = SQLITE là đáp án chính xác!
```

---

## 6. TẠI SAO KHÔNG CHỈ DÙNG SQLITE?

### ❌ SQLite thuần thiếu giao diện người đọc

| Vấn đề | Giải thích |
|---|---|
| **Binary format** | Người dùng không thể mở .db và đọc trực tiếp |
| **Không có UI** | Cần tool (DB Browser) hoặc code để xem |
| **Git-unfriendly** | Không diff được thay đổi |
| **Không trực quan** | Không có Graph view, Canvas |

### ✅ Giải pháp: SQLite + Markdown Sync Layer

```
SQLite (machine)  ←sync→  Markdown (human)
    ↓                          ↓
  Query fast              Read easy
  Vector search           Git version control  
  Relations               Human observable
  ACID                    Direct editing
```

---

## 7. KIẾN TRÚC HYBRID ĐỀ XUẤT

```
┌─────────────────────────────────────────────────────────────┐
│                    BONBO KM ARCHITECTURE                      │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              📄 MARKDOWN INTERFACE LAYER                │ │
│  │  ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌────────────┐  │ │
│  │  │todo.md  │ │activity │ │PROJECT_  │ │Knowledge/  │  │ │
│  │  │(Tasks)  │ │.md      │ │README.md │ │*.md files  │  │ │
│  │  └────┬────┘ └────┬────┘ └────┬─────┘ └─────┬──────┘  │ │
│  │       │           │           │              │          │ │
│  │       └───────────┴─────┬─────┴──────────────┘          │ │
│  │                         │ (2-way sync)                   │ │
│  └─────────────────────────┼───────────────────────────────┘ │
│                            │                                  │
│  ┌─────────────────────────┼───────────────────────────────┐ │
│  │              🗄️ SQLITE KNOWLEDGE ENGINE                 │ │
│  │                         │                                │ │
│  │  ┌──────────┐ ┌────────┴──┐ ┌──────────┐ ┌──────────┐ │ │
│  │  │knowledge │ │ episodes  │ │procedures│ │  tasks    │ │ │
│  │  │(facts)   │ │(events)   │ │(skills)  │ │(work)    │ │ │
│  │  └──────────┘ └───────────┘ └──────────┘ └──────────┘ │ │
│  │  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌──────────┐ │ │
│  │  │relations │ │preferences│ │file_sync │ │ knowledge│ │ │
│  │  │(graph)   │ │(config)   │ │(sync)    │ │  _fts    │ │ │
│  │  └──────────┘ └───────────┘ └──────────┘ └──────────┘ │ │
│  │                                                         │ │
│  │  ┌─────────────────────────────────────────────────────┐│ │
│  │  │  🔍 HYBRID SEARCH ENGINE                            ││ │
│  │  │  ├── FTS5: Full-text keyword search (BM25)          ││ │
│  │  │  ├── sqlite-vec: Vector similarity search (ANN)     ││ │
│  │  │  └── Reciprocal Rank Fusion (RRF): Merge results    ││ │
│  │  └─────────────────────────────────────────────────────┘│ │
│  │                                                         │ │
│  │  PRAGMA journal_mode=WAL;  -- Concurrent + safe         │ │
│  │  File: bonbo_knowledge.db (single file, portable)       │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              🧠 EMBEDDING PIPELINE                       │ │
│  │  Text → Embedding Model → Vector (768-dim) → SQLite    │ │
│  │  (Sử dụng: OpenAI ada-002, hoặc local model)            │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 8. VÍ DỤ TRIỂN KHAI CODE

### Rust Implementation (vì BonBo là Rust expert!):

```rust
// knowledge/mod.rs — BonBo Knowledge Management Module

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};

/// BonBo Knowledge Manager
pub struct KnowledgeManager {
    conn: Connection,
}

/// Một entry tri thức
#[derive(Debug, Serialize, Deserialize)]
pub struct Knowledge {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub source: String,
}

/// Kết quả tìm kiếm hybrid
#[derive(Debug)]
pub struct SearchResult {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub score: f64,          // Combined score (RRF)
    pub keyword_score: f64,  // FTS5 BM25
    pub vector_score: f64,   // Cosine similarity
}

impl KnowledgeManager {
    /// Khởi tạo Knowledge Manager
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        ).context("Failed to open knowledge database")?;

        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let km = Self { conn };
        km.init_schema()?;
        Ok(km)
    }

    /// Khởi tạo database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(include_str!("schema.sql"))?;
        Ok(())
    }

    /// Thêm tri thức mới
    pub fn add_knowledge(&self, knowledge: &Knowledge) -> Result<i64> {
        let tags_json = serde_json::to_string(&knowledge.tags)?;
        
        self.conn.execute(
            "INSERT INTO knowledge (title, content, category, project, tags, confidence, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                knowledge.title,
                knowledge.content,
                knowledge.category,
                knowledge.project,
                tags_json,
                knowledge.confidence,
                knowledge.source,
            ],
        ).context("Failed to insert knowledge")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Tìm kiếm hybrid: Keyword (FTS5) + Vector (sqlite-vec)
    pub fn search(&self, query: &str, query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        // Step 1: Keyword search via FTS5 (BM25 ranking)
        let mut keyword_stmt = self.conn.prepare(
            "SELECT k.id, k.title, k.content, bm25(knowledge_fts) as score
             FROM knowledge_fts f
             JOIN knowledge k ON k.id = f.rowid
             WHERE knowledge_fts MATCH ?1
             ORDER BY score DESC
             LIMIT ?2"
        )?;

        let keyword_results: Vec<(i64, String, String, f64)> = keyword_stmt
            .query_map(params![query, limit * 2], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Step 2: Vector similarity search (simplified - use sqlite-vec in production)
        // TODO: Implement with sqlite-vec extension for production

        // Step 3: Reciprocal Rank Fusion (merge keyword + vector results)
        let mut results = Vec::new();
        for (rank, (id, title, content, kscore)) in keyword_results.iter().enumerate() {
            results.push(SearchResult {
                id: *id,
                title: title.clone(),
                content: content.clone(),
                score: 1.0 / (rank as f64 + 60.0), // RRF formula
                keyword_score: *kscore,
                vector_score: 0.0, // Will be merged with vector results
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    /// Ghi lại sự kiện (Episodic Memory)
    pub fn log_episode(&self, session_id: &str, event_type: &str, content: &str, summary: Option<&str>) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO episodes (session_id, event_type, content, summary)
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, event_type, content, summary],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Lấy ngữ cảnh liên quan cho câu query (RAG-style)
    pub fn get_relevant_context(&self, query: &str, limit: usize) -> Result<Vec<Knowledge>> {
        let mut stmt = self.conn.prepare(
            "SELECT k.* FROM knowledge k
             JOIN knowledge_fts f ON f.rowid = k.id
             WHERE knowledge_fts MATCH ?1
             ORDER BY k.confidence DESC, k.accessed_at DESC
             LIMIT ?2"
        )?;

        let results = stmt.query_map(params![query, limit], |row| {
            Ok(Knowledge {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                category: row.get(3)?,
                project: row.get(4)?,
                tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                confidence: row.get(6)?,
                source: row.get(7)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        Ok(results)
    }

    /// Đồng bộ knowledge ra Markdown file (cho người đọc)
    pub fn sync_to_markdown(&self, output_dir: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT title, content, category, tags, project, updated_at
             FROM knowledge ORDER BY category, title"
        )?;

        let entries: Vec<(String, String, String, String, Option<String>, String)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (title, content, category, tags, project, updated) in entries {
            let dir = format!("{}/{}", output_dir, category);
            std::fs::create_dir_all(&dir)?;
            
            let md_content = format!(
                "---\ntitle: {}\ncategory: {}\ntags: {}\nproject: {}\nupdated: {}\n---\n\n{}",
                title, category, tags, project.unwrap_or_default(), updated, content
            );
            
            let safe_title = title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            let file_path = format!("{}/{}.md", dir, safe_title);
            std::fs::write(&file_path, md_content)?;
        }

        Ok(())
    }
}
```

---

## 9. SO SÁNH 3 PHƯƠNG ÁN

| Tiêu chí | 🔮 Markdown thuần | 🗄️ SQLite thuần | 🏆 **Hybrid (Đề xuất)** |
|---|---|---|---|
| **Tìm kiếm keyword** | ⭐⭐ | ⭐⭐⭐⭐⭐ (FTS5) | ⭐⭐⭐⭐⭐ |
| **Tìm kiếm semantic** | ❌ | ⭐⭐⭐⭐ (sqlite-vec) | ⭐⭐⭐⭐⭐ |
| **Relations/Graph** | ⭐⭐ (backlinks) | ⭐⭐⭐⭐ (FK+CTE) | ⭐⭐⭐⭐⭐ |
| **Structured data** | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Aggregation/Stats** | ❌ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **ACID/Transaction** | ❌ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Human readable** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Git friendly** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Tốc độ query** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Embedding storage** | ❌ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Tổng điểm** | **17/50** | **40/50** | **🏆 49/50** |

---

## 10. LỘ TRÌNH TRIỂN KHAI

### Phase 1: MVP (1-2 tuần)
```
✅ Thiết lập SQLite schema cơ bản
✅ Implement KnowledgeManager struct (Rust)
✅ FTS5 full-text search
✅ Add/query knowledge API
✅ Sync ra Markdown files
```

### Phase 2: Semantic Search (2-3 tuần)
```
⬜ Tích hợp sqlite-vec extension
⬜ Embedding pipeline (OpenAI hoặc local)
⬜ Hybrid search: FTS5 + Vector + RRF
⬜ Auto-embed khi add knowledge mới
```

### Phase 3: Knowledge Graph (1-2 tuần)
```
⬜ Relations table
⬜ Graph traversal queries
⬜ Auto-suggest relations
⬜ Visual export (Mermaid diagram)
```

### Phase 4: Advanced Features (2-4 tuần)
```
⬜ 2-way Markdown sync (đọc thay đổi từ file)
⬜ Memory consolidation (episodic → semantic)
⬜ Procedural memory (workflow learning)
⬜ RAG integration (retrieve context for LLM calls)
⬜ Memory decay (auto-forget old/irrelevant)
```

---

## 🏆 KẾT LUẬN

> **Dùng SQLite làm lõi + Markdown làm giao diện.**
> 
> Obsidian/Markdown là **công cụ suy nghĩ cho con người** — tuyệt vời nhưng không đủ cho AI agent cần truy vấn nhanh, semantic search, relations, và structured data.
> 
> SQLite là **engine lưu trữ cho máy** — vượt trội về query, ACID, embedding, FTS5 — nhưng cần lớp Markdown để người dùng có thể quan sát và tương tác.
> 
> **Kiến trúc Hybrid = Cả hai thế giới: Machine-efficient + Human-readable.**

---

*Nghiên cứu dựa trên: sqlite.org/whentouse.html, SparkCo AI Agent Memory Evaluation 2025, sqlite-memory extension, Cognitive Science 3-layer memory model*
