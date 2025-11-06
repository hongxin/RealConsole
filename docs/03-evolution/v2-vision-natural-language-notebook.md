# RealConsole v2 愿景：自然语言交互式笔记本

**日期**: 2025-11-06
**版本**: v2.0 Roadmap
**核心理念**: 不是创造比 Python 更好的语言，而是用自然语言替代编程语言

---

## 🌟 核心洞察

### 突破性想法

> **Jupyter Notebook 的本质**: 交互式、可重现、可分享的计算环境
> **RealConsole v2 的愿景**: 交互式、可重现、可分享的自然语言任务环境

**关键区别**：
- **Jupyter**: `df.groupby('category').sum()` → 需要掌握 Pandas API
- **RealConsole v2**: "按类别汇总销售数据" → 自然语言表达意图

**核心价值**：
- 降低技术门槛：非程序员也能进行数据分析
- 保持专业能力：程序员可以深入到代码层面
- 可重现性：Cell 级别的执行和结果保存
- 协作性：分享 Notebook 即分享思路和流程

---

## 📊 概念对比

### Jupyter Notebook vs RealConsole v2

| 维度 | Jupyter Notebook | RealConsole v2 (愿景) |
|------|------------------|----------------------|
| **输入方式** | Python/R/Julia 代码 | 自然语言 + DSL（可选） |
| **执行引擎** | IPython Kernel | LLM + Tool Calling + DSL |
| **用户群体** | 程序员、数据科学家 | 所有人（包括非技术人员） |
| **学习曲线** | 需要学习编程语言 | 用自然语言表达即可 |
| **可视化** | Matplotlib/Plotly | LLM 生成代码 → 渲染图表 |
| **调试** | Stack trace | 自然语言错误解释 |
| **版本控制** | .ipynb (JSON) | .rcnb (RealConsole Notebook) |
| **协作** | 分享代码 | 分享意图和流程 |
| **可重现性** | 依赖环境一致 | LLM + 工具一致即可 |

### 核心共同点

✅ **Cell 模型**: 独立的执行单元
✅ **交互式**: 即时反馈，迭代探索
✅ **可视化**: 支持表格、图表、图像
✅ **持久化**: 保存输入、输出、状态
✅ **可分享**: 导出为文件，团队协作

---

## 🎯 典型应用场景

### 场景 1: 数据分析（非技术人员）

**Jupyter 方式**（需要编程知识）:
```python
import pandas as pd
import matplotlib.pyplot as plt

# Cell 1: 加载数据
df = pd.read_csv('sales.csv')
df.head()

# Cell 2: 数据清洗
df = df.dropna()
df['date'] = pd.to_datetime(df['date'])

# Cell 3: 分组统计
result = df.groupby('region')['revenue'].sum().sort_values(ascending=False)
result

# Cell 4: 可视化
result.plot(kind='bar')
plt.title('Revenue by Region')
plt.show()
```

**RealConsole v2 方式**（自然语言）:
```
[Cell 1]
% 加载 sales.csv 文件，显示前 5 行

[Cell 2]
% 清理数据：删除空值，将 date 列转换为日期格式

[Cell 3]
% 按地区汇总收入，从高到低排序

[Cell 4]
% 用柱状图展示各地区收入分布
```

**优势**：
- 业务人员直接表达分析意图
- LLM 自动生成 Python 代码（后台执行）
- 用户看到结果，不需要理解代码细节
- 高级用户可以查看/修改生成的代码

---

### 场景 2: 系统运维（DevOps）

**传统方式**（需要记忆命令）:
```bash
# Cell 1: 检查磁盘使用
df -h | grep -v tmpfs | sort -k5 -rn

# Cell 2: 查找大文件
find /var/log -type f -size +100M -exec ls -lh {} \;

# Cell 3: 清理日志
find /var/log -name "*.log" -mtime +30 -delete
```

**RealConsole v2 方式**:
```
[Cell 1]
% 检查磁盘使用情况，按使用率降序排列

[Cell 2]
% 找出 /var/log 下超过 100MB 的文件

[Cell 3]
% 删除 /var/log 下 30 天前的日志文件
```

**优势**：
- 初级运维也能执行复杂任务
- 自动生成安全的命令（避免误删）
- 可重现的运维流程（Notebook 即文档）

---

### 场景 3: 机器学习实验（数据科学家）

**混合模式**（自然语言 + DSL + 代码）:
```
[Cell 1 - 自然语言]
% 加载 MNIST 数据集，显示样本分布

[Cell 2 - DSL 简化]
@workflow train_model {
  data: mnist
  model: cnn
  epochs: 10
}

[Cell 3 - 自然语言]
% 评估模型在测试集上的准确率

[Cell 4 - 直接代码（高级用户）]
!python
import torch
model.eval()
# ... 自定义评估逻辑
```

**优势**：
- 快速原型：用自然语言描述实验思路
- 结构化：用 DSL 定义标准化流程
- 灵活性：需要时深入代码层面

---

## 🏗️ 架构设计（v2.0）

### 核心组件

```
┌─────────────────────────────────────────────────────────┐
│                  RealConsole v2 Notebook                │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Cell 1: 自然语言输入                           │   │
│  │  "加载 sales.csv，显示前 5 行"                  │   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Intent Parser (DSL + LLM)                      │   │
│  │  - DSL Pattern Matching (快速路径)              │   │
│  │  - LLM Semantic Understanding (智能路径)        │   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Execution Plan Generator                       │   │
│  │  - Tool Calling: file_read, table_display       │   │
│  │  - Code Generation: Python/Shell 脚本          │   │
│  │  - Pipeline Orchestration: 多步任务编排        │   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Execution Engine                               │   │
│  │  - Sandbox (安全执行)                          │   │
│  │  - State Management (变量、环境)               │   │
│  │  - Result Capture (stdout, return, visualization)│   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Cell Output (Rich Display)                     │   │
│  │  - Table (Markdown/HTML)                        │   │
│  │  - Chart (Plotly/D3.js)                         │   │
│  │  - Text (Markdown)                              │   │
│  │  - Code (Syntax Highlighting)                   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Cell 2: 下一个任务...                          │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 关键技术点

#### 1. Cell 状态管理

每个 Cell 有独立状态：
```rust
struct Cell {
    id: String,
    input: CellInput,      // 用户输入（自然语言/DSL/代码）
    execution_count: u32,  // 执行次数
    status: CellStatus,    // Pending, Running, Success, Error
    output: CellOutput,    // 输出（文本、表格、图表、错误）
    metadata: Metadata,    // 执行时间、模型、工具等
    context: CellContext,  // 变量、环境、依赖
}

enum CellInput {
    NaturalLanguage(String),
    DSL(DslExpression),
    Code(CodeBlock),
    Mixed(Vec<InputSegment>),
}

enum CellOutput {
    Text(String),
    Markdown(String),
    Table(DataFrame),
    Chart(ChartSpec),
    Image(ImageData),
    Error(ErrorInfo),
    Mixed(Vec<OutputSegment>),
}
```

#### 2. 跨 Cell 上下文共享

**变量共享**：
```
[Cell 1]
% 加载 sales.csv 到变量 df

[Cell 2]
% 对 df 按地区分组统计
```

**实现**：
```rust
struct NotebookContext {
    variables: HashMap<String, Value>,  // df, result, model 等
    imports: HashSet<String>,            // 已导入的库
    environment: HashMap<String, String>, // 环境变量
    working_dir: PathBuf,                // 工作目录
}
```

**LLM 感知上下文**：
```
系统提示词：
"当前上下文中有以下变量：
- df: DataFrame (100 rows, 5 columns: date, region, product, quantity, revenue)
- total_revenue: float (1234567.89)
用户可以直接引用这些变量。"
```

#### 3. DSL 增强（结构化意图）

**场景 1: 数据转换流水线**
```dsl
@pipeline data_processing {
  load: "sales.csv" -> df
  filter: df[revenue > 1000] -> high_value
  group: high_value by region -> summary
  sort: summary by revenue desc
  visualize: summary as bar_chart
}
```

**场景 2: 机器学习工作流**
```dsl
@ml_workflow train_model {
  data {
    source: "mnist.csv"
    split: [0.8, 0.2]
    preprocess: normalize
  }

  model {
    type: cnn
    layers: [conv2d(32), maxpool, conv2d(64), dense(128), dense(10)]
  }

  training {
    epochs: 20
    batch_size: 128
    optimizer: adam
    loss: cross_entropy
  }

  evaluation {
    metrics: [accuracy, f1_score]
    save_best: true
  }
}
```

**优势**：
- 可读性强（接近自然语言）
- 类型安全（DSL 解析器验证）
- 可重用（保存为模板）
- 可组合（Pipeline 嵌套）

#### 4. 智能执行策略

**自动选择执行路径**：
```rust
fn execute_cell(input: &str, context: &NotebookContext) -> Result<CellOutput> {
    // 1. 尝试 DSL 匹配（高性能、确定性）
    if let Some(dsl_expr) = parse_dsl(input) {
        return execute_dsl(dsl_expr, context);
    }

    // 2. 尝试 Intent 匹配（中等性能）
    if let Some(intent) = match_intent(input) {
        return execute_intent(intent, context);
    }

    // 3. LLM 生成执行计划（高灵活性）
    let plan = llm_generate_plan(input, context).await?;
    return execute_plan(plan, context);
}
```

**渐进式复杂度**：
- 简单任务 → DSL/Intent（100ms 响应）
- 复杂任务 → LLM 生成（2-5s 响应）
- 混合任务 → 自动编排

---

## 🔬 DSL 设计哲学（深度思考）

### 为什么需要 DSL？

**问题 1**: 纯自然语言不够精确
```
用户: "分析销售数据"
LLM: 不确定要做什么分析 → 需要多轮对话
```

**DSL 解决方案**:
```dsl
@analyze sales {
  metrics: [total, average, trend]
  group_by: [region, product]
  time_range: last_30_days
}
```

**问题 2**: 重复性任务效率低
```
用户每次都要描述: "加载数据 → 清洗 → 分组 → 可视化"
```

**DSL 解决方案**:
```dsl
@template sales_analysis {
  # 定义一次，到处重用
}
```

**问题 3**: 复杂逻辑难以用自然语言表达
```
用户: "如果收入大于 1000 且地区是北京或上海，则标记为高价值客户，否则..."
```

**DSL 解决方案**:
```dsl
@rule classify_customer {
  if (revenue > 1000 && region in ["北京", "上海"]) {
    label: "high_value"
  } else if (revenue > 500) {
    label: "medium_value"
  } else {
    label: "low_value"
  }
}
```

### DSL 的三层抽象

```
Layer 3: 自然语言 (最灵活，最慢)
  "帮我分析一下最近的销售趋势"

Layer 2: DSL (结构化，快速)
  @analyze sales { metrics: [trend], time_range: recent }

Layer 1: 原生代码 (最精确，最复杂)
  df.groupby('date')['revenue'].sum().plot()
```

**用户可以自由选择层级**：
- 探索阶段 → 自然语言
- 成熟阶段 → DSL
- 定制需求 → 代码

### DSL 设计原则

1. **声明式优于命令式**
   ```dsl
   # Good (声明意图)
   @load data from "sales.csv"

   # Bad (命令步骤)
   @open file "sales.csv"
   @read content
   @parse as csv
   ```

2. **语义化优于技术化**
   ```dsl
   # Good (业务语言)
   @filter high_revenue_customers

   # Bad (技术实现)
   @sql "SELECT * FROM customers WHERE revenue > 1000"
   ```

3. **组合优于继承**
   ```dsl
   # Good (模块化组合)
   @pipeline {
     @load | @clean | @analyze | @visualize
   }

   # Bad (单体配置)
   @all_in_one_analysis { ... 100 行配置 ... }
   ```

---

## 🚀 实现路线图

### Phase 1: 基础 Cell 模型（v2.0-alpha）

**目标**: 实现基础的 Cell 执行和状态管理

**功能**:
- [x] Web 终端基础（v1.27.0 已完成）
- [ ] Cell 数据结构
- [ ] Cell 执行引擎
- [ ] 简单的输出渲染（文本、Markdown）
- [ ] Cell 持久化（保存/加载）

**技术栈**:
- Frontend: React + Monaco Editor（代码编辑） + Markdown 渲染
- Backend: Rust + tokio + serde
- Storage: .rcnb 文件格式（JSON Lines）

**示例 .rcnb 格式**:
```jsonl
{"type":"cell","id":"c1","input":"加载 sales.csv","output":{"type":"table","data":[...]},"metadata":{...}}
{"type":"cell","id":"c2","input":"按地区汇总","output":{"type":"text","data":"..."},"metadata":{...}}
```

---

### Phase 2: 智能执行（v2.1）

**目标**: 集成 DSL 和 LLM，实现智能意图理解

**功能**:
- [ ] DSL Parser（基于现有 Intent DSL 扩展）
- [ ] LLM Plan Generator（生成执行计划）
- [ ] Tool Calling 集成（复用现有工具系统）
- [ ] 错误处理和自然语言解释

**DSL 示例**:
```dsl
# 数据处理 DSL
@load "sales.csv" as df
@filter df where revenue > 1000
@group df by region aggregate sum(revenue)
@sort by revenue desc
@show top 10
```

**LLM 提示词设计**:
```
你是一个数据分析助手。用户输入自然语言，你需要生成执行计划。

当前上下文：
- 变量: df (DataFrame, 1000 rows)
- 可用工具: file_read, data_filter, data_group, visualize

用户输入: "找出收入最高的 5 个地区"

生成执行计划（JSON）:
{
  "steps": [
    {"tool": "data_group", "params": {"df": "df", "by": "region", "agg": "sum"}},
    {"tool": "data_sort", "params": {"by": "revenue", "ascending": false}},
    {"tool": "data_head", "params": {"n": 5}}
  ]
}
```

---

### Phase 3: 丰富可视化（v2.2）

**目标**: 支持表格、图表、图像等多种输出

**功能**:
- [ ] 表格渲染（类似 Pandas DataFrame）
- [ ] 图表渲染（Plotly.js / D3.js）
- [ ] 图像显示（PNG/SVG）
- [ ] 交互式可视化（筛选、缩放、导出）

**技术选型**:
- 表格: AG Grid 或 TanStack Table
- 图表: Plotly.js（兼容 Python Plotly）
- 交互: React + WebSocket 实时更新

**示例输出**:
```json
{
  "type": "chart",
  "spec": {
    "type": "bar",
    "data": [{"region": "北京", "revenue": 12345}, ...],
    "x": "region",
    "y": "revenue",
    "title": "各地区收入分布"
  }
}
```

---

### Phase 4: 协作和分享（v2.3）

**目标**: 支持 Notebook 分享和团队协作

**功能**:
- [ ] 导出为 HTML（静态页面）
- [ ] 导出为 PDF（报告）
- [ ] 版本控制集成（Git）
- [ ] 多人实时协作（WebRTC / OT）

**分享格式**:
```html
<!-- RealConsole Notebook Export -->
<!DOCTYPE html>
<html>
<head>
  <title>Sales Analysis Report</title>
  <style>/* 内嵌样式 */</style>
</head>
<body>
  <div class="cell">
    <div class="input">加载 sales.csv</div>
    <div class="output">
      <table>...</table>
    </div>
  </div>
  <div class="cell">
    <div class="input">按地区汇总</div>
    <div class="output">
      <div id="chart1"></div>
      <script>/* Plotly 图表 */</script>
    </div>
  </div>
</body>
</html>
```

---

## 🎓 理论基础（深度思考）

### 1. 人机交互的演化

```
第一代: 命令行 (1970s)
  $ ls -la | grep ".txt"
  - 需要记忆命令和参数
  - 高学习曲线

第二代: GUI (1980s)
  [File] -> [Open] -> 选择文件
  - 可视化操作
  - 降低学习曲线，但灵活性受限

第三代: 自然语言 + AI (2020s)
  "显示所有文本文件的详细信息"
  - 用人类语言表达意图
  - LLM 理解并执行
  - 学习曲线最低，灵活性最高
```

**RealConsole v2 的定位**: 第三代交互范式的探索

---

### 2. 抽象层次的权衡

**Von Neumann 层次结构**:
```
高层抽象 (易用，慢)
  ↑
  | 自然语言 → LLM 理解 → 生成代码
  |
  | DSL → 解析器 → 中间表示
  |
  | 编程语言 → 编译器 → 机器码
  ↓
低层抽象 (复杂，快)
```

**RealConsole v2 的创新**: 允许用户在多个层次自由移动
- 初学者停留在高层（自然语言）
- 专家深入到低层（DSL/代码）
- 无缝切换，无需重写整个 Notebook

---

### 3. 声明式 vs 命令式

**命令式**（传统编程）:
```python
result = []
for row in df.iterrows():
    if row['revenue'] > 1000:
        result.append(row)
df_filtered = pd.DataFrame(result)
```

**声明式**（SQL/DSL）:
```sql
SELECT * FROM df WHERE revenue > 1000
```

**自然语言**（RealConsole v2）:
```
筛选出收入大于 1000 的记录
```

**优势**:
- 用户关注"做什么"，而非"怎么做"
- LLM 负责将"做什么"翻译成"怎么做"

---

### 4. 意图 → 执行的映射

**传统编程**: 程序员手动编写映射
```
意图: 找出最大值
代码: max(list)
```

**RealConsole v2**: LLM 自动生成映射
```
意图: "找出收入最高的客户"
LLM 分析:
  - 数据源: customers
  - 字段: revenue
  - 操作: max
  - 输出: customer_id, name, revenue
生成代码:
  df.loc[df['revenue'].idxmax()]
```

**关键技术**: Few-shot Learning
```
示例 1:
  输入: "统计每个类别的数量"
  输出: df.groupby('category').size()

示例 2:
  输入: "找出最新的 10 条记录"
  输出: df.sort_values('date', ascending=False).head(10)

新任务:
  输入: "找出收入最高的客户"
  输出: [LLM 生成基于示例的代码]
```

---

## 💡 关键挑战与解决方案

### 挑战 1: 自然语言的歧义性

**问题**:
```
用户: "分析销售数据"
可能含义:
  1. 显示总销售额？
  2. 按时间展示趋势？
  3. 按地区分组统计？
  4. 找出异常值？
```

**解决方案 A: 交互式澄清**
```
RealConsole: "您想要哪种分析？
  1. 总销售额统计
  2. 时间趋势分析
  3. 地区分布分析
  4. 异常检测
  5. 其他（请描述）"
```

**解决方案 B: 上下文推断**
```
[Cell 1] 加载 sales.csv（包含 date, region, revenue 列）
[Cell 2] "分析销售数据"
         → LLM 推断: 用户可能想看时间趋势或地区分布
         → 生成两个候选分析，让用户选择
```

**解决方案 C: DSL 辅助**
```
@analyze sales {
  # 显式指定分析类型
  type: trend | distribution | summary
}
```

---

### 挑战 2: 性能 vs 灵活性

**问题**:
- LLM 调用慢（2-5s）
- 用户期望即时反馈

**解决方案: 混合策略**
```rust
enum ExecutionStrategy {
    // 快速路径: DSL/Intent 匹配（<100ms）
    FastPath(DslExpr),

    // 智能路径: LLM 生成计划，缓存结果（首次 2-5s，后续 <100ms）
    CachedLLM(CachedPlan),

    // 灵活路径: 实时 LLM 生成（2-5s）
    LiveLLM(Prompt),
}

fn choose_strategy(input: &str, cache: &Cache) -> ExecutionStrategy {
    // 1. 尝试 DSL 解析
    if let Some(dsl) = parse_dsl(input) {
        return FastPath(dsl);
    }

    // 2. 检查缓存（相似查询）
    if let Some(plan) = cache.find_similar(input) {
        return CachedLLM(plan);
    }

    // 3. 实时生成
    LiveLLM(input.into())
}
```

**缓存策略**:
```rust
struct QueryCache {
    // 完全匹配缓存
    exact: HashMap<String, Plan>,

    // 语义相似缓存（向量搜索）
    semantic: VectorDB<String, Plan>,
}
```

---

### 挑战 3: 安全性

**问题**:
```
用户: "删除所有文件"
LLM 生成: rm -rf /
```

**解决方案 A: 危险操作检测**
```rust
fn is_dangerous(plan: &Plan) -> bool {
    let dangerous_patterns = [
        r"rm -rf",
        r"DROP TABLE",
        r"DELETE FROM .* WHERE 1=1",
        // ...
    ];

    plan.commands.iter().any(|cmd| {
        dangerous_patterns.iter().any(|p| cmd.matches(p))
    })
}
```

**解决方案 B: 沙箱执行**
```rust
struct Sandbox {
    allowed_paths: Vec<PathBuf>,  // 白名单目录
    denied_commands: Vec<String>,  // 黑名单命令
    max_execution_time: Duration,  // 超时保护
}

impl Sandbox {
    fn execute(&self, plan: &Plan) -> Result<Output> {
        // 在受限环境中执行
        // - chroot
        // - 资源限制（CPU/内存/网络）
        // - 权限降级
    }
}
```

**解决方案 C: 用户确认**
```
RealConsole: "⚠️  检测到危险操作：
  命令: rm -rf /var/log/*
  影响: 删除所有日志文件

是否继续？ [y/N]"
```

---

### 挑战 4: 可重现性

**问题**:
- 不同时间执行同一 Notebook，结果可能不同
- LLM 生成的代码可能不一致

**解决方案 A: 冻结执行计划**
```json
{
  "cell_id": "c1",
  "input": "加载销售数据",
  "execution_plan": {
    "frozen": true,
    "version": "2024-11-06-hash-abc123",
    "steps": [
      {"tool": "file_read", "params": {"path": "sales.csv"}},
      {"tool": "table_display", "params": {"limit": 5}}
    ]
  }
}
```

**解决方案 B: LLM 参数固定**
```yaml
notebook_metadata:
  llm:
    provider: deepseek
    model: deepseek-chat
    temperature: 0  # 确定性输出
    seed: 42        # 固定随机种子
```

**解决方案 C: 显式依赖声明**
```yaml
dependencies:
  - name: pandas
    version: "2.0.0"
  - name: matplotlib
    version: "3.7.0"
  - tool: data_loader
    version: "1.2.3"
```

---

## 🌈 愿景总结

### 短期目标（v2.0 - 2025 Q2）

✅ **可用的自然语言 Notebook**
- 基础 Cell 模型
- LLM 驱动的任务执行
- Markdown/表格输出
- 持久化和分享

### 中期目标（v2.5 - 2025 Q4）

✅ **生产级数据分析平台**
- 丰富可视化（图表、仪表盘）
- DSL 增强（数据处理、ML 工作流）
- 协作功能（多人编辑、版本控制）
- 插件系统（扩展工具和可视化）

### 长期愿景（v3.0+）

✅ **自然语言编程新范式**
- AI 辅助编程（代码生成、调试、优化）
- 跨领域应用（DevOps、科研、教育）
- 社区生态（Notebook 市场、模板库）
- 多模态交互（语音、图像、视频）

---

## 🔗 与现有技术的关系

### vs Jupyter Notebook
- **互补**: RealConsole 面向非程序员，Jupyter 面向程序员
- **集成**: 可以生成 .ipynb 格式，与 Jupyter 生态兼容

### vs ChatGPT Code Interpreter
- **差异**: ChatGPT 是对话式，RealConsole 是 Notebook 式（更适合复杂分析）
- **优势**: 可重现性、版本控制、本地执行

### vs Observable Notebook
- **相似**: 响应式计算、可视化
- **差异**: Observable 用 JavaScript，RealConsole 用自然语言

### vs Dataiku / KNIME
- **相似**: 低代码数据分析
- **差异**: 他们用拖拽 UI，我们用自然语言

---

## 📚 参考文献

1. **IPython/Jupyter 论文**
   "Jupyter Notebooks – a publishing format for reproducible computational workflows"
   Fernando Pérez, Brian E. Granger (2015)

2. **自然语言编程**
   "Natural Language Programming: A New Paradigm for Software Development"
   Reiss, S.P. (2019)

3. **声明式编程**
   "Out of the Tar Pit"
   Ben Moseley, Peter Marks (2006)

4. **DSL 设计**
   "Domain-Specific Languages"
   Martin Fowler (2010)

5. **交互式计算**
   "The Design of Computational Media"
   Donald Knuth (1984) - Literate Programming

---

**文档状态**: 初稿，持续更新
**下次审阅**: v2.0-alpha 发布前

**核心哲学**:
> 不是创造更好的编程语言，而是让每个人都能用自己的语言编程。

