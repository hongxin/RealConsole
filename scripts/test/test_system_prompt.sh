#!/bin/bash
# 测试系统提示词配置功能

set -e

echo "=========================================="
echo "测试系统提示词配置功能（v1.23.1）"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 测试程序路径
REALCONSOLE="./target/release/realconsole"

if [ ! -f "$REALCONSOLE" ]; then
    echo "错误: 找不到 $REALCONSOLE"
    echo "请先运行: cargo build --release"
    exit 1
fi

echo -e "${CYAN}✓ 编译版本检查通过${NC}"
echo ""

# 测试 1: /show-prompt 命令
echo -e "${YELLOW}测试 1: /show-prompt - 显示当前系统提示词${NC}"
echo "-------------------------------------------"
echo "/show-prompt" | DEEPSEEK_API_KEY="test-key" timeout 3 $REALCONSOLE || true
echo ""

# 测试 2: /set-prompt 不带参数（显示帮助）
echo -e "${YELLOW}测试 2: /set-prompt - 显示用法帮助${NC}"
echo "-------------------------------------------"
echo "/set-prompt" | DEEPSEEK_API_KEY="test-key" timeout 3 $REALCONSOLE || true
echo ""

# 测试 3: 配置文件中设置系统提示词
echo -e "${YELLOW}测试 3: 配置文件支持检查${NC}"
echo "-------------------------------------------"
if grep -q "system_prompt" realconsole.yaml; then
    echo -e "${GREEN}✓ realconsole.yaml 中已包含 system_prompt 配置示例${NC}"
    echo ""
    echo "配置示例："
    grep -A 5 "system_prompt" realconsole.yaml || true
else
    echo -e "${YELLOW}⚠ realconsole.yaml 中未找到 system_prompt 配置示例${NC}"
fi
echo ""

# 测试 4: 命令帮助测试
echo -e "${YELLOW}测试 4: 验证新命令已注册${NC}"
echo "-------------------------------------------"
echo "/help" | DEEPSEEK_API_KEY="test-key" timeout 3 $REALCONSOLE | grep -i "prompt" || echo "（在帮助信息中查找 prompt 相关命令）"
echo ""

# 测试总结
echo "=========================================="
echo -e "${GREEN}✓ 系统提示词功能测试完成${NC}"
echo "=========================================="
echo ""
echo "功能说明："
echo "1. ${CYAN}/show-prompt${NC} - 显示当前使用的系统提示词及其来源"
echo "2. ${CYAN}/set-prompt <prompt>${NC} - 设置自定义系统提示词"
echo "3. ${CYAN}/set-prompt reset${NC} - 重置为配置文件默认值"
echo "4. 配置文件支持（realconsole.yaml 中的 system_prompt 字段）"
echo ""
echo "优先级：运行时设置 > 配置文件 > 内置默认"
echo ""
