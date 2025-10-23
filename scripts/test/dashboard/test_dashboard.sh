#!/bin/bash
# 测试 Dashboard 四象分区功能

echo "=== 测试 Dashboard 功能 ==="
echo ""

{
    # 1. 执行一些命令，生成追踪数据
    echo "pwd"
    sleep 0.5

    echo "!ls -la"
    sleep 0.5

    echo "hello"
    sleep 2.0

    # 2. 查看 Dashboard
    echo "/trace dashboard"
    sleep 1.5

    # 3. 查看统计信息（对比）
    echo "/trace stats"
    sleep 1.0

    echo "exit"
} | ./target/debug/realconsole 2>&1

echo ""
echo "=== 测试完成 ==="
