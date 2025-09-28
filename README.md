# 🚀 RamBooster

一个专为 macOS 设计的高性能、轻量级内存清理工具，使用 Rust 构建，提供强大的内存优化功能。

<div align="center">

![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Version](https://img.shields.io/badge/version-1.0.0-green.svg)

</div>

## ✨ 核心特性

- **🎯 极致轻量**: 纯 Rust 实现，占用资源极少
- **⚡ 超快执行**: 毫秒级内存清理，即时生效
- **🔥 Killer 模式**: 三轮激进清理 + 进程终止
- **📱 交互终端**: 美观的实时监控界面
- **⚡ 直接执行**: `rb b` 一键快速清理
- **🔐 免密操作**: 自动配置 sudo 权限
- **📊 详细报告**: 多级别数据展示
- **📝 操作日志**: 完整的清理历史记录
- **🔄 一键更新**: 自动从 GitHub 拉取最新版本
- **🗑️ 完全卸载**: 干净彻底的移除工具

## 🚀 快速开始

### 安装

```bash
# 方式一：一键安装（推荐）
git clone https://github.com/ink1ing/ram-booster.git
cd ram-booster
./setup_rb.sh

# 方式二：手动构建
git clone https://github.com/ink1ing/ram-booster.git
cd ram-booster
cargo build --release
./setup_sudo.sh  # 设置权限（避免每次输入密码）
cp target/release/cli ~/.local/bin/rb
```

### 使用方法

```bash
# 🎮 交互模式 - 进入完整的管理界面
rb

# ⚡ 快速清理 - 一键执行 Killer 模式
rb b

# 📊 查看状态
rb status

# 📊 查看帮助
rb --help
```

### 更新和卸载

```bash
# 🔄 更新到最新版本
./update.sh

# 🗑️ 完全卸载
./uninstall.sh
```

## 💪 为什么选择 RamBooster？

### 🆚 相比付费软件的优势

| 特性 | RamBooster | CleanMyMac X | Memory Clean 3 | 其他付费软件 |
|------|------------|--------------|----------------|------------|
| **💰 价格** | ✅ 完全免费 | ❌ $89.95/年 | ❌ $9.99 | ❌ $29.99+ |
| **🔓 开源** | ✅ 完全开源 | ❌ 闭源 | ❌ 闭源 | ❌ 闭源 |
| **🎯 专注性** | ✅ 专业内存清理 | ❌ 功能臃肿 | ⚠️ 功能有限 | ⚠️ 各有限制 |
| **⚡ 性能** | ✅ Rust 极速 | ❌ 资源占用高 | ⚠️ 一般 | ⚠️ 因软件而异 |
| **🛡️ 安全性** | ✅ 代码可审计 | ❌ 黑盒操作 | ❌ 黑盒操作 | ❌ 黑盒操作 |
| **🎛️ 可定制** | ✅ 高度可配置 | ❌ 选项有限 | ❌ 选项有限 | ❌ 选项有限 |
| **📱 终端友好** | ✅ 完美CLI体验 | ❌ 仅GUI | ❌ 仅GUI | ❌ 仅GUI |
| **🔥 Killer模式** | ✅ 独有激进清理 | ❌ 无 | ❌ 无 | ❌ 无 |

### 🎯 核心优势

1. **💸 零成本**: 永久免费，无需订阅或购买许可证
2. **🔍 透明安全**: 开源代码，所有操作完全可见
3. **🚀 性能卓越**: Rust 语言带来的极致性能
4. **🎪 专业专注**: 专门针对内存优化，不做无关功能
5. **🛠️ 高度可控**: 用户完全掌控清理策略和参数
6. **👨‍💻 开发者友好**: 完美的命令行界面，支持脚本集成

## 🎮 详细功能

### 清理级别
- **Low**: 轻度清理，保守策略
- **Mid**: 中等清理，平衡性能
- **High**: 高强度清理，释放更多内存
- **Killer**: 🔥 杀手模式，三轮激进清理 + 进程终止

### Killer 模式工作流程
1. **第一轮**: 标准内存清理
2. **第二轮**: 识别并终止高内存占用进程
3. **第三轮**: 深度系统缓存清理

### 数据显示级别
- **Minimal**: 仅显示关键信息
- **Standard**: 标准详细程度
- **Detailed**: 详细的进程和内存信息
- **Verbose**: 最详细的诊断信息

## 🖥️ 平台兼容性

| 系统 | 版本要求 | 状态 | 说明 |
|------|---------|------|------|
| **macOS** | 10.15+ | ✅ 完全支持 | 主要目标平台 |
| **Linux** | - | 🔄 计划中 | 未来版本支持 |
| **Windows** | - | 🔄 计划中 | 未来版本支持 |

### macOS 特性支持
- ✅ Intel Mac (x86_64)
- ✅ Apple Silicon (ARM64)
- ✅ macOS Monterey (12.0+)
- ✅ macOS Ventura (13.0+)
- ✅ macOS Sonoma (14.0+)
- ✅ macOS Sequoia (15.0+)

## 👥 适用人群

### 🎯 最适合的用户

| 用户类型 | 适用度 | 原因 |
|----------|---------|------|
| **👨‍💻 开发者** | ⭐⭐⭐⭐⭐ | CLI友好、可集成、开源透明 |
| **🎮 重度用户** | ⭐⭐⭐⭐⭐ | Killer模式、高效清理 |
| **💰 预算有限用户** | ⭐⭐⭐⭐⭐ | 完全免费、功能强大 |
| **🔍 隐私敏感用户** | ⭐⭐⭐⭐⭐ | 开源、本地运行、无数据收集 |
| **⚡ 性能追求者** | ⭐⭐⭐⭐⭐ | Rust性能、专业优化 |
| **🛠️ 系统管理员** | ⭐⭐⭐⭐⭐ | 脚本友好、批量部署 |
| **📱 GUI偏好用户** | ⭐⭐⭐ | 终端界面可能需要适应 |
| **🆕 新手用户** | ⭐⭐⭐ | 需要基本终端知识 |

### 🎓 学习价值
- **Rust 学习者**: 优秀的系统编程示例
- **macOS 开发者**: 系统API使用参考
- **性能优化**: 内存管理最佳实践

## 🔧 高级配置

### 更新管理

#### 🔄 自动更新功能
```bash
# 检查并更新到最新版本
./update.sh

# 更新过程包括：
# 1. 自动备份当前版本
# 2. 从 GitHub 拉取最新代码
# 3. 编译并安装新版本
# 4. 验证安装结果
```

#### 🗑️ 完全卸载
```bash
# 彻底移除 RAM Booster
./uninstall.sh

# 卸载内容包括：
# • 主执行文件 (~/.local/bin/rb)
# • 所有备份文件 (~/.local/bin/rb.backup.*)
# • 缓存数据 (~/.cache/ram_booster)
# • 配置文件 (~/.config/ram_booster)
# • 日志文件 (~/Library/Logs/ram_booster)
```

### 自定义清理策略
编辑配置文件来自定义清理行为：
```rust
// 示例配置
boost_level: BoostLevel::Killer,
data_level: DataLevel::Detailed,
auto_terminate_threshold: 1024, // MB
```

### 集成到脚本
```bash
#!/bin/bash
# 定时清理脚本
if [ $(rb status | grep "Available" | awk '{print $2}') -lt 2048 ]; then
    rb b  # 内存不足时自动清理
fi
```

### 版本管理最佳实践
```bash
# 查看当前版本
rb --version

# 定期检查更新（建议每月）
./update.sh

# 重要项目前备份当前版本
cp ~/.local/bin/rb ~/.local/bin/rb.backup.manual

# 测试新版本后确认稳定性
rb status && rb b  # 验证基本功能
```

## 🤝 贡献指南

我们欢迎各种形式的贡献！

1. **🐛 报告问题**: 提交 Issue 描述遇到的问题
2. **✨ 功能建议**: 分享你的想法和需求
3. **🔧 代码贡献**: Fork -> 修改 -> Pull Request
4. **📚 文档改进**: 帮助完善文档和说明
5. **🌍 国际化**: 添加其他语言支持

## 📄 开源协议

本项目采用双重许可证：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

你可以选择其中任意一种协议使用本项目。

## 🙏 致谢

- Rust 社区提供的优秀生态
- macOS 系统 API 文档
- 所有贡献者和用户的反馈

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给个星星！**

Made with ❤️ by Rust & Open Source Community

</div>