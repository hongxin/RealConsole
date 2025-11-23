# v1.51.0 自然语言驱动可视化 - 智能 Notebook 核心特性

## 🌟 功能概述

**自然语言驱动可视化**是 RealConsole 智能 Notebook 的核心特性，让用户通过自然语言描述直接生成各种专业图表，无需记忆复杂的命令语法。

### 核心价值

- **零学习成本**: 直接用自然语言表达需求，如"画一个销售趋势图"
- **智能理解**: LLM 自动识别图表类型、提取数据、构造参数
- **即时呈现**: 实时生成专业级可交互图表（基于 ECharts）
- **完整记录**: 自动保存到会话历史，支持回溯

## 🎨 支持的图表类型

1. **折线图** (Line) - 趋势分析
2. **柱状图** (Bar) - 对比分析
3. **饼图** (Pie) - 占比分析
4. **散点图** (Scatter) - 相关性分析
5. **面积图** (Area) - 累积趋势
6. **气泡图** (Bubble) - 三维数据
7. **雷达图** (Radar) - 多维对比
8. **热力图** (Heatmap) - 矩阵数据

## 🚀 使用示例

### 示例 1: 折线图
```
帮我画一个销售趋势折线图，X轴是1月到6月，销售额分别是120、132、101、134、90、230
```

**效果**: 自动生成交互式折线图，显示销售趋势

### 示例 2: 饼图
```
创建一个饼图显示产品份额：产品A 35%，产品B 25%，产品C 40%
```

**效果**: 生成饼图，清晰展示各产品占比

### 示例 3: 对比图
```
画一个对比图，显示2023年和2024年的销售额。2023年Q1到Q4分别是100、120、110、150，2024年是130、145、135、180
```

**效果**: 双系列折线图，对比两年数据

## 🏗️ 技术架构

### 设计哲学

遵循 RealConsole 的三大设计哲学：

1. **易经智慧** - 顺应自然语言流，LLM 作为意图理解层
2. **素书精神** - 最小化学习成本，直接表达即可
3. **极简主义** - 重用现有工具系统，无需新增 DSL

### 实现路径

```
用户自然语言输入
    ↓
LLM 意图理解（OpenAI Function Calling）
    ↓
调用 create_chart 工具
    ├── chart_type: 图表类型
    ├── title: 标题
    ├── x_labels: X轴标签
    ├── series: 数据系列
    ├── labels: 饼图标签（可选）
    └── indicators: 雷达图指标（可选）
    ↓
ToolExecutor 检测特殊标记 __CHART_DATA__
    ↓
WebSocket 解析并转换为 ChartData
    ↓
发送 Chart 消息到前端
    ↓
ECharts 渲染交互式图表
```

### 关键技术点

1. **工具调用系统**: 重用现有的 Tool trait 和 ToolExecutor
2. **特殊标记**: `__CHART_DATA__:{json}` 用于识别图表数据
3. **立即返回**: ToolExecutor 检测到标记后立即返回，避免 LLM 二次处理
4. **参数转换**: convert_tool_params_to_chart_data() 将工具参数转换为 ChartData
5. **历史记录**: 自动添加到 Session 的 chart_history

## 📁 代码结构

```
src/
├── builtin_tools.rs          # ChartTool 定义和注册
├── tool_executor.rs           # 工具结果拦截
├── web/
│   ├── websocket.rs           # WebSocket 集成
│   │   ├── extract_and_process_chart_data()
│   │   └── convert_tool_params_to_chart_data()
│   └── session.rs             # 图表历史管理
└── visualization/
    └── types.rs               # ChartData 数据结构

scripts/test/
└── test_nl_visualization.sh   # 测试脚本

docs/04-reports/visualization/
├── nl-visualization-feature.md      # 本文档
├── nl-visualization-test-plan.md    # 测试计划
└── nl-visualization-test-results.md # 测试结果（待生成）
```

## 🎯 实现细节

### ChartTool 参数定义

```rust
// src/builtin_tools.rs:1049-1108
Tool::new(
    "create_chart",
    "创建数据可视化图表...",
    vec![
        Parameter { name: "chart_type", required: true, ... },
        Parameter { name: "title", required: true, ... },
        Parameter { name: "x_labels", required: false, ... },
        Parameter { name: "series", required: true, ... },
        Parameter { name: "labels", required: false, ... },
        Parameter { name: "indicators", required: false, ... },
    ],
    |args| Ok(format!("__CHART_DATA__:{}", args.to_string()))
)
```

### 工具结果拦截

```rust
// src/tool_executor.rs:333-379
if result.content.starts_with("__CHART_DATA__:") {
    let debug_info = Self::encode_debug_info(&conversation_rounds);
    return Ok(format!("{}__DEBUG__{}__CHART__{}",
        "✅ 图表已生成",
        debug_info,
        chart_json
    ));
}
```

### WebSocket 处理

```rust
// src/web/websocket.rs:1088-1110
let (final_content, chart_data_opt) = extract_and_process_chart_data(&clean_content);

if let Some(chart_data) = chart_data_opt {
    session.add_chart_to_history(chart_data.clone(), Some(round_id.clone()), ...);

    let chart_msg = ServerMessage::Chart { round_id, chart_data };
    sender.send(Message::Text(serde_json::to_string(&chart_msg)?)).await?;
}
```

## 🧪 测试

### 快速测试

```bash
# 1. 设置 API Key
export DEEPSEEK_API_KEY="your-api-key-here"

# 2. 运行测试脚本
./scripts/test/test_nl_visualization.sh

# 3. 在浏览器打开 http://127.0.0.1:7788

# 4. 输入测试用例
"帮我画一个销售趋势折线图，X轴是1月到6月，销售额分别是120、132、101、134、90、230"
```

### 测试用例

详见 `nl-visualization-test-plan.md`，包含：
- 6 个测试用例（折线图、饼图、柱状图、多系列、容错等）
- 完整的验证点
- 预期结果

## 📊 性能指标

- **响应时间**: < 5秒（取决于 LLM）
- **图表类型**: 8 种
- **参数支持**: 6 个（3 必选 + 3 可选）
- **代码增量**: ~200 行（3 个文件）

## 🔮 未来优化

### Phase 2: 数据分析增强
- 集成数据处理工具（统计、聚合）
- 支持简单计算（平均值、增长率）
- 自动数据清洗

### Phase 3: 多模态输入
- 上传 CSV/Excel 文件
- 识别表格数据
- 自动推荐图表类型

### Phase 4: Intent DSL 优化
- 添加专门的图表 Intent
- 提高识别准确率
- 支持更复杂的自然语言

### Phase 5: 智能推荐
- 基于数据特征推荐最佳图表
- 提供多种可视化方案
- 交互式调整参数

## 🤝 贡献指南

### 添加新图表类型

1. 在 `ChartType` 枚举中添加类型
2. 更新 `ChartTool` 的 description
3. 在 `convert_tool_params_to_chart_data` 中添加解析逻辑
4. 前端添加对应的 ECharts 配置
5. 编写测试用例

### 改进参数识别

1. 优化 `ChartTool` 的 Parameter description
2. 提供更多示例（在 description 中）
3. 考虑添加 Intent DSL（可选）

## 📚 相关文档

- [测试计划](./nl-visualization-test-plan.md)
- [ChartData 结构](../../00-core/visualization-vision.md)
- [工具系统设计](../../01-understanding/tool-system.md)
- [WebSocket 协议](../../02-practice/developer/websocket-api.md)

---

**版本**: v1.51.0
**状态**: ✅ 已实现，待测试
**作者**: RealConsole Team
**日期**: 2025-01-23
