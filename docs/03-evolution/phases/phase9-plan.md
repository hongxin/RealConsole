# Phase 9 开发计划：智能化与可视化基础

**版本目标**: v0.9.0
**时间规划**: 2-3 周
**优先级**: 高

## 核心目标

在 v0.8.0 的基础上，夯实系统的**智能化**和**可视化**基础，使 RealConsole 从"能执行"进化到"会思考、能反馈"。

## 一、更智能的对话 🧠

### 1.1 上下文理解增强

#### 目标
- 记住对话历史，理解指代关系
- 跨对话的任务连续性
- 智能推断用户意图

#### 实现方案

**1.1.1 增强的 Memory 系统**
```rust
// src/memory/context_tracker.rs
pub struct ContextTracker {
    /// 最近提到的实体（文件、目录、命令等）
    recent_entities: HashMap<EntityType, Vec<Entity>>,

    /// 任务上下文链
    task_chain: Vec<TaskContext>,

    /// 指代消解历史
    reference_history: LRUCache<String, Entity>,
}

pub enum EntityType {
    File(PathBuf),
    Directory(PathBuf),
    Command(String),
    Variable(String, String),
    Concept(String),
}
```

**1.1.2 指代消解**
```
用户: "请查看 src/main.rs 的内容"
系统: [记录实体: File(src/main.rs)]

用户: "它有多少行？"
系统: [消解 "它" → src/main.rs]
      [执行: wc -l src/main.rs]
```

**实施步骤**:
1. 扩展 Memory 模块，添加 `ContextTracker`
2. 在 Agent 中集成上下文跟踪
3. LLM 提示词中加入上下文信息
4. 测试指代消解准确率

**验收标准**:
- [ ] 能记住最近 5 个提到的实体
- [ ] 正确消解 "它"、"这个"、"那个" 等指代
- [ ] 测试覆盖率 ≥ 80%

### 1.2 任务分解和规划

#### 目标
- 复杂任务自动分解
- 显示执行计划供用户确认
- 步骤间依赖管理

#### 实现方案

**1.2.1 任务规划器**
```rust
// src/planning/task_planner.rs
pub struct TaskPlanner {
    llm: Arc<dyn LlmClient>,
}

pub struct ExecutionPlan {
    /// 任务描述
    description: String,

    /// 分解的步骤
    steps: Vec<PlanStep>,

    /// 预估时间
    estimated_duration: Duration,

    /// 风险等级
    risk_level: RiskLevel,
}

pub struct PlanStep {
    id: String,
    description: String,
    command: String,
    dependencies: Vec<String>,  // 依赖的步骤 ID
    optional: bool,
}
```

**1.2.2 用户交互流程**
```
用户: "帮我初始化一个新的 Rust 项目并配置 GitHub Actions"

系统: 📋 执行计划（共 5 步，预计 2 分钟）

      1️⃣ 创建项目目录
         命令: cargo new my-project

      2️⃣ 初始化 Git 仓库
         命令: cd my-project && git init
         依赖: 步骤 1

      3️⃣ 创建 .github/workflows 目录
         命令: mkdir -p my-project/.github/workflows
         依赖: 步骤 2

      4️⃣ 生成 CI 配置文件
         命令: [写入 YAML 配置]
         依赖: 步骤 3

      5️⃣ 提交初始配置
         命令: git add . && git commit -m "Initial setup"
         依赖: 步骤 4
         可选: 是

      ⚠️ 风险等级: 低

      是否执行？[y/N]:
```

**实施步骤**:
1. 实现 `TaskPlanner` 和 `ExecutionPlan`
2. 设计任务分解的 LLM 提示词
3. 实现步骤依赖检查和执行
4. 添加执行进度显示
5. 支持中断和恢复

**验收标准**:
- [ ] 能分解包含 3-10 步的复杂任务
- [ ] 正确识别步骤间的依赖关系
- [ ] 用户可以在任何步骤中断
- [ ] 测试覆盖主要场景

### 1.3 错误自动修复

#### 目标
- 识别常见错误模式
- 自动建议修复方案
- 学习用户的修复历史

#### 实现方案

**1.3.1 错误分析器**
```rust
// src/error_handling/error_analyzer.rs
pub struct ErrorAnalyzer {
    /// 错误模式库
    patterns: Vec<ErrorPattern>,

    /// LLM 客户端（用于复杂错误）
    llm: Arc<dyn LlmClient>,

    /// 修复历史（学习）
    fix_history: HashMap<String, Vec<FixAttempt>>,
}

pub struct ErrorPattern {
    pattern: Regex,
    category: ErrorCategory,
    suggestions: Vec<FixSuggestion>,
}

pub enum ErrorCategory {
    CommandNotFound,
    PermissionDenied,
    FileNotFound,
    SyntaxError,
    NetworkError,
    Unknown,
}

pub struct FixSuggestion {
    description: String,
    command: String,
    confidence: f32,
    auto_apply: bool,
}
```

**1.3.2 交互流程**
```
用户: "cargo build"

系统: ❌ 错误: error: could not find `Cargo.toml`

      💡 自动分析：
      原因: 当前目录不是 Cargo 项目

      建议修复方案：
      1. [推荐] 进入项目目录
         cd <project-path>

      2. 初始化新项目
         cargo new my-project

      3. 查找附近的 Cargo.toml
         find . -name Cargo.toml -maxdepth 3

      选择方案 [1/2/3] 或输入 'skip':
```

**实施步骤**:
1. 收集常见错误模式
2. 实现 `ErrorAnalyzer` 和模式匹配
3. 集成 LLM 进行复杂错误分析
4. 实现修复建议的交互流程
5. 添加修复历史学习

**验收标准**:
- [ ] 识别 10+ 种常见错误模式
- [ ] 修复建议准确率 ≥ 70%
- [ ] 用户可选择应用或跳过
- [ ] 记录修复成功率

## 二、可视化基础 📊

### 2.1 命令执行流程可视化

#### 目标
- 显示命令的执行流程
- 工具调用链可视化
- 错误点高亮

#### 实现方案

**2.1.1 执行追踪**
```rust
// src/visualization/execution_trace.rs
pub struct ExecutionTrace {
    /// 根节点（用户输入）
    root: TraceNode,

    /// 执行开始时间
    start_time: DateTime<Utc>,

    /// 总耗时
    duration: Duration,

    /// 状态
    status: ExecutionStatus,
}

pub struct TraceNode {
    id: String,
    node_type: NodeType,
    description: String,
    start_time: DateTime<Utc>,
    duration: Option<Duration>,
    status: NodeStatus,
    children: Vec<TraceNode>,
    metadata: HashMap<String, String>,
}

pub enum NodeType {
    UserInput,
    IntentMatch,
    LlmCall,
    ToolCall,
    ShellExecution,
    ConversationTurn,
}
```

**2.1.2 ASCII 流程图渲染**
```
用户输入: "请帮我统计target目录磁盘占用"
│
├─ 🧠 LLM 分析 (0.8s)
│  └─ 识别意图: 查询磁盘占用
│
├─ 🔧 工具调用: shell_execute (0.2s)
│  ├─ 命令: du -sh target
│  ├─ 安全检查: ✅ 通过
│  └─ 执行结果: 8.6G
│
└─ 💬 LLM 总结 (0.5s)
   └─ 输出: "当前目录的总磁盘占用为 8.6GB"

总耗时: 1.5s
状态: ✅ 成功
```

**实施步骤**:
1. 实现 `ExecutionTrace` 和 `TraceNode`
2. 在 Agent 各个环节插入追踪点
3. 实现 ASCII 树状图渲染
4. 添加详细模式和简洁模式
5. 支持导出为 JSON

**验收标准**:
- [ ] 完整追踪执行流程
- [ ] 清晰的层级结构显示
- [ ] 时间信息精确到毫秒
- [ ] 支持错误节点高亮

### 2.2 系统状态仪表板

#### 目标
- 实时系统统计
- 历史趋势分析
- 性能瓶颈识别

#### 实现方案

**2.2.1 统计收集器**
```rust
// src/visualization/dashboard.rs
pub struct SystemDashboard {
    /// LLM 统计
    llm_stats: LlmStats,

    /// 工具统计
    tool_stats: ToolStats,

    /// 命令历史统计
    command_stats: CommandStats,

    /// 性能指标
    performance_metrics: PerformanceMetrics,
}

pub struct LlmStats {
    total_calls: u64,
    success_rate: f32,
    avg_response_time: Duration,
    token_usage: u64,
    cost_estimate: f32,
}

pub struct ToolStats {
    tool_usage: HashMap<String, u64>,
    success_rate_by_tool: HashMap<String, f32>,
    avg_execution_time: HashMap<String, Duration>,
}
```

**2.2.2 仪表板显示**
```
╔════════════════════════════════════════════════════════════════╗
║            RealConsole 系统仪表板 v0.9.0                      ║
╠════════════════════════════════════════════════════════════════╣
║ 📊 会话统计                                                    ║
║   • 运行时间: 2h 34m                                           ║
║   • 总命令数: 127                                              ║
║   • 成功率: 94.5% (120/127)                                    ║
╠════════════════════════════════════════════════════════════════╣
║ 🧠 LLM 统计                                                    ║
║   • 调用次数: 215                                              ║
║   • 平均响应: 0.8s                                             ║
║   • Token 使用: 48,523                                         ║
║   • 预估成本: $0.12                                            ║
╠════════════════════════════════════════════════════════════════╣
║ 🔧 工具使用 Top 5                                              ║
║   1. shell_execute    42 次  ████████████████░░░░  80%         ║
║   2. calculator       18 次  ███████░░░░░░░░░░░░░  34%         ║
║   3. list_dir         12 次  █████░░░░░░░░░░░░░░░  23%         ║
║   4. read_file         8 次  ███░░░░░░░░░░░░░░░░░  15%         ║
║   5. get_datetime      5 次  ██░░░░░░░░░░░░░░░░░░   9%         ║
╠════════════════════════════════════════════════════════════════╣
║ ⚡ 性能指标                                                    ║
║   • P50 响应时间: 1.2s                                         ║
║   • P95 响应时间: 3.5s                                         ║
║   • P99 响应时间: 8.2s                                         ║
║   • 最慢命令: "cargo build" (45.3s)                            ║
╚════════════════════════════════════════════════════════════════╝

输入 /dashboard 查看实时更新
```

**实施步骤**:
1. 实现统计数据收集
2. 设计仪表板布局
3. 实现 ASCII 表格和图表渲染
4. 添加实时更新支持
5. 支持导出统计报告

**验收标准**:
- [ ] 收集完整的统计数据
- [ ] 清晰美观的仪表板显示
- [ ] 支持实时刷新
- [ ] 可导出为 JSON/CSV

### 2.3 性能分析报告

#### 目标
- 识别性能瓶颈
- 提供优化建议
- 趋势分析

#### 实现方案

**2.3.1 性能分析器**
```rust
// src/visualization/performance_analyzer.rs
pub struct PerformanceAnalyzer {
    /// 历史执行记录
    history: Vec<ExecutionRecord>,

    /// 分析配置
    config: AnalyzerConfig,
}

pub struct ExecutionRecord {
    command: String,
    total_time: Duration,
    breakdown: ExecutionBreakdown,
    timestamp: DateTime<Utc>,
}

pub struct ExecutionBreakdown {
    llm_time: Duration,
    tool_time: Duration,
    shell_time: Duration,
    other_time: Duration,
}

pub struct PerformanceReport {
    /// 总体统计
    summary: PerformanceSummary,

    /// 瓶颈识别
    bottlenecks: Vec<Bottleneck>,

    /// 优化建议
    recommendations: Vec<Recommendation>,
}
```

**2.3.2 报告示例**
```
═══════════════════════════════════════════════════════════════
  性能分析报告 - 2025-10-16 14:30:00
═══════════════════════════════════════════════════════════════

📊 总体统计（最近 100 次命令）
─────────────────────────────────────────────────────────────
  平均响应时间: 2.3s
  中位数: 1.5s
  最快: 0.2s (calculator: 2+2)
  最慢: 45.3s (cargo build)

⏱️ 时间分布
─────────────────────────────────────────────────────────────
  LLM 调用:     45% (1.0s)  ████████████████░░░░░░░░░░░░░░░░
  Shell 执行:   35% (0.8s)  ████████████░░░░░░░░░░░░░░░░░░░░
  工具调用:     15% (0.3s)  █████░░░░░░░░░░░░░░░░░░░░░░░░░░░
  其他开销:      5% (0.1s)  ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

🔍 识别的瓶颈
─────────────────────────────────────────────────────────────
  1. [严重] LLM 响应慢
     平均: 1.0s | P95: 2.8s
     建议: 考虑使用本地模型或缓存常见查询

  2. [中等] Shell 命令超时
     5% 的命令超过 10s
     建议: 添加进度提示或异步执行

💡 优化建议
─────────────────────────────────────────────────────────────
  ✓ 启用 LLM 响应缓存（预计节省 30% 时间）
  ✓ 对长时间命令添加进度条
  ✓ 考虑预加载常用工具

═══════════════════════════════════════════════════════════════
```

**实施步骤**:
1. 实现性能数据收集
2. 实现统计分析算法
3. 设计瓶颈识别规则
4. 实现报告生成
5. 添加定期分析任务

**验收标准**:
- [ ] 准确识别性能瓶颈
- [ ] 提供可行的优化建议
- [ ] 支持历史趋势分析
- [ ] 报告格式清晰易读

## 三、实施路线图

### Week 1: 智能化基础
- [ ] Day 1-2: 上下文理解增强（ContextTracker）
- [ ] Day 3-4: 任务规划器基础实现
- [ ] Day 5-7: 错误分析器和修复建议

### Week 2: 可视化基础
- [ ] Day 1-3: 执行追踪和流程图渲染
- [ ] Day 4-5: 系统仪表板实现
- [ ] Day 6-7: 性能分析器和报告

### Week 3: 集成和优化
- [ ] Day 1-2: 功能集成和测试
- [ ] Day 3-4: 性能优化
- [ ] Day 5: 文档编写
- [ ] Day 6-7: 用户测试和反馈收集

## 四、技术挑战

### 挑战 1: LLM 调用成本
- **问题**: 更多的智能功能意味着更多 LLM 调用
- **方案**:
  - 智能缓存策略
  - 混合使用规则和 LLM
  - 可配置的智能级别

### 挑战 2: 可视化在 CLI 的限制
- **问题**: 终端环境的显示限制
- **方案**:
  - 渐进式渲染
  - 自适应终端宽度
  - 可选的 Web 仪表板

### 挑战 3: 性能开销
- **问题**: 追踪和统计会增加开销
- **方案**:
  - 异步统计收集
  - 采样策略
  - 可配置的追踪级别

## 五、成功指标

### 智能化
- [ ] 指代消解准确率 ≥ 85%
- [ ] 任务分解成功率 ≥ 90%
- [ ] 错误修复建议采纳率 ≥ 50%
- [ ] 用户满意度 ≥ 4.0/5.0

### 可视化
- [ ] 执行流程追踪覆盖率 100%
- [ ] 仪表板刷新延迟 < 100ms
- [ ] 性能分析准确识别瓶颈 ≥ 80%
- [ ] 报告可读性评分 ≥ 4.0/5.0

## 六、文档计划

- [ ] 用户指南：智能对话功能
- [ ] 用户指南：可视化功能
- [ ] 开发文档：扩展智能模块
- [ ] 开发文档：自定义可视化
- [ ] API 文档：统计和追踪接口

---

**起草**: 2025-10-16
**状态**: 待审核
**下一步**: 启动 Week 1 开发
