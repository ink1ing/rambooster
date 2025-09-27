# 🚀 RAM Booster 快速上手

## 一步编译启动

```bash
# 进入项目目录
cd "ram booster"

# 编译项目
cargo build --release

# 直接使用（无需安装）
./target/release/cli status
```

## 🧪 运行测试

```bash
# 快速测试（推荐）
./scripts/test.sh

# 或单独测试
cargo test --lib                      # 单元测试 (29个)
cargo test --test integration_tests   # 集成测试 (5个)
```

## 📝 常用命令

```bash
# 查看内存状态
./target/release/cli status

# 查看系统诊断
./target/release/cli doctor

# 查看进程建议
./target/release/cli suggest

# 日志管理
./target/release/cli logs info
./target/release/cli logs list

# 内存释放（需要Xcode Command Line Tools）
./target/release/cli boost

# JSON输出
./target/release/cli status --json
```

## ⚙️ 系统要求

- **macOS** (使用mach系统调用)
- **Rust 1.70+**
- **Xcode Command Line Tools** (可选，用于boost功能)

## 🔧 安装系统命令行工具

如果需要使用内存释放功能：

```bash
xcode-select --install
```

## 📊 性能表现

- 内存统计读取：~930 ns
- 进程列表获取：~14.4 ms
- 常驻内存占用：~15 MB
- 所有测试通过：29个单元测试 + 5个集成测试

## 🛡️ 安全特性

- ✅ 进程终止功能默认关闭
- ✅ 系统进程自动保护
- ✅ 操作前二次确认
- ✅ 完整操作日志记录

---

**🎉 项目已可本地使用！更多详情见 `docs/USAGE.md`**