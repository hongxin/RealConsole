#!/usr/bin/env bash
# RealConsole Web 终端快速启动示例

set -e

echo "🚀 RealConsole Web 终端启动向导"
echo "================================"
echo ""

# 检查是否设置了 API Key
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "⚠️  未检测到 DEEPSEEK_API_KEY 环境变量"
    echo ""
    echo "请先设置 API Key："
    echo "  export DEEPSEEK_API_KEY='your-api-key-here'"
    echo ""
    echo "或者只使用系统命令和 Shell 命令（不需要 API Key）"
    echo ""
    read -p "是否继续启动（仅系统命令）？ [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
fi

# 检查配置文件
if [ ! -f "realconsole.yaml" ]; then
    echo "📝 创建默认配置文件..."
    cat > realconsole.yaml << 'EOF'
prefix: "/"

llm:
  primary:
    provider: "deepseek"
    model: "deepseek-chat"
    endpoint: "https://api.deepseek.com/v1"
    api_key: "${DEEPSEEK_API_KEY}"

web:
  enabled: true
  bind: "127.0.0.1"
  port: 7788
EOF
    echo "✅ 配置文件已创建"
fi

# 检查编译
if [ ! -f "./target/release/realconsole" ]; then
    echo "📦 编译项目..."
    cargo build --release
fi

echo ""
echo "🌐 启动 Web 服务..."
echo "   访问地址: http://127.0.0.1:7788"
echo "   按 Ctrl+C 停止服务"
echo ""

# 启动服务
./target/release/realconsole web
