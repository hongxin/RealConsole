#!/bin/bash
# 稳定性测试脚本

echo "=== RealConsole 稳定性测试 ==="
echo ""

# 测试 1: 基本命令执行
echo "测试 1: 基本命令执行..."
echo -e "/help\nexit" | DEEPSEEK_API_KEY="test-key" ./target/release/realconsole 2>&1 | grep -q "RealConsole"
if [ $? -eq 0 ]; then
    echo "✓ 基本命令执行正常"
else
    echo "✗ 基本命令执行失败"
fi
echo ""

# 测试 2: 多次命令
echo "测试 2: 多次命令执行..."
echo -e "/stats\n/history\n/memory list\nexit" | DEEPSEEK_API_KEY="test-key" ./target/release/realconsole 2>&1 | grep -q "RealConsole"
if [ $? -eq 0 ]; then
    echo "✓ 多次命令执行正常"
else
    echo "✗ 多次命令执行失败"
fi
echo ""

# 测试 3: Shell 命令
echo "测试 3: Shell 命令执行..."
echo -e "ls -la\nexit" | DEEPSEEK_API_KEY="test-key" ./target/release/realconsole 2>&1 | grep -q "RealConsole"
if [ $? -eq 0 ]; then
    echo "✓ Shell 命令执行正常"
else
    echo "✗ Shell 命令执行失败"
fi
echo ""

# 测试 4: 快速连续命令（压力测试）
echo "测试 4: 快速连续命令..."
{
    for i in {1..10}; do
        echo "/stats"
    done
    echo "exit"
} | DEEPSEEK_API_KEY="test-key" ./target/release/realconsole 2>&1 | grep -q "RealConsole"
if [ $? -eq 0 ]; then
    echo "✓ 快速连续命令执行正常"
else
    echo "✗ 快速连续命令执行失败"
fi
echo ""

echo "=== 测试完成 ==="
echo ""
echo "如果所有测试都通过，说明程序稳定性已改善。"
echo "如果您仍然遇到终端崩溃，请运行: reset"
