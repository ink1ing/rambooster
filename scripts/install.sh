#!/bin/bash
# RAM Booster (rambo) 本地安装脚本

set -e

echo "=== RAM Booster 本地安装脚本 ==="

# 检查 Rust 环境
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ 错误: 未找到 cargo。请先安装 Rust: https://rustup.rs/"
    exit 1
fi

echo "✓ 检测到 Rust 环境"

# 获取当前目录
CURRENT_DIR="$(pwd)"
PROJECT_DIR="$CURRENT_DIR"

echo "📁 项目目录: $PROJECT_DIR"

# 编译项目
echo "🔨 编译项目..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi

echo "✓ 编译成功"

# 检查可执行文件
EXECUTABLE="$PROJECT_DIR/target/release/cli"
if [ ! -f "$EXECUTABLE" ]; then
    echo "❌ 可执行文件不存在: $EXECUTABLE"
    exit 1
fi

# 创建符号链接到 /usr/local/bin (需要 sudo)
SYMLINK_TARGET="/usr/local/bin/rambo"

echo "🔗 创建符号链接..."
echo "   目标: $SYMLINK_TARGET -> $EXECUTABLE"

if [ -L "$SYMLINK_TARGET" ] || [ -f "$SYMLINK_TARGET" ]; then
    echo "⚠️  符号链接已存在，将删除旧链接"
    sudo rm -f "$SYMLINK_TARGET"
fi

sudo ln -sf "$EXECUTABLE" "$SYMLINK_TARGET"

if [ $? -eq 0 ]; then
    echo "✓ 符号链接创建成功"
else
    echo "❌ 符号链接创建失败"
    echo "💡 你可以手动运行: $EXECUTABLE"
fi

# 检查 Xcode Command Line Tools
echo ""
echo "🔍 检查系统依赖..."
if [ -f "/usr/bin/purge" ]; then
    echo "✓ Xcode Command Line Tools 已安装"
else
    echo "⚠️  未找到 /usr/bin/purge"
    echo "   内存释放功能需要 Xcode Command Line Tools"
    echo "   安装命令: xcode-select --install"
fi

# 运行诊断
echo ""
echo "🩺 运行系统诊断..."
"$EXECUTABLE" doctor

echo ""
echo "🎉 安装完成！"
echo ""
echo "📚 使用方法:"
echo "   rambo status          # 查看内存状态"
echo "   rambo boost           # 释放内存"
echo "   rambo suggest         # 建议终止的进程"
echo "   rambo logs info       # 查看日志信息"
echo "   rambo doctor          # 系统诊断"
echo "   rambo --help          # 查看完整帮助"
echo ""
echo "📖 详细文档: docs/USAGE.md"