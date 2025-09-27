#!/bin/bash
# 设置 Control+R+B 快捷键的脚本

echo "⌨️  RAM Booster 快捷键设置"
echo "================================="

RB_PATH="$(pwd)/target/release/rb"

echo "正在创建快捷键脚本..."

# 创建快捷键脚本
cat > /tmp/rb_shortcut.sh << EOF
#!/bin/bash
# RAM Booster 快捷键脚本
osascript -e '
tell application "Terminal"
    activate
    do script "$RB_PATH b"
end tell
'
EOF

chmod +x /tmp/rb_shortcut.sh

echo "📋 快捷键设置说明:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 打开 系统偏好设置 > 键盘 > 快捷键"
echo "2. 选择 '服务' 或 '应用快捷键'"
echo "3. 添加新快捷键:"
echo "   应用程序: 终端.app 或 所有应用程序"
echo "   快捷键: Control+R+B"
echo "   脚本路径: /tmp/rb_shortcut.sh"
echo ""
echo "💡 或者使用 Automator 创建快速操作:"
echo "   1. 打开 Automator"
echo "   2. 创建新的 '快速操作'"
echo "   3. 添加 '运行Shell脚本' 操作"
echo "   4. 输入: $RB_PATH b"
echo "   5. 保存为 'RAM Booster'"
echo "   6. 在系统偏好设置中设置快捷键 Control+R+B"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ 快捷键脚本已创建: /tmp/rb_shortcut.sh"
echo "🔗 现在可以按照上述说明设置系统快捷键"