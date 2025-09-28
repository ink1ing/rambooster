#!/bin/bash

# RAM Booster 一键更新脚本
# 从 GitHub 拉取最新版本并替换当前安装

set -e

echo "🔄 RAM Booster 更新程序"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查必要的工具
check_dependencies() {
    local missing_deps=()

    if ! command -v git >/dev/null 2>&1; then
        missing_deps+=("git")
    fi

    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}❌ 缺少必要依赖: ${missing_deps[*]}${NC}"
        echo -e "${YELLOW}请先安装 Git 和 Rust 工具链${NC}"
        exit 1
    fi
}

# 备份当前版本
backup_current() {
    if [ -f ~/.local/bin/rb ]; then
        echo "📦 备份当前版本..."
        cp ~/.local/bin/rb ~/.local/bin/rb.backup.$(date +%Y%m%d_%H%M%S) 2>/dev/null || true
        echo -e "${GREEN}✅ 已备份到 ~/.local/bin/rb.backup.*${NC}"
    fi
}

# 获取最新版本
update_from_github() {
    local temp_dir=$(mktemp -d)
    local repo_url="https://github.com/ink1ing/rambooster.git"

    echo "🌐 从 GitHub 下载最新版本..."

    cd "$temp_dir"
    git clone "$repo_url" rambooster
    cd rambooster

    echo "🔨 编译最新版本..."
    cargo build --release

    # 确保目标目录存在
    mkdir -p ~/.local/bin

    # 安装新版本
    echo "📦 安装新版本..."
    cp target/release/cli ~/.local/bin/rb
    chmod +x ~/.local/bin/rb

    # 同时安装更新和卸载脚本到全局位置
    echo "📦 安装管理脚本..."
    cp update.sh ~/.local/bin/rb-update
    cp uninstall.sh ~/.local/bin/rb-uninstall
    chmod +x ~/.local/bin/rb-update
    chmod +x ~/.local/bin/rb-uninstall

    # 清理临时文件
    cd /
    rm -rf "$temp_dir"

    echo -e "${GREEN}✅ 更新完成！${NC}"
}

# 验证安装
verify_installation() {
    if ~/.local/bin/rb --version >/dev/null 2>&1; then
        local version=$(~/.local/bin/rb --version 2>/dev/null || echo "未知版本")
        echo -e "${GREEN}🎉 RAM Booster 已成功更新！${NC}"
        echo "📊 当前版本: $version"
    else
        echo -e "${RED}❌ 更新可能失败，请检查安装${NC}"
        exit 1
    fi
}

# 显示使用提示
show_usage_tips() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}💡 使用提示:${NC}"
    echo "• 运行程序: rb"
    echo "• 快速清理: rb boost"
    echo "• 查看状态: rb status"
    echo "• 查看帮助: rb --help"
    echo ""
    echo -e "${BLUE}🔧 管理命令 (可在任意目录运行):${NC}"
    echo "• 更新程序: rb-update"
    echo "• 卸载程序: rb-uninstall"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# 主要执行流程
main() {
    echo "🔍 检查依赖..."
    check_dependencies

    echo "💾 备份当前版本..."
    backup_current

    echo "⬇️  更新程序..."
    update_from_github

    echo "✅ 验证安装..."
    verify_installation

    show_usage_tips

    echo -e "${GREEN}🎊 RAM Booster 更新完成！${NC}"
}

# 错误处理
trap 'echo -e "\n${RED}❌ 更新过程中发生错误${NC}"; exit 1' ERR

# 执行主函数
main "$@"