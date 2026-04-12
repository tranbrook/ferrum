# 🔬 Báo Cáo Nghiên Cứu: GitHub Spec Kit - Áp Dụng Vào BonBo Agent Coding

> **Ngày nghiên cứu**: 2025-04-04  
> **Nguồn**: https://github.com/github/spec-kit  
> **Mục tiêu**: Đánh giá khả năng áp dụng SpecKit vào BonBo agent coding workflow

---

## 1. TỔNG QUAN VỀ GITHUB SPEC KIT

### 1.1 Spec Kit là gì?
GitHub Spec Kit là một **open-source toolkit chính thức từ GitHub** (MIT License) giúp thực hiện **Spec-Driven Development (SDD)** — một phương pháp phát triển phần mềm trong đó **đặc tả (specification) là trung tâm** thay vì code.

### 1.2 Triết lý cốt lõi: "Code serves specifications"
- **Đặc tả trở thành artifact thực thi (executable)** — không chỉ là tài liệu tham khảo
- **Code là output được sinh ra từ đặc tả**, không phải ngược lại
- Thay vì "vibe coding" (nhập prompt mông lung → hy vọng code đúng), SDD yêu cầu **định nghĩa rõ WHAT & WHY trước**, sau đó mới đến HOW

### 1.3 Số liệu đáng chú ý
- ⭐ 8,000+ GitHub stars
- 🤖 Hỗ trợ **30+ AI coding agents** (Claude Code, Gemini CLI, Copilot, Cursor, Codex, Windsurf, Amp, Junie, Kilo Code, etc.)
- 🧩 **35+ community extensions** đã có sẵn
- 📦 CLI tool: `specify-cli` (Python, cài qua `uv`)

---

## 2. WORKFLOW CỦA SPEC KIT

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│ Constitution │───▶│  Specify    │───▶│  Clarify    │───▶│    Plan     │───▶│    Tasks     │───▶│  Implement  │
│ (Nguyên tắc) │    │ (Đặc tả)   │    │ (Làm rõ)   │    │ (Kế hoạch) │    │ (Công việc) │    │ (Thực thi)  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘    └──────────────┘    └─────────────┘
     Bước 0            Bước 1            Bước 2             Bước 3             Bước 4            Bước 5
```

### Bước 0: Constitution (`/speckit.constitution`)
- Thiết lập **nguyên tắc bất biến** của dự án (kiến trúc, testing, quality, UX...)
- Tạo file `.specify/memory/constitution.md`
- Đóng vai trò như "DNA kiến trúc" — mọi quyết định sau đều phải tuân thủ

### Bước 1: Specify (`/speckit.specify`)
- Định nghĩa **WHAT** cần xây và **WHY** (không bàn về tech stack)
- Focus vào: user journeys, experiences, success criteria
- Tự động: đánh số feature, tạo branch, sinh spec từ template

### Bước 2: Clarify (`/speckit.clarify`)
- Làm rõ các điểm chưa rõ trong spec
- Đánh dấu `[NEEDS CLARIFICATION]` để không đoán mò
- Review checklist để đảm bảo completeness

### Bước 3: Plan (`/speckit.plan`)
- Định nghĩa **HOW** — tech stack, architecture, constraints
- Sinh: plan.md, data-model.md, contracts/, research.md, quickstart.md
- Constitutional compliance check — đảm bảo plan tuân thủ constitution

### Bước 4: Tasks (`/speckit.tasks`)
- Phá nhỏ plan thành tasks cụ thể, actionable
- Đánh dấu dependency, parallelizable tasks `[P]`
- Test-first structure: test tasks trước implementation tasks

### Bước 5: Implement (`/speckit.implement`)
- Thực thi từng task theo thứ tự
- TDD: tests trước, code sau
- Incremental delivery theo user story phases

---

## 3. CẤU TRÚC DỰ ÁN SPEC KIT

```
project/
├── .specify/
│   ├── memory/
│   │   └── constitution.md          # Nguyên tắc dự án
│   ├── scripts/                     # Helper scripts (bash + powershell)
│   │   ├── create-new-feature.sh
│   │   ├── setup-plan.sh
│   │   └── update-agent-context.sh
│   ├── specs/
│   │   ├── 001-feature-name/
│   │   │   ├── spec.md              # Đặc tả chức năng
│   │   │   ├── plan.md              # Kế hoạch kỹ thuật
│   │   │   ├── tasks.md             # Danh sách tasks
│   │   │   ├── data-model.md        # Mô hình dữ liệu
│   │   │   ├── research.md          # Nghiên cứu kỹ thuật
│   │   │   ├── quickstart.md        # Hướng dẫn nhanh
│   │   │   └── contracts/           # API contracts
│   │   │       ├── api-spec.json
│   │   │       └── ...
│   │   └── 002-next-feature/
│   ├── templates/                   # Templates cho spec, plan, tasks
│   └── extensions/                  # Extensions (optional)
├── .claude/commands/                # Agent-specific commands (nếu dùng Claude)
└── CLAUDE.md / AGENTS.md            # Agent context file
```

---

## 4. ĐÁNH GIÁ: SPEC KIT CÓ ÁP DỤNG ĐƯỢC VÀO BONBO KHÔNG?

### 4.1 ✅ ĐIỂM TƯƠNG THÍCH CAO

| Khía cạnh | BonBo hiện tại | Spec Kit | Mức độ phù hợp |
|-----------|---------------|----------|----------------|
| **Task tracking** | `tasks/todo.md` với checkboxes | `tasks.md` với phases & checkpoints | ⭐⭐⭐⭐⭐ |
| **Activity logging** | `docs/activity.md` với timestamps | Branch-based feature tracking | ⭐⭐⭐⭐ |
| **Project README** | `docs/PROJECT_README.md` | `constitution.md` + agent context files | ⭐⭐⭐⭐ |
| **Template-driven** | System prompt định rõ format | Template files (.md) hướng dẫn LLM | ⭐⭐⭐⭐⭐ |
| **Phased development** | Plan → Execute → Verify | Specify → Plan → Tasks → Implement | ⭐⭐⭐⭐⭐ |
| **Knowledge persistence** | SQLite knowledge base | `.specify/memory/` markdown files | ⭐⭐⭐⭐ |

### 4.2 ✅ NHỮNG GÌ BONBO CÓ THỂ HỌC HỎI VÀ ÁP DỤNG

#### A. Template-Driven Quality (Giá trị cao nhất)
Spec Kit sử dụng templates như "unit tests cho specifications":
- ✅ **Buộc LLM phân tầng đúng**: Spec chỉ nói WHAT/WHY, không bàn HOW
- ✅ **Đánh dấu uncertainty**: `[NEEDS CLARIFICATION]` thay vì đoán mò
- ✅ **Checklist tự review**: Templates có sẵn checklist để LLM self-check
- ✅ **Phase gates**: Kiểm tra trước khi chuyển bước tiếp

**→ Áp dụng**: BonBo có thể tạo internal template system áp dụng cùng nguyên tắc.

#### B. Constitution-as-Code (Rất giá trị)
- Constitution đóng vai trò như **"DNA kiến trúc"** — immutable principles
- Mọi quyết định implement đều phải pass "constitutional gates"
- Giúp maintain **consistency across sessions** — vấn đề lớn nhất của AI agents

**→ Áp dụng**: BonBo có thể thêm constitution layer vào knowledge base.

#### C. Feature-Branch SDD Workflow
- Mỗi feature = 1 branch + 1 thư mục spec riêng
- Tự động đánh số (001, 002, 003...)
- Tất cả artifacts (spec, plan, tasks, contracts) nằm cùng nhau

**→ Áp dụng**: BonBo có thể áp dụng structured feature numbering và artifact co-location.

#### D. Test-First AI Implementation
- Template ép LLM viết tests TRƯỚC khi implement
- Integration-first testing (real DB, real services) thay vì mocks
- Contract tests mandatory

**→ Áp dụng**: BonBo có thể thêm TDD enforcement vào implement workflow.

#### E. Extensions Ecosystem
- Spec Kit có plugin architecture cho extensions
- Categories: docs, code, process, integration, visibility
- Đã có 35+ community extensions (Jira, Azure DevOps, QA, Code Review...)

**→ Áp dụng**: BonBo có thể thiết kế plugin system tương tự.

### 4.3 ⚠️ THÁCH THỨC & ĐIỀU CHỈNH CẦN THIẾT

| Thách thức | Chi tiết | Giải pháp đề xuất |
|------------|----------|-------------------|
| **BonBo không phải CLI tool riêng** | Spec Kit thiết kế cho CLI agents (Claude Code, Gemini CLI...) | Tạo **SpecKit-inspired internal workflow** thay vì dùng trực tiếp |
| **Overhead cho small tasks** | Full SDD workflow quá nặng cho bug fixes nhỏ | Linh hoạt: full SDD cho features, lightweight cho bugs |
| **BonBo đã có knowledge base** | SpecKit dùng file markdown, BonBo dùng SQLite | Kết hợp: dùng SQLite để store constitution & specs |
| **Language barrier** | Templates toàn tiếng Anh | Có thể tạo preset tiếng Việt / đa ngôn ngữ |

### 4.4 📊 MA TRẬN ĐÁNH GIÁ TỔNG THỂ

```
Khả năng áp dụng trực tiếp:    ████████░░  80%
Giá trị lý thuyết:             █████████░  90%
Độ phức tạp tích hợp:          ██████░░░░  60%
ROI (nếu áp dụng):             █████████░  90%
Tương thích kiến trúc BonBo:   ███████░░░  70%
```

---

## 5. ĐỀ XUẤT KẾ HOẠCH TÍCH HỢP VÀO BONBO

### Phase 1: Áp dụng Concept (Ngay lập tức)
1. **Thêm Constitution layer** vào BonBo knowledge base
   - Lưu project principles vào `knowledge_add(category="constitution")`
   - Tự động load constitution khi bắt đầu session mới
   
2. **Tạo internal SDD templates**
   - Spec template → Áp dụng cho `docs/PROJECT_README.md`
   - Plan template → Áp dụng cho `tasks/todo.md` structure
   - Tasks template → Áp dụng cho task breakdown format

3. **Thêm Clarification phase**
   - Khi nhận task mới, BonBo tự động mark `[NEEDS CLARIFICATION]` 
   - Hỏi user trước khi implement

### Phase 2: Enhanced Workflow (2-4 tuần)
4. **Feature numbering system**
   - Mỗi feature được đánh số (F001, F002...)
   - Tạo artifact directory structure tự động

5. **TDD enforcement**
   - Khi `/implement`, yêu cầu tạo test trước
   - Checklist validation trước khi mark task complete

6. **Cross-artifact consistency check**
   - Tương tự `/speckit.analyze` — kiểm tra spec ↔ plan ↔ tasks đồng nhất

### Phase 3: Advanced Features (1-2 tháng)
7. **Plugin/Extension system**
   - Cho phép user thêm custom workflow steps
   - Inspired by SpecKit's extension architecture

8. **Multi-variant implementation**
   - Từ 1 spec, generate nhiều implementation approaches
   - So sánh và chọn best approach

---

## 6. KẾT LUẬN

### 🟢 CÂU TRẢ LỜI: CÓ, RẤT ĐÁNG ÁP DỤNG!

**GitHub Spec Kit mang lại một phương pháp luận cực kỳ giá trị** mà BonBo nên học hỏi và tích hợp:

1. **Spec-Driven thay vì Code-Driven**: Giảm "vibe coding", tăng chất lượng output
2. **Template-based quality control**: Ép LLM tuân thủ format và quy trình
3. **Constitution-as-DNA**: Đảm bảo consistency across sessions
4. **Phased workflow với gates**: Giảm rework, tăng predictability
5. **TDD enforcement**: Output có test, có verification

### Điểm mấu chốt:
> **Không cần dùng SpecKit directly** (vì BonBo có kiến trúc riêng), nhưng **NÊN áp dụng triết lý SDD** vào BonBo's internal workflow. Đặc biệt:
> - Constitution layer cho project principles
> - Template-driven spec/plan/tasks generation  
> - Phase gates và clarification phase
> - TDD enforcement trong implement

### So sánh nhanh:

| | Vibe Coding (hiện tại nhiều agent) | BonBo + SDD (đề xuất) |
|---|---|---|
| Input | Prompt mông lung | Structured spec + plan |
| Output | Code hy vọng đúng | Code verified theo spec |
| Consistency | Mỗi session khác nhau | Constitution đảm bảo consistency |
| Quality | Phụ thuộc may rủi | Template-driven, predictable |
| Maintainability | Khó, code-first | Spec-first, dễ evolve |

---

*Tài liệu này được nghiên cứu và soạn bởi BonBo Agent — dựa trên phân tích từ GitHub repo chính thức, blog GitHub, và Microsoft Developer Blog.*
