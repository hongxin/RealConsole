#!/bin/bash
# v1.38.0: Cell 重新执行功能测试脚本

set -e

echo "==================================="
echo "  v1.38.0 Cell 重新执行功能测试"
echo "==================================="
echo ""

# 检查编译
if [ ! -f "./target/release/realconsole" ]; then
    echo "❌ 未找到编译后的二进制文件"
    echo "正在编译..."
    cargo build --release
fi

# 读取 API Key
if [ -z "$DEEPSEEK_API_KEY" ]; then
    if [ -f ".env" ]; then
        echo "从 .env 文件加载 API Key..."
        export $(grep "^DEEPSEEK_API_KEY=" .env | xargs)
    fi
fi

# 清理日志
LOG_FILE="/tmp/v1.38.0_test.log"
rm -f "$LOG_FILE"

echo "正在启动 Web 服务器（端口 7799）..."
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" ./target/release/realconsole web --port 7799 > "$LOG_FILE" 2>&1 &
WEB_PID=$!

echo "✅ Web 服务器已启动 (PID: $WEB_PID)"
echo ""
sleep 3

# 检查服务器
if curl -s http://127.0.0.1:7799 > /dev/null 2>&1; then
    echo "✅ Web 服务器运行正常"
else
    echo "⚠️  等待服务器启动..."
    sleep 2
fi

echo ""
echo "==================================="
echo "  🎯 测试准备完成！"
echo "==================================="
echo ""
echo "📍 访问：http://127.0.0.1:7799"
echo "📝 日志：$LOG_FILE"
echo "🔧 PID：$WEB_PID"
echo ""

echo "==================================="
echo "  📋 测试步骤"
echo "==================================="
echo ""
echo "第一步：创建一些 Round"
echo "----------------------------------------"
echo "在 Web 终端中执行以下命令："
echo ""
echo "1. Shell 命令："
echo "   !date"
echo ""
echo "2. LLM 对话："
echo "   你好"
echo ""
echo "3. Shell 命令："
echo "   !ls"
echo ""

echo "第二步：测试重新执行按钮"
echo "----------------------------------------"
echo "在每个 Round 卡片右上角，你应该看到："
echo "  🔄 重新执行 （青色到绿色渐变按钮）"
echo ""
echo "点击按钮观察："
echo "  ✅ 按钮变为\"⏳ 执行中...\"并禁用"
echo "  ✅ 输出区域显示\"🔄 正在重新执行...\""
echo "  ✅ 命令重新执行"
echo "  ✅ 新输出替换旧输出"
echo "  ✅ 按钮恢复为\"🔄 重新执行\""
echo ""

echo "第三步：测试不同类型的 Round"
echo "----------------------------------------"
echo "1. Shell 命令重执行："
echo "   - !date 应该显示新的时间"
echo ""
echo "2. LLM 对话重执行："
echo "   - \"你好\" 可能得到不同的回复"
echo ""
echo "3. 系统命令重执行："
echo "   - /system help 应该重新显示帮助"
echo ""

echo "第四步：测试边界情况"
echo "----------------------------------------"
echo "1. 快速连续点击（测试防抖）"
echo "2. 执行中断开 WebSocket（测试错误处理）"
echo "3. 执行大量输出的命令（测试性能）"
echo ""

echo "==================================="
echo "  🎨 UI 验证要点"
echo "==================================="
echo ""
echo "✅ 按钮样式："
echo "   - 青色到绿色渐变背景"
echo "   - 黑色粗体文字"
echo "   - Hover 时放大 1.05 倍"
echo "   - 发光效果"
echo ""
echo "✅ 按钮位置："
echo "   - Round 头部右侧"
echo "   - 折叠按钮之前"
echo "   - margin-left: auto（自动靠右）"
echo ""
echo "✅ 执行状态："
echo "   - Loading: \"🔄 正在重新执行...\""
echo "   - 错误: 红色\"❌ WebSocket 未连接\""
echo ""

echo "==================================="
echo "  💡 提示"
echo "==================================="
echo ""
echo "停止服务器："
echo "  kill $WEB_PID"
echo ""
echo "查看日志："
echo "  tail -f $LOG_FILE"
echo ""
echo "清理进程："
echo "  pkill -f 'realconsole web'"
echo ""

# 保存 PID
echo "$WEB_PID" > /tmp/v1.38.0_test_web.pid

echo "==================================="
echo "  ✨ 开始测试吧！"
echo "==================================="
echo ""

# macOS 自动打开浏览器
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "正在打开浏览器..."
    sleep 1
    open http://127.0.0.1:7799
    echo "🌐 浏览器已打开"
fi

echo ""
