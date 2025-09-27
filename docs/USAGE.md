# RAM Booster (rambo) 使用指南

## 概述

RAM Booster (rambo) 是一个 macOS 内存管理工具，提供内存监控、释放和进程管理功能。

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone <repository-url>
cd ram\ booster

# 编译发布版本
cargo build --release

# 创建符号链接到 /usr/local/bin（可选）
sudo ln -sf $(pwd)/target/release/cli /usr/local/bin/rambo
```

## CLI 命令详解

### 1. 查看内存状态 (`status`)

显示当前内存统计信息和占用内存最多的进程：

```bash
# 基本用法
rambo status

# 显示前20个进程
rambo status --top 20

# JSON 格式输出
rambo status --json
```

**输出示例：**
```
--- Memory Stats ---
  Total: 18432 MB
  Free: 1178 MB
  Active: 5406 MB
  Inactive: 5075 MB
  Wired: 2660 MB
  Compressed: 3450 MB
  Pressure: Normal

--- Top 10 Processes (by memory) ---
PID    Name                        RSS (MB)
------ ------------------------- ----------
10067  Chrome Helper (Renderer)        601
5668   Xcode                           461
5837   lldb-rpc-server                 385
```

### 2. 内存释放 (`boost`)

执行内存清理操作，释放可用内存：

```bash
# 基本用法
rambo boost

# JSON 格式输出结果
rambo boost --json
```

**注意：** 需要安装 Xcode Command Line Tools（包含 `/usr/bin/purge` 命令）

**输出示例：**
```
Boosting memory... This may take a moment.

--- Boost Result ---
  Time taken: 2.34s
  Memory freed: 512 MB

  Before: 1178 MB free
  After:  1690 MB free
```

### 3. 进程建议 (`suggest`)

列出可以安全终止的候选进程：

```bash
# 基本用法
rambo suggest

# 设置内存阈值（默认50MB）
rambo suggest --rss-threshold 100

# JSON 格式输出
rambo suggest --json
```

**输出示例：**
```
--- Candidate Processes to Terminate ---
PID    Name                        RSS (MB)
------ ------------------------- ----------
12345  Chrome Helper                   256
12346  Some Background App             128
```

### 4. 终止进程 (`kill`)

安全终止指定PID的进程：

```bash
# 终止进程（需要确认）
rambo kill 12345

# 强制终止（SIGKILL）
rambo kill 12345 --force

# 启用进程终止功能
rambo --enable-termination kill 12345
```

**安全特性：**
- 默认关闭进程终止功能
- 系统进程和前台进程受到保护
- 需要二次确认
- 多级安全检查（Safe/Risky/Dangerous/Forbidden）

### 5. 查看日志 (`log`)

显示内存操作的历史记录：

```bash
# 查看今天的日志
rambo log

# 查看指定日期的日志
rambo log 2025-09-27
```

### 6. 系统诊断 (`doctor`)

检查系统配置和依赖：

```bash
rambo doctor
```

**检查项目：**
- `/usr/bin/purge` 命令可用性
- 内存统计访问权限
- 进程列表访问权限
- 配置和日志目录权限
- LaunchAgent 状态

### 7. 后台守护 (`daemon`)

运行后台监控服务：

```bash
# 前台运行守护进程
rambo daemon --foreground

# 安装为 LaunchAgent（自动启动）
rambo daemon --install

# 卸载 LaunchAgent
rambo daemon --uninstall
```

**守护进程功能：**
- 监听系统内存压力事件
- 自动触发内存释放
- 节流控制（避免频繁操作）
- 记录操作日志

## 配置文件

配置文件位置：`~/.config/rambo/config.toml`

```toml
# RSS 阈值（MB），超过此值的进程会被标记为候选
rss_threshold_mb = 50

# 日志后端（jsonl 或 sqlite）
log_backend = "jsonl"

# 日志保留天数
log_retention_days = 30

# 是否启用进程终止功能（默认关闭）
enable_process_termination = false

# 节流间隔（秒），避免短时间内重复执行boost
throttle_interval_seconds = 300

# 白名单进程（永不终止）
whitelist_processes = ["kernel_task", "launchd", "WindowServer"]

# 黑名单进程（禁止终止）
blacklist_processes = []
```

## 环境变量配置

可以通过环境变量覆盖配置：

```bash
export RAMBO_RSS_THRESHOLD_MB=100
export RAMBO_LOG_BACKEND=sqlite
export RAMBO_ENABLE_PROCESS_TERMINATION=true
export RAMBO_THROTTLE_INTERVAL_SECONDS=600
```

## CLI 全局参数

```bash
# 覆盖RSS阈值
rambo --rss-threshold 100 suggest

# 覆盖日志后端
rambo --log-backend sqlite status

# 启用进程终止
rambo --enable-termination kill 12345
```

## 常见用例

### 1. 定期内存监控
```bash
# 每隔1分钟检查内存状态
while true; do
    rambo status --top 5
    sleep 60
done
```

### 2. 内存不足时自动释放
```bash
# 检查可用内存，低于阈值时释放
FREE_MB=$(rambo status --json | jq '.mem_stats.free_mb')
if [ $FREE_MB -lt 1000 ]; then
    rambo boost
fi
```

### 3. 查找内存泄漏
```bash
# 监控特定进程的内存使用
rambo status --json | jq '.processes[] | select(.name=="MyApp")'
```

## 日志文件

日志存储在：`~/.local/share/rambo/logs/`

- 文件格式：`YYYY-MM-DD.jsonl`
- 每行一个JSON事件
- 包含时间戳、操作类型、内存变化等信息

## 安全考虑

1. **进程终止默认关闭**：防止意外终止重要进程
2. **多级安全检查**：系统进程、前台进程受保护
3. **白名单保护**：重要进程永不被建议终止
4. **操作确认**：危险操作需要用户确认
5. **日志记录**：所有操作都有详细日志

## 故障排除

### 1. boost 命令失败
```bash
# 检查是否安装了 Xcode Command Line Tools
rambo doctor

# 安装命令行工具
xcode-select --install
```

### 2. 权限问题
```bash
# 检查权限状态
rambo doctor

# 某些系统可能需要额外权限来访问进程信息
```

### 3. 守护进程不工作
```bash
# 检查 LaunchAgent 状态
rambo doctor

# 手动加载 LaunchAgent
launchctl load ~/Library/LaunchAgents/com.rambo.daemon.plist

# 查看守护进程日志
tail -f ~/Library/Logs/rambo-daemon.log
```

## 性能基准

基于基准测试的性能指标：

- **内存统计读取**: ~930 ns
- **进程列表获取**: ~14.4 ms
- **进程排序**: ~64 µs
- **boost操作**: 通常 2-5 秒
- **常驻内存占用**: ~15 MB

## 限制

1. **macOS 专用**：使用 mach 系统调用，仅支持 macOS
2. **需要命令行工具**：boost 功能需要 `/usr/bin/purge`
3. **权限要求**：某些功能可能需要管理员权限
4. **安全限制**：系统进程保护可能限制某些操作

## API 参考

### 配置优先级
1. CLI 参数（最高优先级）
2. 环境变量
3. 配置文件
4. 默认值（最低优先级）

### 进程安全等级
- **Safe**: 可以安全终止
- **Risky**: 可能影响用户体验，需要确认
- **Dangerous**: 可能导致系统不稳定，需要强确认
- **Forbidden**: 禁止终止（系统进程等）