#!/bin/bash
# 测试重复错误检测功能

echo "=== 测试重复错误检测 ==="
echo ""

{
    # 1. 制造大量重复的相同错误（触发重复错误检测）
    for i in {1..5}; do
        echo "!nonexistent_cmd"
        sleep 0.2
    done

    # 2. 制造一些其他错误
    echo "!another_bad_cmd"
    sleep 0.2
    echo "!another_bad_cmd"
    sleep 0.2
    echo "!another_bad_cmd"
    sleep 0.2

    # 3. 执行一些成功的命令
    echo "pwd"
    sleep 0.2

    # 4. 查看 Dashboard（应该显示重复错误异常）
    echo "/trace dashboard"
    sleep 1.5

    echo "exit"
} | ./target/debug/realconsole 2>&1

echo ""
echo "=== 测试完成 ==="
