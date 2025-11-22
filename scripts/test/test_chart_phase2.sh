#!/bin/bash
# Phase 2 可视化功能端到端测试脚本
# v1.45.0: 测试饼图、散点图和 CSV 文件支持

set -e  # 遇到错误立即退出

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  RealConsole Phase 2 可视化测试${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 创建测试 CSV 文件
echo -e "${YELLOW}[1/4] 创建测试 CSV 文件...${NC}"

CSV_FILE="/tmp/realconsole_test_sales.csv"
cat > "$CSV_FILE" << 'EOF'
月份,销售额,成本,利润
1月,120,80,40
2月,132,85,47
3月,101,70,31
4月,134,90,44
5月,90,65,25
6月,230,150,80
EOF

echo -e "${GREEN}✓ CSV 文件已创建: $CSV_FILE${NC}"
echo ""

# 显示测试用例
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  测试用例列表${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

echo -e "${YELLOW}【饼图测试】${NC}"
echo -e "${GREEN}用例 1:${NC} 简单饼图（带 labels）"
echo '!chart pie --title "市场份额分布" --labels "产品A,产品B,产品C,产品D" --series "份额:35,25,30,10"'
echo ""

echo -e "${GREEN}用例 2:${NC} 饼图（不带 labels）"
echo '!chart pie --title "销售占比" --series "销售额:120,230,180,90"'
echo ""

echo -e "${YELLOW}【散点图测试】${NC}"
echo -e "${GREEN}用例 3:${NC} 简单散点图"
echo '!chart scatter --title "身高体重分布" --x-name "身高(cm)" --y-name "体重(kg)" --data "170,65 175,70 160,55 180,80 165,58"'
echo ""

echo -e "${GREEN}用例 4:${NC} 多系列散点图"
echo '!chart scatter --title "测试成绩分布" --x-name "数学" --y-name "英语" --data "85,90 78,82 92,88" --data "70,75 65,68 72,78"'
echo ""

echo -e "${GREEN}用例 5:${NC} 大数据散点图"
echo '!chart scatter --title "随机分布" --data "10,20 15,25 20,15 25,30 30,20 35,35 40,25 45,40 50,35"'
echo ""

echo -e "${YELLOW}【CSV 图表测试】${NC}"
echo -e "${GREEN}用例 6:${NC} CSV 折线图（单系列）"
echo "!chart csv $CSV_FILE --type line --title \"月度销售趋势\" --x-col \"月份\" --y-col \"销售额\""
echo ""

echo -e "${GREEN}用例 7:${NC} CSV 折线图（多系列）"
echo "!chart csv $CSV_FILE --type line --title \"销售成本对比\" --x-col \"月份\" --y-col \"销售额\" --y-col \"成本\""
echo ""

echo -e "${GREEN}用例 8:${NC} CSV 柱状图"
echo "!chart csv $CSV_FILE --type bar --title \"月度利润\" --x-col \"月份\" --y-col \"利润\""
echo ""

echo -e "${YELLOW}【错误处理测试】${NC}"
echo -e "${GREEN}用例 9:${NC} 饼图验证失败（labels 长度不匹配）"
echo '!chart pie --labels "A,B" --series "1,2,3"'
echo ""

echo -e "${GREEN}用例 10:${NC} 散点图格式错误"
echo '!chart scatter --data "1,2 3"'
echo ""

echo -e "${GREEN}用例 11:${NC} CSV 文件不存在"
echo '!chart csv /tmp/nonexistent.csv --type line --x-col "A" --y-col "B"'
echo ""

echo -e "${GREEN}用例 12:${NC} CSV 列不存在"
echo "!chart csv $CSV_FILE --type line --x-col \"不存在的列\" --y-col \"销售额\""
echo ""

# 启动服务器
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  启动 Web 服务器${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

echo -e "${YELLOW}即将启动 RealConsole Web 终端...${NC}"
echo -e "${YELLOW}服务器地址: http://127.0.0.1:7788${NC}"
echo ""
echo -e "${GREEN}请在浏览器中测试以上用例${NC}"
echo ""
echo -e "${BLUE}验收检查清单:${NC}"
echo ""
echo "【饼图】"
echo "  [ ] 1. 扇区正确显示（紫、绿、金等颜色）"
echo "  [ ] 2. Labels 正确标注"
echo "  [ ] 3. 鼠标悬停显示百分比"
echo "  [ ] 4. 图例可点击切换"
echo ""
echo "【散点图】"
echo "  [ ] 5. 散点正确定位"
echo "  [ ] 6. 多系列使用不同颜色"
echo "  [ ] 7. 悬停散点放大"
echo "  [ ] 8. 坐标轴显示轴名称"
echo ""
echo "【CSV 图表】"
echo "  [ ] 9. CSV 文件正确读取"
echo "  [ ] 10. 多列数据显示为多系列"
echo "  [ ] 11. X 轴使用第一列数据"
echo "  [ ] 12. 图表类型正确（折线/柱状）"
echo ""
echo "【错误处理】"
echo "  [ ] 13. 错误提示清晰友好"
echo "  [ ] 14. 包含使用示例"
echo "  [ ] 15. Round 状态正确（失败标记）"
echo ""
echo "【通用功能】"
echo "  [ ] 16. 主题切换（深色/浅色）正常"
echo "  [ ] 17. 工具栏功能正常（保存图片、缩放）"
echo "  [ ] 18. 窗口调整大小时图表响应式调整"
echo ""

echo -e "${YELLOW}按 Ctrl+C 停止服务器${NC}"
echo ""

# 设置环境变量并启动
export DEEPSEEK_API_KEY="${DEEPSEEK_API_KEY:-test-key}"
./target/release/realconsole web --port 7788

# 清理
echo ""
echo -e "${YELLOW}清理测试文件...${NC}"
rm -f "$CSV_FILE"
echo -e "${GREEN}✓ 测试文件已清理${NC}"
