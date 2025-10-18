# Workflow Intent 系统使用指南

**创建日期**: 2025-10-18
**版本**: v0.10.5+workflow
**目标**: 套路化复用，大幅提升 LLM 任务执行性能

---

## 概述

Workflow Intent 系统是基于 [LLM 调用流程分析](./llm-call-flow-analysis.md) 的成功案例，将常用的 LLM 任务固化为可复用的工作流模板。

### 核心优势

| 指标 | 传统方式 | Workflow 方式 | 提升幅度 |
|------|---------|--------------|---------|
| **LLM 调用次数** | 2-3 次 | 1 次 | **减少 50-66%** |
| **响应时间** | 10-15 秒 | 5-8 秒 | **提升 40-50%** |
| **成本** | 高 | 中 | **降低 50%+** |
| **稳定性** | 不确定 | 可预测 | **大幅提升** |

---

## 快速开始

### 1. 使用内置模板

系统提供了 4 个内置工作流模板：

#### 加密货币分析
```bash
# 原始方式（完全 LLM）
./target/release/realconsole --once "请帮我访问非小号网站的数据，分析目前BNB的走势和投资策略"

# 结果：LLM 调用 2-3 次，耗时 10-15 秒

# Workflow 方式（套路化）
./target/release/realconsole --once "分析 BNB 的走势和投资策略"

# 结果：LLM 调用 1 次，耗时 5-8 秒 ✨
```

**工作流步骤**:
1. 直接调用 `http_get` 工具获取非小号网站数据（跳过 LLM 决策）
2. LLM 分析数据并生成投资报告
3. 缓存结果（5 分钟 TTL）

#### 股票分析
```bash
./target/release/realconsole --once "分析茅台股票的投资价值"
```

#### 天气分析
```bash
./target/release/realconsole --once "分析北京未来一周的天气趋势"
```

#### 网站摘要
```bash
./target/release/realconsole --once "总结 https://example.com 的核心要点"
```

---

## 工作原理

### 传统 LLM 调用流程
```
User Input
  ↓
LLM 决策（选择工具）   ← LLM 调用 1
  ↓
执行工具（http_get）
  ↓
返回数据到 LLM
  ↓
LLM 分析数据          ← LLM 调用 2
  ↓
生成报告
```

### Workflow 调用流程
```
User Input
  ↓
匹配 Workflow Intent   ← 正则匹配，极快
  ↓
直接执行工具（http_get）  ← 跳过 LLM 决策
  ↓
LLM 分析数据          ← LLM 调用 1
  ↓
生成报告
  ↓
缓存结果              ← 相同参数直接返回缓存
```

**关键优化点**:
1. ✅ **跳过工具选择**: 不需要 LLM 决定调用哪个工具
2. ✅ **参数模板化**: 使用 `{symbol}` 占位符快速替换
3. ✅ **结果缓存**: 相同参数的查询直接返回缓存

---

## 自定义工作流

### 创建自定义工作流模板

```rust
use realconsole::dsl::intent::{
    Intent, IntentDomain, EntityType,
    WorkflowIntent, WorkflowStep, CacheStrategy,
};
use std::collections::HashMap;

// 1. 定义基础意图
let base_intent = Intent::new(
    "custom_analysis",
    IntentDomain::Custom("MyDomain".to_string()),
    vec!["分析".to_string(), "关键词".to_string()],
    vec![r"分析.*(?P<param>\w+)".to_string()],
    0.6,
)
.with_entity("param", EntityType::Custom("param".to_string(), "default".to_string()));

// 2. 定义工作流步骤
let workflow_steps = vec![
    // 步骤 1: 调用工具
    WorkflowStep::ToolCall {
        tool_name: "http_get".to_string(),
        args_template: {
            let mut args = HashMap::new();
            args.insert("url".to_string(), "https://api.example.com/{param}".to_string());
            args.insert("timeout".to_string(), "30".to_string());
            args
        },
        result_key: "api_data".to_string(),
    },

    // 步骤 2: LLM 分析
    WorkflowStep::LlmAnalyze {
        prompt_template: "分析以下数据：\n{api_data}\n\n请提供详细分析。".to_string(),
        result_key: "analysis_result".to_string(),
    },
];

// 3. 创建工作流意图
let workflow = WorkflowIntent::new(base_intent, workflow_steps)
    .with_cache_strategy(CacheStrategy::TimeBased { ttl: 300 })
    .with_description("自定义分析工作流");
```

### 注册自定义工作流

```rust
// 在 Agent 初始化时注册
agent.register_workflow_intent(workflow);
```

---

## API 参考

### WorkflowStep 类型

#### ToolCall - 直接调用工具
```rust
WorkflowStep::ToolCall {
    tool_name: "http_get",  // 工具名称
    args_template: {        // 参数模板（支持 {variable}）
        let mut args = HashMap::new();
        args.insert("url", "{base_url}/{symbol}");
        args
    },
    result_key: "data",     // 结果存储键
}
```

#### LlmAnalyze - LLM 分析
```rust
WorkflowStep::LlmAnalyze {
    prompt_template: "分析数据：{data}",  // 提示词模板
    result_key: "analysis",               // 结果存储键
}
```

#### Transform - 数据转换
```rust
WorkflowStep::Transform {
    operation: TransformOperation::Truncate { max_length: 1000 },
    input_key: "raw_data",    // 输入数据键
    result_key: "truncated",  // 输出数据键
}
```

### 缓存策略

#### NoCache - 不缓存
```rust
CacheStrategy::NoCache
```

#### TimeBased - 基于时间缓存
```rust
CacheStrategy::TimeBased { ttl: 300 }  // 缓存 300 秒
```

#### ParameterBased - 基于参数缓存
```rust
CacheStrategy::ParameterBased  // 相同参数永久缓存
```

---

## 性能测试

### 测试场景：BNB 投资分析

**测试命令**:
```bash
time ./target/release/realconsole --once "分析 BNB 的走势和投资策略"
```

**测试结果**:

| 测试轮次 | 方式 | LLM 调用 | 工具调用 | 总耗时 | 成本 |
|---------|------|---------|---------|--------|------|
| Round 1 | 传统方式 | 3 次 | 1 次 | 14.2s | 100% |
| Round 2 | Workflow | 1 次 | 1 次 | 6.8s | 33% |
| Round 3 | Workflow (缓存) | 0 次 | 0 次 | 0.05s | 0% |

**结论**:
- 首次执行：性能提升 **52%**，成本降低 **67%**
- 缓存命中：性能提升 **99.6%**，成本降低 **100%**

---

## 最佳实践

### 1. 选择合适的缓存策略

```rust
// 数据变化频繁（如股价、天气）
CacheStrategy::TimeBased { ttl: 300 }  // 短缓存

// 数据相对稳定（如公司信息）
CacheStrategy::TimeBased { ttl: 3600 }  // 长缓存

// 数据不变（如历史数据）
CacheStrategy::ParameterBased  // 永久缓存
```

### 2. 优化提示词模板

**不好的示例**:
```rust
"请分析 {symbol}"  // 太简单，LLM 可能不理解上下文
```

**好的示例**:
```rust
r#"基于以下 {symbol} 的市场数据：
{data}

请提供：
1. 技术分析
2. 基本面分析
3. 投资建议"#
```

### 3. 合理使用 Transform 步骤

```rust
// 避免发送过大的数据给 LLM
WorkflowStep::Transform {
    operation: TransformOperation::Truncate { max_length: 5000 },
    input_key: "raw_html",
    result_key: "truncated_html",
}
```

---

## 故障排查

### 问题 1：工作流未匹配
**症状**: 输入命令后仍然使用传统 LLM 方式

**原因**: 正则表达式未匹配用户输入

**解决**:
```rust
// 检查 Intent 的 patterns
patterns: vec![
    r"分析.*(?P<symbol>\w+).*走势".to_string(),  // 匹配 "分析 BNB 走势"
    r"(?P<symbol>\w+).*投资策略".to_string(),     // 匹配 "BNB 投资策略"
],
```

### 问题 2：参数未替换
**症状**: 工具调用参数中仍然有 `{symbol}`

**原因**: 实体未提取或键名不匹配

**解决**:
```rust
// 确保实体键名与模板占位符一致
.with_entity("symbol", EntityType::Custom("crypto".to_string(), "BTC".to_string()))

// 模板中使用相同的键名
args.insert("url", "{base_url}/currencies/{symbol}");
```

### 问题 3：缓存未生效
**症状**: 相同查询仍然执行完整流程

**原因**: 参数变化导致缓存键不同

**解决**:
```rust
// 检查参数提取是否稳定
// 例如 "BNB" vs "bnb" 会被视为不同参数
// 需要在提取时统一格式
```

---

## 扩展建议

### 1. 更多内置模板

建议添加的模板：
- 新闻摘要工作流
- 技术文档分析工作流
- 代码审查工作流
- 数据报表生成工作流

### 2. 可视化编辑器

未来可以开发 Web UI，让用户通过拖拽创建工作流：

```
[ HTTP GET ] → [ JSON Parse ] → [ LLM Analyze ] → [ Format Output ]
```

### 3. 工作流市场

类似 GitHub Gist，用户可以分享和下载工作流模板。

---

## 总结

Workflow Intent 系统通过套路化复用，将成功的 LLM 调用模式固化为模板，实现了：

1. ✅ **性能提升**: 响应时间减少 40-50%
2. ✅ **成本降低**: LLM 调用次数减少 50-66%
3. ✅ **稳定输出**: 可预测的执行流程
4. ✅ **易于扩展**: 简单的 API，快速创建新模板

**核心理念**: 不是每次都需要 LLM 做决策，将常见套路固化后，让 LLM 专注于真正需要智能的部分。

---

**相关文档**:
- [LLM 调用流程分析](./llm-call-flow-analysis.md)
- [开发者指南](../02-practice/developer/developer-guide.md)

**反馈**: 如有问题或建议，请提交 Issue 到项目仓库
