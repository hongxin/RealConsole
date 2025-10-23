#!/bin/bash
# 测试 Dashboard 异常检测功能

echo "=== 测试 Dashboard 异常检测 ==="
echo ""

{
    # 1. 制造一些失败的命令（触发异常检测）
    echo "!nonexistent_command_1"
    sleep 0.3

    echo "!nonexistent_command_2"
    sleep 0.3

    echo "!nonexistent_command_3"
    sleep 0.3

    echo "!nonexistent_command_4"
    sleep 0.3

    echo "!nonexistent_command_5"
    sleep 0.3

    # 2. 执行一些成功的命令
    echo "pwd"
    sleep 0.3

    echo "!echo test"
    sleep 0.3

    # 3. 查看 Dashboard（应该显示异常检测）
    echo "/trace dashboard"
    sleep 1.5

    echo "exit"
} | ./target/debug/realconsole 2>&1

echo ""
echo "=== 测试完成 ==="
