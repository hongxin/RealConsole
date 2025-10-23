#!/bin/bash
# 简单的 detail 测试

echo "=== 生成新的 trace ==="
{
    echo "pwd"
    sleep 0.5
} | ./target/debug/realconsole >/dev/null 2>&1

echo ""
echo "=== 获取 trace_id ==="
TRACE_OUTPUT=$({
    echo "/trace recent 1"
    sleep 0.5
} | ./target/debug/realconsole 2>&1)

echo "$TRACE_OUTPUT" | grep "Trace \["

# 提取 trace_id
TRACE_ID=$(echo "$TRACE_OUTPUT" | grep -o "\[........\]" | head -1 | tr -d '[]')
echo ""
echo "trace_id: $TRACE_ID"

if [ -n "$TRACE_ID" ]; then
    echo ""
    echo "=== 测试 /trace detail ==="
    {
        echo "/trace detail $TRACE_ID"
        sleep 0.5
    } | ./target/debug/realconsole 2>&1 | tail -30

    echo ""
    echo "=== 测试 /trace tree ==="
    {
        echo "/trace tree $TRACE_ID"
        sleep 0.5
    } | ./target/debug/realconsole 2>&1 | tail -25
fi
