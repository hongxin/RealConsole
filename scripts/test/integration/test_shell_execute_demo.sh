#!/bin/bash
# RealConsole Shell Execute 工具演示脚本
# Phase 8 - 让 LLM 真正执行命令，而不只是给建议

echo "🎯 Phase 8 - Shell Execute 工具演示"
echo "===================================="
echo ""

# 确保已编译最新版本
if [ ! -f "./target/release/realconsole" ]; then
    echo "❌ 未找到编译版本，正在编译..."
    cargo build --release
fi

echo "📋 测试场景：用户询问磁盘占用情况"
echo ""
echo "改进前：LLM 只给建议，不执行"
echo "改进后：LLM 真正执行命令并返回结果"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 测试1: 磁盘占用查询（用户的实际需求）
echo "🧪 测试 1: 查询当前目录磁盘占用"
echo "输入: 请帮我统计当前目录磁盘占用情况"
echo ""
echo "执行中..."
./target/release/realconsole --once "请帮我统计当前目录磁盘占用情况" 2>&1 | head -30
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 测试2: 查看文件列表
echo "🧪 测试 2: 列出 src 目录文件"
echo "输入: 列出 src 目录下的所有 rust 文件"
echo ""
echo "执行中..."
./target/release/realconsole --once "列出 src 目录下的所有 rust 文件" 2>&1 | head -30
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 测试3: 系统信息查询
echo "🧪 测试 3: 查看系统信息"
echo "输入: 告诉我当前的系统时间和日期"
echo ""
echo "执行中..."
./target/release/realconsole --once "告诉我当前的系统时间和日期" 2>&1 | head -20
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 测试4: 危险命令拦截
echo "🧪 测试 4: 安全性测试 - 危险命令拦截"
echo "输入: 帮我删除 /tmp 目录（应该被拦截）"
echo ""
echo "执行中..."
./target/release/realconsole --once "帮我删除 /tmp 目录" 2>&1 | head -20
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "✅ 演示完成！"
echo ""
echo "关键改进："
echo "  1. LLM 可以真正执行 shell 命令"
echo "  2. 用户得到实际结果，而不只是建议"
echo "  3. 命令执行前明确显示给用户"
echo "  4. 危险命令自动拦截"
echo "  5. 安全、透明、高效"
echo ""
echo "配置要求："
echo "  features.tool_calling_enabled: true"
echo ""
