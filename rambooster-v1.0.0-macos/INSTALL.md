# 🚀 RAM Booster v1.0.0 安装指南

## 快速安装

1. **解压下载的文件**
   ```bash
   unzip rambooster-v1.0.0-macos.zip
   cd rambooster-v1.0.0-macos
   ```

2. **设置权限**
   ```bash
   chmod +x rb cli setup_sudo.sh setup_rb.sh
   ```

3. **配置sudo权限（避免输入密码）**
   ```bash
   ./setup_sudo.sh
   ```

4. **全局安装（推荐）**
   ```bash
   ./setup_rb.sh
   ```

## 使用方法

### 🎮 交互模式
```bash
rb
```
进入完整的交互式终端界面，支持所有功能。

### ⚡ 快速清理
```bash
rb b
```
一键执行 Killer 模式清理，无需进入交互界面。

### 📊 查看帮助
```bash
rb --help
```

## 功能特色

- **💀 Killer 模式**: 三轮激进清理 + 进程终止
- **🎨 Rust 主题**: 美观的螃蟹红色 ASCII 横幅
- **📊 多级显示**: 4种可视化级别
- **⚡ 超快执行**: 纯 Rust 实现
- **🔐 免密操作**: 自动配置 sudo 权限

## 系统要求

- macOS 10.15+ (Catalina 或更高版本)
- Intel Mac 或 Apple Silicon 均支持

---

💡 如需帮助或反馈：https://github.com/ink1ing/rambooster
