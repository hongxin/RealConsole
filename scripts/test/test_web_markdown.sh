#!/bin/bash
# Web 版本 Markdown 渲染测试脚本
# v1.26.0

set -e

echo "================================================"
echo "  RealConsole Web Markdown 渲染测试"
echo "================================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查环境
echo -e "${BLUE}[1/4] 检查环境...${NC}"

if [ ! -f "./target/release/realconsole" ]; then
    echo -e "${YELLOW}编译可执行文件...${NC}"
    cargo build --release
fi

if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo -e "${YELLOW}警告: DEEPSEEK_API_KEY 未设置${NC}"
    echo "设置测试用 API Key..."
    export DEEPSEEK_API_KEY="test-key"
fi

echo -e "${GREEN}✓ 环境检查完成${NC}"
echo ""

# 编译检查
echo -e "${BLUE}[2/4] 编译检查...${NC}"
cargo clippy --quiet 2>&1 | grep "src/web/server.rs" || echo -e "${GREEN}✓ Web 服务器代码无警告${NC}"
echo ""

# 代码验证
echo -e "${BLUE}[3/4] 代码验证...${NC}"

# 检查 marked.js CDN
if grep -q "marked@11.1.1" src/web/server.rs; then
    echo -e "${GREEN}✓ marked.js CDN 已添加${NC}"
else
    echo -e "${YELLOW}✗ marked.js CDN 未找到${NC}"
    exit 1
fi

# 检查 markdown-overlay 容器
if grep -q "markdown-overlay" src/web/server.rs; then
    echo -e "${GREEN}✓ Markdown 覆盖层容器已添加${NC}"
else
    echo -e "${YELLOW}✗ Markdown 覆盖层未找到${NC}"
    exit 1
fi

# 检查 CSS 样式
if grep -q "markdown-content" src/web/server.rs; then
    echo -e "${GREEN}✓ Markdown CSS 样式已添加${NC}"
else
    echo -e "${YELLOW}✗ Markdown CSS 样式未找到${NC}"
    exit 1
fi

# 检查 MarkdownRenderer 类
if grep -q "class MarkdownRenderer" src/web/server.rs; then
    echo -e "${GREEN}✓ MarkdownRenderer 类已实现${NC}"
else
    echo -e "${YELLOW}✗ MarkdownRenderer 类未找到${NC}"
    exit 1
fi

# 检查 WebSocket 消息处理
if grep -q "markdownRenderer.render" src/web/server.rs; then
    echo -e "${GREEN}✓ WebSocket Markdown 渲染已集成${NC}"
else
    echo -e "${YELLOW}✗ WebSocket Markdown 渲染未集成${NC}"
    exit 1
fi

echo ""

# 手动测试说明
echo -e "${BLUE}[4/4] 手动测试说明${NC}"
echo "================================================"
echo ""
echo -e "${YELLOW}请执行以下步骤进行手动测试：${NC}"
echo ""
echo "1. 启动 Web 服务器:"
echo -e "   ${GREEN}DEEPSEEK_API_KEY='your-key' ./target/release/realconsole web${NC}"
echo ""
echo "2. 打开浏览器访问:"
echo -e "   ${GREEN}http://127.0.0.1:7788${NC}"
echo ""
echo "3. 测试 Markdown 渲染:"
echo ""
echo "   测试用例 1 - 标题和粗体:"
echo -e "   ${BLUE}请介绍一下 Rust 语言${NC}"
echo "   期望: 看到蓝色标题和白色粗体文字"
echo ""
echo "   测试用例 2 - 代码块:"
echo -e "   ${BLUE}写一个 Hello World 的 Rust 程序${NC}"
echo "   期望: 看到绿色代码块，深灰背景"
echo ""
echo "   测试用例 3 - 列表:"
echo -e "   ${BLUE}列出 Rust 的三个优点${NC}"
echo "   期望: 看到蓝色 bullet 点"
echo ""
echo "   测试用例 4 - 流式输出:"
echo -e "   ${BLUE}详细解释什么是所有权${NC}"
echo "   期望: 流式输出时平滑渲染 Markdown"
echo ""
echo "   测试用例 5 - 纯文本:"
echo -e "   ${BLUE}!ls${NC}"
echo "   期望: 普通终端输出（不渲染 Markdown）"
echo ""
echo "================================================"
echo ""
echo -e "${GREEN}✓ 所有自动检查通过！${NC}"
echo -e "${YELLOW}请按照上述说明进行手动测试${NC}"
echo ""
