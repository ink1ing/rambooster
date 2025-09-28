#!/bin/bash
# RAM Booster 完整安装脚本 - 支持全局执行

echo "🚀 RAM Booster 完整安装程序"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 获取当前目录
CURRENT_DIR="$(pwd)"
RB_EXECUTABLE="$CURRENT_DIR/target/release/cli"

# 检查并编译可执行文件
compile_project() {
    if [ ! -f "$RB_EXECUTABLE" ]; then
        echo -e "${YELLOW}🔨 正在编译 RAM Booster...${NC}"
        cargo build --release
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ 编译失败${NC}"
            exit 1
        fi
        echo -e "${GREEN}✅ 编译成功${NC}"
    else
        echo -e "${GREEN}✅ 可执行文件已存在${NC}"
    fi
    echo -e "${BLUE}📁 可执行文件: $RB_EXECUTABLE${NC}"
}

# 安装到用户本地目录
install_local() {
    echo -e "${YELLOW}📦 安装到用户目录...${NC}"

    # 确保目录存在
    mkdir -p ~/.local/bin

    # 复制主程序
    cp "$RB_EXECUTABLE" ~/.local/bin/rb
    chmod +x ~/.local/bin/rb
    echo -e "${GREEN}✅ 主程序已安装到 ~/.local/bin/rb${NC}"

    # 复制管理脚本
    if [ -f "$CURRENT_DIR/update.sh" ]; then
        cp "$CURRENT_DIR/update.sh" ~/.local/bin/rb-update
        chmod +x ~/.local/bin/rb-update
        echo -e "${GREEN}✅ 更新脚本已安装到 ~/.local/bin/rb-update${NC}"
    fi

    if [ -f "$CURRENT_DIR/uninstall.sh" ]; then
        cp "$CURRENT_DIR/uninstall.sh" ~/.local/bin/rb-uninstall
        chmod +x ~/.local/bin/rb-uninstall
        echo -e "${GREEN}✅ 卸载脚本已安装到 ~/.local/bin/rb-uninstall${NC}"
    fi
}

# 检查 PATH 配置
check_path() {
    echo -e "${YELLOW}🔍 检查 PATH 配置...${NC}"

    # 检查 ~/.local/bin 是否在 PATH 中
    if [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
        echo -e "${GREEN}✅ ~/.local/bin 已在 PATH 中${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  ~/.local/bin 不在 PATH 中${NC}"
        return 1
    fi
}

# 添加 PATH 配置
setup_path() {
    local shell_rc=""
    case $SHELL in
        */zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        */bash)
            shell_rc="$HOME/.bashrc"
            if [ ! -f "$shell_rc" ]; then
                shell_rc="$HOME/.bash_profile"
            fi
            ;;
        */fish)
            shell_rc="$HOME/.config/fish/config.fish"
            mkdir -p "$(dirname "$shell_rc")"
            ;;
        *)
            echo -e "${YELLOW}⚠️  未识别的 shell: $SHELL${NC}"
            echo -e "${YELLOW}请手动添加 ~/.local/bin 到 PATH${NC}"
            return 1
            ;;
    esac

    echo -e "${BLUE}📝 添加 PATH 配置到 $shell_rc${NC}"

    if [[ "$SHELL" == */fish ]]; then
        echo 'fish_add_path ~/.local/bin' >> "$shell_rc"
    else
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
    fi

    echo -e "${GREEN}✅ PATH 配置已添加${NC}"
    echo -e "${YELLOW}💡 请运行以下命令或重启终端生效:${NC}"
    echo -e "${BLUE}    source $shell_rc${NC}"
}

# 显示使用指南
show_usage_guide() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}🎉 RAM Booster 安装完成！${NC}"
    echo ""
    echo -e "${YELLOW}💡 基本使用:${NC}"
    echo "• 启动程序: rb"
    echo "• 快速清理: rb boost"
    echo "• 查看状态: rb status"
    echo "• 查看帮助: rb --help"
    echo ""
    echo -e "${BLUE}🔧 管理命令 (可在任意目录运行):${NC}"
    echo "• 更新程序: rb-update"
    echo "• 卸载程序: rb-uninstall"
    echo ""
    echo -e "${YELLOW}🎮 交互模式命令:${NC}"
    echo "• 设置强度: /level [low|mid|high|killer]"
    echo "• 设置详细度: /data [minimal|standard|detailed|verbose]"
    echo "• 导出结果: /export [json|csv|txt|markdown]"
    echo "• 查看帮助: /help"
    echo "• 退出程序: exit"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# 主函数
main() {
    echo -e "${BLUE}🔍 开始安装 RAM Booster...${NC}"

    # 编译项目
    compile_project

    # 本地安装
    install_local

    # 检查和配置 PATH
    if ! check_path; then
        echo -e "${YELLOW}是否自动配置 PATH? (推荐) [Y/n]:${NC}"
        read -r response
        if [[ "$response" =~ ^[Nn]$ ]]; then
            echo -e "${YELLOW}跳过 PATH 配置，你需要手动添加 ~/.local/bin 到 PATH${NC}"
        else
            setup_path
        fi
    fi

    # 显示使用指南
    show_usage_guide
}

# 错误处理
trap 'echo -e "\n${RED}❌ 安装过程中发生错误${NC}"; exit 1' ERR

# 执行主函数
main "$@"