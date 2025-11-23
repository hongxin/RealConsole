#!/bin/bash
# 测试 v1.52.0 图像查看功能修复
# Bug: 图像数据返回但不显示（__IMAGE____IMAGE_DATA__ 标记暴露）
# Fix: 修复提取逻辑，不再依赖 __DEBUG__ 标记

set -e

echo "=========================================="
echo "v1.52.0 图像查看功能测试"
echo "=========================================="

# 1. 检查测试图像是否存在
if [ ! -f "images/test.png" ]; then
    echo "创建测试图像..."
    mkdir -p images
    echo "iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFUlEQVR42mNk+M9Qz0AEYBxVSF+FAP0QDiectvQpAAAAAElFTkSuQmCC" | base64 -D > images/test.png
    echo "✓ 测试图像已创建: images/test.png ($(ls -lh images/test.png | awk '{print $5}'))"
fi

# 2. 运行单元测试
echo ""
echo "=========================================="
echo "1. 运行单元测试"
echo "=========================================="
cargo test --lib websocket::tests::test_extract_image_data --quiet

echo "✓ 单元测试通过"

# 3. 说明
echo ""
echo "=========================================="
echo "2. Web 测试说明"
echo "=========================================="
echo "要测试 Web UI 中的图像显示功能:"
echo ""
echo "1. 启动 Web 服务器:"
echo "   ./target/release/realconsole web"
echo ""
echo "2. 在浏览器中打开: http://127.0.0.1:7788"
echo ""
echo "3. 输入以下命令测试:"
echo "   view image images/test.png"
echo ""
echo "预期结果:"
echo "  ✓ 应该显示图像,而不是 __IMAGE____IMAGE_DATA__ 标记"
echo "  ✓ 图像应该正确渲染在网页中"
echo "  ✓ 控制台应该看到 [v1.52.0] Rendering image 日志"
echo ""
echo "如果看到 __IMAGE__ 标记,说明提取失败,检查控制台错误日志"
echo ""
echo "=========================================="
echo "Bug 修复说明"
echo "=========================================="
echo "问题: extract_and_process_image_data 依赖 __DEBUG__ 标记"
echo "      但 remove_debug_info 在提取之前就移除了该标记"
echo ""
echo "修复: 移除对 __DEBUG__ 标记的依赖,直接解析 JSON"
echo "      与 chart 提取逻辑保持一致"
echo ""
echo "=========================================="
