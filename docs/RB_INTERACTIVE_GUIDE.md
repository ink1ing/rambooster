# RB 交互式终端使用说明书

## 概述

RB（RAM Booster）交互式终端是一个强大的 macOS 内存管理工具，提供直观的命令行界面进行实时内存监控、清理和优化。

## 🚀 快速开始

### 安装
```bash
# 进入项目目录
cd "/Users/inkling/Desktop/ram booster"

# 运行安装脚本（自动编译和设置全局命令）
./setup_rb.sh
```

### 启动
```bash
# 方式一：全局命令（推荐）
rb

# 方式二：直接运行
./target/release/rb
```

## 📋 界面说明

启动后您将看到：
```
🚀 RAM Booster 交互式终端
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💡 输入 'boost' 开始清理内存
📊 输入 'status' 查看当前状态
⚙️  输入 '/help' 查看所有命令
🚪 输入 'exit' 或 'quit' 退出
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💙 📈 rb>
```

### 提示符说明
- `💚` = 轻度清理模式
- `💙` = 标准清理模式（默认）
- `💜` = 激进清理模式

- `📊` = 最少信息显示
- `📈` = 标准信息显示（默认）
- `📋` = 详细信息显示
- `📜` = 冗长信息显示

## 🎯 核心命令

### boost
执行内存清理操作
```bash
rb> boost
```
- 根据当前清理强度进行内存释放
- 显示清理前后的内存状态对比
- 记录清理日志

### status
查看当前内存状态
```bash
rb> status
```
- 显示总内存、可用内存、活跃内存等
- 根据数据详细度显示不同级别的信息
- 显示内存压力级别和进程排行

### clear
清屏
```bash
rb> clear
```

### exit / quit / q
退出程序
```bash
rb> exit
```

### help / ?
显示帮助信息
```bash
rb> help
```

## ⚙️ 配置命令

### /level - 设置清理强度
```bash
rb> /level light        # 轻度清理
rb> /level standard     # 标准清理（默认）
rb> /level aggressive   # 激进清理

# 简写形式
rb> /level l    # 轻度
rb> /level s    # 标准
rb> /level a    # 激进

# 查看当前设置
rb> /level
```

**清理强度说明：**
- **Light（轻度）**：温和的内存清理，适合日常使用
- **Standard（标准）**：平衡的清理强度，推荐设置
- **Aggressive（激进）**：最大程度的内存释放

### /data - 设置显示详细度
```bash
rb> /data minimal     # 最少信息
rb> /data standard    # 标准信息（默认）
rb> /data detailed    # 详细信息
rb> /data verbose     # 冗长信息

# 简写形式
rb> /data m    # 最少
rb> /data s    # 标准
rb> /data d    # 详细
rb> /data v    # 冗长

# 查看当前设置
rb> /data
```

**显示级别对比：**
- **Minimal**：仅显示基本内存和压力信息
- **Standard**：显示内存详情 + Top 5 进程
- **Detailed**：显示完整内存信息 + Top 10 进程
- **Verbose**：显示全面报告 + Top 15 进程 + 候选清理进程

## 📤 导出功能

### /export - 导出清理结果
```bash
rb> /export json       # JSON 格式
rb> /export csv        # CSV 格式
rb> /export txt        # TXT 格式
rb> /export markdown   # Markdown 格式

# 简写形式
rb> /export j    # JSON
rb> /export c    # CSV
rb> /export t    # TXT
rb> /export md   # Markdown

# 查看可用格式
rb> /export
```

**注意：** 需要先执行 `boost` 命令才能导出结果

**导出文件格式：**
- 文件名：`boost_result_YYYYMMDD_HHMMSS.格式`
- 保存位置：当前工作目录

## 📋 其他功能

### /history - 查看命令历史
```bash
rb> /history
```
显示本次会话的所有命令记录

### /logs - 日志管理
```bash
rb> /logs info    # 查看日志信息（大小、文件数量）
rb> /logs list    # 列出所有日志文件
```

### /help - 显示帮助
```bash
rb> /help
```
显示所有可用命令的详细说明

## 🔄 典型使用流程

### 1. 基础内存检查和清理
```bash
rb> status              # 查看当前状态
rb> boost               # 执行清理
rb> status              # 查看清理效果
```

### 2. 自定义清理设置
```bash
rb> /level aggressive   # 设置激进清理
rb> /data verbose       # 设置详细显示
rb> boost               # 执行清理
rb> /export json        # 导出结果
```

### 3. 监控和分析
```bash
rb> /data detailed      # 设置详细显示
rb> status              # 查看进程排行
rb> /logs info          # 查看日志统计
rb> /history            # 查看操作历史
```

## 🛠️ 安装和卸载

### 完整安装（推荐）
```bash
# 运行安装脚本
./setup_rb.sh

# 选择 'y' 创建全局命令
# 现在可以在任意目录使用 'rb' 命令
```

### 仅编译（不安装全局命令）
```bash
cargo build --release --bin rb
# 使用 ./target/release/rb 启动
```

### 卸载
```bash
# 删除全局命令
sudo rm /usr/local/bin/rb

# 完全删除项目（可选）
rm -rf "/Users/inkling/Desktop/ram booster"
```

## ⚠️ 注意事项

### 系统要求
- macOS 系统
- 已安装 Xcode Command Line Tools
- Rust 环境（用于编译）

### 安全性
- 程序仅使用 macOS 官方的 `/usr/bin/purge` 命令
- 不会修改或删除用户文件
- 可随时安全退出

### 权限
- 内存清理无需 sudo 权限
- 创建全局命令需要 sudo 权限（仅安装时）

## 🚨 故障排除

### 命令找不到
```bash
# 错误：rb: command not found
# 解决：使用完整路径或重新运行安装脚本
./target/release/rb
# 或
./setup_rb.sh
```

### purge 命令失败
```bash
# 错误：CommandNotFound
# 解决：安装 Xcode Command Line Tools
xcode-select --install
```

### 权限被拒绝
```bash
# 错误：permission denied
# 解决：添加执行权限
chmod +x setup_rb.sh
```

## 📊 日志和数据

### 日志位置
日志文件存储在项目目录的 `logs/` 文件夹中

### 导出文件位置
导出的清理报告保存在当前工作目录

### 数据格式
所有数据采用标准化格式，支持进一步分析和处理

---

**版本：** v0.1.0
**更新日期：** 2024年9月27日
**支持：** macOS 系统