```markdown
# RAM-Booster (rambo) — 任务清单（TODO.md）

## 0. 项目初始化（已完成）
- [x] 创建仓库结构：`/crates/core`、`/crates/cli`、`/docs/`、`/scripts/`
- [x] 添加 Rust 工具链检查（rustup、cargo、clippy、fmt）
- [x] 配置 CI（lint + test）
- [x] 添加 LICENSE（MIT/Apache-2.0）和 README

---

## 1. 内存统计核心（已完成）
- [x] 封装 `MemStats` 结构
- [x] 实现 `read_mem_stats()`：调用 `mach` → `host_statistics64`
- [x] 计算各类内存页数（active/inactive/wired/compressed/free）
- [x] 换算 MB 与总量校准
- [x] 压力等级推导函数（normal / warning / critical）
- [x] 集成 `sysinfo` 作为兜底（feature flag）

---

## 2. 进程信息采集（已完成）
- [x] 使用 `libproc` → `proc_listpids` 获取 pid 列表
- [x] 使用 `proc_pid_rusage` 获取 RSS、CPU 时间
- [x] 解析进程名，结合命令行参数
- [x] 前台判定：通过 `NSWorkspace` / `CGWindowList`
- [x] 排序：按 RSS 降序，取 Top N

---

## 3. 日志系统（已完成）
- [x] 定义日志数据结构（JSON schema）
- [x] 实现 JSONL 写入：`~/.local/share/rambo/logs/YYYY-MM-DD.jsonl`
- [x] 日志滚动：按日新建文件
- [x] 可选 SQLite：`rusqlite`，建表 `events`
- [x] 实现 `log_event(action, before, after, delta, details)`

---

## 4. 释放策略（已完成）
- [x] 封装 `purge()`：执行 `/usr/bin/purge`，返回结果与耗时
- [x] 错误处理：检测 CLT 未安装 → 降级提示
- [x] 候选进程筛选：RSS > 阈值、非前台、白/黑名单校验
- [x] 封装 `terminate(pid)`：SIGTERM → 等待 → 可选 SIGKILL（二次确认） (已完成)
- [x] 封装 `boost()`：组合动作（before/after → 日志 → delta_mb） (已完成)

---

## 5. CLI 子命令（已完成）
- [x] `rambo status [--json] [--top N]`：打印内存概览 + Top N 进程 (已完成)
- [x] `rambo boost [--json]`：执行 `purge`，输出前后差值 (已完成)
- [x] `rambo suggest [--json]`：列出候选进程 (已完成)
- [x] `rambo kill <pid>`：确认后温和终止 (已完成)
- [x] `rambo log --today|--range`：输出摘要/详情 (已完成)
- [x] `rambo doctor`：检查 purge、权限、launchd 状态 (已完成)
- [x] `rambo daemon`：后台守护，监听压力事件 (已完成)

---

## 6. 配置管理（已完成）
- [x] 定义配置文件路径：`~/.config/rambo/config.toml` (已完成)
- [x] 加载顺序：默认值 → 文件 → 环境变量 → CLI flag (已完成 - 2025-09-27 14:35 北京时间)
- [x] 配置项：日志后端、保留天数、RSS 阈值、是否启用终止、节流间隔、白/黑名单 (已完成 - 2025-09-27 14:35 北京时间)
- [x] 提供 `doctor` 输出当前配置 (已完成 - 2025-09-27 14:35 北京时间)

---

## 7. 后台与自动触发（已完成）
- [x] 封装 `daemon` 子命令 (已完成 - 2025-09-27 14:40 北京时间)
- [x] 监听 GCD 内存压力事件（normal / warning / critical） (已完成 - 2025-09-27 14:40 北京时间)
- [x] 压力升高时触发 `boost()` 或提醒 (已完成 - 2025-09-27 14:40 北京时间)
- [x] 节流控制：避免短时间重复执行 (已完成 - 2025-09-27 14:40 北京时间)
- [x] 安装/卸载 `launchd` Agent 脚本 (已完成 - 2025-09-27 14:40 北京时间)

---

## 8. 菜单栏 UI（可选）
- [ ] 初始化 Tauri 项目（菜单栏模式）
- [ ] 实现 Tauri Command：调用 `rambo-core` 的 `status/boost/suggest`
- [ ] 菜单栏：显示可用内存百分比
- [ ] 面板：Boost 按钮、今日记录、设置项
- [ ] 打包 DMG + 签名/公证脚本

---

## 9. 质量保证 & 本地可用性（已完成 - 2025-09-27 18:00 北京时间）
- [x] 单元测试：统计换算、日志序列化、候选筛选 (已完成 - 2025-09-27 14:30 北京时间)
- [x] 集成测试：`boost` 前后差值、日志写入 (已完成 - 2025-09-27 17:30 北京时间)
- [x] 基准测试：常驻 RSS、`boost` 耗时（冷/热） (已完成 - 2025-09-27 17:35 北京时间)
- [x] 文档化 API 与 CLI 用例 (已完成 - 2025-09-27 17:45 北京时间)
- [x] 本地安装和测试脚本 (已完成 - 2025-09-27 17:55 北京时间)
- [x] 完整功能验证和使用指南 (已完成 - 2025-09-27 18:00 北京时间)
- [ ] CI 集成 `cargo-nextest`、覆盖率统计

---

## 10. 发布与分发
- [ ] Homebrew Tap 配方（rambo.rb）
- [ ] CI 自动更新 formula SHA
- [ ] GUI：签名、公证、DMG 发布
- [ ] 文档：README 使用指南、FAQ、风险提示、隐私声明
- [ ] 截图/GIF 展示 CLI 与 UI 效果

---

## 11. 安全与隐私
- [x] 默认关闭进程终止功能 (已完成 - 2025-09-27 15:00 北京时间)
- [x] 开启时需二次确认，前台/系统进程强制保护 (已完成 - 2025-09-27 15:00 北京时间)
- [x] 日志仅本地存储；提供一键清理功能 (已完成 - 2025-09-27 17:50 北京时间)
- [ ] 可配置脱敏模式（哈希化进程名）

---

## 12. Backlog（未来增强）
- [ ] 内存压力趋势预测（滑窗统计）
- [ ] 浏览器标签级占用提示
- [ ] Tauri 通知 & 快捷键支持
- [ ] SwiftUI 原生壳（上架 App Store）
- [ ] 多语言支持（中文/英文）
```


