#!/bin/bash
# 测试 /trace detail 和 /trace tree 功能

echo "=== 测试 /trace detail 和 /trace tree ==="
echo ""

{
    echo "pwd"
    sleep 0.3
    echo "/trace recent 1"
    sleep 0.3
    # 从输出中提取 trace_id 并测试（需要手动）
} | ./target/debug/realconsole

echo ""
echo "现在手动测试 /trace detail <id>..."
