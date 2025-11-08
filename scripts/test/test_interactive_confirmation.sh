#!/bin/bash
# v1.37.0 交互式确认功能测试脚本
# 目的：验证 v1.29.2-v1.29.3 已实现的编辑模式功能

set -e

echo "==================================="
echo "  v1.37.0 交互式确认功能测试"
echo "==================================="
echo ""

# 检查是否已编译
if [ ! -f "./target/release/realconsole" ]; then
    echo "❌ 错误：未找到编译后的二进制文件"
    echo "请先运行：cargo build --release"
    exit 1
fi

# 检查 API Key（使用真实的环境变量）
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "⚠️  警告：未设置 DEEPSEEK_API_KEY 环境变量"

    # 尝试从 .env 文件读取
    if [ -f ".env" ]; then
        echo "尝试从 .env 文件读取 API Key..."
        export $(grep "^DEEPSEEK_API_KEY=" .env | xargs)

        if [ -n "$DEEPSEEK_API_KEY" ]; then
            echo "✅ 已从 .env 文件加载 API Key"
        else
            echo "❌ .env 文件中未找到 DEEPSEEK_API_KEY"
            exit 1
        fi
    else
        echo "❌ 未找到 .env 文件"
        echo "请设置环境变量：export DEEPSEEK_API_KEY=your-api-key"
        exit 1
    fi
else
    echo "✅ 使用环境变量中的真实 API Key"
fi

# 清理旧的日志文件
LOG_FILE="/tmp/realconsole_interactive_test.log"
rm -f "$LOG_FILE"

echo "📋 测试步骤："
echo ""
echo "1️⃣  启动 Web 服务器（端口 7799）"
echo "2️⃣  访问 http://127.0.0.1:7799"
echo "3️⃣  测试交互式确认功能"
echo ""

# 启动 Web 服务器（使用真实的 API Key）
echo "正在启动 Web 服务器..."
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" ./target/release/realconsole web --port 7799 > "$LOG_FILE" 2>&1 &
WEB_PID=$!

echo "✅ Web 服务器已启动 (PID: $WEB_PID)"
echo ""

# 等待服务器启动
echo "等待服务器启动..."
sleep 3

# 检查服务器是否正常运行
if curl -s http://127.0.0.1:7799 > /dev/null 2>&1; then
    echo "✅ Web 服务器运行正常"
else
    echo "⚠️  等待服务器完全启动（再等2秒）..."
    sleep 2
    if curl -s http://127.0.0.1:7799 > /dev/null 2>&1; then
        echo "✅ Web 服务器现在正常了"
    else
        echo "❌ 服务器启动失败，请检查日志："
        echo "   cat $LOG_FILE"
        kill $WEB_PID 2>/dev/null
        exit 1
    fi
fi

echo ""
echo "==================================="
echo "  🎯 测试准备完成！"
echo "==================================="
echo ""
echo "📍 访问地址：http://127.0.0.1:7799"
echo "📝 日志文件：$LOG_FILE"
echo "🔧 服务器 PID：$WEB_PID"
echo ""

echo "==================================="
echo "  📋 测试清单（使用真实 LLM）"
echo "==================================="
echo ""
echo "第一步：生成执行计划"
echo "----------------------------------------"
echo "在 Web 终端输入以下命令："
echo ""
echo "  /decompose 读取 realconsole.yaml 文件并显示前10行"
echo ""
echo "预期结果："
echo "  ✅ LLM 理解意图并生成执行计划"
echo "  ✅ 显示\"意图理解\"卡片"
echo "  ✅ 显示步骤列表（例如：打开文件、读取内容、显示结果）"
echo "  ✅ 显示\"修改计划\"和\"执行计划\"按钮"
echo ""

echo "第二步：测试编辑模式"
echo "----------------------------------------"
echo "1. 点击\"修改计划\"按钮"
echo ""
echo "预期结果："
echo "  ✅ 步骤前出现 checkbox（绿色勾选框）"
echo "  ✅ 按钮变为\"保存\"（💾）和\"取消\"（❌）"
echo "  ✅ 可以勾选/取消任意步骤"
echo ""

echo "第三步：测试保存功能"
echo "----------------------------------------"
echo "1. 取消勾选某个步骤（例如第 2 个步骤）"
echo "2. 点击\"保存\"按钮"
echo ""
echo "预期结果："
echo "  ✅ 界面恢复正常模式"
echo "  ✅ 被取消的步骤显示为禁用状态（灰色/划线）"
echo "  ✅ 按钮恢复为\"修改计划\"和\"执行计划\""
echo ""

echo "第四步：测试执行功能"
echo "----------------------------------------"
echo "1. 点击\"执行计划\"按钮"
echo ""
echo "预期结果："
echo "  ✅ 只执行启用的步骤（跳过被取消的步骤）"
echo "  ✅ 显示执行进度（每个步骤的状态）"
echo "  ✅ 执行成功，显示最终结果"
echo ""

echo "第五步：体悟用户体验"
echo "----------------------------------------"
echo "思考以下问题："
echo "  1. 整个流程是否顺畅？"
echo "  2. 3 步操作（修改 → 保存 → 执行）是否繁琐？"
echo "  3. 是否希望\"保存\"后直接执行（变成 2 步）？"
echo "  4. 内联编辑 vs 弹窗编辑，哪个更好？"
echo ""

echo "==================================="
echo "  🎯 Day 2 决策方向"
echo "==================================="
echo ""
echo "基于测试体验，选择优化方向："
echo ""
echo "方向 A（推荐）：简化流程"
echo "  - 改动：1 行代码"
echo "  - 时间：5 分钟"
echo "  - 效果：3 步变 2 步（保存并执行）"
echo ""
echo "方向 B：弹窗式编辑"
echo "  - 改动：约 200 行代码"
echo "  - 时间：1-2 天"
echo "  - 效果：现代化交互体验"
echo ""
echo "方向 C：保持现状"
echo "  - 改动：0 行代码"
echo "  - 时间：0 天"
echo "  - 效果：观察 1-2 周后再决定"
echo ""

echo "==================================="
echo "  💡 提示"
echo "==================================="
echo ""
echo "- 测试完成后，按 Ctrl+C 停止服务器"
echo "- 或运行：kill $WEB_PID"
echo "- 查看日志：cat $LOG_FILE"
echo "- 实时监控日志：tail -f $LOG_FILE"
echo ""
echo "详细的 Day 2 行动计划："
echo "  /tmp/v1.37.0-day2-action-plan.md"
echo ""

# 保存 PID 以便后续清理
echo "$WEB_PID" > /tmp/realconsole_test_web.pid

echo "==================================="
echo "  ✨ 开始测试吧！"
echo "==================================="
echo ""
echo "访问：http://127.0.0.1:7799"
echo ""

# 可选：自动打开浏览器（macOS）
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "正在打开浏览器..."
    sleep 1
    open http://127.0.0.1:7799
    echo ""
    echo "🌐 浏览器已打开，请在 Web 终端中执行测试命令"
fi

echo ""
echo "🔍 温馨提示："
echo "  - 测试命令：/decompose 读取 realconsole.yaml 文件并显示前10行"
echo "  - 观察 LLM 生成的执行步骤"
echo "  - 体验编辑模式的完整流程"
echo ""
