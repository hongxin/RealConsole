#!/bin/bash
# 测试任务系统输出格式验证脚本

set -e

BINARY="./target/release/realconsole"
CONFIG="realconsole.yaml"

echo "========================================="
echo "测试任务系统输出格式验证"
echo "========================================="
echo ""

# 测试1: /plan 命令输出格式
echo "【测试1】验证 /plan 命令输出格式"
echo "-----------------------------------------"
echo "命令: /plan 创建一个名为testproject的目录，在里面创建src和docs子目录"
echo ""

OUTPUT=$($BINARY --config $CONFIG --once "/plan 创建一个名为testproject的目录，在里面创建src和docs子目录" 2>&1)

# 检查关键输出元素
echo "检查输出包含的关键元素："

# 1. 检查是否有目标描述
if echo "$OUTPUT" | grep -q "创建一个名为testproject的目录"; then
    echo "✓ 包含目标描述"
else
    echo "✗ 缺少目标描述"
fi

# 2. 检查是否有阶段信息（树状结构符号）
if echo "$OUTPUT" | grep -qE "├─|└─|│"; then
    echo "✓ 包含树状结构符号"
else
    echo "✗ 缺少树状结构符号"
fi

# 3. 检查是否有执行模式符号
if echo "$OUTPUT" | grep -qE "→|⇉"; then
    echo "✓ 包含执行模式符号"
else
    echo "✗ 缺少执行模式符号"
fi

# 4. 检查是否有紧凑摘要（▸符号）
if echo "$OUTPUT" | grep -q "▸"; then
    echo "✓ 包含紧凑摘要指示器"
else
    echo "✗ 缺少紧凑摘要指示器"
fi

# 5. 检查是否有命令提示
if echo "$OUTPUT" | grep -qE "mkdir|touch|创建"; then
    echo "✓ 包含具体命令"
else
    echo "✗ 缺少具体命令"
fi

echo ""
echo "实际输出："
echo "-----------------------------------------"
echo "$OUTPUT"
echo "-----------------------------------------"
echo ""

# 测试2: 检查输出行数是否符合极简主义（应该在10-15行左右，不超过20行）
LINE_COUNT=$(echo "$OUTPUT" | grep -v "^$" | wc -l | tr -d ' ')
echo "【测试2】验证输出行数（极简主义要求）"
echo "-----------------------------------------"
echo "非空行数: $LINE_COUNT"

if [ "$LINE_COUNT" -le 20 ]; then
    echo "✓ 输出行数符合极简主义要求 (≤20行)"
else
    echo "⚠ 输出行数偏多，可能需要进一步优化"
fi

echo ""
echo "========================================="
echo "测试完成"
echo "========================================="
