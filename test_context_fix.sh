#!/bin/bash
# 测试 context 功能修复效果

echo "================================"
echo "Context 功能修复验证测试"
echo "================================"
echo ""

echo "测试 1: 检查版本"
realconsole --version
echo ""

echo "测试 2: 测试 context 命令（非交互模式）"
echo "注意：由于是非交互模式，某些命令可能需要在 REPL 中测试"
echo ""

echo "测试 3: 检查 emoji 是否已移除"
echo "检查 context_cmd.rs..."
if grep -q "🟢\|🔴\|👤\|🤖\|⚠️\|ℹ️" src/commands/context_cmd.rs 2>/dev/null; then
    echo "❌ 发现残留的 emoji"
    exit 1
else
    echo "✓ context_cmd.rs 中的 emoji 已全部移除"
fi

echo "检查 memory_core.rs..."
if grep -q "⭐⭐" src/memory/memory_core.rs 2>/dev/null; then
    echo "❌ 发现残留的 emoji"
    exit 1
else
    echo "✓ memory_core.rs 中的 emoji 已全部移除"
fi
echo ""

echo "测试 4: 检查异步锁改进"
if grep -q "try_current" src/commands/context_cmd.rs; then
    echo "✓ 异步锁已改进为使用 try_current"
else
    echo "❌ 未找到 try_current 改进"
    exit 1
fi

if grep "block_in_place" src/commands/context_cmd.rs | grep -v "^[[:space:]]*//"; then
    echo "❌ 仍在使用 block_in_place"
    exit 1
else
    echo "✓ 已移除 block_in_place（代码中）"
fi
echo ""

echo "================================"
echo "✓ 所有自动化测试通过！"
echo "================================"
echo ""
echo "建议进行的手动测试："
echo "1. 启动 realconsole"
echo "2. 在 REPL 中执行以下命令："
echo "   /context"
echo "   /context start"
echo "   /context status"
echo "   /context show"
echo "   /context stop"
echo ""
echo "3. 观察："
echo "   - 终端是否稳定（不崩溃）"
echo "   - 显示是否正常（使用 [OK], [ON], [OFF] 等文本替代 emoji）"
echo "   - 快速连续执行命令是否流畅"
echo ""
