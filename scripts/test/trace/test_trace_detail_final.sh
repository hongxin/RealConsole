#!/bin/bash
# 完整测试 trace detail 和 tree 功能

echo "=== 完整测试 trace detail 和 tree ==="
echo ""

{
    # 1. 执行几个命令生成 trace
    echo "pwd"
    sleep 0.4

    echo "echo 'hello world'"
    sleep 0.4

    # 2. 查看最近的 trace
    echo "/trace recent 2"
    sleep 1.0

    # 3. 测试 detail（使用第二个 trace 的 ID）
    echo "/trace detail $(echo 'ls' | ./target/debug/realconsole >/dev/null 2>&1 && echo '/trace recent 1' | ./target/debug/realconsole 2>&1 | grep -o '\[........\]' | head -1 | tr -d '[]' || echo 'unknown')"

    # 简化：直接在会话中执行
    echo "ls src/*.rs | head -3"
    sleep 0.4

    echo "/trace recent 1"
    sleep 1.0

    # 获取并使用最新的 trace_id
    # 这里我们手动硬编码测试
    echo "/trace detail"
    sleep 0.3

    echo "/trace help"
    sleep 0.5

    echo "exit"
} | ./target/debug/realconsole 2>&1

echo ""
echo "=== 测试完成 ==="
