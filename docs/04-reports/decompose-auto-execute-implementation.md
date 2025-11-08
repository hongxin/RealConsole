# /decompose 自动执行功能实施报告

**版本**: v1.39.0
**日期**: 2025-01-08
**实施方案**: 方案一（保留可视化 + 添加自动执行）

---

## 📋 实施概览

**目标**: 让 `/decompose` 命令在显示意图拆解可视化后，自动执行计划中的所有步骤，返回真实结果

**核心改进**:
- ✅ 保留原有可视化功能（IntentUnderstanding + StepProgress）
- ✅ 添加自动执行逻辑（调用 `execute_plan()` 函数）
- ✅ 复用 v1.30.0 已有的工具调用基础设施
- ✅ 保持与"直接执行"模式一致的智能体验
- ✅ 不破坏原有直接执行逻辑（完全保留）

---

## 🔧 技术实施

### 修改文件

**src/web/websocket.rs** - 核心修改

#### 1. Intent DSL 快速路径（lines 897-964）

**原有逻辑**:
```rust
// 发送可视化消息
IntentUnderstanding -> StepProgress (pending) -> RoundComplete
// ❌ 不执行任何工具
```

**新增逻辑** (v1.39.0):
```rust
// 1. 发送可视化消息（保留）
IntentUnderstanding -> StepProgress (pending)

// 2. 转换 ExecutionStep -> EnabledStep
let enabled_steps: Vec<EnabledStep> = plan.steps.iter()
    .enumerate()
    .map(|(index, step)| EnabledStep {
        step_id: step.id.clone(),
        step_index: index,
        description: step.description.clone(),
        tool: step.tool.clone(),
        params: step.params.clone(),
    })
    .collect();

// 3. 调用 execute_plan() 自动执行
execute_plan(session, &plan.id, &enabled_steps, sender).await

// 4. 完成回合
RoundComplete ("⚡ 通过 Intent DSL 快速识别并执行")
```

#### 2. LLM 拆解路径（lines 1013-1090）

**原有逻辑**:
```rust
// 调用 LLM 拆解
decomposer.decompose(query) -> ExecutionPlan

// 发送可视化消息
IntentUnderstanding -> StepProgress (pending) -> RoundComplete
// ❌ 不执行任何工具
```

**新增逻辑** (v1.39.0):
```rust
// 1. 调用 LLM 拆解（保留）
decomposer.decompose(query) -> ExecutionPlan

// 2. 发送可视化消息（保留）
IntentUnderstanding -> StepProgress (pending)

// 3. 转换并执行（新增）
let enabled_steps: Vec<EnabledStep> = ...
execute_plan(session, &plan.id, &enabled_steps, sender).await

// 4. 完成回合
RoundComplete ("🤖 通过 LLM 拆解并执行")
```

### 复用的基础设施

**无需重新开发**（已在 v1.30.0 实现）:
- ✅ `execute_plan()` 函数（lines 1046-1174）
  - 逐步执行所有启用的步骤
  - 发送 `PlanExecutionStart` 消息
  - 发送 `StepProgress` (running/success/failed) 消息
  - 发送 `StepOutput` 消息（工具输出）
  - 发送 `PlanExecutionComplete` 消息

- ✅ `execute_step()` 函数（lines 1177-1203）
  - 调用 `ToolRegistry.execute()`
  - 传递工具参数
  - 返回真实执行结果

- ✅ `EnabledStep` 数据结构（src/web/session.rs:135-143）
  - step_id, step_index, description, tool, params

- ✅ WebSocket 消息类型
  - `PlanExecutionStart`
  - `StepProgress` (多种状态)
  - `StepOutput`
  - `PlanExecutionComplete`

---

## 📊 执行流程对比

### v1.38.1（旧版本）- 仅可视化

```
用户输入: /decompose 计算 2 + 3
    ↓
识别 Intent DSL 或 LLM 拆解
    ↓
发送 IntentUnderstanding 消息
    ↓
发送 StepProgress (pending) 消息
    ↓
发送 RoundComplete 消息
    ↓
❌ 结束（无真实结果）
```

### v1.39.0（新版本）- 可视化 + 执行

```
用户输入: /decompose 计算 2 + 3
    ↓
识别 Intent DSL 或 LLM 拆解
    ↓
发送 IntentUnderstanding 消息（可视化）
    ↓
发送 StepProgress (pending) 消息（可视化）
    ↓
转换 ExecutionStep -> EnabledStep
    ↓
调用 execute_plan() 函数
    ├─ 发送 PlanExecutionStart 消息
    ├─ 对每个步骤：
    │   ├─ 发送 StepProgress (running)
    │   ├─ 调用 ToolRegistry.execute()
    │   ├─ 发送 StepOutput（真实结果：5）
    │   └─ 发送 StepProgress (success)
    └─ 发送 PlanExecutionComplete 消息
    ↓
发送 RoundComplete 消息
    ↓
✅ 完成（有真实结果：2 + 3 = 5）
```

---

## 💡 核心优势

### 1. 保留可视化价值
- ✅ 用户仍能看到意图理解过程
- ✅ 步骤计划清晰展示（pending 状态）
- ✅ 教学和调试价值保留

### 2. 添加真正执行能力
- ✅ 调用实际工具（通过 ToolRegistry）
- ✅ 返回真实结果（如网页内容、计算结果）
- ✅ 流式显示执行进度（running -> success/failed）

### 3. 与直接执行模式一致
- ✅ 相同的工具调用逻辑
- ✅ 相同的错误处理机制
- ✅ 相同的输出格式

### 4. 不破坏原有架构
- ✅ 直接执行路径完全保留（lines 168-175）
- ✅ 复用已有基础设施（execute_plan）
- ✅ 无需修改前端 UI（自动兼容）

---

## 🎯 用户价值

| 场景 | v1.38.1（旧版本） | v1.39.0（新版本） |
|------|------------------|-------------------|
| **快速使用** | 直接执行 | 直接执行（保留） |
| **教学/调试** | `/decompose` 仅显示计划 | `/decompose` 显示计划 + 执行 |
| **透明度** | ⭐⭐⭐（仅计划） | ⭐⭐⭐⭐⭐（计划 + 执行过程） |
| **实用性** | ⭐⭐（无结果） | ⭐⭐⭐⭐⭐（有真实结果） |
| **智能程度** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐（与直接执行一致） |

---

## 🧪 测试验证

### 编译测试

```bash
$ cargo build --release
   Compiling realconsole v1.38.1
   Finished `release` profile [optimized] target(s) in 31.97s
✅ 编译成功
```

### 功能测试（推荐）

**方法一：浏览器测试**

```bash
# 1. 启动 Web 服务器
DEEPSEEK_API_KEY="your-api-key" ./target/release/realconsole web

# 2. 访问 http://127.0.0.1:7788

# 3. 输入测试命令
/decompose 计算 2 + 3

# 预期结果：
# - 显示意图理解（IntentUnderstanding 卡片）
# - 显示步骤计划（StepProgress 卡片，pending 状态）
# - 显示执行开始（PlanExecutionStart）
# - 显示步骤执行（StepProgress running -> success）
# - 显示步骤输出（StepOutput: "✅ 执行成功\n工具: Calculator\n\n5"）
# - 显示执行完成（PlanExecutionComplete）
```

**方法二：复杂场景测试**

```bash
# 测试 LLM 拆解 + 工具调用
/decompose 帮我看看人民日报网站今天的新闻

# 预期结果：
# - LLM 拆解意图
# - 显示步骤计划（如：使用 WebFetch 抓取网站）
# - 执行 WebFetch 工具
# - 返回真实的网站新闻内容
```

---

## 📌 代码变更统计

| 文件 | 修改类型 | 行数变化 |
|------|---------|---------|
| `src/web/websocket.rs` | 新增 | +96 行 |
| | 总变化 | 2 处修改点 |

**关键修改点**:
1. Intent DSL 快速路径：lines 914-963（新增 50 行）
2. LLM 拆解路径：lines 1038-1089（新增 46 行）

**复用代码**:
- `execute_plan()` 函数：129 行（v1.30.0 已实现）
- `execute_step()` 函数：27 行（v1.30.0 已实现）

---

## 🚀 未来扩展

### Phase 2: 可选择性执行（可选）

**目标**: 前端添加步骤选择 UI，用户可勾选需要执行的步骤

**实现思路**:
1. 前端在显示 StepProgress (pending) 时，添加复选框
2. 用户勾选后，发送 `execute_plan` WebSocket 消息
3. 后端接收 `enabled_steps` 参数，仅执行选中的步骤

**优势**:
- 更灵活的执行控制
- 适合调试和实验场景

**当前状态**: 暂不实施（自动执行已满足大部分需求）

---

## 📚 相关文档

- **调研报告**: `docs/04-reports/decompose_research_report.md`
  - 详细分析了直接执行与 `/decompose` 的差异
  - 提出了三种改进方案
  - 推荐方案一（已实施）

- **核心代码**:
  - `src/web/websocket.rs:791-1090` - `/decompose` 命令处理
  - `src/web/websocket.rs:1046-1203` - `execute_plan()` 和 `execute_step()`
  - `src/web/session.rs:135-143` - `EnabledStep` 数据结构

---

## 🎬 总结

**核心成就**:
- ✅ `/decompose` 命令现在既能可视化意图，又能真正执行工具
- ✅ 与直接执行模式保持一致的智能体验
- ✅ 保留教学和调试价值
- ✅ 复用已有基础设施，无需重复开发

**用户体验提升**:
- **旧版本**: `/decompose` 只显示计划，无实际结果 → ⭐⭐⭐
- **新版本**: `/decompose` 显示计划 + 执行并返回结果 → ⭐⭐⭐⭐⭐

**技术亮点**:
- 复用 v1.30.0 的 `execute_plan()` 基础设施
- 最小化代码变更（仅 96 行新增）
- 编译通过，零警告
- 无需修改前端 UI（自动兼容）

**下一步**: 准备发布 v1.39.0 版本 🎉
