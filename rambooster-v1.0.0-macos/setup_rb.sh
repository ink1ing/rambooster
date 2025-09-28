#!/bin/bash
# 设置 rb 命令的快速安装脚本

echo "🚀 设置 RAM Booster RB 交互式终端"
echo "=================================="

# 获取当前目录
CURRENT_DIR="$(pwd)"
RB_EXECUTABLE="$CURRENT_DIR/target/release/rb"

# 检查rb可执行文件是否存在
if [ ! -f "$RB_EXECUTABLE" ]; then
    echo "❌ rb可执行文件不存在，正在编译..."
    cargo build --release --bin rb
    if [ $? -ne 0 ]; then
        echo "❌ 编译失败"
        exit 1
    fi
    echo "✅ 编译成功"
fi

echo "📁 RB可执行文件: $RB_EXECUTABLE"

# 检查是否可以使用sudo创建符号链接
SYMLINK_TARGET="/usr/local/bin/rb"

echo ""
echo "🔗 设置全局rb命令访问..."
echo "   将创建符号链接: $SYMLINK_TARGET -> $RB_EXECUTABLE"
echo ""

read -p "是否创建全局rb命令? 需要sudo权限 (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -L "$SYMLINK_TARGET" ] || [ -f "$SYMLINK_TARGET" ]; then
        echo "⚠️  符号链接已存在，将删除旧链接"
        sudo rm -f "$SYMLINK_TARGET"
    fi

    sudo ln -sf "$RB_EXECUTABLE" "$SYMLINK_TARGET"

    if [ $? -eq 0 ]; then
        echo "✅ 全局rb命令设置成功！"
        echo "💡 现在你可以在任何地方运行: rb"
    else
        echo "❌ 符号链接创建失败"
        echo "💡 你可以直接运行: $RB_EXECUTABLE"
    fi
else
    echo "💡 跳过全局安装，你可以直接运行: $RB_EXECUTABLE"
fi

echo ""
echo "🎯 快速使用指南:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -L "$SYMLINK_TARGET" ]; then
    echo "🚀 启动: rb"
else
    echo "🚀 启动: $RB_EXECUTABLE"
fi
echo "💾 查看内存: 输入 'status'"
echo "🔄 清理内存: 输入 'boost'"
echo "⚙️  设置强度: 输入 '/level [light|standard|aggressive]'"
echo "📊 设置详细度: 输入 '/data [minimal|standard|detailed|verbose]'"
echo "📤 导出结果: 输入 '/export [json|csv|txt|markdown]'"
echo "❓ 查看帮助: 输入 'help'"
echo "🚪 退出程序: 输入 'exit'"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎉 设置完成！享受使用 RAM Booster 交互式终端！"