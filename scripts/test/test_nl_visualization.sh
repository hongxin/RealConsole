#!/bin/bash
# v1.51.0: 测试自然语言驱动的图表生成功能
# 这是智能 Notebook 的核心特性

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "🚀 v1.51.0 自然语言驱动可视化测试"
echo "========================================="
echo ""

# 检查环境变量
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "❌ 错误: 未设置 DEEPSEEK_API_KEY 环境变量"
    echo "请运行: export DEEPSEEK_API_KEY='your-api-key'"
    exit 1
fi

# 编译项目
echo "📦 正在编译项目..."
cd "$PROJECT_ROOT"
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true
echo ""

# 测试用例
declare -a TEST_CASES=(
    "帮我画一个销售趋势折线图，X轴是1月到6月，销售额分别是120、132、101、134、90、230"
    "创建一个饼图显示产品份额：产品A 35%，产品B 25%，产品C 40%"
    "画一个柱状图，显示各部门人数：研发部50人，销售部30人，市场部20人，运营部15人"
)

echo "========================================="
echo "📋 测试计划"
echo "========================================="
echo ""
for i in "${!TEST_CASES[@]}"; do
    echo "$((i+1)). ${TEST_CASES[$i]}"
done
echo ""

echo "========================================="
echo "🎯 开始测试"
echo "========================================="
echo ""

echo "💡 提示："
echo "  1. 脚本将启动 Web 服务器（端口 7788）"
echo "  2. 请在浏览器打开: http://127.0.0.1:7788"
echo "  3. 在聊天界面依次输入上述测试用例"
echo "  4. 观察图表是否正确生成"
echo "  5. 按 Ctrl+C 停止服务器"
echo ""

# 启动 Web 服务器
echo "🌐 启动 Web 服务器..."
echo "----------------------------------------"
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" ./target/release/realconsole web --port 7788

echo ""
echo "========================================="
echo "✅ 测试完成"
echo "========================================="
