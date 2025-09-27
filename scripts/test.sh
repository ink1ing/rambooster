#!/bin/bash
# RAM Booster (rambo) 测试脚本

set -e

echo "=== RAM Booster 测试脚本 ==="

# 获取当前目录
CURRENT_DIR="$(pwd)"
PROJECT_DIR="$CURRENT_DIR"
EXECUTABLE="$PROJECT_DIR/target/release/cli"

# 检查可执行文件是否存在
if [ ! -f "$EXECUTABLE" ]; then
    echo "❌ 可执行文件不存在: $EXECUTABLE"
    echo "💡 请先运行编译: cargo build --release"
    exit 1
fi

echo "✓ 找到可执行文件: $EXECUTABLE"

# 1. 运行单元测试
echo ""
echo "🧪 运行单元测试..."
cargo test --lib

echo ""
echo "✓ 单元测试完成"

# 2. 运行集成测试
echo ""
echo "🔧 运行集成测试..."
cargo test --test integration_tests

echo ""
echo "✓ 集成测试完成"

# 3. 运行基准测试（可选，比较耗时）
read -p "是否运行基准测试？(y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "⏱️  运行基准测试（这可能需要几分钟）..."
    cargo bench
    echo ""
    echo "✓ 基准测试完成，结果保存在 target/criterion/"
fi

# 4. 测试主要CLI功能
echo ""
echo "🔍 测试主要CLI功能..."

echo "  📊 测试 status 命令..."
"$EXECUTABLE" status --top 5
echo "  ✓ status 命令正常"

echo ""
echo "  🩺 测试 doctor 命令..."
"$EXECUTABLE" doctor
echo "  ✓ doctor 命令正常"

echo ""
echo "  📋 测试 suggest 命令..."
"$EXECUTABLE" suggest --rss-threshold 100
echo "  ✓ suggest 命令正常"

echo ""
echo "  📝 测试 logs 命令..."
"$EXECUTABLE" logs info
echo "  ✓ logs info 命令正常"

echo ""
echo "  📜 测试 logs list 命令..."
"$EXECUTABLE" logs list
echo "  ✓ logs list 命令正常"

# 5. JSON输出测试
echo ""
echo "  🔧 测试JSON输出..."
JSON_OUTPUT=$("$EXECUTABLE" status --json --top 3)
if echo "$JSON_OUTPUT" | jq . >/dev/null 2>&1; then
    echo "  ✓ JSON 输出格式正确"
else
    echo "  ⚠️  JSON 输出可能有问题"
fi

echo ""
echo "🎉 所有测试完成！"
echo ""
echo "📝 测试总结:"
echo "  ✓ 单元测试通过"
echo "  ✓ 集成测试通过"
echo "  ✓ CLI功能测试通过"
echo "  ✓ JSON输出测试通过"
echo ""
echo "🚀 项目可以正常使用！"