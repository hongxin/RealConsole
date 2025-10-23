#!/bin/bash
# 测试 /trace detail 显示完整调用链

echo "=== 测试 /trace detail 显示 LLM 调用链 ==="
echo ""

{
    # 1. 执行一个 LLM 查询
    echo "hello world"
    sleep 3.0

    # 2. 查看最近的 trace
    echo "/trace recent 1"
    sleep 1.0

    # 3. 使用 detail 查看详细信息（使用实际的 trace_id）
    echo "/trace detail 679edda4"
    sleep 1.0

    echo "exit"
} | ./target/debug/realconsole 2>&1

echo ""
echo "=== 测试完成 ==="
