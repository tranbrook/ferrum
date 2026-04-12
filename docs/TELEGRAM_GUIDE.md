# 🤖 Vybrid Telegram Bot — Hướng dẫn sử dụng

## Tổng quan

Vybrid Telegram Bot là kênh giao tiếp với AI Assistant Vybrid qua Telegram, cung cấp chức năng tương tự CLI. Bot có thể:

- 💬 Chat với AI assistant (GLM-5.1)
- 🔧 Gọi tools tự động (đọc/ghi file, chạy lệnh shell, tìm kiếm code, web search...)
- 🧠 Hỗ trợ thinking/reasoning mode
- 💬 Giữ ngữ cảnh hội thoại theo từng chat
- 🔒 Hạn chế quyền truy cập theo user/chat ID

---

## 1. Cài đặt

### Yêu cầu

- Rust (edition 2021) đã cài đặt
- API key Z.AI (`ZAI_API_KEY`)
- Telegram Bot Token (từ @BotFather)

### Build từ source

```bash
cd vybrid-rust
cargo build --release
```

Binary nằm tại `target/release/vybrid`.

### Install (tùy chọn)

```bash
./install.sh
source ~/.bashrc
```

---

## 2. Tạo Telegram Bot

### Bước 1: Chat với @BotFather trên Telegram

1. Mở Telegram, tìm **@BotFather**
2. Gửi lệnh `/newbot`
3. Chọn tên bot (ví dụ: `Vybrid Assistant`)
4. Chọn username cho bot (phải kết thúc bằng `bot`, ví dụ: `vybrid_assistant_bot`)
5. BotFather sẽ trả về **Bot Token** — dạng `1234567890:ABCdefGHIjklMNOpqrSTUvwxYZ`

### Bước 2: Lấy User ID của bạn (nếu muốn giới hạn truy cập)

1. Chat với **@userinfobot** trên Telegram
2. Gửi bất kỳ tin nhắn nào
3. Bot sẽ trả về **User ID** của bạn (dạng số, ví dụ: `123456789`)

---

## 3. Cấu hình

### Biến môi trường cần thiết

Thêm vào file `~/.vybrid/.env` hoặc `vybrid-rust/.env`:

```bash
# === Bắt buộc ===
ZAI_API_KEY=your_zai_api_key_here
TELEGRAM_BOT_TOKEN=your_telegram_bot_token_here

# === Tùy chọn ===
# Danh sách User ID được phép sử dụng bot (phân tách bằng dấu phẩy)
TELEGRAM_ALLOWED_USERS=123456789,987654321

# Danh sách Chat ID (group/channel) được phép
TELEGRAM_ALLOWED_CHATS=-1001234567890

# Override model (mặc định: glm-5.1)
ZAI_MODEL=glm-5.1

# Override API base URL (mặc định: https://api.z.ai/api/coding/paas/v4)
ZAI_API_BASE_URL=https://api.z.ai/api/coding/paas/v4
```

### Ví dụ file `.env` hoàn chỉnh

```bash
ZAI_API_KEY=abc123xyz456
TELEGRAM_BOT_TOKEN=1234567890:ABCdefGHIjklMNOpqrSTUvwxYZ
TELEGRAM_ALLOWED_USERS=123456789
```

### Bảo mật

> ⚠️ **QUAN TRỌNG**: Nếu không cấu hình `TELEGRAM_ALLOWED_USERS` hoặc `TELEGRAM_ALLOWED_CHATS`, **bất kỳ ai** đều có thể sử dụng bot. Khuyến nghị luôn thiết lập danh sách user được phép.

---

## 4. Chạy Bot

### Chạy từ source

```bash
cd vybrid-rust
cargo run -- --telegram
```

### Chạy từ binary đã build

```bash
./target/release/vybrid --telegram
# hoặc
vybrid --telegram    # nếu đã install
```

### Shortcut

```bash
# -t là shorthand cho --telegram
vybrid -t
```

### Dừng bot

Nhấn `Ctrl+C` trong terminal đang chạy bot.

### Chạy dưới dạng background service (Linux)

```bash
# Dùng nohup
nohup vybrid --telegram > vybrid-bot.log 2>&1 &

# Hoặc dùng systemd (khuyến nghị cho production)
```

#### Ví dụ systemd service

Tạo file `/etc/systemd/system/vybrid-bot.service`:

```ini
[Unit]
Description=Vybrid Telegram Bot
After=network.target

[Service]
Type=simple
User=your_user
WorkingDirectory=/home/your_user
EnvironmentFile=/home/your_user/.vybrid/.env
ExecStart=/home/your_user/.local/bin/vybrid --telegram
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable vybrid-bot
sudo systemctl start vybrid-bot
sudo systemctl status vybrid-bot
```

---

## 5. Sử dụng trên Telegram

### Lệnh cơ bản

| Lệnh | Mô tả |
|------|--------|
| `/start` | Khởi động bot, hiển thị thông báo chào |
| `/help` | Hiển thị danh sách lệnh |
| `/new` | Bắt đầu cuộc hội thoại mới |
| `/clear` | Xóa toàn bộ lịch sử hội thoại |
| `/tools` | Liệt kê các tools AI có thể sử dụng |
| `/status` | Xem trạng thái hội thoại hiện tại |

### Chat với AI

Đơn giản gửi tin nhắn text cho bot — AI sẽ phản hồi. AI có thể tự động gọi tools khi cần:

**Ví dụ:**

```
Bạn: Viết cho tôi một file Rust hello world
Bot: (AI tạo file bằng tool create_file và trả lời)

Bạn: Đọc file Cargo.toml
Bot: (AI đọc file bằng tool read_file và hiển thị nội dung)

Bạn: Tìm tất cả file .rs trong project
Bot: (AI chạy lệnh shell find hoặc dùng grep)
```

### Tính năng

#### 🔧 Tool tự động
AI tự quyết định khi nào cần dùng tools. Các tools có sẵn:

- **read_file** / **read_multiple_files** — Đọc file
- **create_file** / **create_multiple_files** — Tạo/ghi file
- **edit_file** — Sửa file (thay thế đoạn code chính xác)
- **execute_bash_command** — Chạy lệnh shell
- **enhanced_grep** — Tìm kiếm code với regex
- **google_search** — Tìm kiếm Google (cần SERPAPI_KEY)
- **ddg_search** — Tìm kiếm web (DuckDuckGo + Brave)
- **create_project_structure** — Tạo cấu trúc project
- **get_current_todo_items** — Đọc danh sách task
- **mark_todo_complete** — Đánh dấu task hoàn thành

#### 🧠 Thinking Mode
AI sử dụng GLM-5.1 với thinking mode enabled — AI sẽ "suy nghĩ" trước khi trả lời.

#### 📱 Markdown Formatting
Bot tự động format response với Telegram MarkdownV2:
- Code blocks với syntax highlighting
- Bold, italic text
- Escaping ký tự đặc biệt

#### 💬 Multi-chat
Mỗi chat (private/group) có conversation history riêng, không bị trộn lẫn.

#### ⚠️ Typing Indicator
Bot tự động gửi "typing..." indicator khi đang xử lý để bạn biết bot đang hoạt động.

---

## 6. Kiến trúc

```
src/telegram/
├── mod.rs          # Module declaration
├── bot.rs          # Bot startup, dispatcher setup
├── config.rs       # Telegram config (token, allowed users)
├── formatter.rs    # MarkdownV2 formatting, message splitting
├── handlers.rs     # Message handlers, AI processing loop
└── state.rs        # Per-chat state management
```

### Luồng hoạt động

```
Telegram User → Telegram Server → Bot (teloxide) → Handler
                                                     ↓
                                              GlmClient (streaming)
                                                     ↓
                                              Tool Execution (if needed)
                                                     ↓
                                              Format Response → Send to Telegram
```

---

## 7. Troubleshooting

### Bot không phản hồi

1. Kiểm tra `TELEGRAM_BOT_TOKEN` đã đúng chưa
2. Kiểm tra bot đang chạy: `ps aux | grep vybrid`
3. Kiểm tra logs trong terminal
4. Kiểm tra kết nối mạng

### Lỗi "No API key configured"

Thêm `ZAI_API_KEY` vào `~/.vybrid/.env`:

```bash
echo 'ZAI_API_KEY=your_key_here' >> ~/.vybrid/.env
```

### Lỗi "You are not authorized"

Kiểm tra `TELEGRAM_ALLOWED_USERS` chứa đúng User ID của bạn:

```bash
# Xem cấu hình hiện tại
grep TELEGRAM_ALLOWED ~/.vybrid/.env
```

### Lỗi "TELEGRAM_BOT_TOKEN not set"

```bash
echo 'TELEGRAM_BOT_TOKEN=your_token_here' >> ~/.vybrid/.env
```

### Tin nhắn bị cắt

Telegram giới hạn 4096 ký tự/tin nhắn. Bot tự động chia nhỏ tin nhắn dài. Nếu vẫn bị lỗi, kiểm tra nội dung có chứa ký tự MarkdownV2 không hợp lệ.

### Bot chậm phản hồi

- AI model (GLM-5.1) cần thời gian suy nghĩ và phản hồi
- Nếu AI gọi tools, thời gian sẽ lâu hơn
- Kiểm tra tốc độ mạng đến `api.z.ai`

---

## 8. So sánh CLI vs Telegram Bot

| Tính năng | CLI | Telegram Bot |
|-----------|-----|--------------|
| Chat với AI | ✅ | ✅ |
| Tool calling | ✅ | ✅ |
| Streaming response | ✅ | ❌ (gửi khi hoàn tất) |
| Shell mode (`!`) | ✅ | ❌ |
| `/menu` setup | ✅ | ❌ (cấu hình file .env) |
| `/docs` project context | ✅ | ✅ (tự động inject) |
| Project docs add/read | ✅ | ❌ |
| Multi-user | ❌ | ✅ |
| Remote access | ❌ | ✅ |
| Persistent session | In-memory | In-memory |
| Output formatting | Terminal colors | Telegram MarkdownV2 |

---

## 9. Ghi chú phát triển

### Thêm command mới

1. Thêm xử lý trong `src/telegram/handlers.rs` → hàm `handle_message()`
2. Nếu cần format đặc biệt, thêm helper trong `src/telegram/formatter.rs`

### Thay đổi system prompt

Sửa hàm `get_system_prompt()` trong `src/telegram/handlers.rs`.

### Thay đổi model/API

Cấu hình qua biến môi trường:
- `ZAI_MODEL` — đổi model
- `ZAI_API_BASE_URL` — đổi API endpoint

---

*Viết bởi Vybrid Assistant • Version 1.0.0 • 2026-04-01*
