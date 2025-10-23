#!/bin/bash
# 测试 trace 功能

echo "=== 测试 trace 功能 ==="
echo ""

# 执行一些命令来生成追踪记录
{
    echo "pwd"
    sleep 0.5
    echo "ls -la | head -5"
    sleep 0.5
    echo "/trace"
    sleep 0.5
    echo "exit"
} | ./target/debug/realconsole

echo ""
echo "=== 测试完成 ==="
