#!/bin/bash
# 设置 purge 命令免密执行

echo "🔐 设置 purge 命令免密执行"
echo "================================="

USERNAME=$(whoami)
SUDOERS_FILE="/etc/sudoers.d/rambooster"

echo "正在为用户 $USERNAME 设置 purge 命令免密权限..."

# 创建 sudoers 规则
sudo tee "$SUDOERS_FILE" > /dev/null << EOF
# RAM Booster purge 命令免密规则
$USERNAME ALL=(ALL) NOPASSWD: /usr/sbin/purge
EOF

# 验证语法
if sudo visudo -c -f "$SUDOERS_FILE"; then
    echo "✅ sudoers 规则创建成功"
    echo "📝 文件位置: $SUDOERS_FILE"
    echo "🎯 现在 purge 命令可以免密执行"
else
    echo "❌ sudoers 规则创建失败"
    sudo rm -f "$SUDOERS_FILE"
    exit 1
fi

echo ""
echo "🧪 测试免密执行..."
if sudo -n /usr/sbin/purge; then
    echo "✅ 免密执行测试成功"
else
    echo "⚠️  免密执行测试失败，可能需要重启终端"
fi

echo ""
echo "📋 如需撤销免密设置:"
echo "sudo rm $SUDOERS_FILE"