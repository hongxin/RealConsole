# Workflow 工作流系统 - 开发故事

> **实验性功能**: LLM 任务自动化工作流
>
> **开发日期**: 2025-10-18
> **状态**: 🔵 实验性（Proof of Concept）

---

## 📖 功能概述

### 设计动机

**问题**: 重复的 LLM 任务流程效率低、成本高

**案例**（BNB 投资分析）:
```
用户输入: "分析 BNB 的投资机会"

传统流程:
1. LLM 理解意图
2. LLM 选择工具 (http_get)      ← 每次都选同一个工具
3. LLM 生成参数 (symbol=BNB)    ← 参数可预测
4. 执行工具调用
5. LLM 分析结果
6. 返回投资建议

问题:
- 2-3 次 LLM 调用（工具选择 + 参数生成 + 分析）
- 10-15 秒响应时间
- 高额 API 成本
```

### 核心思路

**Workflow Intent**: 将常用流程固化为可复用模板

```
用户输入: "分析 BNB 的投资机会"

Workflow 流程:
1. Intent 匹配（正则）              ← 无需 LLM
2. 提取参数 (symbol=BNB)           ← 模板替换
3. 直接调用 http_get               ← 跳过工具选择
4. LLM 分析结果                    ← 仅 1 次 LLM 调用
5. 返回投资建议

收益:
- 1 次 LLM 调用（减少 50-66%）
- 5-8 秒响应时间（减少 40-50%）
- 成本大幅降低
```

---

## ⚡ 实施历程

### Phase 1: 流程分析

**目标**: 深入分析现有 LLM 调用流程

**方法**: 追踪 BNB 投资分析的完整 12 个阶段
1. 输入接收
2. Intent 匹配
3. LLM 工具选择
4. 工具参数生成
5. 工具执行
6. 结果处理
7. LLM 分析
8. ...

**发现**:
- 工具选择环节可预测（总是选 http_get）
- 参数生成可模板化（symbol、url 等）
- 结果可缓存（相同参数查询）

**产出**: 31KB 深度分析报告（已归档）

---

### Phase 2: 数据结构设计

**目标**: 设计灵活可扩展的工作流结构

**核心结构**:
```rust
pub struct WorkflowIntent {
    pub base_intent: Intent,           // 复用 Intent DSL
    pub workflow_steps: Vec<WorkflowStep>,
    pub cache_strategy: CacheStrategy,
    pub description: String,
}

pub enum WorkflowStep {
    ToolCall {                         // 工具调用
        tool_name: String,
        args_template: String,         // 参数模板
        result_key: String,
    },
    LlmAnalyze {                       // LLM 分析
        prompt_template: String,
        result_key: String,
    },
    Transform {                        // 数据转换
        operation: TransformOp,
        input_key: String,
        result_key: String,
    },
}

pub enum CacheStrategy {
    NoCache,
    TimeBased { ttl: Duration },       // 时间缓存
    ParameterBased,                    // 参数缓存
}
```

**设计亮点**:
1. **复用 Intent DSL** - 不重新发明轮子
2. **步骤类型清晰** - 易于理解和扩展
3. **灵活缓存** - 多种策略可选

---

### Phase 3: 执行器实现

**目标**: 高效执行工作流

**WorkflowExecutor 核心逻辑**:
```rust
pub async fn execute(
    &self,
    workflow_intent: &WorkflowIntent,
    intent_match: &IntentMatch,
) -> Result<WorkflowResult> {
    // 1. 提取参数（从 IntentMatch）
    let params = extract_params(intent_match);

    // 2. 检查缓存
    if let Some(cached) = check_cache(&params) {
        return Ok(cached);  // 0.05秒返回！
    }

    // 3. 执行工作流步骤
    for step in &workflow_intent.workflow_steps {
        match step {
            ToolCall { tool_name, args_template, .. } => {
                // 直接调用，跳过 LLM 决策
                let args = substitute_params(args_template, &params);
                let result = execute_tool(tool_name, &args).await?;
            }
            LlmAnalyze { prompt_template, .. } => {
                // 仅在需要时调用 LLM
                let prompt = substitute_params(prompt_template, &params);
                let result = call_llm(&prompt).await?;
            }
            Transform { operation, .. } => {
                // 数据转换（JSON 提取等）
                let result = apply_transform(operation, &data)?;
            }
        }
    }

    // 4. 更新缓存
    update_cache(&params, &result);

    // 5. 返回结果 + 性能统计
    Ok(WorkflowResult { /* ... */ })
}
```

**性能统计**:
```rust
pub struct WorkflowResult {
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,       // 执行耗时
    pub steps_executed: usize,  // 步骤数
    pub llm_calls: usize,       // LLM 调用次数
    pub tool_calls: usize,      // 工具调用次数
}
```

---

### Phase 4: 模板系统集成

**目标**: 提供常用工作流模板

**内置模板**:
```rust
// 1. 投资分析模板
WorkflowIntent::investment_analysis(&symbol) {
    steps: [
        ToolCall("http_get", "url=...{symbol}..."),
        LlmAnalyze("分析以下数据: {price_data}"),
    ],
    cache: ParameterBased,
}

// 2. 技术文档查询模板
WorkflowIntent::doc_query(&keyword) {
    steps: [
        ToolCall("web_search", "query={keyword}"),
        Transform(JsonExtract, "results[0].content"),
        LlmAnalyze("总结: {content}"),
    ],
    cache: TimeBased { ttl: 1h },
}

// 3. 数据转换模板
WorkflowIntent::data_transform(&input) {
    steps: [
        Transform(ParseJson, "{input}"),
        Transform(Extract, "data.metrics"),
        LlmAnalyze("解释: {metrics}"),
    ],
    cache: NoCache,
}
```

---

### Phase 5: 测试与验证

**测试覆盖**: 5 个单元测试 ✅

```rust
#[test]
fn test_workflow_intent_creation() { /* ... */ }

#[test]
fn test_workflow_execution() { /* ... */ }

#[test]
fn test_cache_hit() { /* ... */ }

#[test]
fn test_parameter_substitution() { /* ... */ }

#[test]
fn test_multi_step_workflow() { /* ... */ }
```

**验证案例**: BNB 投资分析

| 指标 | 传统流程 | Workflow | 改进 |
|------|---------|----------|------|
| LLM 调用 | 2-3 次 | 1 次 | ⬇️ 50-66% |
| 响应时间 | 10-15 秒 | 5-8 秒 | ⬇️ 40-50% |
| 缓存命中 | N/A | 0.05 秒 | ⬇️ 99.6% |
| Token 消耗 | ~3000 | ~1000 | ⬇️ 66% |

---

## 🎯 最终成果

### 功能特性

✅ **Workflow 定义** - 灵活的工作流结构
✅ **步骤类型** - ToolCall / LlmAnalyze / Transform
✅ **缓存策略** - 时间 / 参数 / 无缓存
✅ **参数替换** - 模板引擎（支持 `{var}`）
✅ **性能统计** - 完整的执行指标
✅ **内置模板** - 3 个常用场景

### 技术指标

| 指标 | 数值 |
|------|------|
| 代码量 | ~800 行 |
| 测试覆盖 | 5 个测试 ✅ |
| 性能提升 | 40-50% |
| 成本降低 | 50-66% |

---

## 💡 设计亮点

### 1. 基于 Intent DSL 扩展

**复用成熟架构**:
```rust
pub struct WorkflowIntent {
    pub base_intent: Intent,  // 复用！
    // ... workflow 特有字段
}
```

**好处**:
- 不重新发明轮子
- 保持一致性
- 易于集成

### 2. 步骤化设计

**清晰的步骤类型**:
- `ToolCall` - 工具调用（跳过 LLM 决策）
- `LlmAnalyze` - LLM 分析（仅在需要时）
- `Transform` - 数据转换（无 LLM 开销）

**可组合**:
```rust
workflow.steps = [
    ToolCall("http_get", ...),      // 获取数据
    Transform(JsonExtract, ...),    // 提取字段
    LlmAnalyze("分析: {data}"),     // 智能分析
];
```

### 3. 灵活的缓存

**三种策略**:
- `NoCache` - 每次执行（动态内容）
- `TimeBased` - 时间过期（准实时数据）
- `ParameterBased` - 参数缓存（确定性查询）

**智能选择**:
```rust
// 投资数据 - 参数缓存（同一 symbol 可复用）
cache: ParameterBased

// 天气数据 - 时间缓存（10分钟过期）
cache: TimeBased { ttl: 10min }

// AI 创作 - 无缓存（每次不同）
cache: NoCache
```

---

## 🐛 挑战与限制

### 挑战 1: 适用场景有限

**适合**:
- 重复性高的任务（投资分析、文档查询）
- 流程可预测（总是相同的工具链）
- 参数可模板化

**不适合**:
- 高度动态的任务（每次流程不同）
- 复杂决策（需要 LLM 推理）
- 创造性任务（AI 写作等）

### 挑战 2: 维护成本

**问题**: 每个模板都需要维护

**现状**: 仅 3 个内置模板

**决策**: 暂不大规模推广，保持实验性

### 挑战 3: 缓存一致性

**问题**: 参数缓存可能返回过时数据

**解决**:
- 支持 TTL 配置
- 提供手动清除缓存
- 缓存命中显示时间戳

---

## 🎓 经验教训

### 成功经验

1. **深度分析先行** - 31KB 流程分析报告，找准优化点
2. **复用现有架构** - 基于 Intent DSL，快速实现
3. **性能数据验证** - 实测数据支撑决策

### 需要改进

1. **适用性评估** - 并非所有任务都适合 Workflow
2. **用户界面** - 目前仅有代码接口，缺少 UI
3. **模板市场** - 需要社区贡献更多模板

### 未来方向

**如果继续发展**:
- 🔵 可视化工作流编辑器
- 🔵 模板市场（社区分享）
- 🔵 更智能的缓存策略
- 🔵 工作流执行日志

**但当前**:
- ⚪ 保持实验性状态
- ⚪ 观察用户需求
- ⚪ 不投入大量资源

---

## 📚 相关文档

**原始报告**（已归档，供参考）:
- `llm-call-flow-analysis.md` - 31KB 深度分析
- `workflow-implementation-summary.md` - 实施总结
- `workflow-integration-plan.md` - 集成计划
- `workflow-integration-complete.md` - 完成报告
- `workflow-system-usage.md` - 使用指南

**代码位置**:
- `src/dsl/intent/workflow.rs` - 核心实现
- `src/dsl/intent/workflow_templates.rs` - 内置模板

---

## 🚀 总结

**Workflow 系统是一个成功的 PoC（概念验证）**:

- ⚡ **显著性能提升** - 40-50% 响应时间优化
- 🔥 **大幅成本降低** - 50-66% LLM 调用减少
- ✅ **技术可行性** - 800 行代码，5 个测试通过

**但仍是实验性功能**:
- 适用场景有限
- 需要更多用户验证
- 维护成本需要评估

**Vibe Coding 的智慧**:
- 快速验证想法（1 天实现）
- 数据驱动决策（实测数据）
- 保持开放态度（不过度投入）

**下一步**: 观察用户反馈，再决定是否大规模推广 🎯

---

**最后更新**: 2025-10-22
**归档原因**: 简化文档结构，合并 5 个报告
**原始文档**: 5 个文件，2,825 行（已合并到 ~500 行）
**状态**: 实验性功能，持续观察
