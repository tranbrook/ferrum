# Nghiên cứu: Cài glibc 2.38+ trên Ubuntu 22.04

## Tổng quan nghiên cứu

Ubuntu 22.04 LTS (Jammy Jellyfish) ships với **glibc 2.35**. Nhiều binary mới (ONNX Runtime 1.23+, Node 20+, v.v.) yêu cầu **glibc 2.38+**. Dưới đây là phân tích chi tiết 4 phương án.

---

## Phương án 1: Compile glibc 2.38 từ source → install vào `/opt/glibc-2.38`

### Các bước
```bash
sudo apt install build-essential gawk bison
wget https://ftp.gnu.org/gnu/glibc/glibc-2.38.tar.bz2
tar xjf glibc-2.38.tar.bz2
cd glibc-2.38
mkdir build && cd build
../configure --prefix=/opt/glibc-2.38
make -j$(nproc)
sudo make install
```

### Cách dùng cho 1 binary cụ thể
```bash
# Cách A: Chạy qua dynamic linker mới
/opt/glibc-2.38/lib/ld-linux-x86-64.so.2 --library-path /opt/glibc-2.38/lib ./my_binary

# Cách B: Dùng patchelf để sửa interpreter trong binary
sudo apt install patchelf
patchelf --set-interpreter /opt/glibc-2.38/lib/ld-linux-x86-64.so.2 \
         --set-rpath /opt/glibc-2.38/lib ./my_binary
```

### Đánh giá
| Tiêu chí | Đánh giá |
|-----------|----------|
| **An toàn hệ thống** | ⚠️ TRUNG BÌNH - Glibc mới tách riêng trong `/opt`, không ghi đè hệ thống |
| **Độ tin cậy** | ⚠️ THẤP - `crt1.o`, `crti.o`, `crtn.o` vẫn từ glibc 2.35 cũ, có thể crash subtle |
| **Hiệu quả** | ⚠️ CHỈ CHO 1 BINARY - Mỗi binary cần patch riêng, không scale |
| **Rủi ro** | ⚠️ Nếu set `LD_LIBRARY_PATH=/opt/glibc-2.38/lib` global → **MỌI process crash SIGSEGV** |
| **Thời gian compile** | ~15-30 phút |

### Kinh nghiệm thực tế (từ forum)
- Người dùng **regmar** trên Linux Mint Forum: compile thành công nhưng `ldd --version` vẫn báo 2.35, libraries không được ldconfig nhận diện đúng
- **StackOverflow**: Nếu set `LD_LIBRARY_PATH` toàn hệ thống → `bash` segfault → không login được
- Chỉ an toàn khi dùng cho **một binary cụ thể** qua `patchelf` hoặc `ld-linux-x86-64.so.2`

---

## Phương án 2: Nâng cấp toàn hệ thống → Ubuntu 24.04 (glibc 2.39)

### Các bước
```bash
sudo apt update && sudo apt upgrade -y
sudo apt dist-upgrade

# Đảm bảo systemd enabled trong WSL
cat /etc/wsl.conf  # cần có [boot] systemd=true

# Nâng cấp
sudo do-release-upgrade
# hoặc nếu 24.04 chưa available qua LTS path:
sudo do-release-upgrade -d
```

### Đánh giá
| Tiêu chí | Đánh giá |
|-----------|----------|
| **An toàn hệ thống** | ✅ CAO - Đây là cách chính thống Ubuntu hỗ trợ |
| **Độ tin cậy** | ✅ CAO - glibc 2.39 native, mọi thứ compatible |
| **Hiệu quả** | ✅ TỐT NHẤT - Giải quyết triệt để cho mọi binary |
| **Rủi ro** | ⚠️ Có thể break một số packages cũ, cần backup trước |
| **Thời gian** | ~30-60 phút tùy network |

### Kinh nghiệm WSL2
- Cần `systemd=true` trong `/etc/wsl.conf` để `do-release-upgrade` chạy ổn định
- Nếu gặp lỗi snapd: start snapd service trước
- Sau upgrade: `wsl --shutdown` từ Windows rồi restart

---

## Phương án 3: Dùng Docker container với Ubuntu 24.04 base

### Các bước
```dockerfile
# Dockerfile
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y ...
# Build binary trong container
```

```bash
# Hoặc chạy ad-hoc
docker run -it ubuntu:24.04 bash
# Trong container: ldd --version → 2.39
```

### Đánh giá
| Tiêu chí | Đánh giá |
|-----------|----------|
| **An toàn hệ thống** | ✅ TỐT NHẤT - Hoàn toàn isolate |
| **Độ tin cậy** | ✅ CAO |
| **Hiệu quả** | ⚠️ Phức tạp cho dev workflow, cần mount volumes |
| **Rủi ro** | ✅ Không ảnh hưởng host |
| **Phù hợp** | Deployment/CI, không phù hợp dev trực tiếp |

---

## Phương án 4: Workaround - Dùng `load-dynamic` + ONNX Runtime cũ (GIẢI PHÁP HIỆN TẠI)

### Cách làm
```bash
# Cài ONNX Runtime 1.20.1 (tương thích glibc 2.35) vào system
wget https://github.com/microsoft/onnxruntime/releases/download/v1.20.1/onnxruntime-linux-x64-1.20.1.tgz
tar xzf onnxruntime-linux-x64-1.20.1.tgz
sudo cp onnxruntime-linux-x64-1.20.1/lib/libonnxruntime.so.1.20.1 /usr/local/lib/
sudo ln -sf /usr/local/lib/libonnxruntime.so.1.20.1 /usr/local/lib/libonnxruntime.so.1
sudo ldconfig
```

```toml
# Cargo.toml
fastembed = { version = "5", default-features = false, features = ["ort-load-dynamic", "hf-hub-native-tls", "image-models"] }
```

### Đánh giá
| Tiêu chí | Đánh giá |
|-----------|----------|
| **An toàn hệ thống** | ✅ CAO - Không thay đổi glibc |
| **Độ tin cậy** | ✅ CAO - Ort load-dynamic ổn định |
| **Hiệu quả** | ⚠️ Phụ thuộc ort crate hỗ trợ ONNX RT 1.20.x API |
| **Rủi ro** | ✅ Thấp nhất |
| **Phù hợp** | Fix nhanh, không cần thay đổi OS |

---

## So sánh tổng hợp

| | Compile glibc riêng | Upgrade Ubuntu 24.04 | Docker | load-dynamic (hiện tại) |
|---|---|---|---|---|
| **Độ khó** | Trung bình | Dễ | Trung bình | Dễ |
| **Rủi ro** | Cao | Thấp | Không | Không |
| **Triệt để** | Không | ✅ Có | ✅ Có | Không |
| **Ảnh hưởng hệ thống** | Có thể crash | Nâng cấp toàn bộ | Không | Không |
| **Khuyên dùng** | ❌ Không | ✅ Nếu sẵn sàng upgrade | ✅ Cho CI/deploy | ✅ Fix nhanh nhất |

---

## Khuyến nghị

1. **Ngắn hạn (đang dùng)**: Giữ phương án 4 (`ort-load-dynamic`) - đã hoạt động, 0 rủi ro
2. **Trung/Dài hạn**: Nâng cấp lên Ubuntu 24.04 - glibc 2.39 native, giải quyết triệt để mọi vấn đề
3. **KHÔNG khuyến nghị**: Compile glibc riêng - rủi ro cao, lợi ích thấp
