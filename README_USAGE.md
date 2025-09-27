# RAM Booster (rambo) 本地使用指南

## 快速开始

### 1. 安装和编译

```bash
# 克隆或下载项目到本地
cd "ram booster"

# 运行安装脚本（推荐）
./scripts/install.sh

# 或者手动编译
cargo build --release
```

### 2. 基本使用

```bash
# 查看内存状态和Top进程
rambo status

# 释放内存（需要Xcode Command Line Tools）
rambo boost

# 查看可以安全终止的进程建议
rambo suggest

# 查看系统诊断信息
rambo doctor
```

### 3. 日志管理

```bash
# 查看日志信息
rambo logs info

# 列出所有日志文件
rambo logs list

# 清理过期日志（基于配置的保留天数）
rambo logs cleanup

# 清除所有日志（需确认）
rambo logs clear
```

## 测试和验证

### 运行测试套件

```bash
# 运行完整测试脚本
./scripts/test.sh

# 或者分别运行测试
cargo test --lib                    # 单元测试
cargo test --test integration_tests # 集成测试
cargo bench                        # 基准测试（可选）
```

### 验证功能

1. **内存监控**：`rambo status --json` 应该返回有效的JSON格式
2. **系统检查**：`rambo doctor` 应该显示所有检查项状态
3. **日志功能**：`rambo logs info` 应该显示日志统计信息

## 配置

配置文件位置：`~/.config/rambo/config.toml`

```toml
# 示例配置
rss_threshold_mb = 50
log_backend = "jsonl"
log_retention_days = 30
enable_process_termination = false
throttle_interval_seconds = 300
whitelist_processes = ["kernel_task", "launchd", "WindowServer"]
blacklist_processes = []
```

## 后台守护进程（可选）

```bash
# 前台运行守护进程（测试用）
rambo daemon --foreground

# 安装为系统服务（自动启动）
rambo daemon --install

# 卸载系统服务
rambo daemon --uninstall
```

## 安全特性

- 🔒 **进程终止默认关闭**：防止意外终止重要进程
- 🛡️ **多级安全检查**：系统进程、前台进程自动保护
- 📋 **白名单机制**：重要进程永不被建议终止
- ⚠️ **操作确认**：危险操作需要用户二次确认
- 📝 **完整日志**：所有操作都有详细记录

## 性能指标

基于基准测试的典型性能：
- 内存统计读取：~930 ns
- 进程列表获取：~14.4 ms
- 进程排序：~64 µs
- Boost操作：2-5秒
- 常驻内存：~15 MB

## 故障排除

### 常见问题

1. **`boost` 命令失败**
   ```bash
   # 检查Xcode Command Line Tools
   rambo doctor

   # 安装命令行工具
   xcode-select --install
   ```

2. **权限问题**
   ```bash
   # 检查权限状态
   rambo doctor

   # macOS可能需要允许终端访问系统信息
   ```

3. **编译错误**
   ```bash
   # 清理并重新编译
   cargo clean
   cargo build --release
   ```

### 调试模式

```bash
# 使用调试版本（更详细的错误信息）
cargo build
./target/debug/cli status

# 查看详细输出
RUST_BACKTRACE=1 ./target/release/cli boost
```

## 文件结构

```
ram booster/
├── crates/
│   ├── core/           # 核心功能库
│   └── cli/            # 命令行接口
├── scripts/
│   ├── install.sh      # 安装脚本
│   └── test.sh         # 测试脚本
├── docs/
│   └── USAGE.md        # 详细使用文档
└── README_USAGE.md     # 本文件
```

## 环境要求

- **操作系统**：macOS（使用mach系统调用）
- **Rust版本**：1.70+
- **系统工具**：Xcode Command Line Tools（可选，用于boost功能）
- **权限**：读取进程信息、内存统计

## 获取帮助

```bash
# 查看完整帮助
rambo --help

# 查看子命令帮助
rambo status --help
rambo boost --help
rambo logs --help
```

更详细的文档请参考：`docs/USAGE.md`