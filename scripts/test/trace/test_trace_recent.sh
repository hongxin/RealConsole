#!/bin/bash
# 测试 /trace recent 功能

echo "=== 测试 /trace recent 功能 ==="
echo ""

{
    echo "pwd"
    sleep 0.3
    echo "ls"
    sleep 0.3
    echo "echo 'hello world'"
    sleep 0.3
    echo "/trace recent"
    sleep 0.3
    echo "exit"
} | ./target/debug/realconsole

echo ""
echo "=== 测试完成 ==="
