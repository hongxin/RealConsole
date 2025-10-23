#!/bin/bash
# v1.6.0 Dashboard 功能演示
# 展示系统健康度监控、四象分区视图、异常检测和智能建议

echo "=========================================="
echo "  RealConsole v1.6.0 Dashboard 演示"
echo "=========================================="
echo ""

{
    # Phase 1: 正常使用阶段
    echo "=== Phase 1: 正常使用 ==="
    echo "pwd"
    sleep 0.3
    echo "!ls -la | head -5"
    sleep 0.5
    echo "hello, how are you?"
    sleep 2.0

    # Phase 2: 查看健康 Dashboard
    echo "/trace dashboard"
    sleep 1.5

    echo ""
    echo "=== Phase 2: 制造异常（重复错误）==="
    # Phase 3: 制造重复错误
    for i in {1..5}; do
        echo "!bad_command_$i"
        sleep 0.2
    done

    # Phase 4: 再次查看 Dashboard（应该检测到异常）
    echo "/trace dashboard"
    sleep 1.5

    echo ""
    echo "=== Phase 3: 查看追踪历史 ==="
    # Phase 5: 查看其他追踪信息
    echo "/trace stats"
    sleep 1.0

    echo "/trace recent 3"
    sleep 1.0

    echo "exit"
} | ./target/release/realconsole 2>&1 | grep -A 100 "RealConsole v1.6.0"

echo ""
echo "=========================================="
echo "  演示完成"
echo "=========================================="
