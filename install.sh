#!/usr/bin/env bash
# RealConsole 用户级安装脚本
#
# 本脚本将 RealConsole 安装到当前用户的本地目录：
#   - 可执行文件: ~/.local/bin/realconsole
#   - 配置目录:   ~/.realconsole/
#   - 数据目录:   ~/.realconsole/memory/
#
# 不需要 root 权限，不影响其他用户

set -e

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 安装目录
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.realconsole"
BINARY_NAME="realconsole"

echo -e "${GREEN}=== RealConsole 用户级安装 ===${NC}\n"

# 检查是否已编译
if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo -e "${YELLOW}⚠ 未找到编译后的可执行文件${NC}"
    echo "正在编译 RealConsole..."
    cargo build --release
    echo -e "${GREEN}✓ 编译完成${NC}\n"
fi

# 创建安装目录
if [ ! -d "$INSTALL_DIR" ]; then
    echo "创建安装目录: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
    echo -e "${GREEN}✓ 目录已创建${NC}\n"
fi

# 创建配置目录
if [ ! -d "$CONFIG_DIR" ]; then
    echo "创建配置目录: $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$CONFIG_DIR/memory"
    echo -e "${GREEN}✓ 配置目录已创建${NC}\n"
fi

# 复制可执行文件
echo "安装可执行文件到: $INSTALL_DIR/$BINARY_NAME"
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo -e "${GREEN}✓ 可执行文件已安装${NC}\n"

# 复制配置示例（如果不存在）
if [ ! -f "$CONFIG_DIR/realconsole.yaml" ]; then
    if [ -f "config/minimal.yaml" ]; then
        echo "复制配置示例到: $CONFIG_DIR/"
        cp "config/minimal.yaml" "$CONFIG_DIR/realconsole.yaml.example"
        echo -e "${GREEN}✓ 配置示例已复制${NC}\n"
    fi
fi

# 检查 PATH
echo "检查 PATH 配置..."
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠ $INSTALL_DIR 不在 PATH 中${NC}"
    echo ""
    echo "请将以下行添加到你的 shell 配置文件中："
    echo ""

    # 检测 shell 类型
    if [ -n "$BASH_VERSION" ]; then
        echo -e "${GREEN}  # For Bash (add to ~/.bashrc or ~/.bash_profile)${NC}"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    elif [ -n "$ZSH_VERSION" ]; then
        echo -e "${GREEN}  # For Zsh (add to ~/.zshrc)${NC}"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    else
        echo -e "${GREEN}  # Add to your shell config file${NC}"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi

    echo ""
    echo "或者临时添加到当前会话："
    echo -e "${GREEN}  export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    echo ""
else
    echo -e "${GREEN}✓ PATH 已正确配置${NC}\n"
fi

# 显示安装信息
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ RealConsole 安装完成！${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "安装位置："
echo "  • 可执行文件: $INSTALL_DIR/$BINARY_NAME"
echo "  • 配置目录:   $CONFIG_DIR/"
echo "  • 数据目录:   $CONFIG_DIR/memory/"
echo ""
echo "下一步："
echo "  1. 运行配置向导创建配置文件："
echo -e "     ${GREEN}realconsole wizard${NC}"
echo ""
echo "  2. 或者手动配置："
echo -e "     ${GREEN}cd $CONFIG_DIR${NC}"
echo "     编辑 realconsole.yaml 和 .env"
echo ""
echo "  3. 开始使用："
echo -e "     ${GREEN}realconsole${NC}"
echo ""
echo "需要帮助？运行："
echo -e "  ${GREEN}realconsole --help${NC}"
echo ""
echo "卸载："
echo -e "  ${GREEN}./uninstall.sh${NC}"
echo ""
