# RAM Booster v1.2.0 安装指南

## 🚀 快速安装

1. **解压文件到任意目录**
2. **运行安装脚本**:
   ```bash
   cd rambooster-v1.2.0-macos
   ./setup_rb.sh
   ```

## ✨ 新功能亮点

### 🌍 全局命令支持
- **rb-update**: 在任意目录更新程序
- **rb-uninstall**: 在任意目录卸载程序
- **自动PATH配置**: 安装时自动配置环境变量

### 📱 版本显示
- 启动时显示当前版本号 v1.2.0
- 快速清理模式也显示版本信息

### 🔧 增强安装
- 智能检测Shell类型 (zsh/bash/fish)
- 自动配置PATH环境变量
- 完整的安装后验证

## 💡 使用方法

### 基本命令
```bash
# 启动交互模式
rb

# 快速清理
rb b

# 查看状态
rb status

# 查看帮助
rb --help
```

### 全局管理命令
```bash
# 更新到最新版本 (任意目录可执行)
rb-update

# 完全卸载程序 (任意目录可执行)
rb-uninstall
```

## 🎯 系统要求

- macOS 10.15+
- Intel Mac (x86_64) 或 Apple Silicon (ARM64)
- Xcode Command Line Tools (推荐)

## 🆘 故障排除

如果遇到问题：

1. **确保有执行权限**:
   ```bash
   chmod +x setup_rb.sh
   ```

2. **手动安装Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```

3. **重新加载shell配置**:
   ```bash
   source ~/.zshrc  # 或 ~/.bashrc
   ```

---

🚀 **Happy Memory Boosting!**