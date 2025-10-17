#!/usr/bin/env bash
# RealConsole 卸载脚本
#
# 删除用户级安装的 RealConsole，包括：
#   - 可执行文件: ~/.local/bin/realconsole
#   - 配置目录:   ~/.realconsole/ (可选保留)

set -e

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 安装目录
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.realconsole"
BINARY_NAME="realconsole"

echo -e "${BLUE}=== RealConsole 卸载程序 ===${NC}\n"

# 检查是否安装
if [ ! -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo -e "${YELLOW}⚠ RealConsole 未安装在 $INSTALL_DIR${NC}"
    echo ""
    exit 1
fi

# 显示将要删除的内容
echo "将要删除："
echo "  • 可执行文件: $INSTALL_DIR/$BINARY_NAME"

if [ -d "$CONFIG_DIR" ]; then
    echo "  • 配置目录:   $CONFIG_DIR/ (包含配置和数据)"
fi

echo ""

# 询问是否保留配置
KEEP_CONFIG="n"
if [ -d "$CONFIG_DIR" ]; then
    read -p "是否保留配置和数据文件？[y/N] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        KEEP_CONFIG="y"
    fi
    echo ""
fi

# 确认卸载
read -p "确认卸载 RealConsole？[y/N] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}已取消卸载${NC}"
    exit 0
fi

echo ""
echo "开始卸载..."

# 删除可执行文件
if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    rm "$INSTALL_DIR/$BINARY_NAME"
    echo -e "${GREEN}✓ 已删除可执行文件${NC}"
fi

# 删除配置目录（如果用户选择）
if [ "$KEEP_CONFIG" = "n" ] && [ -d "$CONFIG_DIR" ]; then
    # 再次确认删除配置
    echo ""
    echo -e "${YELLOW}警告: 即将删除所有配置和数据文件！${NC}"
    read -p "最后确认，是否删除 $CONFIG_DIR？[y/N] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        echo -e "${GREEN}✓ 已删除配置目录${NC}"
    else
        echo -e "${YELLOW}已保留配置目录: $CONFIG_DIR${NC}"
    fi
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ RealConsole 卸载完成${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ "$KEEP_CONFIG" = "y" ]; then
    echo "配置已保留在: $CONFIG_DIR"
    echo "如需完全删除，请手动运行："
    echo -e "  ${RED}rm -rf $CONFIG_DIR${NC}"
    echo ""
fi

echo "感谢使用 RealConsole！"
echo ""
