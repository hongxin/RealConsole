#!/bin/bash
# 测试 LLM 和 Tool 追踪功能

echo "=== 测试 LLM Span 追踪 ==="
echo ""

{
    # 1. 执行一个 Shell 命令（有 Shell Span）
    echo "pwd"
    sleep 0.5

    # 2. 执行一个自然语言查询（有 LLM Span）
    # 注意：这会触发 LLM 调用，需要配置 API
    echo "hello"
    sleep 2.0

    # 3. 查看最近的 trace
    echo "/trace recent 3"
    sleep 1.0

    echo "exit"
} | ./target/debug/realconsole 2>&1

echo ""
echo "=== 测试完成 ==="
