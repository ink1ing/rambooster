#!/bin/bash

# RAM Booster 一键卸载脚本
# 完全移除 RAM Booster 及其所有文件

set -e

echo "🗑️  RAM Booster 卸载程序"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查是否安装了 RAM Booster
check_installation() {
    local found_files=()

    # 检查主执行文件
    if [ -f ~/.local/bin/rb ]; then
        found_files+=("~/.local/bin/rb")
    fi

    # 检查全局管理脚本
    if [ -f ~/.local/bin/rb-update ]; then
        found_files+=("~/.local/bin/rb-update")
    fi

    if [ -f ~/.local/bin/rb-uninstall ]; then
        found_files+=("~/.local/bin/rb-uninstall")
    fi

    # 检查备份文件
    if ls ~/.local/bin/rb.backup.* >/dev/null 2>&1; then
        found_files+=("备份文件")
    fi

    # 检查日志文件
    if [ -d ~/.cache/ram_booster ]; then
        found_files+=("~/.cache/ram_booster")
    fi

    # 检查配置文件
    if [ -d ~/.config/ram_booster ]; then
        found_files+=("~/.config/ram_booster")
    fi

    if [ ${#found_files[@]} -eq 0 ]; then
        echo -e "${YELLOW}ℹ️  未检测到 RAM Booster 安装${NC}"
        echo "可能已经卸载，或安装在其他位置"
        exit 0
    fi

    echo -e "${BLUE}📋 检测到以下 RAM Booster 文件:${NC}"
    for file in "${found_files[@]}"; do
        echo "  • $file"
    done
    echo ""
}

# 确认卸载
confirm_uninstall() {
    echo -e "${YELLOW}⚠️  警告: 这将完全移除 RAM Booster 及其所有数据${NC}"
    echo "包括:"
    echo "  • 主执行文件 (rb)"
    echo "  • 全局管理脚本 (rb-update, rb-uninstall)"
    echo "  • 所有备份文件"
    echo "  • 日志和缓存数据"
    echo "  • 配置文件"
    echo ""

    read -p "确定要卸载吗? (y/N): " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${GREEN}✅ 卸载已取消${NC}"
        exit 0
    fi
}

# 移除主执行文件
remove_executable() {
    echo "🗑️  移除主执行文件..."

    if [ -f ~/.local/bin/rb ]; then
        rm -f ~/.local/bin/rb
        echo -e "${GREEN}✅ 已移除 ~/.local/bin/rb${NC}"
    fi

    # 移除全局管理脚本
    if [ -f ~/.local/bin/rb-update ]; then
        rm -f ~/.local/bin/rb-update
        echo -e "${GREEN}✅ 已移除 ~/.local/bin/rb-update${NC}"
    fi

    if [ -f ~/.local/bin/rb-uninstall ]; then
        rm -f ~/.local/bin/rb-uninstall
        echo -e "${GREEN}✅ 已移除 ~/.local/bin/rb-uninstall${NC}"
    fi
}

# 移除备份文件
remove_backups() {
    echo "🗑️  移除备份文件..."

    local backup_count=0
    for backup in ~/.local/bin/rb.backup.*; do
        if [ -f "$backup" ]; then
            rm -f "$backup"
            ((backup_count++))
        fi
    done

    if [ $backup_count -gt 0 ]; then
        echo -e "${GREEN}✅ 已移除 $backup_count 个备份文件${NC}"
    else
        echo "ℹ️  未找到备份文件"
    fi
}

# 移除数据文件
remove_data() {
    echo "🗑️  移除数据和缓存文件..."

    # 移除缓存目录
    if [ -d ~/.cache/ram_booster ]; then
        rm -rf ~/.cache/ram_booster
        echo -e "${GREEN}✅ 已移除缓存目录 ~/.cache/ram_booster${NC}"
    fi

    # 移除配置目录
    if [ -d ~/.config/ram_booster ]; then
        rm -rf ~/.config/ram_booster
        echo -e "${GREEN}✅ 已移除配置目录 ~/.config/ram_booster${NC}"
    fi

    # 移除可能的日志文件
    if [ -d ~/Library/Logs/ram_booster ]; then
        rm -rf ~/Library/Logs/ram_booster
        echo -e "${GREEN}✅ 已移除日志目录 ~/Library/Logs/ram_booster${NC}"
    fi
}

# 清理 PATH 环境变量提示
cleanup_path() {
    echo "🔧 检查 PATH 环境变量..."

    local shell_rc=""
    case $SHELL in
        */zsh)
            shell_rc="~/.zshrc"
            ;;
        */bash)
            shell_rc="~/.bashrc 或 ~/.bash_profile"
            ;;
        */fish)
            shell_rc="~/.config/fish/config.fish"
            ;;
    esac

    if [ -n "$shell_rc" ]; then
        echo -e "${YELLOW}💡 提示: 如果您手动添加了 ~/.local/bin 到 PATH，${NC}"
        echo -e "${YELLOW}    可考虑从 $shell_rc 中移除相关配置${NC}"
    fi
}

# 验证卸载
verify_uninstall() {
    echo "✅ 验证卸载结果..."

    local remaining_files=()

    if [ -f ~/.local/bin/rb ]; then
        remaining_files+=("~/.local/bin/rb")
    fi

    if [ -f ~/.local/bin/rb-update ]; then
        remaining_files+=("~/.local/bin/rb-update")
    fi

    if [ -f ~/.local/bin/rb-uninstall ]; then
        remaining_files+=("~/.local/bin/rb-uninstall")
    fi

    if [ -d ~/.cache/ram_booster ]; then
        remaining_files+=("~/.cache/ram_booster")
    fi

    if [ -d ~/.config/ram_booster ]; then
        remaining_files+=("~/.config/ram_booster")
    fi

    if [ ${#remaining_files[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ RAM Booster 已完全卸载${NC}"
        return 0
    else
        echo -e "${RED}⚠️  以下文件可能未完全移除:${NC}"
        for file in "${remaining_files[@]}"; do
            echo "  • $file"
        done
        echo -e "${YELLOW}请手动检查并删除${NC}"
        return 1
    fi
}

# 显示卸载完成信息
show_completion() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}🎉 RAM Booster 卸载完成！${NC}"
    echo ""
    echo -e "${BLUE}如果将来需要重新安装:${NC}"
    echo "• 克隆仓库: git clone https://github.com/ink1ing/rambooster.git"
    echo "• 或下载预编译版本"
    echo ""
    echo -e "${YELLOW}感谢您使用 RAM Booster！${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# 主要执行流程
main() {
    echo "🔍 检查 RAM Booster 安装..."
    check_installation

    confirm_uninstall

    echo ""
    echo "🗑️  开始卸载 RAM Booster..."

    remove_executable
    remove_backups
    remove_data
    cleanup_path

    echo ""
    verify_uninstall

    show_completion
}

# 错误处理
trap 'echo -e "\n${RED}❌ 卸载过程中发生错误${NC}"; exit 1' ERR

# 执行主函数
main "$@"