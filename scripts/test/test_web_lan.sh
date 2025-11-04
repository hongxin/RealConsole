#!/usr/bin/env bash
# RealConsole Web 终端局域网访问测试脚本

set -e

echo "🌐 RealConsole Web 终端局域网访问测试"
echo "=========================================="
echo ""

# 获取本机 IP 地址
LOCAL_IP=$(ifconfig | grep "inet " | grep -v 127.0.0.1 | awk '{print $2}' | head -1)

if [ -z "$LOCAL_IP" ]; then
    echo "❌ 无法获取本机 IP 地址"
    exit 1
fi

echo "📍 检测到本机 IP: $LOCAL_IP"
echo ""

# 检查是否已经编译
if [ ! -f "./target/release/realconsole" ]; then
    echo "📦 编译项目..."
    cargo build --release
fi

echo "✅ 准备就绪"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 测试计划："
echo ""
echo "1️⃣  测试 1: 本地访问 (127.0.0.1:7788)"
echo "   用途：验证服务基本功能"
echo "   访问：http://127.0.0.1:7788"
echo ""
echo "2️⃣  测试 2: 局域网访问 (0.0.0.0:7788)"
echo "   用途：验证局域网可访问性"
echo "   访问：http://$LOCAL_IP:7788"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 选择测试模式
read -p "请选择测试模式 [1/2]: " choice

case $choice in
    1)
        echo ""
        echo "🚀 启动测试 1: 本地访问模式"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "✅ 绑定地址: 127.0.0.1 (默认)"
        echo "✅ 端口: 7788"
        echo "✅ 访问地址: http://127.0.0.1:7788"
        echo ""
        echo "⚠️  注意：此模式仅支持本机访问，局域网内其他设备无法访问"
        echo ""
        echo "按 Ctrl+C 停止服务"
        echo ""

        # 启动服务（使用默认配置）
        ./target/release/realconsole web
        ;;

    2)
        echo ""
        echo "🚀 启动测试 2: 局域网访问模式"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "✅ 绑定地址: 0.0.0.0 (所有网络接口)"
        echo "✅ 端口: 7788"
        echo "✅ 本机访问: http://127.0.0.1:7788"
        echo "✅ 局域网访问: http://$LOCAL_IP:7788"
        echo ""
        echo "⚠️  安全提醒："
        echo "   • 0.0.0.0 会暴露在整个局域网中"
        echo "   • 确保在受信任的网络环境中使用"
        echo "   • 当前版本没有身份验证机制"
        echo ""
        echo "📱 其他设备访问步骤："
        echo "   1. 确保设备连接到同一局域网"
        echo "   2. 在浏览器中访问: http://$LOCAL_IP:7788"
        echo "   3. 如果无法访问，检查防火墙设置"
        echo ""
        echo "🔧 macOS 防火墙检查："
        echo "   系统偏好设置 > 安全性与隐私 > 防火墙 > 防火墙选项"
        echo "   确保允许 realconsole 接收传入连接"
        echo ""
        echo "按 Ctrl+C 停止服务"
        echo ""

        # 启动服务（绑定到 0.0.0.0）
        ./target/release/realconsole web --bind 0.0.0.0
        ;;

    *)
        echo "❌ 无效的选择"
        exit 1
        ;;
esac
