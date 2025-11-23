#!/bin/bash
# v1.51.0: 测试会话历史加载时图表的正确性
# 验证修复：刷新页面后图表能正确显示，文本不重复

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "🚀 v1.51.0 会话历史图表加载测试"
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

echo "========================================="
echo "📋 测试说明"
echo "========================================="
echo ""
echo "本测试验证 Bug 修复：页面刷新后图表能正确加载"
echo ""
echo "测试步骤："
echo "  1. 启动 Web 服务器（端口 7788）"
echo "  2. 生成一个测试图表"
echo "  3. 保存会话"
echo "  4. 刷新页面或重新加载会话"
echo "  5. 验证图表正确显示且文本不重复"
echo ""
echo "验证点："
echo "  ✅ 图表渲染正确"
echo "  ✅ 文本只显示一次 \"✅ 图表已生成\""
echo "  ✅ 图表数据与首次生成时一致"
echo ""

echo "========================================="
echo "🧪 测试用例"
echo "========================================="
echo ""
echo "用例 1: 单个折线图"
echo "输入: \"帮我画一个销售趋势折线图，X轴是1月到6月，销售额分别是120、132、101、134、90、230\""
echo ""
echo "用例 2: 饼图"
echo "输入: \"创建一个饼图显示产品份额：产品A 35%，产品B 25%，产品C 40%\""
echo ""
echo "用例 3: 多个图表混合"
echo "输入: 先生成折线图，再生成饼图，然后保存会话并刷新"
echo ""

echo "========================================="
echo "🎯 开始测试"
echo "========================================="
echo ""

echo "💡 操作指引："
echo ""
echo "  第一部分：生成图表"
echo "  ----------------"
echo "  1. 在浏览器打开: http://127.0.0.1:7788"
echo "  2. 输入测试用例 1 的内容"
echo "  3. 验证图表正确显示"
echo "  4. 可选：继续输入测试用例 2 和 3"
echo ""
echo "  第二部分：保存会话"
echo "  ----------------"
echo "  5. 点击页面右上角的 \"保存会话\" 按钮"
echo "  6. 记住会话名称（或使用默认名称）"
echo "  7. 验证会话已保存（查看会话列表）"
echo ""
echo "  第三部分：验证修复"
echo "  ----------------"
echo "  8. 方式A：刷新浏览器页面（F5 或 Cmd+R）"
echo "     方式B：点击 \"加载会话\" 按钮，选择刚才保存的会话"
echo "  9. 等待会话加载完成"
echo "  10. 验证以下内容："
echo "      ✅ 图表正确显示（与首次生成时一致）"
echo "      ✅ 文本只显示一次 \"✅ 图表已生成\"（不重复）"
echo "      ✅ 所有回合和图表都完整恢复"
echo ""
echo "  第四部分：清理（可选）"
echo "  --------------------"
echo "  11. 删除测试会话（避免污染会话列表）"
echo ""
echo "按 Ctrl+C 停止服务器"
echo ""

# 显示会话文件位置
echo "========================================="
echo "📁 会话文件位置"
echo "========================================="
echo ""
echo "会话保存在: ~/.realconsole/sessions/"
echo "可以查看会话文件内容:"
echo "  ls -lh ~/.realconsole/sessions/"
echo "  cat ~/.realconsole/sessions/session-<id>.json | jq ."
echo ""

# 启动 Web 服务器
echo "========================================="
echo "🌐 启动 Web 服务器"
echo "========================================="
echo ""
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" ./target/release/realconsole web --port 7788

echo ""
echo "========================================="
echo "✅ 测试完成"
echo "========================================="
