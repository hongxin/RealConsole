#!/bin/bash
# 在同一个会话中测试 trace 功能

echo "=== 测试 trace detail 和 tree（同一会话） ==="
echo ""

{
    # 1. 执行一个命令生成 trace
    echo "pwd"
    sleep 0.5

    # 2. 查看最近的 trace
    echo "/trace recent 1"
    sleep 0.8

    # 3. 使用短 ID 测试 detail（注意：需要从上面的输出中获取）
    # 这里我们先测试一个简单的方式：执行多个命令然后查看
    echo "ls -la | head -3"
    sleep 0.5

    echo "/trace recent 2"
    sleep 0.8

    # 4. 手动输入 trace_id (这里用占位符，实际需要从输出获取)
    # echo "/trace detail <id>"

    echo "exit"
} | ./target/debug/realconsole

echo ""
echo "=== 测试完成 ==="
echo "提示：从上面的输出中可以看到 trace_id，可以手动测试 /trace detail <id>"
