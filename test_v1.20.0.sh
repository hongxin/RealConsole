#!/bin/bash
# v1.20.0 环境变量共享测试脚本

echo "=== v1.20.0 环境变量共享测试 ==="
echo ""

# 测试命令（模拟 /plan + /execute 的输出）
echo "测试场景：计算1到100的和，然后乘以50"
echo ""

# 使用修复后的逻辑：合并执行
MERGED_CMD='SUM=$(seq 1 100 | awk '"'"'{sum+=$1} END {print sum}'"'"') && echo "1到100的和是: $SUM" ; echo '"'"'__REALCONSOLE_TASK_0_END__'"'"' && RESULT=$((SUM * 50)) && echo "最终结果是: $RESULT" ; echo '"'"'__REALCONSOLE_TASK_1_END__'"'"' && echo "验证: 5050 × 50 = $((5050 * 50))" ; echo '"'"'__REALCONSOLE_TASK_2_END__'"'"''

echo "执行合并后的命令..."
echo ""

OUTPUT=$(bash -c "$MERGED_CMD")

echo "原始输出："
echo "$OUTPUT"
echo ""

# 拆分输出
echo "=== 任务输出拆分 ==="
echo ""

# Task 0
TASK_0=$(echo "$OUTPUT" | sed -n '1,/__REALCONSOLE_TASK_0_END__/p' | grep -v '__REALCONSOLE_TASK_0_END__')
echo "Task 0 (计算1到100的和):"
echo "$TASK_0"
echo ""

# Task 1
TASK_1=$(echo "$OUTPUT" | sed -n '/__REALCONSOLE_TASK_0_END__/,/__REALCONSOLE_TASK_1_END__/p' | grep -v '__REALCONSOLE_TASK_._END__')
echo "Task 1 (将和与50相乘):"
echo "$TASK_1"
echo ""

# Task 2
TASK_2=$(echo "$OUTPUT" | sed -n '/__REALCONSOLE_TASK_1_END__/,/__REALCONSOLE_TASK_2_END__/p' | grep -v '__REALCONSOLE_TASK_._END__')
echo "Task 2 (验证计算结果):"
echo "$TASK_2"
echo ""

# 验证结果
echo "=== 验证结果 ==="
if echo "$TASK_0" | grep -q "5050"; then
    echo "✅ Task 0: 正确计算出 5050"
else
    echo "❌ Task 0: 失败"
fi

if echo "$TASK_1" | grep -q "252500"; then
    echo "✅ Task 1: 正确计算出 252500 (环境变量 SUM 成功传递！)"
else
    echo "❌ Task 1: 失败（环境变量未传递）"
fi

if echo "$TASK_2" | grep -q "252500"; then
    echo "✅ Task 2: 验证通过"
else
    echo "❌ Task 2: 失败"
fi

echo ""
echo "=== 测试完成 ==="
