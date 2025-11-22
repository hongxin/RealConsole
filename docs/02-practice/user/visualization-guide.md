# RealConsole 数据可视化指南

> **版本**: v1.44.0 - v1.45.0
> **状态**: 稳定功能
> **适用场景**: Web 终端模式

## 快速开始

RealConsole 内置了基于 ECharts 5 的专业数据可视化功能，支持在 Web 终端中生成交互式图表。

### 启动 Web 终端

```bash
realconsole web
# 访问 http://127.0.0.1:7788
```

## 支持的图表类型

### 1. 折线图 (Line Chart)

**用途**: 趋势分析、时间序列数据

```bash
# 单系列折线图
!chart line --title "月度销售" --x-axis "1月,2月,3月,4月" --series "销售额:120,132,145,138"

# 多系列折线图
!chart line --title "销售对比" --x-axis "Q1,Q2,Q3,Q4" --series "去年:100,110,105,120" --series "今年:120,132,145,138"
```

**参数说明**:
- `--title`: 图表标题
- `--x-axis`: X 轴数据（逗号分隔）
- `--series`: 数据系列，格式为 `名称:值1,值2,值3`（可多个）

### 2. 柱状图 (Bar Chart)

**用途**: 对比分析、分类数据

```bash
# 单系列柱状图
!chart bar --title "月度利润" --x-axis "1月,2月,3月" --series "利润:40,47,31"

# 多系列柱状图
!chart bar --title "部门对比" --x-axis "研发,销售,市场" --series "预算:100,80,60" --series "实际:95,85,55"
```

### 3. 饼图 (Pie Chart)

**用途**: 占比展示、百分比分析

```bash
# 带标签饼图
!chart pie --title "市场份额" --labels "产品A,产品B,产品C,产品D" --series "份额:35,25,30,10"

# 不带标签（自动编号）
!chart pie --title "销售占比" --series "销售额:120,230,180,90"
```

**参数说明**:
- `--labels`: 扇区标签（可选，不提供则自动编号）
- `--series`: 数据系列，格式同上

**视觉效果**:
- 悬停显示百分比
- 扇区高亮阴影
- 图例点击切换

### 4. 散点图 (Scatter Plot)

**用途**: 相关性分析、分布展示

```bash
# 单系列散点图
!chart scatter --title "身高体重分布" --x-name "身高(cm)" --y-name "体重(kg)" --data "170,65 175,70 160,55 180,80 165,58"

# 多系列散点图
!chart scatter --title "测试成绩分布" --x-name "数学" --y-name "英语" --data "85,90 78,82 92,88" --data "70,75 65,68 72,78"
```

**参数说明**:
- `--x-name`: X 轴名称
- `--y-name`: Y 轴名称
- `--data`: 坐标点，格式为 `x1,y1 x2,y2 ...`（可多个系列）

**视觉效果**:
- 散点大小 10px
- 悬停放大至 15px
- 多系列不同颜色

### 5. CSV 文件图表

**用途**: 快速可视化 CSV 数据

#### 准备 CSV 文件

创建测试文件 `/tmp/sales.csv`:
```csv
月份,销售额,成本,利润
1月,120,80,40
2月,132,85,47
3月,101,70,31
4月,134,90,44
```

#### 生成图表

```bash
# 单系列折线图
!chart csv /tmp/sales.csv --type line --title "月度销售趋势" --x-col "月份" --y-col "销售额"

# 多系列折线图
!chart csv /tmp/sales.csv --type line --title "销售成本对比" --x-col "月份" --y-col "销售额" --y-col "成本"

# 柱状图
!chart csv /tmp/sales.csv --type bar --title "月度利润" --x-col "月份" --y-col "利润"
```

**参数说明**:
- `--type`: 图表类型（`line` 或 `bar`）
- `--title`: 图表标题（可选）
- `--x-col`: X 轴列名
- `--y-col`: Y 轴列名（可多个，生成多系列）

**注意事项**:
- 仅支持服务器本地 CSV 文件
- 建议文件大小 < 1MB
- 第一行必须是列名（header）
- 数值列会自动转换为数字类型

## 图表功能

### 交互功能

所有图表支持：
- **悬停提示** (Tooltip): 显示详细数值
- **图例交互**: 点击图例切换系列显示/隐藏
- **工具栏**: 缩放、还原、保存图片

### 保存图片

点击图表右上角的"保存图片"按钮：
- 格式：PNG
- 分辨率：自动适配（2x）
- 背景：白色

### 主题适配

图表自动适配 RealConsole 主题：
- **深色主题**: 暗色背景，柔和文字
- **浅色主题**: 白色背景，深色文字
- **配色体系**: 紫绿金三色（#A371F7, #0ECB81, #F0B90B）

### 响应式布局

- 自动适配窗口大小
- 移动端友好（触屏操作）
- 折叠/展开与回合卡片集成

## 实用示例

### 示例 1：项目进度可视化

```bash
!chart bar --title "Sprint 进度" --x-axis "需求,开发,测试,部署" --series "已完成:8,12,5,2" --series "进行中:2,3,4,1"
```

### 示例 2：性能趋势分析

```bash
!chart line --title "响应时间趋势" --x-axis "Mon,Tue,Wed,Thu,Fri" --series "API响应(ms):120,115,135,110,105"
```

### 示例 3：资源占比

```bash
!chart pie --title "服务器资源占用" --labels "CPU,内存,磁盘,网络" --series "占用率:45,62,38,25"
```

### 示例 4：相关性分析

```bash
!chart scatter --title "代码复杂度 vs Bug 数" --x-name "圈复杂度" --y-name "Bug 数量" --data "10,2 15,3 25,8 30,12 35,15"
```

### 示例 5：日志分析可视化

假设有 `/tmp/logs.csv`：
```csv
时间,错误数,警告数
00:00,2,5
01:00,1,3
02:00,0,2
03:00,3,7
```

```bash
!chart csv /tmp/logs.csv --type line --title "日志趋势" --x-col "时间" --y-col "错误数" --y-col "警告数"
```

## 最佳实践

### 1. 选择合适的图表类型

| 场景 | 推荐图表 |
|------|---------|
| 时间趋势 | 折线图 |
| 类别对比 | 柱状图 |
| 占比分析 | 饼图 |
| 相关性 | 散点图 |

### 2. 数据准备

- **X 轴数据**: 清晰的标签（日期、类别等）
- **Y 轴数据**: 数值类型，避免缺失值
- **系列名称**: 简洁明了（如"销售额"而非"2024年Q1销售额统计"）

### 3. 标题和轴名称

- **标题**: 简短有力，突出重点
- **轴名称**: 包含单位（如"销售额(万元)"、"响应时间(ms)"）

### 4. CSV 文件规范

- 使用 UTF-8 编码
- 第一行为列名
- 数值列避免混入文本
- 日期建议使用统一格式（如"2024-01"）

## 错误处理

### 常见错误及解决

**1. 数据长度不匹配**
```
❌ 系列 '销售额' 数据长度(3)与X轴(2)不匹配
```
**解决**: 确保每个 `--series` 的数据数量与 `--x-axis` 一致

**2. CSV 文件不存在**
```
❌ 文件不存在: /tmp/data.csv
```
**解决**: 检查文件路径，确保服务器可访问

**3. CSV 列不存在**
```
❌ 找不到列: 不存在的列
```
**解决**: 检查列名拼写，注意大小写

**4. 饼图标签长度不匹配**
```
❌ 饼图 labels 长度(2)与 data 长度(3)不匹配
```
**解决**: 移除 `--labels` 参数或确保数量匹配

## 限制与未来增强

### 当前限制

- CSV 文件仅支持服务器本地路径
- 导出格式仅支持 PNG
- 饼图不支持环形图（Donut）
- 散点图不支持气泡图（可变大小）

### 未来计划 (Phase 3)

- 面积图 (Area Chart)
- 混合图表（折线+柱状）
- 双 Y 轴支持
- 浏览器文件上传
- SVG 矢量图导出
- 数据导出（Excel）

## 技术架构

### 后端

- **模块**: `src/visualization/`
- **解析器**: `parser.rs` (命令行参数解析)
- **数据结构**: `types.rs` (ChartData, Series)
- **CSV 支持**: `csv.rs` (csv = "1.3" 库)

### 前端

- **渲染引擎**: ECharts 5.4.3
- **集成位置**: `src/web/frontend.rs`
- **消息类型**: `ServerMessage::Chart`

### 通信流程

```
用户输入 → WebSocket → 命令解析 → ChartData → JSON → 前端 → ECharts 渲染
```

## 参考资料

- **实施计划**: [docs/04-reports/visualization/phase2-implementation-plan.md](../../04-reports/visualization/phase2-implementation-plan.md)
- **进度报告**: [docs/04-reports/visualization/phase2-progress-report.md](../../04-reports/visualization/phase2-progress-report.md)
- **完成总结**: [docs/04-reports/visualization/phase2-completion-summary.md](../../04-reports/visualization/phase2-completion-summary.md)
- **ECharts 文档**: https://echarts.apache.org/zh/index.html

---

**最后更新**: 2025-01-22
**版本**: v1.45.0
**反馈**: [GitHub Issues](https://github.com/hongxin/RealConsole/issues)
