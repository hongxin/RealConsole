#!/bin/bash
# v1.44.0: Chart 可视化功能端到端测试脚本
#
# 用途：验证图表可视化系统的完整功能
# 测试范围：
# - chart 命令解析
# - WebSocket 消息传输
# - ECharts 前端渲染
# - 主题适配

set -e

echo "========================================="
echo "RealConsole Chart 可视化功能测试"
echo "v1.44.0"
echo "========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试步骤
echo -e "${BLUE}📋 测试准备${NC}"
echo "1. 确保已安装 DEEPSEEK_API_KEY (可选，用于 LLM 功能)"
echo "2. 确保 7788 端口未被占用"
echo ""

# 检查端口
if lsof -Pi :7788 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  端口 7788 已被占用，尝试关闭...${NC}"
    lsof -ti:7788 | xargs kill -9 2>/dev/null || true
    sleep 1
fi

echo -e "${GREEN}✅ 端口检查完成${NC}"
echo ""

# 启动 Web 服务器
echo -e "${BLUE}🚀 启动 Web 服务器${NC}"
echo "命令: DEEPSEEK_API_KEY=\"test-key\" ./target/release/realconsole web --port 7788"
echo ""

# 后台启动服务器
DEEPSEEK_API_KEY="test-key" ./target/release/realconsole web --port 7788 &
SERVER_PID=$!

# 等待服务器启动
sleep 3

# 检查服务器是否启动成功
if ! lsof -Pi :7788 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}❌ 服务器启动失败${NC}"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo -e "${GREEN}✅ 服务器启动成功 (PID: $SERVER_PID)${NC}"
echo ""

# 显示测试命令
echo -e "${BLUE}📊 图表测试命令${NC}"
echo ""
echo "打开浏览器访问: ${GREEN}http://127.0.0.1:7788${NC}"
echo ""
echo "========================================="
echo "测试用例 1: 简单折线图"
echo "========================================="
echo '!chart line --title "月度销售趋势" --x-axis "1月,2月,3月,4月,5月,6月" --series "销售额:120,132,101,134,90,230"'
echo ""

echo "========================================="
echo "测试用例 2: 多系列对比图"
echo "========================================="
echo '!chart line --title "年度销售对比" --x-axis "Q1,Q2,Q3,Q4" --series "2023:100,120,90,150" --series "2024:120,140,110,180"'
echo ""

echo "========================================="
echo "测试用例 3: 平滑曲线"
echo "========================================="
echo '!chart line --title "温度变化" --x-axis "00:00,06:00,12:00,18:00" --series "温度:18,15,25,20" --smooth'
echo ""

echo "========================================="
echo "测试用例 4: 柱状图"
echo "========================================="
echo '!chart bar --title "产品销量" --x-axis "产品A,产品B,产品C,产品D" --series "销量:45,67,89,56"'
echo ""

echo "========================================="
echo "测试用例 5: 自动 X 轴"
echo "========================================="
echo '!chart line --title "简单数据" --series "数据:10,20,15,25,30"'
echo ""

echo "========================================="
echo "测试用例 6: 错误处理 - 数据长度不匹配"
echo "========================================="
echo '!chart line --x-axis "A,B" --series "1,2,3"'
echo "(预期结果: 显示错误提示)"
echo ""

echo "========================================="
echo "测试用例 7: 错误处理 - 无效图表类型"
echo "========================================="
echo '!chart invalid --series "1,2,3"'
echo "(预期结果: 显示错误提示)"
echo ""

echo "========================================="
echo "验收检查清单"
echo "========================================="
echo "[ ] 1. 折线图正确渲染"
echo "[ ] 2. 多系列图表显示多条线"
echo "[ ] 3. 平滑曲线效果生效"
echo "[ ] 4. 柱状图正确显示"
echo "[ ] 5. 自动 X 轴标签（1, 2, 3...）"
echo "[ ] 6. 图表标题正确显示（紫色主题色）"
echo "[ ] 7. 悬停显示数据点详细信息"
echo "[ ] 8. 工具栏功能正常（保存、缩放、还原）"
echo "[ ] 9. 主题切换（深色/浅色）正常"
echo "[ ] 10. 错误提示清晰友好"
echo "[ ] 11. 图表响应式调整（窗口大小变化）"
echo "[ ] 12. Round 卡片样式美观"
echo ""

echo -e "${YELLOW}⏳ 服务器运行中...${NC}"
echo "按 Ctrl+C 停止服务器"
echo ""

# 等待用户中断
wait $SERVER_PID

# 清理
echo ""
echo -e "${GREEN}🧹 测试结束，清理资源${NC}"
kill $SERVER_PID 2>/dev/null || true
