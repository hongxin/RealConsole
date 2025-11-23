#!/bin/bash
# 测试图像查看功能
# v1.52.0: 验证 view_image 工具的输出格式

set -e

echo "=========================================="
echo "测试图像查看工具"
echo "=========================================="

# 1. 创建测试图像
TEST_DIR="/tmp/realconsole_image_test"
mkdir -p "$TEST_DIR"

# 创建一个小的测试图像 (10x10 像素的 PNG)
# 使用 base64 解码一个最小的 PNG 图像
echo "iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFUlEQVR42mNk+M9Qz0AEYBxVSF+FAP0QDiectvQpAAAAAElFTkSuQmCC" | base64 -d > "$TEST_DIR/test.png"

echo "✓ 创建测试图像: $TEST_DIR/test.png"

# 2. 构建项目
echo ""
echo "构建项目..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true

# 3. 测试: 直接调用 view_image 工具（通过 LLM）
echo ""
echo "=========================================="
echo "测试 1: 通过 LLM 调用 view_image 工具"
echo "=========================================="

timeout 10 ./target/release/realconsole <<EOF 2>&1 | head -100 || true
view image $TEST_DIR/test.png
/exit
EOF

# 4. 检查输出格式
echo ""
echo "=========================================="
echo "测试 2: 检查输出是否包含正确的标记"
echo "=========================================="

OUTPUT=$(timeout 10 ./target/release/realconsole <<EOF 2>&1 || true
view image $TEST_DIR/test.png
/exit
EOF
)

echo "$OUTPUT" | head -50

if echo "$OUTPUT" | grep -q "__IMAGE_DATA__:"; then
    echo "✓ 找到 __IMAGE_DATA__ 标记"
else
    echo "✗ 未找到 __IMAGE_DATA__ 标记"
fi

if echo "$OUTPUT" | grep -q '"image_type"'; then
    echo "✓ 找到 image_type 字段"
else
    echo "✗ 未找到 image_type 字段"
fi

if echo "$OUTPUT" | grep -q '"data"'; then
    echo "✓ 找到 data 字段"
else
    echo "✗ 未找到 data 字段"
fi

# 清理
rm -rf "$TEST_DIR"
echo ""
echo "✓ 测试完成"
