
# RAM-Booster（rambo）— 总体计划与架构（plan.md）

> 目标：做一个**诚实、轻量、可解释**的 macOS 内存助手。核心能力：精确监控、可控释放（以清缓存为主，进程温和终止为辅）、可审计日志；优先保证低资源占用与用户掌控感。

---

## 1. 产品定位与边界

### 1.1 场景与价值
- **卡顿应急**：一键“Boost”以释放可回收内存、缓解短时卡顿。
- **长期卫生**：记录内存压力与操作历史，帮助发现“高占用/异常进程”。
- **透明可控**：与商业清理软件相比，**可解释**、**可配置**、**不装神秘加速**。

### 1.2 非目标（初期不做）
- 伪造内存压力的激进技巧（大规模 `malloc/free`）。
- 内核扩展或越权能力。
- 跨平台（初期仅 macOS 13+）。

---

## 2. 平台与技术栈

- **系统**：macOS 13+（Apple Silicon 优先，兼容 Intel）。
- **语言**：Rust 1.80+（核心与 CLI），可选 Tauri 2.x（菜单栏 UI）。
- **系统工具**：`/usr/bin/purge`（清文件缓存；若不可用则降级为提示）。
- **分发**：CLI（Homebrew Tap），GUI（签名/公证 DMG）。

---

## 3. 总体架构

```

/rambo
/crates
/core   ← 采集、策略、日志（纯 Rust）
/cli    ← 命令行接口（clap）
/ui     ← 可选：Tauri 菜单栏壳
/docs     ← 文档（本文件等）
/scripts  ← 构建/签名/公证/发布脚本

````

- **rambo-core（库）**
  - 统计采集：`mach` + `host_statistics64`（`vm_statistics64`），`sysinfo` 兜底。
  - 进程信息：`libproc` / `proc_pid_rusage`，前台判定（NSWorkspace/CGWindowList，经绑定）。
  - 压力事件：`DISPATCH_SOURCE_TYPE_MEMORYPRESSURE`（GCD）。
  - 释放策略：`purge()`；可选 `terminate(pid)`（SIGTERM → 超时 → 可选 SIGKILL）。
  - 日志记录：JSONL（默认）/ SQLite（可选），事件化写入。
  - 配置管理：TOML + 环境变量 + CLI 覆盖。

- **rambo-cli（可执行）**
  - 子命令：`status / boost / suggest / kill / log / doctor / daemon`。
  - 输出：人类可读（表格/摘要）+ JSON（`--json`）。

- **rambo-ui（可选）**
  - 菜单栏：实时可用内存百分比、`Boost` 按钮、今日日志列表、设置面板。
  - 调用核心：Tauri Command 直连 `rambo-core`。

- **后台常驻（可选）**
  - `launchd` Agent：开机自启；压力事件触发；动作节流。

---

## 4. 能力说明

### 4.1 监控与评估
- 读取 total/free/active/inactive/wired/compressed。
- 计算 **可用内存**、**压缩比**、**压力等级**（`normal/warning/critical`，基于可用、压缩占比、趋势）。

### 4.2 释放策略（分级）
1. **温和级（默认）**：执行 `purge`（仅清文件缓存），记录前后对比。
2. **建议级**：列出“可安全重启”的候选后台进程（浏览器/Electron Helper/模拟器等），不自动杀。
3. **执行级（可选开关）**：对候选进程发 `SIGTERM`，超时后**要求二次确认**方可 `SIGKILL`。
> 坚持“可解释 & 可撤销”原则：每次操作均记录细节与差值。

### 4.3 日志模型（JSON 事件）
```json
{
  "ts": "2025-09-25T10:23:45.123Z",
  "action": "purge|suggest|terminate|noop",
  "before": {"free": 1024, "inactive": 512, "compressed": 800, "wired": 300},
  "after":  {"free": 2100, "inactive": 430, "compressed": 680, "wired": 305},
  "delta_mb": 1076,
  "pressure": "warning",
  "details": {"pids":[{"pid":123,"name":"Chromium Helper","rss_mb":540,"signal":"TERM"}]},
  "trigger": "manual|auto"
}
````

---

## 5. 配置与策略

* **配置文件**：`~/.config/rambo/config.toml`

```toml
[log]
backend = "jsonl"     # or "sqlite"
retain_days = 14

[boost]
enable_terminate = false
candidate_rss_mb = 500
confirm_kill = true
throttle_seconds = 60   # 自动触发节流

[ui]
menu_bar = true
show_delta_chart = true

[detect]
exclude_frontmost = true
whitelist = ["Chromium Helper", "Electron Helper", "qemu-system*"]
blacklist = ["com.apple.*", "WindowServer", "Dock"]
```

* **优先级**：默认 → 配置文件 → 环境变量 → CLI flag。
* **安全网**：前台进程禁止终止；系统关键进程黑名单；强制二次确认。

---

## 6. 里程碑与交付物

### M0 — 项目起步（1–2 天）

* 仓库与 CI（fmt/clippy/test）
* `rambo-core`/`rambo-cli` 骨架，`doctor` 自检（CLT/purge/权限）
* 文档：README、plan.md（本文件）

### M1 — **MVP（CLI 可用）**

* 统计采集（mach + 兜底）与表格输出（`status`）
* 一键 `boost`（仅 `purge`），记录 JSONL，打印前后差值
* `log --today`/`--range` 摘要（次数、平均/中位提升）
* Top N 进程监控（RSS/CPU），`suggest` 候选列表

### M2 — **自动化 & 终止候选**

* `launchd` Agent + `daemon` 子命令（压力事件触发）
* 候选筛选规则（RSS 阈值/前台排除/白黑名单/近活跃度）
* `kill <pid>` 温和终止（TERM→等待→可选 KILL），二次确认

### M3 — **Tauri 菜单栏（可选）**

* 菜单栏图标、`Boost` 按钮、今日日志列表
* 设置页（阈值/白黑名单/自动触发）
* 打包签名/公证脚本

### M4 — **发布与优化**

* Homebrew Tap 配方与自动更新 SHA
* CLI 性能基线（常驻 < 10MB，Boost 95% ≤ 2s）
* 隐私与撤销：日志清理、脱敏选项

---

## 7. 接口与 CLI 规格（草案）

* `rambo status [--json] [--top N]`
* `rambo boost [--no-purge] [--dry-run] [--json]`
* `rambo suggest [--top N] [--json]`
* `rambo kill <pid> [--force] [--json]`
* `rambo log (--today | --range YYYY-MM-DD:YYYY-MM-DD) [--json]`
* `rambo doctor`
* `rambo daemon`（后台守护，用于 launchd 调用）
* 全局：`--config <path>`、`--verbose`、`--quiet`

---

## 8. 质量与测试

* **单测**：统计换算、压力等级、候选筛选、日志序列化。
* **集测**：`boost` 前后差值记录；`suggest/kill` 流程。
* **基准**：常驻 RSS、Boost 耗时（冷/热；多次均值）。
* **工具**：`cargo-nextest` 并行、`criterion` 基准（可选）。

---

## 9. 风险与应对

| 风险               | 说明                 | 对策                       |
| ---------------- | ------------------ | ------------------------ |
| `purge` 不可用/权限失败 | 无 Xcode CLT、返回码非 0 | `doctor` 检测并引导安装；提供仅建议模式 |
| 误杀进程             | 终止后台引发数据丢失         | 默认关闭终止；白/黑名单；二次确认；前台保护   |
| 体验反直觉            | 清缓存后首次访问变慢         | UI/CLI 清晰提示；可关闭自动触发      |
| 高开销              | 常驻占用过高             | 事件驱动、节流、优化枚举频率           |
| 兼容性              | 不同 macOS 版本字段差异    | mach/系统调用做版本分支；兜底库       |

---

## 10. 隐私与合规

* 日志默认**仅本地**，不开启任何上报。
* 提供一键清理日志与脱敏（进程名/命令行可哈希化，默认不脱敏，用户可切换）。
* 许可建议：MIT 或 Apache-2.0。

---

## 11. 路线图（摘要）

* **MVP**：CLI 监控 + `purge` + JSONL
* **Beta**：`launchd` 自动触发 + 候选终止 + `doctor`
* **v1.0**：Tauri 菜单栏 + 设置 + 发布渠道完善
* **v1.x**（增强）：SQLite 报表、快捷键、通知、简单趋势预测、SwiftUI 原生壳（可上架）

---

## 12. 验收指标（KPI）

* 常驻 RSS **< 10MB**；`boost` 动作 **P95 ≤ 2s**。
* `boost` 后**可用内存提升**中位数 ≥ **300MB**（机型差异需注明）。
* 误杀率 0（默认策略下）；用户满意度（问卷/反馈）≥ 4/5。

```
```
