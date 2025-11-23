# 可视化输出架构演进

**日期**: 2025-01-23
**版本**: v1.51.0+
**状态**: 架构设计 / 待实施

## 🎯 核心问题

当前 RealConsole 有两条不同的执行路径，它们在可视化输出上存在架构不一致：

1. **Intent DSL 路径**: 传统意图识别 → 步骤分解 → 执行
2. **Tool Calling 路径**: LLM 工具调用 → 特殊标记 → 拦截处理

这导致：
- 路径分裂，功能孤立
- 特殊标记 hack（`__CHART__`），脆弱且难以扩展
- 缺少统一的结构化输出抽象

## 🔍 当前实现分析 (v1.51.0)

### 执行流程

```
用户输入: "画一个销售趋势图"
    ↓
LLM 理解意图 (with_tools)
    ↓
调用 create_chart 工具
    ├── 参数: chart_type, title, series, ...
    └── 返回: "__CHART_DATA__:{json}"
    ↓
ToolExecutor 检测标记
    └── 返回: "✅ 图表已生成__CHART__{...}__DEBUG__{...}"
    ↓
WebSocket 层 remove_debug_info()
    └── 移除: __DEBUG__ 及后续内容
    └── 保留: "✅ 图表已生成__CHART__{...}"
    ↓
extract_and_process_chart_data()
    ├── 提取: __CHART_DATA__:{json}
    ├── 解析: JSON → ChartData
    └── 发送: Chart 消息
    ↓
前端 ECharts 渲染
```

### 问题点

1. **脆弱的字符串标记**
   - 依赖特定的字符串顺序
   - 容易被其他处理逻辑破坏（如 v1.51.0 的 bug）
   - 缺乏类型安全

2. **意图分解路径缺失**
   - Intent DSL 无法利用 Tool Calling
   - 两条路径互不兼容
   - 重复的参数提取逻辑

3. **输出类型散落**
   - 文本: Output 消息
   - 图表: Chart 消息（通过标记）
   - 表格: 未来需求（如何实现？）
   - 缺少统一抽象

## 💡 架构改进方案

### 方案 A: 统一结构化输出

引入 `StructuredOutput` 作为第一类公民：

```rust
// src/output/mod.rs
pub enum StructuredOutput {
    /// 纯文本输出
    Text(String),
    /// 图表数据
    Chart(ChartData),
    /// 表格数据 (未来)
    Table(TableData),
    /// 图像数据 (未来)
    Image(ImageData),
    /// Markdown 文档 (未来)
    Document(MarkdownDocument),
}

pub struct ExecutionResult {
    /// 多种输出类型
    pub outputs: Vec<StructuredOutput>,
    /// 执行元数据
    pub metadata: ExecutionMetadata,
    /// 错误信息（如果有）
    pub error: Option<String>,
}
```

**优势**：
- ✅ 类型安全，编译时检查
- ✅ 易于扩展新输出类型
- ✅ 统一处理逻辑
- ✅ 支持一次执行产生多种输出

**集成点**：
- `LlmResponse` 增加 `structured_outputs: Vec<StructuredOutput>`
- `ToolExecutor` 直接返回 `StructuredOutput` 而非字符串
- `ServerMessage` 统一发送 `StructuredOutputMessage`

### 方案 B: Intent + Tool 融合

让 Intent DSL 支持关联 Tool：

```rust
// src/dsl/intent/builtin.rs
intent! {
    name: "create_visualization",
    aliases: ["绘图", "画图", "可视化"],
    patterns: [
        r"画.*图",
        r"创建.*图表",
        r"生成.*可视化"
    ],
    // ✨ 新增：关联的 Tool
    tool: Some("create_chart"),
    // ✨ 参数提取器（从自然语言中提取 Tool 参数）
    params_extractor: |text| {
        // 使用 LLM 或规则提取参数
        extract_chart_params(text)
    },
}
```

**执行流程**：
```
用户输入
    ↓
IntentMatcher 匹配到 "create_visualization"
    ↓
调用 params_extractor 提取参数
    ↓
调用关联的 Tool: create_chart(params)
    ↓
Tool 返回 StructuredOutput::Chart
    ↓
统一发送 Chart 消息
```

**优势**：
- ✅ 保留 Intent 的高层意图理解
- ✅ 利用 Tool 的底层执行能力
- ✅ 统一两条路径
- ✅ 可选：Intent 可以不关联 Tool

### 方案 C: Middleware 管道

引入输出处理中间件：

```rust
// src/output/middleware.rs
pub trait OutputMiddleware {
    fn process(&self, result: &mut ExecutionResult) -> anyhow::Result<()>;
}

pub struct ChartDetector;
impl OutputMiddleware for ChartDetector {
    fn process(&self, result: &mut ExecutionResult) -> anyhow::Result<()> {
        for output in &mut result.outputs {
            if let StructuredOutput::Text(text) = output {
                if let Some(chart_data) = extract_chart_data(text) {
                    result.outputs.push(StructuredOutput::Chart(chart_data));
                }
            }
        }
        Ok(())
    }
}

// 注册中间件
pipeline
    .add(ChartDetector)
    .add(TableDetector)
    .add(ImageDetector);
```

**优势**：
- ✅ 解耦检测逻辑
- ✅ 易于扩展
- ✅ 可插拔设计
- ✅ 职责清晰

## 🚀 推荐实施路线

### Phase 1: 基础重构（v1.52.0）

**目标**: 引入 StructuredOutput 抽象

1. **新增 `src/output/mod.rs`**
   ```rust
   pub enum StructuredOutput { ... }
   pub struct ExecutionResult { ... }
   ```

2. **修改 `LlmResponse`**
   ```rust
   pub struct LlmResponse {
       pub text: String,
       pub structured_outputs: Vec<StructuredOutput>,  // 新增
       ...
   }
   ```

3. **重构 `ToolExecutor`**
   - Tool handler 返回 `Result<ExecutionResult, String>`
   - 检测 ChartTool 直接返回 `StructuredOutput::Chart`

4. **修改 WebSocket 层**
   - 遍历 `structured_outputs` 发送对应消息
   - 移除字符串标记 hack

**预期成果**：
- ✅ 移除 `__CHART__` 标记
- ✅ 类型安全的输出处理
- ✅ 为未来扩展打好基础

### Phase 2: Intent 集成（v1.53.0）

**目标**: Intent DSL 支持 Tool 关联

1. **扩展 Intent 定义**
   ```rust
   pub struct Intent {
       ...
       pub associated_tool: Option<String>,
       pub params_extractor: Option<ParamsExtractor>,
   }
   ```

2. **实现参数提取**
   - 简单规则提取（regex）
   - LLM 辅助提取（复杂情况）

3. **统一执行路径**
   - Intent 匹配 → Tool 调用 → 统一输出

**预期成果**：
- ✅ 两条路径统一
- ✅ Intent 复用 Tool 能力
- ✅ 更灵活的扩展性

### Phase 3: 多模态输出（v1.54.0+）

**目标**: 支持更多输出类型

1. **表格输出**
   ```rust
   StructuredOutput::Table(TableData)
   ```

2. **图像输出**
   ```rust
   StructuredOutput::Image(ImageData)
   ```

3. **复合输出**
   一次执行返回多种类型（文本 + 图表 + 表格）

**预期成果**：
- ✅ 丰富的可视化能力
- ✅ 真正的"智能 Notebook"

## 📊 架构对比

| 维度 | 当前架构 (v1.51.0) | 改进架构 (v1.52.0+) |
|------|-------------------|-------------------|
| **输出抽象** | 字符串标记 | `StructuredOutput` enum |
| **类型安全** | ❌ 运行时字符串匹配 | ✅ 编译时类型检查 |
| **扩展性** | ⚠️ 需要添加新标记 | ✅ 添加新 enum 变体 |
| **Intent 集成** | ❌ 不支持 | ✅ 支持 Tool 关联 |
| **多种输出** | ❌ 一次一种 | ✅ 一次多种 |
| **维护性** | ⚠️ 脆弱（顺序依赖） | ✅ 健壮（类型保证） |

## 🎓 设计原则

1. **类型安全优先**: 尽可能在编译时捕获错误
2. **逐步演进**: 不破坏现有功能，渐进式改进
3. **职责分离**: 输出检测、转换、发送各司其职
4. **易于扩展**: 新增输出类型不影响现有代码
5. **哲学一致**: 符合易经变化、素书策略、极简主义

## 🤝 向后兼容

Phase 1 实施时需要保证：
- ✅ 现有 Tool Calling 继续工作
- ✅ Intent DSL 不受影响
- ✅ 前端无需修改（消息格式不变）
- ✅ 测试全部通过

## 📝 下一步行动

1. **立即**: 继续完成 v1.51.0 测试（确认 bug fix 有效）
2. **短期**: 设计 `StructuredOutput` API（v1.52.0）
3. **中期**: 实施 Phase 1 重构
4. **长期**: Intent + Tool 融合

---

**作者**: Claude Code Agent
**审阅**: 待团队讨论
**参考**: [可视化愿景](../../00-core/visualization-vision.md), [工具系统](../tool-system.md)
