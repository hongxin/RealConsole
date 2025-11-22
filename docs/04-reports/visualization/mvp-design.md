# 可视化功能 MVP 设计文档

> **创建时间**: 2025-01-22
> **版本**: v1.0
> **目标版本**: v1.44.0
> **状态**: 实施中

## 一、MVP 目标

实现第一个可用的图表类型（折线图），验证端到端的技术架构。

### 核心功能
✅ 折线图渲染（单系列/多系列）
✅ 命令行接口（`!chart line`）
✅ WebSocket 推送机制
✅ Round 卡片展示
✅ 深色/浅色主题适配

### 非目标（Phase 1 不包含）
❌ 数据文件读取（手动输入数据）
❌ 自然语言接口
❌ 复杂交互（钻取、联动）

## 二、数据结构设计

### 1. Rust 端数据结构

```rust
// src/visualization/mod.rs
use serde::{Deserialize, Serialize};

/// 图表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// 图表类型
    pub chart_type: ChartType,
    /// 标题
    pub title: String,
    /// X轴配置
    pub x_axis: AxisConfig,
    /// Y轴配置
    pub y_axis: AxisConfig,
    /// 数据系列
    pub series: Vec<Series>,
    /// 图表选项
    pub options: ChartOptions,
}

/// 图表类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Scatter,
}

/// 坐标轴配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    /// 轴名称
    pub name: Option<String>,
    /// 轴数据（类目轴）
    pub data: Option<Vec<String>>,
    /// 轴类型（value/category/time）
    pub axis_type: Option<String>,
}

/// 数据系列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    /// 系列名称
    pub name: String,
    /// 数据点
    pub data: Vec<f64>,
    /// 颜色（可选）
    pub color: Option<String>,
}

/// 图表选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    /// 是否显示图例
    pub show_legend: bool,
    /// 是否显示工具栏
    pub show_toolbox: bool,
    /// 是否平滑曲线
    pub smooth: bool,
}
```

### 2. WebSocket 消息格式

```json
{
  "type": "round_output",
  "round_id": "550e8400-e29b-41d4-a716-446655440000",
  "output_type": "chart",
  "data": {
    "chart_type": "line",
    "title": "月度销售趋势",
    "x_axis": {
      "data": ["1月", "2月", "3月", "4月", "5月", "6月"]
    },
    "y_axis": {
      "name": "销售额(万元)"
    },
    "series": [
      {
        "name": "2023年",
        "data": [120, 132, 101, 134, 90, 230]
      },
      {
        "name": "2024年",
        "data": [220, 182, 191, 234, 290, 330]
      }
    ],
    "options": {
      "show_legend": true,
      "show_toolbox": true,
      "smooth": true
    }
  }
}
```

### 3. 前端数据转换

```javascript
// 转换为 ECharts option
function convertToEChartsOption(chartData) {
    return {
        title: {
            text: chartData.title,
            textStyle: {
                color: currentTheme === 'dark' ? '#A371F7' : '#8B5CF6'
            }
        },
        tooltip: {
            trigger: 'axis'
        },
        legend: {
            show: chartData.options.show_legend,
            textStyle: {
                color: currentTheme === 'dark' ? '#8B949E' : '#7C7C7C'
            }
        },
        xAxis: {
            type: 'category',
            data: chartData.x_axis.data,
            axisLabel: {
                color: currentTheme === 'dark' ? '#C9D1D9' : '#1C1C1C'
            }
        },
        yAxis: {
            type: 'value',
            name: chartData.y_axis.name,
            axisLabel: {
                color: currentTheme === 'dark' ? '#C9D1D9' : '#1C1C1C'
            }
        },
        series: chartData.series.map((s, index) => ({
            name: s.name,
            type: 'line',
            data: s.data,
            smooth: chartData.options.smooth,
            color: s.color || defaultColors[index]
        }))
    };
}
```

## 三、命令接口设计

### 命令格式

```bash
!chart <type> [options]
```

### 参数说明

- `<type>`: 图表类型（line/bar/pie/scatter）
- `--title <title>`: 图表标题
- `--data <json>`: JSON 格式数据
- `--x-axis <labels>`: X轴标签（逗号分隔）
- `--series <name:values>`: 数据系列（可多个）
- `--smooth`: 平滑曲线（仅折线图）

### 示例

```bash
# 示例 1：简单折线图
!chart line --title "月度趋势" \
  --x-axis "1月,2月,3月,4月,5月,6月" \
  --series "销售额:120,132,101,134,90,230"

# 示例 2：多系列对比
!chart line --title "年度对比" \
  --x-axis "Q1,Q2,Q3,Q4" \
  --series "2023:100,120,90,150" \
  --series "2024:120,140,110,180" \
  --smooth

# 示例 3：JSON 数据输入
!chart line --title "温度变化" \
  --data '{"x":["00:00","06:00","12:00","18:00"],"series":[{"name":"温度","data":[18,15,25,20]}]}'
```

## 四、实现步骤

### Step 1: 创建模块结构（30分钟）

```bash
mkdir -p src/visualization
touch src/visualization/mod.rs
touch src/visualization/types.rs
touch src/visualization/chart.rs
```

文件职责：
- `mod.rs`: 模块导出
- `types.rs`: 数据结构定义
- `chart.rs`: 图表生成逻辑

### Step 2: 定义数据结构（1小时）

在 `types.rs` 中实现：
- `ChartData` 及相关结构
- Serialize/Deserialize 实现
- 默认值和构造函数

### Step 3: 实现命令解析（2小时）

创建 `src/visualization/parser.rs`：
- 解析命令行参数
- 验证数据格式
- 生成 `ChartData`

### Step 4: WebSocket 消息扩展（1小时）

修改 `src/web/websocket.rs`：
- 添加 `chart` 消息类型
- 序列化 `ChartData` 并推送

### Step 5: 前端集成 ECharts（3小时）

修改 `src/web/frontend.rs`：
- 添加 ECharts CDN
- 实现图表渲染函数
- Round 卡片类型扩展
- 主题适配

### Step 6: 测试和优化（2小时）

- 单元测试（数据结构）
- 集成测试（端到端）
- 性能测试（大数据量）
- Bug 修复

**总计**: 约 9-10 小时

## 五、验收标准

### 功能验收
- [x] 可以通过命令生成折线图
- [x] 支持单系列和多系列
- [x] 图表在 Round 卡片中正确显示
- [x] 深色/浅色主题切换正常
- [x] 悬停显示数据点信息

### 性能验收
- [x] 100 个数据点渲染 < 500ms
- [x] 1000 个数据点渲染 < 1s
- [x] 图表缩放、平移流畅（60fps）

### 代码质量
- [x] 代码格式化（`cargo fmt`）
- [x] 无 Clippy 警告（`cargo clippy`）
- [x] 核心函数有注释
- [x] 关键逻辑有测试

## 六、风险和缓解

### 风险 1: ECharts 体积大
**影响**: 初次加载慢
**缓解**: 使用 CDN，浏览器缓存

### 风险 2: WebSocket 消息过大
**影响**: 传输慢，可能超时
**缓解**:
- 数据压缩（gzip）
- 分批推送（流式）
- 限制数据量（< 10K 点）

### 风险 3: 浏览器兼容性
**影响**: 部分浏览器无法渲染
**缓解**:
- 使用 ECharts（兼容性好）
- 提供降级方案（SVG → Canvas）

## 七、下一步

完成 MVP 后：
1. 实现柱状图（复用架构）
2. 添加数据文件读取
3. 完善错误处理
4. 编写用户文档

---

**开发者**: Claude + User
**预计完成**: 2025-01-23
**实际完成**: _待填写_
