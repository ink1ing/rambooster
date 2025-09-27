
# RAM-Booster (rambo) — 细节实现（details.md）

---

## 1. 核心依赖与模块

### 1.1 Rust crates
- 系统接口：
  - `mach`（调用 `host_statistics64` 读取内存）
  - `libc`（底层 syscall）
  - `sysinfo`（跨平台兜底）
- CLI：
  - `clap`（命令行解析）
  - `anyhow`、`thiserror`（错误处理）
- 日志：
  - `tracing`、`tracing-subscriber`
  - `serde`、`serde_json`
  - `rusqlite`（SQLite 可选）
- 后台：
  - `daemonize`（非 launchd 场景）
- UI（可选）：
  - `tauri`、`tauri-plugin-positioner`

### 1.2 目录结构
```

/rambo
/crates
/core     # 统计、策略、日志
/cli      # 命令行接口
/ui       # 可选，Tauri UI
/docs       # plan.md, details.md, TODO.md
/scripts    # 构建、签名、公证脚本

````

---

## 2. 内存统计实现

### 2.1 API
- 调用 `host_statistics64` → `vm_statistics64`
- 字段：`active_count`、`inactive_count`、`wire_count`、`compressor_page_count`、`free_count`
- 页大小：`host_page_size`

### 2.2 数据结构
```rust
pub struct MemStats {
    pub total_mb: u64,
    pub free_mb: u64,
    pub active_mb: u64,
    pub inactive_mb: u64,
    pub wired_mb: u64,
    pub compressed_mb: u64,
    pub pressure: PressureLevel,
}

pub enum PressureLevel { Normal, Warning, Critical }
````

### 2.3 压力等级推导

* `warning`：可用内存 < 15% 或压缩 > 20%
* `critical`：可用内存 < 5% 或压缩 > 30%

---

## 3. 进程监控

### 3.1 数据采集

* `proc_listpids` 获取所有 pid
* `proc_pid_rusage` → RSS
* `proc_pidpath` → 进程名
* CPU 使用：`rusage_info_v2` 中 `ri_user_time` + `ri_system_time`

### 3.2 前台排除

* 绑定 Objective-C：`NSWorkspace.shared.frontmostApplication`
* 或：使用 `CGWindowListCopyWindowInfo` 检查前台 app

### 3.3 排序与筛选

* 默认按 RSS 排序
* 提供 Top N 进程列表
* 候选终止条件：RSS > 阈值、非前台、不在黑名单

---

## 4. 内存压力事件

* 使用 `dispatch_source_create(DISPATCH_SOURCE_TYPE_MEMORYPRESSURE, …)`
* 监听：

  * `DISPATCH_MEMORYPRESSURE_NORMAL`
  * `DISPATCH_MEMORYPRESSURE_WARN`
  * `DISPATCH_MEMORYPRESSURE_CRITICAL`
* 在 `warn`/`critical` 时触发 `boost()` 或通知

---

## 5. 释放策略

### 5.1 清缓存

```rust
use std::process::Command;

pub fn purge() -> anyhow::Result<()> {
    let status = Command::new("/usr/bin/purge").status()?;
    if !status.success() {
        anyhow::bail!("purge failed with code {:?}", status.code());
    }
    Ok(())
}
```

* 错误处理：未安装 CLT → 返回提示

### 5.2 温和终止进程

```rust
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

pub fn terminate(pid: i32) -> anyhow::Result<()> {
    kill(Pid::from_raw(pid), Signal::SIGTERM)?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    // 如果仍在运行，提示用户是否升级到 SIGKILL
    Ok(())
}
```

### 5.3 Boost 组合

* 记录前后 `MemStats`
* 执行 `purge()`
* （可选）终止候选进程
* 输出差值并写日志

---

## 6. 日志系统

### 6.1 JSONL

* 路径：`~/.local/share/rambo/logs/YYYY-MM-DD.jsonl`
* 每次操作追加一行 JSON

### 6.2 SQLite

* 表结构：

```sql
CREATE TABLE events (
  ts TEXT PRIMARY KEY,
  action TEXT,
  before_json TEXT,
  after_json TEXT,
  delta_mb INTEGER,
  pressure TEXT,
  details_json TEXT
);
```

### 6.3 数据模型

```rust
#[derive(Serialize, Deserialize)]
pub struct LogEvent {
    ts: String,
    action: String,
    before: MemStats,
    after: MemStats,
    delta_mb: i64,
    pressure: PressureLevel,
    details: serde_json::Value,
}
```

---

## 7. CLI 设计

### 7.1 命令

* `rambo status [--json]`
* `rambo boost [--json]`
* `rambo suggest`
* `rambo kill <pid>`
* `rambo log --today|--range`
* `rambo doctor`
* `rambo daemon`

### 7.2 输出

* 默认：表格/文本
* `--json`：输出 JSON，方便脚本化

---

## 8. 配置系统

### 8.1 文件路径

* `~/.config/rambo/config.toml`

### 8.2 示例

```toml
[log]
backend = "jsonl"
retain_days = 14

[boost]
enable_terminate = false
candidate_rss_mb = 500
confirm_kill = true

[ui]
menu_bar = true
```

### 8.3 优先级

默认值 < 配置文件 < 环境变量 < CLI 参数

---

## 9. 后台常驻

### 9.1 Launchd Agent

`~/Library/LaunchAgents/io.ink.rambo.agent.plist`

```xml
<dict>
  <key>Label</key><string>io.ink.rambo.agent</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/rambo</string>
    <string>daemon</string>
  </array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
</dict>
```

### 9.2 功能

* 监听压力事件
* 自动调用 `boost()`（带节流）

---

## 10. 菜单栏 UI（Tauri）

### 10.1 功能

* 菜单栏图标：显示可用内存百分比
* 面板：Boost 按钮 + 今日日志
* 设置页：阈值、白/黑名单、自动触发

### 10.2 架构

* Rust → `tauri::command` 暴露核心 API
* 前端：React/Vue + Tailwind，显示数据

---

## 11. 测试与验证

* 单元测试：

  * `read_mem_stats()`
  * 压力等级推导
  * 候选进程筛选
* 集成测试：

  * `boost()` 前后差值
  * 日志写入与读取
* 基准测试：

  * CLI 常驻内存 < 10MB
  * Boost 耗时 P95 < 2s

---

## 12. 发布与分发

### 12.1 CLI

* 构建 universal binary（arm64 + x86_64）
* Homebrew Tap：`ink/tap/rambo.rb`

### 12.2 GUI

* DMG 打包
* `codesign` + `notarytool` 公证
* GitHub Release + 下载链接

---

## 13. 隐私与安全

* 默认不开启进程终止
* 前台/关键系统进程黑名单保护
* 日志仅本地保存
* 提供一键清理与脱敏模式

---

## 14. Backlog（增强功能）

* 内存趋势预测（滑窗/阈值预警）
* 浏览器标签级占用提示
* 快捷键 / 系统通知
* SwiftUI 原生壳（可上架 App Store）
* 多语言支持（中文/英文）

```

