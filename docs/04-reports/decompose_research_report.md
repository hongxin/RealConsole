# Web 版本意图拆解功能调研报告

**日期**: 2025-01-08
**目的**: 对比直接执行与 `/decompose` 命令的差异，改进意图拆解功能

---

## 📊 测试场景

**测试输入**: "请经过认真的思考，帮我看看人民日报网站今天的新闻"

### 方式一：直接执行（无前缀）
```
请经过认真的思考，帮我看看人民日报网站今天的新闻
```

### 方式二：意图拆解执行
```
/decompose 请经过认真的思考，帮我看看人民日报网站今天的新闻
```

---

## 🔍 核心差异分析

### 直接执行流程 (`src/web/websocket.rs:168-175`)

```
用户输入（自然语言）
    ↓
智能路由判断 (CommandRouter)
    ↓
识别为 NaturalLanguage
    ↓
尝试 Intent DSL 匹配
    ├─ 匹配成功 → execute_intent()
    │    ↓
    │  执行实际工具/命令
    │    ↓
    │  返回真实结果
    │
    └─ 匹配失败 → execute_llm_chat()
         ↓
       调用 LLM 对话
         ↓
       LLM 可能调用 Function Calling
         ↓
       执行工具（如 WebFetch）
         ↓
       返回实际抓取的网站内容
```

**关键特点**：
- ✅ **会真正执行工具调用**
- ✅ **返回实际结果**（如真实的网站内容）
- ✅ **支持 LLM Function Calling**
- ✅ **用户体验智能流畅**

### /decompose 执行流程 (`src/web/websocket.rs:791-1043`)

```
/decompose <query>
    ↓
execute_decompose_command()
    ↓
【快速路径】尝试 Intent DSL 预识别 (line 862-932)
    ├─ 匹配成功
    │    ↓
    │  发送 IntentUnderstanding 消息
    │    ↓
    │  发送 StepProgress 消息（pending 状态）
    │    ↓
    │  返回 "⚡ 通过 Intent DSL 快速识别"
    │    ↓
    │  ❌ 不执行任何步骤
    │
    └─ 匹配失败
         ↓
    【LLM 拆解路径】(line 934-1043)
         ↓
       发送 Thinking 消息
         ↓
       调用 decomposer.decompose(query)
         ↓
       发送 IntentUnderstanding 消息
         ↓
       发送 StepProgress 消息（pending 状态）
         ↓
       返回 "🤖 通过 LLM 拆解执行"
         ↓
       ❌ 不执行任何步骤
```

**关键特点**：
- ❌ **只分析意图，不执行步骤**
- ❌ **只返回计划，无实际结果**
- ⚠️  **存在 `execute_plan()` 函数（line 1045），但未被调用**
- 💡 **适合教学、可视化、调试场景**

---

## 🎯 问题总结

### 1. 根本差异

| 维度 | 直接执行 | /decompose |
|------|----------|------------|
| 意图识别 | ✅ Intent DSL / LLM | ✅ Intent DSL / LLM |
| 工具调用 | ✅ 实际执行 | ❌ 不执行 |
| 返回结果 | ✅ 真实数据 | ❌ 仅计划 |
| 用户体验 | ⭐⭐⭐⭐⭐ 智能 | ⭐⭐⭐ 教学用 |

### 2. 用户反馈

**用户观点**: "原来的执行效果好更加智能"

**原因分析**:
- **直接执行**: 调用 LLM → Function Calling → WebFetch 工具 → 返回人民日报网站真实新闻
- **/decompose**: 分析意图 → 显示计划（"步骤1: 使用 WebFetch 抓取..."）→ **但不执行** → 无实际结果

### 3. 代码证据

**直接执行会调用工具** (`src/web/websocket.rs:168-175`):
```rust
CommandType::NaturalLanguage(text) => {
    if let Some(intent_match) = try_match_intent(&text, &agent) {
        execute_intent(&intent_match, &text, &agent, session, sender).await
    } else {
        execute_llm_chat(&text, &agent, session, sender).await  // ← 会执行工具
    }
}
```

**/decompose 不执行工具** (`src/web/websocket.rs:915, 1007`):
```rust
// 快速路径
let output_content = "\n⚡ 通过 Intent DSL 快速识别".to_string();  // ← 只返回文字
...
// LLM 路径
let output_content = "\n🤖 通过 LLM 拆解执行".to_string();  // ← 只返回文字
```

---

## 💡 改进方案

### 方案一：保留现有逻辑 + 添加执行功能

**目标**: `/decompose` 命令在显示计划后，提供"执行"选项

**实现思路**:
1. `/decompose` 显示计划后，发送 `ExecutePlanPrompt` 消息
2. 前端显示"执行"按钮（每个步骤可勾选）
3. 用户点击"执行"→ 调用 `execute_plan()` 函数
4. 流式返回执行结果

**优点**:
- ✅ 保留教学/可视化功能
- ✅ 添加真正的执行能力
- ✅ 用户可选择性执行

**缺点**:
- 需要修改前端 UI（已有基础实现）
- 需要完善 `execute_plan()` 函数

### 方案二：/decompose 自动执行（不推荐）

**目标**: `/decompose` 直接执行所有步骤

**问题**:
- ❌ 失去可视化/教学价值
- ❌ 与直接执行功能重复
- ❌ 无法选择性执行步骤

### 方案三：改进直接执行，添加可视化

**目标**: 直接执行时也显示意图拆解过程

**实现思路**:
1. 直接执行时，先调用 `decomposer.decompose()`
2. 显示意图理解 + 步骤计划
3. 自动执行所有步骤
4. 流式返回结果

**优点**:
- ✅ 保留智能执行
- ✅ 增加透明度
- ✅ 用户理解 AI 思考过程

**缺点**:
- 增加延迟（多一次 LLM 调用）
- 可能影响性能

---

## 🎯 推荐方案

**推荐**: **方案一（保留 + 添加执行）**

**理由**:
1. **保持现有架构**: 直接执行保持高效智能
2. **完善 /decompose**: 添加真正的执行功能
3. **用户自主选择**: 教学场景用 `/decompose`，快速使用直接执行
4. **代码复用**: `execute_plan()` 函数已存在（line 1045）

**具体改进**:
1. **完善 `execute_plan()` 函数**
   - 读取计划中的步骤
   - 逐步执行工具调用
   - 流式返回结果

2. **前端交互改进**
   - 显示计划后，添加"执行计划"按钮
   - 每个步骤可勾选（已有 `enabled_steps` 参数）
   - 执行时显示进度

3. **消息流程设计**
   ```
   /decompose → IntentUnderstanding → StepProgress(pending)
              ↓
           [用户点击执行]
              ↓
           ExecutePlan 消息 → StepProgress(running) →
           ToolResult → StepProgress(completed) → PlanComplete
   ```

---

## 📌 现有基础设施

**已有功能**（无需重新开发）:
- ✅ `execute_plan()` 函数框架 (`src/web/websocket.rs:1045`)
- ✅ `IntentUnderstanding` 消息类型
- ✅ `StepProgress` 消息类型
- ✅ `EnabledStep` 参数（可选择性执行）
- ✅ 前端 Plan 卡片 UI（v1.29.3）

**需要补充**:
- 🔧 完善 `execute_plan()` 函数实现
- 🔧 添加工具调用逻辑
- 🔧 前端添加"执行"按钮

---

## 🚀 实施优先级

### Phase 1: 核心执行逻辑（高优先级）
- [ ] 实现 `execute_plan()` 函数
- [ ] 集成工具调用系统
- [ ] 流式返回执行结果

### Phase 2: 前端交互（中优先级）
- [ ] 添加"执行计划"按钮
- [ ] 步骤选择 UI
- [ ] 执行进度显示

### Phase 3: 体验优化（低优先级）
- [ ] 错误处理优化
- [ ] 执行日志记录
- [ ] 性能监控

---

## 📚 相关代码位置

| 功能 | 文件 | 行号 |
|------|------|------|
| 直接执行路由 | `src/web/websocket.rs` | 168-175 |
| /decompose 命令 | `src/web/websocket.rs` | 791-1043 |
| Intent DSL 快速路径 | `src/web/websocket.rs` | 862-932 |
| LLM 拆解路径 | `src/web/websocket.rs` | 934-1043 |
| execute_plan 框架 | `src/web/websocket.rs` | 1045+ |
| CLI decompose | `src/agent/mod.rs` | 2222-2317 |

---

## 🎬 总结

**核心问题**: `/decompose` 命令只做意图分析和可视化，**不执行实际工具调用**，导致无法返回真实结果。

**解决方向**: 完善 `execute_plan()` 函数，让 `/decompose` 支持真正的执行能力，同时保留可视化/教学价值。

**用户价值**: 既能快速智能执行（直接输入），也能可视化+执行（/decompose），满足不同场景需求。
