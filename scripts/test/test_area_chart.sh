#!/bin/bash
# Test script for Area Chart feature (v1.47.0)

set -e

echo "========================================="
echo "测试面积图功能 (v1.47.0)"
echo "========================================="

# 创建测试 CSV 文件
TEST_CSV="/tmp/sales_trend.csv"
cat > "$TEST_CSV" <<'EOF'
Month,Revenue,Cost
Jan,12000,8000
Feb,15000,9000
Mar,18000,10000
Apr,22000,12000
May,25000,14000
Jun,28000,15000
EOF

echo ""
echo "1. 创建测试 CSV 文件: $TEST_CSV"
cat "$TEST_CSV"

echo ""
echo "2. 测试基本面积图命令"
echo "   命令: !chart csv $TEST_CSV --type area --x-col Month --y-col Revenue"
echo ""

# 测试面积图解析
echo "3. 测试多系列面积图（收入 vs 成本）"
echo "   命令: !chart csv $TEST_CSV --type area --x-col Month --y-col Revenue,Cost --title '收入与成本趋势'"
echo ""

echo "4. 测试平滑面积图"
echo "   命令: !chart csv $TEST_CSV --type area --x-col Month --y-col Revenue --smooth"
echo ""

echo "========================================="
echo "✅ 测试脚本已准备就绪"
echo ""
echo "请在 Web 终端中手动测试以下命令:"
echo ""
echo "1. 启动 Web 服务:"
echo "   DEEPSEEK_API_KEY=\"test-key\" ./target/release/realconsole web"
echo ""
echo "2. 在浏览器中 (http://127.0.0.1:7788) 执行:"
echo "   !chart csv $TEST_CSV --type area --x-col Month --y-col Revenue"
echo "   !chart csv $TEST_CSV --type area --x-col Month --y-col Revenue,Cost"
echo "   !chart csv $TEST_CSV --type area --x-col Month --y-col Revenue --smooth"
echo ""
echo "3. 验证渐变填充效果:"
echo "   - 顶部 50% 不透明度"
echo "   - 底部 6% 不透明度"
echo "   - 平滑曲线（如果指定 --smooth）"
echo "========================================="

# 清理
echo ""
echo "清理测试文件: rm -f $TEST_CSV"
