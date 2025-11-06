# RealConsole v1 → v2 平滑过渡计划

**日期**: 2025-11-06
**目标**: 渐进式演进，在 v1.x 中演练 v2.0 核心概念
**核心思想**: Cell 是结构化的对话轮次，AI 介入需要意图拆解和用户确认

---

## 🎯 核心洞察

### Cell vs 多轮对话的本质关系

**传统理解**：
- 多轮对话：连续的问答序列
- Cell：独立的执行单元

**深层理解**：
```
多轮对话 = 隐式 Cell 序列

[对话 1]                    [Cell 1]
User: 加载数据               Input: "加载数据"
AI: 已加载 100 行    ≈      Output: DataFrame (100 rows)
                            Status: Success

[对话 2]                    [Cell 2]
User: 筛选收入>1000         Input: "筛选收入>1000"
AI: 筛选完成，50 行  ≈      Output: DataFrame (50 rows)
                            Status: Success
```

**关键区别**：
- **隐式 vs 显式**：对话是隐式的，Cell 是显式的边界
- **临时 vs 持久**：对话存在于内存，Cell 可持久化
- **线性 vs 非线性**：对话是时间线性的，Cell 可以重新排序、插入、删除

---

## 🔄 AI Notebook vs 传统 Notebook

### 传统 Notebook（Jupyter）

**执行模型**：确定性
```python
# Cell 1
df = pd.read_csv('data.csv')  # 100% 确定执行什么

# Cell 2
df.groupby('category').sum()  # 100% 确定结果
```

**用户心智模型**：
1. 编写代码
2. 执行
3. 看到结果
4. （如果不对）修改代码，重新执行

---

### AI Notebook（RealConsole v2）

**执行模型**：意图驱动
```
# Cell 1
User: "加载销售数据"

AI 内部流程：
  1. 意图理解：需要加载文件
  2. 意图拆解：
     - 找到销售相关文件
     - 确定文件格式
     - 选择加载工具
  3. 生成执行计划：
     Plan: [
       { tool: "file_search", pattern: "*sales*.csv" },
       { tool: "file_read", path: "sales_2024.csv" },
     ]
  4. 用户确认：
     "找到 2 个文件：
       - sales_2024.csv (最新)
       - sales_2023.csv
      是否加载 sales_2024.csv？ [Y/n]"
  5. 执行
  6. 返回结果
```

**关键差异**：
| 维度 | 传统 Notebook | AI Notebook |
|------|--------------|-------------|
| **输入** | 确定的代码 | 模糊的意图 |
| **处理** | 直接执行 | 理解 → 拆解 → 计划 → 确认 → 执行 |
| **错误** | 语法/运行时错误 | 意图理解错误 + 执行错误 |
| **反馈** | 报错信息 | 自然语言解释 + 建议 |
| **迭代** | 修改代码 | 澄清意图 + 修改代码 |

**用户心智模型**：
1. 用自然语言描述意图
2. AI 拆解意图并展示理解
3. 用户确认或修正理解
4. AI 执行
5. 看到结果
6. （如果不对）追加澄清或重新描述

---

## 🚶 渐进式演进路径

### 当前状态（v1.27.0）

**已有能力**：
- ✅ 多轮对话上下文（9 轮历史）
- ✅ 工具调用（14+ 工具）
- ✅ 流式输出
- ✅ Markdown 渲染
- ✅ Web 终端界面

**缺失能力**：
- ❌ 显式 Cell 边界
- ❌ Cell 独立执行/重新执行
- ❌ 意图拆解可视化
- ❌ 用户确认机制
- ❌ 执行计划展示

---

### v1.28.0 - 引入"对话回合"概念（2025-11）

**目标**：在不改变交互方式的前提下，内部建立 Cell 概念

**功能**：
1. **可视化对话回合边界**
   ```
   ┌─────────────────────────────────────┐
   │ 回合 #1                             │
   ├─────────────────────────────────────┤
   │ % 加载 sales.csv                    │
   │                                     │
   │ ✓ 已加载 100 行数据                 │
   │ 列：date, region, product, revenue  │
   └─────────────────────────────────────┘

   ┌─────────────────────────────────────┐
   │ 回合 #2                             │
   ├─────────────────────────────────────┤
   │ % 按地区汇总收入                    │
   │                                     │
   │ ✓ 汇总完成                          │
   │ | 地区   | 收入      |              │
   │ |--------|-----------|              │
   │ | 北京   | 1,234,567 |              │
   │ | 上海   | 987,654   |              │
   └─────────────────────────────────────┘
   ```

2. **回合元数据显示**
   ```
   回合 #1 | 执行时间 1.2s | 使用工具 file_read | 模型 deepseek-chat
   ```

3. **回合折叠/展开**
   ```
   ▼ 回合 #1 - 加载 sales.csv ✓
   ▶ 回合 #2 - 按地区汇总收入 ✓
   ▶ 回合 #3 - 生成柱状图 ✓
   ▼ 回合 #4 - 找出异常值
     [展开显示详细内容]
   ```

**技术实现**：
```rust
// src/web/session.rs
pub struct ConversationRound {
    pub id: String,           // round-{uuid}
    pub index: usize,         // 1, 2, 3...
    pub user_input: String,
    pub ai_response: String,
    pub tools_used: Vec<String>,
    pub execution_time: Duration,
    pub status: RoundStatus,  // Success, Error, Pending
    pub timestamp: DateTime<Utc>,
}

enum RoundStatus {
    Pending,
    Running,
    Success,
    Error(String),
}
```

**UI 变化**：
- 每个对话轮次显示为一个"卡片"
- 卡片有明确的边界（圆角、阴影、间距）
- 显示执行状态（✓ 成功 / ✗ 失败 / ⏳ 执行中）

**意义**：
- 用户开始习惯"回合"的概念（为 Cell 做准备）
- 建立内部数据结构（Round ≈ Cell）

---

### v1.29.0 - 意图拆解可视化（2025-12）

**目标**：让用户看到 AI 如何理解和拆解任务

**功能**：
1. **意图理解展示**
   ```
   ┌─────────────────────────────────────┐
   │ 回合 #5                             │
   ├─────────────────────────────────────┤
   │ % 分析销售趋势并生成报告            │
   │                                     │
   │ 🤔 理解您的意图...                  │
   │                                     │
   │ 📋 任务拆解：                        │
   │   1. 按时间分组统计收入             │
   │   2. 计算增长率                     │
   │   3. 生成趋势图表                   │
   │   4. 生成 Markdown 报告             │
   │                                     │
   │ 🔧 需要的工具：                     │
   │   - data_group (数据分组)           │
   │   - calculate (计算)                │
   │   - visualize (可视化)              │
   │   - report_generate (报告生成)      │
   │                                     │
   │ ⏱️ 预计耗时：~5 秒                  │
   │                                     │
   │ [继续执行] [修改计划]               │
   └─────────────────────────────────────┘
   ```

2. **执行进度展示**
   ```
   执行中...
   ✓ 1/4 按时间分组统计收入 (1.2s)
   ✓ 2/4 计算增长率 (0.5s)
   ⏳ 3/4 生成趋势图表...
   ⏸ 4/4 生成报告（等待中）
   ```

3. **可修改的执行计划**
   ```
   用户点击 [修改计划]：

   ┌─────────────────────────────────────┐
   │ 调整执行计划                        │
   ├─────────────────────────────────────┤
   │ ☑ 1. 按时间分组统计收入             │
   │ ☑ 2. 计算增长率                     │
   │ ☐ 3. 生成趋势图表                   │
   │   └─ 图表类型：                     │
   │       ○ 折线图  ● 柱状图  ○ 饼图    │
   │ ☑ 4. 生成 Markdown 报告             │
   │   └─ 包含内容：                     │
   │       ☑ 数据摘要  ☑ 图表  ☐ 原始数据│
   │                                     │
   │ [取消] [确认并执行]                 │
   └─────────────────────────────────────┘
   ```

**技术实现**：
```rust
// src/agent/intent_decomposer.rs
pub struct IntentDecomposer {
    llm: Arc<dyn LlmClient>,
}

impl IntentDecomposer {
    /// 将自然语言意图拆解为多步任务
    pub async fn decompose(&self, input: &str) -> Result<ExecutionPlan> {
        let prompt = format!(
            "将用户意图拆解为具体的执行步骤：

            用户输入：{}

            可用工具：{}

            请以 JSON 格式返回执行计划：
            {{
              \"understanding\": \"对用户意图的理解\",
              \"steps\": [
                {{
                  \"description\": \"步骤描述\",
                  \"tool\": \"工具名称\",
                  \"params\": {{...}},
                  \"estimated_time\": 1.2
                }}
              ],
              \"total_estimated_time\": 5.0
            }}",
            input,
            self.available_tools()
        );

        let response = self.llm.chat(&prompt).await?;
        let plan: ExecutionPlan = serde_json::from_str(&response)?;
        Ok(plan)
    }
}

pub struct ExecutionPlan {
    pub understanding: String,
    pub steps: Vec<ExecutionStep>,
    pub total_estimated_time: f64,
}

pub struct ExecutionStep {
    pub id: String,
    pub description: String,
    pub tool: String,
    pub params: serde_json::Value,
    pub estimated_time: f64,
    pub status: StepStatus,
}
```

**WebSocket 消息扩展**：
```rust
// src/web/session.rs
#[derive(Serialize)]
pub enum ServerMessage {
    // ... 现有消息 ...

    /// 意图理解结果
    IntentUnderstanding {
        understanding: String,
        plan: ExecutionPlan,
    },

    /// 执行步骤进度
    StepProgress {
        step_index: usize,
        total_steps: usize,
        description: String,
        status: StepStatus,
    },

    /// 请求用户确认
    ConfirmationRequest {
        question: String,
        plan: ExecutionPlan,
        options: Vec<String>,
    },
}
```

**意义**：
- 用户理解 AI 的思考过程
- 建立信任（AI 不是黑盒）
- 引入交互式确认机制

---

### v1.30.0 - 用户确认与反馈循环（2026-01）

**目标**：建立 AI - 用户协作机制

**场景 1：自动确认（高置信度）**
```
User: "加载 sales.csv"

AI 内部：
  - 意图清晰度：95%
  - 文件存在：是
  - 操作风险：低

→ 直接执行，无需确认
```

**场景 2：交互确认（中等置信度）**
```
User: "加载销售数据"

AI 检测到歧义：
  ┌─────────────────────────────────────┐
  │ 🤔 发现多个可能的文件：             │
  ├─────────────────────────────────────┤
  │ 1. sales_2024.csv (最新, 100 行)    │
  │ 2. sales_2023.csv (归档, 1000 行)   │
  │ 3. sales_summary.xlsx (汇总)        │
  │                                     │
  │ 您想加载哪个文件？                  │
  │                                     │
  │ [1] [2] [3] [其他文件...]           │
  └─────────────────────────────────────┘
```

**场景 3：强制确认（高风险操作）**
```
User: "删除所有测试数据"

AI 检测到危险操作：
  ┌─────────────────────────────────────┐
  │ ⚠️  危险操作警告                    │
  ├─────────────────────────────────────┤
  │ 您即将删除以下文件：                │
  │ - test_data_1.csv (1000 行)         │
  │ - test_data_2.csv (500 行)          │
  │ - test_results.json (重要)          │
  │                                     │
  │ ⚠️  此操作不可撤销！                │
  │                                     │
  │ 请输入 "DELETE" 确认：              │
  │ [_______________]                   │
  │                                     │
  │ [取消]                              │
  └─────────────────────────────────────┘
```

**场景 4：澄清式确认（低置信度）**
```
User: "分析数据"

AI 需要更多信息：
  ┌─────────────────────────────────────┐
  │ 🤔 需要更多信息来理解您的意图       │
  ├─────────────────────────────────────┤
  │ 请选择分析类型：                    │
  │ □ 统计摘要（均值、中位数、分布）    │
  │ □ 时间趋势分析                      │
  │ □ 分组对比分析                      │
  │ □ 异常检测                          │
  │ □ 相关性分析                        │
  │ □ 其他（请描述）_______________     │
  │                                     │
  │ 分析维度：                          │
  │ □ 按地区  □ 按产品  □ 按时间        │
  │                                     │
  │ [确认] [让 AI 自动选择]             │
  └─────────────────────────────────────┘
```

**技术实现**：
```rust
// src/agent/confirmation.rs
pub struct ConfirmationEngine {
    config: ConfirmationConfig,
}

pub struct ConfirmationConfig {
    /// 自动执行的最低置信度阈值
    pub auto_execute_threshold: f64,  // 0.9

    /// 需要确认的操作类型
    pub require_confirmation: Vec<OperationType>,

    /// 危险操作关键词
    pub dangerous_keywords: Vec<String>,
}

impl ConfirmationEngine {
    /// 判断是否需要用户确认
    pub fn should_confirm(&self, plan: &ExecutionPlan) -> ConfirmationDecision {
        // 1. 检查意图置信度
        if plan.confidence < self.config.auto_execute_threshold {
            return ConfirmationDecision::Required(
                ConfirmationType::Clarification
            );
        }

        // 2. 检查操作风险
        if self.is_dangerous(&plan) {
            return ConfirmationDecision::Required(
                ConfirmationType::Safety
            );
        }

        // 3. 检查是否有歧义
        if plan.ambiguities.len() > 0 {
            return ConfirmationDecision::Required(
                ConfirmationType::Disambiguation
            );
        }

        ConfirmationDecision::NotRequired
    }

    fn is_dangerous(&self, plan: &ExecutionPlan) -> bool {
        plan.steps.iter().any(|step| {
            // 检查删除操作
            step.tool == "file_delete" ||
            step.tool == "shell" && self.contains_dangerous_keywords(&step.params)
        })
    }
}

pub enum ConfirmationDecision {
    NotRequired,
    Required(ConfirmationType),
}

pub enum ConfirmationType {
    /// 澄清意图
    Clarification,
    /// 消除歧义
    Disambiguation,
    /// 安全确认
    Safety,
}
```

**意义**：
- 建立 AI - 用户协作的反馈循环
- 提高任务执行的准确性和安全性
- 为 v2.0 的交互式 Cell 打基础

---

### v1.31.0 - Cell 独立执行（2026-02）

**目标**：引入真正的 Cell 模型

**功能 1：Cell 重新执行**
```
┌─────────────────────────────────────┐
│ Cell #1 ✓                           │
│ % 加载 sales.csv                    │
│ 已加载 100 行                       │
│ [🔄 重新执行] [✏️ 编辑] [🗑️ 删除]   │
└─────────────────────────────────────┘

点击 [🔄 重新执行]：
→ 重新运行 Cell #1，更新输出
→ 如果数据源变化（sales.csv 更新），显示新数据
```

**功能 2：Cell 编辑**
```
点击 [✏️ 编辑]：

┌─────────────────────────────────────┐
│ Cell #1 (编辑模式)                  │
│ ┌─────────────────────────────────┐ │
│ │ 加载 sales.csv 并显示前 10 行   │ │ ← 可修改的输入
│ └─────────────────────────────────┘ │
│                                     │
│ [取消] [保存] [保存并执行]          │
└─────────────────────────────────────┘

修改后执行 → 更新输出
```

**功能 3：Cell 插入**
```
┌─────────────────────────────────────┐
│ Cell #1 - 加载数据 ✓                │
└─────────────────────────────────────┘
          [➕ 在下方插入 Cell]
┌─────────────────────────────────────┐
│ Cell #3 - 生成报告 ✓                │
└─────────────────────────────────────┘

点击 [➕]：
→ 创建 Cell #2（新编号）
→ 可以在任意位置插入新的分析步骤
```

**功能 4：Cell 依赖追踪**
```
Cell #1: 加载数据 → 变量 df
Cell #2: 筛选数据 → 使用 df，生成 df_filtered
Cell #3: 分组统计 → 使用 df_filtered，生成 summary

依赖图：
  Cell #1 (df)
     ↓
  Cell #2 (df_filtered)
     ↓
  Cell #3 (summary)

如果重新执行 Cell #1：
  → 提示：Cell #2 和 Cell #3 依赖此 Cell，是否重新执行它们？
```

**技术实现**：
```rust
// src/notebook/cell.rs
pub struct Cell {
    pub id: String,
    pub index: usize,
    pub input: String,
    pub output: Option<CellOutput>,
    pub status: CellStatus,
    pub execution_count: u32,
    pub metadata: CellMetadata,

    // 依赖追踪
    pub variables_defined: Vec<String>,   // 此 Cell 定义的变量
    pub variables_used: Vec<String>,      // 此 Cell 使用的变量
    pub depends_on: Vec<String>,          // 依赖的 Cell ID
}

impl Cell {
    /// 执行 Cell
    pub async fn execute(&mut self, context: &NotebookContext) -> Result<CellOutput> {
        self.execution_count += 1;
        self.status = CellStatus::Running;

        // 执行并捕获输出
        let output = execute_user_intent(&self.input, context).await?;

        // 分析变量定义
        self.variables_defined = extract_defined_variables(&output);

        self.output = Some(output.clone());
        self.status = CellStatus::Success;

        Ok(output)
    }

    /// 分析依赖关系
    pub fn analyze_dependencies(&mut self, all_cells: &[Cell]) {
        self.variables_used = extract_used_variables(&self.input);

        // 找出哪些 Cell 定义了我需要的变量
        for other_cell in all_cells {
            if other_cell.index < self.index {
                for var in &self.variables_used {
                    if other_cell.variables_defined.contains(var) {
                        self.depends_on.push(other_cell.id.clone());
                    }
                }
            }
        }
    }
}
```

**UI 增强**：
```javascript
// src/web/server.rs - Cell 组件
class NotebookCell extends React.Component {
    render() {
        return (
            <div className="cell">
                <div className="cell-header">
                    <span className="cell-number">Cell #{this.props.index}</span>
                    <div className="cell-actions">
                        <button onClick={this.handleRerun}>🔄 重新执行</button>
                        <button onClick={this.handleEdit}>✏️ 编辑</button>
                        <button onClick={this.handleDelete}>🗑️ 删除</button>
                    </div>
                </div>

                <div className="cell-input">
                    {this.state.editing ? (
                        <textarea value={this.state.input} onChange={...} />
                    ) : (
                        <div className="input-display">{this.props.input}</div>
                    )}
                </div>

                <div className="cell-output">
                    {this.renderOutput()}
                </div>

                <div className="cell-metadata">
                    <span>执行时间: {this.props.executionTime}</span>
                    <span>工具: {this.props.toolsUsed.join(', ')}</span>
                    {this.props.dependsOn.length > 0 && (
                        <span>依赖: Cell {this.props.dependsOn.join(', ')}</span>
                    )}
                </div>
            </div>
        );
    }
}
```

**意义**：
- 真正的 Cell 模型（可重新执行、编辑、插入）
- 建立非线性的分析流程（不再是单向对话）
- v2.0 Notebook 的核心基础

---

### v1.32.0 - Notebook 持久化（2026-03）

**目标**：保存和加载完整的分析流程

**功能 1：保存 Notebook**
```
┌─────────────────────────────────────┐
│ 文件菜单                            │
├─────────────────────────────────────┤
│ 💾 保存 Notebook                    │
│    └─ 文件名: sales_analysis.rcnb  │
│    └─ 位置: ~/Documents/            │
│                                     │
│ 📂 加载 Notebook                    │
│                                     │
│ 📤 导出为...                        │
│    ├─ HTML (静态网页)               │
│    ├─ Markdown (文档)               │
│    └─ PDF (报告)                    │
└─────────────────────────────────────┘
```

**功能 2：.rcnb 文件格式**
```json
{
  "version": "1.32.0",
  "metadata": {
    "title": "Sales Analysis",
    "author": "User",
    "created_at": "2026-03-01T10:00:00Z",
    "last_modified": "2026-03-01T12:30:00Z"
  },
  "context": {
    "variables": {
      "df": {
        "type": "DataFrame",
        "shape": [100, 4],
        "columns": ["date", "region", "product", "revenue"]
      },
      "summary": {
        "type": "DataFrame",
        "shape": [3, 2]
      }
    },
    "working_dir": "/Users/hongxin/projects/sales"
  },
  "cells": [
    {
      "id": "c1",
      "index": 1,
      "input": "加载 sales.csv",
      "output": {
        "type": "table",
        "data": [...],
        "display": "DataFrame (100 rows × 4 columns)"
      },
      "status": "success",
      "execution_count": 1,
      "execution_time": 1.2,
      "tools_used": ["file_read"],
      "variables_defined": ["df"],
      "variables_used": [],
      "timestamp": "2026-03-01T10:05:00Z"
    },
    {
      "id": "c2",
      "index": 2,
      "input": "按地区汇总收入",
      "output": {
        "type": "table",
        "data": [...]
      },
      "execution_count": 2,
      "tools_used": ["data_group"],
      "variables_defined": ["summary"],
      "variables_used": ["df"],
      "depends_on": ["c1"],
      "timestamp": "2026-03-01T10:10:00Z"
    }
  ]
}
```

**功能 3：增量保存**
```
自动保存：
  - 每执行一个 Cell 后自动保存
  - 显示保存状态：💾 已保存 (2 秒前)

手动保存：
  - Ctrl+S / Cmd+S 快捷键
  - 菜单栏保存按钮
```

**功能 4：加载 Notebook**
```
加载 sales_analysis.rcnb：

  ┌─────────────────────────────────────┐
  │ 📂 加载 Notebook                    │
  ├─────────────────────────────────────┤
  │ 文件: sales_analysis.rcnb           │
  │ 创建: 2026-03-01                    │
  │ Cell 数: 5                          │
  │ 变量: df, summary, report           │
  │                                     │
  │ 加载选项：                          │
  │ ● 加载所有内容（包括输出）          │
  │ ○ 仅加载输入（重新执行）            │
  │                                     │
  │ [取消] [加载]                       │
  └─────────────────────────────────────┘

加载后：
  → 恢复所有 Cell 和输出
  → 恢复变量上下文（如果选择）
  → 可以继续编辑和执行
```

**意义**：
- Notebook 成为可分享的分析流程
- 支持长期项目（不需要在一个会话内完成）
- 建立知识库（保存常用分析模板）

---

## 🎯 v2.0-alpha 准备就绪检查清单

完成 v1.28 - v1.32 后，以下能力应该已具备：

**核心能力**：
- ✅ Cell 模型（显式边界、独立执行）
- ✅ 意图拆解（可视化、可修改）
- ✅ 用户确认（交互式、分级）
- ✅ 依赖追踪（变量、Cell 关系）
- ✅ 持久化（保存/加载 .rcnb）

**UI/UX**：
- ✅ Cell 界面（输入、输出、元数据、操作）
- ✅ 执行进度可视化
- ✅ 确认对话框
- ✅ 文件菜单（保存/加载/导出）

**技术基础**：
- ✅ NotebookContext（变量管理）
- ✅ IntentDecomposer（意图拆解）
- ✅ ConfirmationEngine（确认机制）
- ✅ Cell 依赖分析

**用户习惯**：
- ✅ 用户习惯 Cell 边界
- ✅ 用户理解 AI 拆解过程
- ✅ 用户习惯交互式确认

---

## 🔄 v2.0 的核心差异

v1.32 已经有了 Notebook 的基本形态，v2.0 的主要增强：

### 1. DSL 深度集成
```dsl
# v1.32: 纯自然语言
% 加载数据，筛选收入>1000，按地区汇总

# v2.0: 自然语言 + DSL 混合
@pipeline {
  load: "sales.csv" -> df
  filter: df[revenue > 1000]
  group: by region aggregate sum(revenue)
}
```

### 2. 丰富可视化
```
# v1.32: 表格 + Markdown
输出：DataFrame 表格

# v2.0: 交互式图表
输出：Plotly 柱状图（可缩放、筛选、导出）
```

### 3. 模板和复用
```
# v2.0: 保存为模板
@template sales_analysis {
  # ... 分析流程 ...
}

# 其他 Notebook 可以重用
@use sales_analysis with { file: "sales_2025.csv" }
```

### 4. 协作功能
```
# v2.0: 多人协作
- 实时共享 Notebook
- 版本历史
- 评论和讨论
```

---

## 📊 进度时间表

```
2025-11  v1.28.0  对话回合可视化          [2 周开发]
2025-12  v1.29.0  意图拆解可视化          [3 周开发]
2026-01  v1.30.0  用户确认与反馈循环      [3 周开发]
2026-02  v1.31.0  Cell 独立执行           [4 周开发]
2026-03  v1.32.0  Notebook 持久化         [2 周开发]
────────────────────────────────────────────────────────
2026-04  v2.0-alpha  DSL 集成 + 可视化    [6 周开发]
2026-06  v2.0-beta   协作功能             [4 周开发]
2026-08  v2.0-rc     稳定性测试           [2 周]
2026-09  v2.0        正式发布             🚀
```

---

## 💡 关键设计原则

### 1. 渐进增强（Progressive Enhancement）
- 每个版本都是可用的产品，不是半成品
- 新功能不破坏旧用法
- 用户可以选择使用新功能或继续旧方式

### 2. 向后兼容（Backward Compatibility）
- v1.28 的 Notebook 可以在 v2.0 中打开
- 旧的对话式交互仍然支持
- 配置文件格式保持兼容

### 3. 用户教育（User Education）
- 通过 UI 提示引导用户发现新功能
- 提供示例 Notebook
- 文档和教程同步更新

### 4. 数据驱动（Data-Driven）
- 收集用户使用数据（匿名）
- 分析哪些功能最常用
- 根据反馈调整优先级

---

## 🎓 理论基础

### 意图拆解的层次理论

```
Level 0: 原子操作（不可再拆）
  - 读取文件
  - 执行 Shell 命令
  - 调用 API

Level 1: 简单任务（1-3 步）
  - 加载并显示数据
  - 筛选特定条件
  - 生成简单图表

Level 2: 复杂任务（4-10 步）
  - 数据清洗和转换
  - 多维度统计分析
  - 生成分析报告

Level 3: 项目级任务（>10 步）
  - 完整数据分析流程
  - 机器学习实验
  - 系统运维自动化
```

**AI 的作用**：
- Level 0：直接执行
- Level 1：简单拆解（规则 + DSL）
- Level 2：智能拆解（LLM 生成计划）
- Level 3：交互式拆解（多轮对话 + 用户确认）

### 确认机制的置信度模型

```
置信度计算公式：
  confidence = w1 * intent_clarity +
               w2 * (1 - ambiguity_count / max_ambiguity) +
               w3 * context_relevance +
               w4 * (1 - risk_score)

其中：
  - intent_clarity: 意图清晰度（0-1）
  - ambiguity_count: 歧义数量
  - context_relevance: 上下文相关性（0-1）
  - risk_score: 操作风险（0-1）
  - w1, w2, w3, w4: 权重（可配置）

确认策略：
  if confidence >= 0.9:
      自动执行
  elif confidence >= 0.7:
      简单确认（是/否）
  elif confidence >= 0.5:
      交互式确认（选择选项）
  else:
      拒绝执行，请求澄清
```

---

**文档状态**: 详细计划，待评审
**下次更新**: 开始 v1.28.0 开发前

**核心理念**：
> Cell 是结构化的对话轮次，AI 介入需要意图拆解和用户确认。
> 通过渐进式演进，在 v1.x 中逐步建立 v2.0 的核心能力。

