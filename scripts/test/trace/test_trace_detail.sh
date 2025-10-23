#!/bin/bash
# 测试 trace detail 功能

echo "=== 测试 trace detail 功能 ==="
echo ""

# 1. 先执行一些命令生成 trace
# 2. 然后通过修改代码打印 trace_id，或者测试 /trace stats 看看数据

{
    echo "pwd"
    sleep 0.3
    echo "/trace stats"
    sleep 0.3
    echo "exit"
} | ./target/debug/realconsole

echo ""
echo "=== 测试完成 ==="
