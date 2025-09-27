#!/bin/bash
# RAM Booster RB 交互式终端演示脚本

echo "🎯 RAM Booster RB 交互式终端演示"
echo "================================"
echo ""

echo "💡 启动rb命令后，你可以："
echo ""
echo "🚀 基本命令:"
echo "   rb> status          # 查看内存状态"
echo "   rb> boost           # 执行内存清理"
echo "   rb> help            # 查看帮助"
echo "   rb> clear           # 清屏"
echo "   rb> exit            # 退出"
echo ""
echo "⚙️  配置命令:"
echo "   rb> /level light    # 设置轻度清理"
echo "   rb> /level standard # 设置标准清理"
echo "   rb> /level aggressive # 设置激进清理"
echo ""
echo "   rb> /data minimal   # 最少信息显示"
echo "   rb> /data standard  # 标准信息显示"
echo "   rb> /data detailed  # 详细信息显示"
echo "   rb> /data verbose   # 冗长信息显示"
echo ""
echo "📤 导出命令:"
echo "   rb> /export json    # 导出JSON格式"
echo "   rb> /export csv     # 导出CSV格式"
echo "   rb> /export txt     # 导出TXT格式"
echo "   rb> /export markdown # 导出Markdown格式"
echo ""
echo "📋 其他命令:"
echo "   rb> /history        # 查看命令历史"
echo "   rb> /logs info      # 查看日志信息"
echo "   rb> /logs list      # 列出日志文件"
echo ""

read -p "🎮 现在启动rb交互式终端吗? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🚀 启动中..."
    echo ""
    ./target/release/rb
else
    echo "💡 你可以随时运行: ./target/release/rb"
fi