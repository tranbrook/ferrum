# ⚔️ SO SÁNH: OBSIDIAN vs SQLITE — Phương pháp Quản lý Dữ liệu Cá nhân

> **Ngày:** Tháng 7, 2025  
> **Mục tiêu:** So sánh chuyên sâu 2 phương pháp quản lý dữ liệu cá nhân截然 khác nhau: **Obsidian** (Markdown file-based) vs **SQLite** (Relational Database)

---

## 📋 MỤC LỤC

1. [Bản chất hai hệ thống](#1-bản-chất-hai-hệ-thống)
2. [Kiến trúc & Mô hình dữ liệu](#2-kiến-trúc--mô-hình-dữ-liệu)
3. [So sánh chi tiết từng tiêu chí](#3-so-sánh-chi-tiết-từng-tiêu-chí)
4. [Ưu/nhược điểm từng hệ thống](#4-ưunhược-điểm-từng-hệ-thống)
5. [Trường hợp sử dụng tốt nhất](#5-trường-hợp-sử-dụng-tốt-nhất)
6. [Khi nào kết hợp cả hai?](#6-khi-nào-kết-hợp-cả-hai)
7. [Kết luận & Khuyến nghị](#7-kết-luận--khuyến-nghị)

---

## 1. BẢN CHẤT HAI HỆ THỐNG

### 🔮 Obsidian — "Mở file, viết, liên kết"
```
Bản chất: Ứng dụng ghi chú / Knowledge Management
Mô hình: Flat Markdown Files (File phẳng)
Triết lý: "Suy nghĩ của bạn là của bạn"
Đối tượng: Con người ghi chú, liên kết ý tưởng
```

### 🗄️ SQLite — "Cơ sở dữ liệu quan hệ siêu nhẹ"
```
Bản chất: Thư viện cơ sở dữ liệu quan hệ (RDBMS)
Mô hình: Single-file Relational Database
Triết lý: "Small. Fast. Reliable. Choose any three."
Đối tượng: Lập trình viên, ứng dụng, phân tích dữ liệu
```

> **⚡ Nhận định quan trọng:** Obsidian và SQLite **không cạnh tranh trực tiếp** — chúng giải quyết **hai bài toán khác nhau**. Obsidian cạnh tranh với `fopen()` (file văn bản), còn SQLite cũng cạnh tranh với `fopen()` nhưng theo hướng **structured data**. Tuy nhiên, cả hai đều là **phương pháp quản lý dữ liệu cá nhân** cục bộ (local-first), nên việc so sánh cách tiếp cận là rất giá trị.

---

## 2. KIẾN TRÚC & MÔ HÌNH DỮ LIỆU

### 📁 Obsidian — File Markdown phân tán

```
📁 My Vault/
├── 📁 Daily Notes/
│   ├── 2025-07-01.md
│   ├── 2025-07-02.md
│   └── 2025-07-03.md
├── 📁 Projects/
│   ├── Project Alpha.md
│   └── Project Beta.md
├── 📁 Ideas/
│   ├── Machine Learning.md
│   └── Recipe Ideas.md
├── 📄 MOC - Home.md          (Map of Content)
├── 🎨 Brainstorm.canvas       (JSON Canvas)
└── ⚙️ .obsidian/
    ├── app.json
    ├── appearance.json
    └── plugins/
```

**Đặc điểm:**
- Mỗi ghi chú = **1 file Markdown (.md) riêng biệt**
- Liên kết qua `[[wikilink]]` hoặc `[markdown link](path)`
- Metadata qua **YAML frontmatter** (key-value)
- Canvas lưu dạng **JSON Canvas** (mở)
- Dữ liệu **phi cấu trúc** → bán cấu trúc (với frontmatter)

### 🗄️ SQLite — Cơ sở dữ liệu quan hệ tập trung

```sql
-- Tất cả dữ liệu nằm trong 1 file: mydata.db

CREATE TABLE notes (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    tags TEXT,            -- hoặc bảng riêng
    folder_id INTEGER REFERENCES folders(id)
);

CREATE TABLE links (
    source_id INTEGER REFERENCES notes(id),
    target_id INTEGER REFERENCES notes(id),
    link_type TEXT,
    created_at DATETIME
);

CREATE TABLE tags (
    note_id INTEGER REFERENCES notes(id),
    tag TEXT
);

-- Truy vấn: Tất cả notes liên kết với "Machine Learning"
SELECT n2.title, n2.content
FROM notes n1
JOIN links l ON l.source_id = n1.id
JOIN notes n2 ON l.target_id = n2.id
WHERE n1.title = 'Machine Learning';
```

**Đặc điểm:**
- Tất cả dữ liệu = **1 file database (.db, .sqlite) duy nhất**
- Dữ liệu **có cấu trúc chặt chẽ** (schema)
- Quan hệ được định nghĩa qua **Foreign Keys**
- Truy vấn bằng **SQL** — ngôn ngữ chuẩn hóa

---

## 3. SO SÁNH CHI TIẾT TỪNG TIÊU CHÍ

### 3.1 📊 BẢNG TỔNG HỢP

| Tiêu chí | 🔮 Obsidian (Markdown) | 🗄️ SQLite (Database) |
|---|---|---|
| **Loại dữ liệu** | Phi cấu trúc (văn bản tự do) | Có cấu trúc (bảng, hàng, cột) |
| **Định dạng file** | `.md` (Markdown), `.canvas` (JSON) | `.db`, `.sqlite` (Binary) |
| **Số lượng file** | Nhiều file (mỗi note = 1 file) | 1 file duy nhất |
| **Cách lưu trữ** | Thư mục/file hệ điều hành | Single-file database |
| **Đọc/ghi** | Bất kỳ text editor nào | Cần thư viện SQLite hoặc工具 |
| **Kích thước** | Mỗi file nhỏ (vài KB) | Tổng hợp trong 1 file (lên đến 281 TB) |
| **Mô hình quan hệ** | Wikilinks, Backlinks (adjacency) | Foreign Keys, JOIN, Relations |
| **Truy vấn** | Tìm kiếm text, backlinks, graph | SQL (SELECT, JOIN, WHERE...) |
| **Schema** | Không có (flexible) | Chặt chẽ (CREATE TABLE) |
| **Tốc độ đọc/ghi** | Chậm hơn với lượng lớn file | **Nhanh hơn filesystem** (theo SQLite.org) |
| **Đồng thời (Concurrency)** | Không giới hạn (mỗi file riêng) | Đọc: không giới hạn; Viết: **1 writer** tại một thời điểm |
| **Tính toán/Phân tích** | Hạn chế (cần plugin) | **Mạnh mẽ** (SQL aggregations, GROUP BY) |
| **Backup** | Copy thư mục, Git | Copy file .db |
| **Khả năng đọc bởi con người** | ✅ **Hoàn hảo** — Plaintext | ❌ Binary — cần tool |
| **Đường học tập** | Thấp — Markdown dễ học | **Cao** — Cần biết SQL |
| **Tùy biến giao diện** | ✅ Themes, CSS, plugins | ❌ Không có UI sẵn |
| **Đa nền tảng** | ✅ macOS, Windows, Linux, iOS, Android | ✅ Mọi nền tảng (library) |
| **Offline** | ✅ Hoàn hảo | ✅ Hoàn hảo |
| **Mã hóa** | Tùy chọn (via plugin/OS) | Mã hóa extension (SEE, SQLCipher) |
| **Giá** | **Miễn phí** (core) | **Miễn phí** (Public Domain) |
| **License** | Proprietary (app) + open format | **Public Domain** (code) |

---

### 3.2 🔍 PHÂN TÍCH CHI TIẾT

#### A. KHẢ NĂNG ĐỌC BỞI CON NGƯỜI (Human Readability)

| | Obsidian | SQLite |
|---|---|---|
| **Đọc trực tiếp** | ✅ Mở bằng Notepad, VS Code, bất kỳ editor | ❌ Binary, cần SQLite Browser/tool |
| **Tính portable** | ✅ Gửi email 1 file .md | ⚠️ Gửi nguyên database hoặc export |
| **Granhular access** | ✅ Mở từng note riêng | ⚠️ Cần query để trích xuất |
| **Longevity** | ✅ Markdown sẽ tồn tại hàng thập kỷ | ⚠️ SQLite format ổn định nhưng là binary |

> **🏆 Thắng: Obsidian** — Văn bản thô (plaintext) là định dạng bền vững nhất. File Markdown có thể đọc bởi bất kỳ hệ điều hành nào trong 50 năm tới.

#### B. HIỆU SUẤT & QUẢN LÝ DỮ LIỆU LỚN

| Khối lượng | Obsidian | SQLite |
|---|---|---|
| **< 100 notes** | ⚡ Nhanh, mượt | ⚡ Nhanh |
| **1,000 notes** | ⚡ Vẫn nhanh | ⚡ Tối ưu |
| **10,000 notes** | ⚠️ Bắt đầu chậm (indexing) | ⚡ Vẫn nhanh (B-tree index) |
| **100,000+ notes** | ❌ Chậm đáng kể | ⚡ Vẫn ổn (optimized engine) |
| **Structured queries** | ❌ Hạn chế (full-text search) | ✅ SQL mạnh mẽ |
| **Relations/JOINs** | ❌ Graph traversal chậm | ✅ JOIN nhanh (indexed) |

> Theo SQLite.org: *"SQLite can be **faster than the filesystem** for reading and writing content to disk."*

> **🏆 Thắng: SQLite** — Với dữ liệu lớn và có cấu trúc, SQLite vượt trội hoàn toàn.

#### C. LINH HOẠT & TỰ DO (Flexibility)

| Khía cạnh | Obsidian | SQLite |
|---|---|---|
| **Loại nội dung** | ✅ Văn bản tự do, image, audio, video, embed | ⚠️ Structured data (text, numbers, blob) |
| **Thay đổi cấu trúc** | ✅ Không cần schema, viết tự do | ⚠️ Cần ALTER TABLE, migration |
| **Nội dung đa phương tiện** | ✅ Nhúng hình, video, PDF dễ dàng | ❌ Chỉ lưu BLOB hoặc đường dẫn |
| **Long-form writing** | ✅ Xuất sắc — bài viết, nhật ký, sách | ❌ Không phù hợp |
| **Free-form thinking** | ✅ Brainstorming, mind mapping, canvas | ❌ Cần schema trước |

> **🏆 Thắng: Obsidian** — Cho tư duy tự do, sáng tạo, và nội dung phi cấu trúc.

#### D. TRUY VẤN & PHÂN TÍCH (Query & Analysis)

| Nhu cầu | Obsidian | SQLite |
|---|---|---|
| **Tìm kiếm text** | ✅ Full-text search built-in | ✅ FTS5 extension |
| **Lọc theo metadata** | ⚠️ Dataview plugin (truy vấn phức tạp) | ✅ `WHERE`, `HAVING` native |
| **Thống kê** | ❌ Rất hạn chế | ✅ `COUNT`, `SUM`, `AVG`, `GROUP BY` |
| **Cross-reference** | ✅ Backlinks, Graph view | ✅ JOIN, subqueries |
| **Phân tích dữ liệu** | ❌ Không phù hợp | ✅ **Xuất sắc** — data analysis native |
| **Aggregation** | ❌ Không hỗ trợ | ✅ SQL aggregation mạnh mẽ |

```sql
-- SQLite: Thống kê số notes theo tag, chỉ lấy tag có > 5 notes
SELECT tag, COUNT(*) as count
FROM tags
GROUP BY tag
HAVING count > 5
ORDER BY count DESC;

-- Obsidian: Cần plugin Dataview với cú pháp riêng
-- ```dataview
-- TABLE length(rows) as count
-- FROM #tag
-- GROUP BY tag
-- WHERE count > 5
-- SORT count DESC
-- ```
```

> **🏆 Thắng: SQLite** — SQL là ngôn ngữ truy vấn mạnh mẽ nhất cho dữ liệu có cấu trúc.

#### E. SỞ HỮU & TỰ CHỦ DỮ LIỆU (Data Ownership)

| Khía cạnh | Obsidian | SQLite |
|---|---|---|
| **Vendor lock-in** | ❌ Không (Markdown mở) | ❌ Không (format chuẩn, Public Domain) |
| **Tự host** | ✅ Dữ liệu cục bộ 100% | ✅ Dữ liệu cục bộ 100% |
| **Export** | ✅ Mở bằng bất cứ đâu | ✅ Export sang CSV, JSON, SQL |
| **Khả năng đọc 10-20 năm** | ✅ Plaintext = chắc chắn | ⚠️ Binary nhưng format ổn định |

> **🏆 Hòa** — Cả hai đều local-first, không lock-in.

#### F. ĐỒNG BỘ & CHIA SẺ (Sync & Sharing)

| Khía cạnh | Obsidian | SQLite |
|---|---|---|
| **Git versioning** | ✅ Xuất sắc (text diff) | ⚠️ Binary diff khó |
| **Sync qua cloud** | ✅ iCloud, GDrive, Dropbox (file riêng lẻ) | ⚠️ 1 file lớn, xung đột dễ hỏng |
| **Obsidian Sync** | ✅ Tích hợp sẵn ($4-8/tháng) | ❌ Không có |
| **Chia sẻ từng phần** | ✅ Gửi 1 file .md | ⚠️ Cần export hoặc gửi cả DB |
| **Conflict resolution** | ✅ File riêng lẻ = ít xung đột | ❌ **1 writer** = dễ conflict khi sync |
| **Multi-device** | ✅ Có (nhưng trả phí) | ⚠️ File lock = rủi ro trên network FS |

> **🏆 Thắng: Obsidian** — File Markdown phân tán dễ đồng bộ hơn file DB tập trung.

#### G. BẢO MẬT (Security)

| Khía cạnh | Obsidian | SQLite |
|---|---|---|
| **Mã hóa at-rest** | ⚠️ OS-level (BitLocker, FileVault) | ⚠️ SQLCipher, SEE extension |
| **Mã hóa E2E (Sync)** | ✅ AES-256 (Obsidian Sync) | ❌ Không có sẵn |
| **Không thu thập data** | ✅ Không telemetry | ✅ Không telemetry (library) |
| **Obscurity** | ✅ Plaintext = transparent | ⚠️ Binary = ít transparent hơn |

> **🏆 Thắng: Obsidian** — Có sẵn mã hóa E2E với Sync.

---

## 4. ƯU/NHƯỢC ĐIỂM TỪNG HỆ THỐNG

### 🔮 OBSIDIAN — Ưu điểm

| # | Ưu điểm | Giải thích |
|---|---|---|
| 1 | **Human-readable** | Plaintext Markdown, đọc bằng bất kỳ editor |
| 2 | **Không schema** | Viết tự do, không cần lên cấu trúc trước |
| 3 | **UI/UX xuất sắc** | Editor, Graph view, Canvas, plugins |
| 4 | **Liên kết tự nhiên** | `[[wikilink]]` — liên kết như tư duy con người |
| 5 | **Graph visualization** | Trực quan hóa mối quan hệ ý tưởng |
| 6 | **Canvas** | Sáng tạo không gian vô hạn |
| 7 | **Plugin ecosystem** | Hàng nghìn plugin mở rộng |
| 8 | **Git-friendly** | Text diff, version control hoàn hảo |
| 9 | **Easy backup/sync** | File nhỏ riêng lẻ, dễ sync qua cloud |
| 10 | **Đường học tập thấp** | Markdown dễ học, UI trực quan |
| 11 | **Đa phương tiện** | Nhúng ảnh, video, PDF, audio, webpage |
| 12 | **Community** | Cộng đồng lớn, tài liệu phong phú |

### 🔮 OBSIDIAN — Nhược điểm

| # | Nhược điểm | Giải thích |
|---|---|---|
| 1 | **Truy vấn yếu** | Không thể SELECT/JOIN/AGGREGATE như SQL |
| 2 | **Dữ liệu lớn chậm** | 100K+ notes = hiệu suất giảm |
| 3 | **Không có transaction** | Không ACID, không rollback |
| 4 | **Tính toán hạn chế** | Không GROUP BY, COUNT, SUM native |
| 5 | **Backlinks chậm** | Cần reindex toàn bộ vault |
| 6 | **Dataview phức tạp** | Plugin Dataview DSL không mạnh bằng SQL |
| 7 | **No enforced structure** | Dễ messy, không có constraint |
| 8 | **Relation yếu** | Liên kết chỉ là text reference, không FK |

---

### 🗄️ SQLITE — Ưu điểm

| # | Ưu điểm | Giải thích |
|---|---|---|
| 1 | **Hiệu suất vượt trội** | Nhanh hơn filesystem (theo sqlite.org) |
| 2 | **SQL mạnh mẽ** | JOIN, GROUP BY, subquery, CTE, window functions |
| 3 | **ACID compliance** | Transaction, rollback, durability |
| 4 | **Indexing** | B-tree index, FTS5 full-text search |
| 5 | **Schema enforcement** | Data integrity, constraints, foreign keys |
| 6 | **Scalable** | 281 TB max, hàng triệu rows mượt |
| 7 | **Public Domain** | Hoàn toàn miễn phí, không license |
| 8 | **Phân tích dữ liệu** | Aggregation, statistics, reporting |
| 9 | **Nhúng được** | Library, không cần server |
| 10 | **Cross-platform** | Mọi OS, mọi ngôn ngữ lập trình |
| 11 | **Single file** | Portable, dễ backup |
| 12 | **Zero config** | Không cần cài đặt, không cần admin |

### 🗄️ SQLITE — Nhược điểm

| # | Nhược điểm | Giải thích |
|---|---|---|
| 1 | **Binary format** | Không đọc được bằng text editor |
| 2 | **Cần biết SQL** | Đường học tập cao cho người không kỹ thuật |
| 3 | **Không có UI** | Không có sẵn giao diện người dùng |
| 4 | **Không tốt cho văn bản tự do** | Không phù hợp cho long-form writing |
| 5 | **1 writer limit** | Chỉ 1 writer tại một thời điểm |
| 6 | **Network sync rủi ro** | File lock trên network FS = corruption |
| 7 | **Git-unfriendly** | Binary diff, không xem thay đổi |
| 8 | **Không trực quan hóa** | Không Graph view, Canvas |
| 9 | **Không đa phương tiện** | Chỉ lưu BLOB hoặc path reference |
| 10 | **Schema migration** | Thay đổi cấu trúc = ALTER TABLE phức tạp |

---

## 5. TRƯỜNG HỢP SỬ DỤNG TỐT NHẤT

### ✅ Dùng OBSIDIAN khi:

```
📝 Ghi chú, nhật ký cá nhân
📖 Viết bài, viết sách, blog
🧠 Brainstorming, mind mapping
📚 Quản lý kiến thức (PKM, Zettelkasten)
📋 Task management cá nhân
🎓 Học tập, ghi chú bài giảng
💡 Quản lý ý tưởng, dự án sáng tạo
🌐 Xuất bản wiki/knowledge base (Publish)
🗂️ Mô hình dữ liệu: PHI CẤU TRÚC → BÁN CẤU TRÚC
```

### ✅ Dùng SQLITE khi:

```
📊 Quản lý dữ liệu có cấu trúc (danh sách, bảng)
💰 Quản lý tài chính cá nhân (thu/chi, đầu tư)
🏃 Theo dõi thói quen, sức khỏe, thể thao
📋 Inventory management (kho hàng, sưu tầm)
🔬 Phân tích dữ liệu, thống kê
🌐 Lưu trữ dữ liệu ứng dụng (backend)
📈 Time-series data (báo cáo, metrics)
🔐 Dữ liệu cần ACID, transaction, integrity
🗂️ Mô hình dữ liệu: CÓ CẤU TRÚC
```

### ❌ KHÔNG phù hợp:

| Hệ thống | Không nên dùng khi... |
|---|---|
| **Obsidian** | Cần tính toán phức tạp, reporting, data analysis, relations phức tạp, dữ liệu > 100K bản ghi |
| **SQLite** | Cần viết văn bản tự do, brainstorming, visual thinking, long-form content, team real-time collaboration |

---

## 6. KHI NÀO KẾT HỢP CẢ HAI?

### 🔮 + 🗄️ Obsidian + SQLite = **Siêu bộ não cá nhân**

Trong thực tế, nhiều người dùng **kết hợp cả hai** để tận dụng điểm mạnh của từng hệ thống:

```
┌─────────────────────────────────────────────────┐
│           SIÊU BỘ NÃO CÁ NHÂN                    │
├──────────────────────┬──────────────────────────┤
│   🔮 OBSIDIAN        │   🗄️ SQLITE              │
│   (Suy nghĩ tự do)   │   (Dữ liệu cấu trúc)    │
├──────────────────────┼──────────────────────────┤
│ • Nhật ký, ghi chú   │ • Tài chính cá nhân      │
│ • Brainstorming       │ • Habit tracking         │
│ • Wiki kiến thức      │ • Contact management     │
│ • Viết blog, sách     │ • Inventory/Danh sách    │
│ • Dự án sáng tạo      │ • Data analysis          │
│ • MOC, learning notes │ • API cache              │
└──────────────────────┴──────────────────────────┘
```

### Công cụ kết hợp:
1. **[obsidian-sqlite3](https://github.com/windily-cloud/obsidian-sqlite3)** — Plugin Obsidian cho phép truy vấn SQLite từ trong Obsidian
2. **Dataview Plugin** — Truy vấn YAML frontmatter giống SQL
3. **DB Folder Plugin** — Quản lý database-like trong Obsidian
4. **Custom Scripts** — Python/Node.js đọc SQLite và tạo Markdown report cho Obsidian

### Ví dụ Workflow kết hợp:

```python
# Python script: Đọc dữ liệu tài chính từ SQLite → Tạo report Markdown cho Obsidian

import sqlite3
from datetime import datetime

conn = sqlite3.connect('personal_finance.db')
cursor = conn.cursor()

# Truy vấn thống kê tháng này
cursor.execute("""
    SELECT category, SUM(amount) as total, COUNT(*) as transactions
    FROM expenses
    WHERE date >= date('now', 'start of month')
    GROUP BY category
    ORDER BY total DESC
""")

# Tạo báo cáo Markdown
md = f"# 📊 Báo cáo tài chính - {datetime.now().strftime('%m/%Y')}\n\n"
md += "| Danh mục | Tổng chi | Số giao dịch |\n|---|---|---|\n"
for row in cursor.fetchall():
    md += f"| {row[0]} | {row[1]:,.0f}đ | {row[2]} |\n"

# Lưu vào Obsidian Vault
with open('/path/to/vault/Finance/Monthly Report.md', 'w') as f:
    f.write(md)

conn.close()
```

---

## 7. KẾT LUẬN & KHUYẾN NGHỊ

### 🏆 BẢNG TỔNG KẾT

| Tiêu chí | 🔮 Obsidian | 🗄️ SQLite | 🏆 Người thắng |
|---|---|---|---|
| **Ghi chú tự do** | ⭐⭐⭐⭐⭐ | ⭐⭐ | 🟣 Obsidian |
| **Quản lý kiến thức** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | 🟣 Obsidian |
| **Data analysis** | ⭐⭐ | ⭐⭐⭐⭐⭐ | 🔵 SQLite |
| **Hiệu suất dữ liệu lớn** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 🔵 SQLite |
| **Tính toán/Thống kê** | ⭐⭐ | ⭐⭐⭐⭐⭐ | 🔵 SQLite |
| **Trực quan hóa** | ⭐⭐⭐⭐⭐ | ⭐⭐ | 🟣 Obsidian |
| **Tính portable** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | 🟣 Obsidian |
| **Truy vấn phức tạp** | ⭐⭐ | ⭐⭐⭐⭐⭐ | 🔵 SQLite |
| **Dễ sử dụng** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | 🟣 Obsidian |
| **Đồng bộ/Sao lưu** | ⭐⭐⭐⭐ | ⭐⭐⭐ | 🟣 Obsidian |
| **Sở hữu dữ liệu** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 🤝 Hòa |
| **Chi phí** | ⭐⭐⭐⭐⭐ (miễn phí) | ⭐⭐⭐⭐⭐ (miễn phí) | 🤝 Hòa |

### 💡 KHUYẾN NGHỊ CUỐI CÙNG

> **Obsidian và SQLite không thay thế mà BỔ SUNG cho nhau.**

| Bạn là ai? | Khuyến nghị |
|---|---|
| 🧑‍💻 **Lập trình viên** | **Cả hai** — SQLite cho data, Obsidian cho notes |
| 📝 **Nhà văn/Sinh viên** | **Obsidian** — Ghi chú, viết, học tập |
| 📊 **Data Analyst** | **SQLite** — Phân tích, thống kê |
| 🏢 **Quản lý dự án** | **Obsidian** + plugin DB cho data nhẹ |
| 🏠 **Quản lý cá nhân tổng hợp** | **Cả hai** — Obsidian cho suy nghĩ, SQLite cho số liệu |
| 🧠 **"Second Brain" builder** | **Obsidian** (chính) + SQLite (bổ trợ) |

### Tóm lại bằng 1 câu:

> **🔮 Obsidian = "Nơi tư duy tự do"** — cho suy nghĩ, sáng tạo, liên kết ý tưởng  
> **🗄️ SQLite = "Nơi dữ liệu có tổ chức"** — cho con số, cấu trúc, phân tích  
> **🔮 + 🗄️ Cùng nhau = "Siêu bộ não cá nhân hoàn chỉnh"**

---

*Báo cáo so sánh dựa trên thông tin từ https://obsidian.md/, https://sqlite.org/whentouse.html, và nghiên cứu cộng đồng — Tháng 7, 2025*
