#!/usr/bin/env bash
# RealConsole Web 终端测试脚本

set -e

echo "🧪 RealConsole Web 终端测试"
echo "=============================="
echo ""

# 检查编译
echo "📦 检查编译..."
cargo build --release
echo "✅ 编译成功"
echo ""

# 检查 Web 子命令帮助
echo "📝 检查 Web 子命令..."
./target/release/realconsole web --help
echo "✅ Web 子命令可用"
echo ""

echo "🎉 基础测试通过！"
echo ""
echo "⚠️  使用前请配置 LLM："
echo "  export DEEPSEEK_API_KEY='your-api-key-here'"
echo ""
echo "💡 手动测试步骤："
echo "  1. 确保配置了 API Key（见上方）"
echo "  2. 运行: ./target/release/realconsole web"
echo "  3. 打开浏览器访问: http://127.0.0.1:7788"
echo "  4. 在终端中输入命令测试交互："
echo "     - 系统命令: /help"
echo "     - Shell 命令: !ls"
echo "     - LLM 对话: hello (需要配置 API Key)"
echo ""
echo "⚙️  自定义端口："
echo "  ./target/release/realconsole web --port 9000"
echo ""
echo "🌐 局域网访问（谨慎使用）："
echo "  ./target/release/realconsole web --bind 0.0.0.0"
