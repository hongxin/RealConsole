# RealConsole 领域特定语言 (DSL) 设计方案

## 一、设计目标与哲学

### 1.1 核心目标

在构建智能引擎逻辑过程中，我们需要一种**抽象的 DSL 表示系统**，作为逻辑推理、意图识别的工具，在**可读性与可计算性之间取得平衡**。

**关键平衡点**：
- **可读性**：领域专家（非程序员）能理解和编写
- **可计算性**：机器能解析、验证和执行
- **可扩展性**：易于添加新的意图和工具
- **安全性**：内置沙箱和约束检查

### 1.2 设计哲学

基于 FCToken 框架 DSL 的六大原则：

1. **抽象（Abstraction）**：隐藏技术细节，聚焦领域概念
2. **泛化（Generalization）**：统一模式，减少概念数量
3. **优化（Optimization）**：高效执行，最小化开销
4. **符号表示（Notation）**：清晰、自然的语法
5. **压缩（Compression）**：简洁表达，避免冗余
6. **吸收（Absorption）**：隐式表达领域共性

## 二、Python 版本核心实现分析

### 2.1 Execute Shell 核心逻辑

Python 版本的 `execute_shell` 采用**分层安全架构**：

```
Layer 1: 命令分类（safe/moderate/dangerous）
Layer 2: 智能解析（shlex-based, not regex）
Layer 3: 风险评估（intent-based）
Layer 4: 策略执行（configurable sandbox）
```

**核心设计模式：管道处理器（Pipeline Processor）**

```python
STAGE_MAP: Dict[str, StageFunc] = {
    'echo': stage_echo,
    'ls': stage_ls,
    'find': stage_find,
    # ... 27+ 命令
}

# 管道执行流程
stages = split_pipeline(command)  # "find . | grep py"
data = None
for stage in stages:
    cmd, args = stage[0], stage[1:]
    handler = STAGE_MAP[cmd]
    data, err = handler(args, data)  # 链式数据传递
```

**关键抽象：Stage Function**

```python
StageFunc = Callable[
    [List[str], Optional[List[str]]], # (args, stdin_data)
    Tuple[Optional[List[str]], Optional[str]] # (stdout_data, error)
]
```

这是一个**数据流编程模型**（Dataflow Programming）：
- 每个命令是一个 actor（参与者）
- 通过 stdin/stdout 传递数据
- 保证原子性和可组合性

### 2.2 Planner 核心逻辑

Python 版本的 `planner` 采用**意图识别 + 模板匹配**：

```python
# 三层架构
1. 意图检测（Intent Detection）
   - 关键词匹配（keyword matching）
   - 正则模式匹配（pattern matching）
   - 实体提取（entity extraction）
   - 置信度计算（confidence scoring）

2. 模板匹配（Template Matching）
   - 预定义场景模板（scenario templates）
   - 启发式规则（heuristic rules）
   - 上下文适配（context adaptation）

3. 步骤生成（Step Generation）
   - 通用步骤生成器（generic step generator）
   - 工具感知增强（tool awareness）
   - 沙箱安全检查（sandbox validation）
```

**核心数据结构**：

```python
@dataclass
class Intent:
    name: str          # 意图名称
    confidence: float  # 置信度 0-1
    entities: Dict[str, Any]  # 提取的实体

@dataclass
class Step:
    number: int
    description: str
    command: Optional[str]       # shell 命令
    tool: Optional[str]          # 工具名称
    note: Optional[str]
    tool_available: Optional[bool]    # 工具是否可用
    sandbox_safe: Optional[bool]      # 沙箱安全性
    sandbox_reason: Optional[str]     # 阻断原因

@dataclass
class PlanResult:
    task: str
    intents: List[Intent]
    steps: List[Step]
    suggestions: List[str]
    template_name: Optional[str]
```

**工具感知机制**：

```python
def _enhance_with_tool_awareness(steps: List[Step]) -> List[Step]:
    """增强步骤，添加工具可用性和沙箱安全性检查"""
    for step in steps:
        if step.tool:
            # 检查工具是否存在
            step.tool_available = check_tool_availability(step.tool)

            # 检查沙箱安全性
            if step.tool_available and step.command:
                ok, reason = check_sandbox_safety(step.tool, step.command)
                step.sandbox_safe = ok
                step.sandbox_reason = reason if not ok else None
```

## 三、RealConsole DSL 设计方案

### 3.1 整体架构

采用**三层 DSL 架构**：

```
┌─────────────────────────────────────────────────┐
│   应用层 (Application Layer)                    │
│   - Intent DSL: 意图表达语言                    │
│   - Plan DSL: 规划描述语言                      │
│   - Tool DSL: 工具定义语言                      │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│   中间层 (Intermediate Layer)                   │
│   - Pipeline IR: 管道中间表示                   │
│   - Dataflow Graph: 数据流图                    │
│   - Execution Plan: 执行计划                    │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│   基础层 (Foundation Layer)                     │
│   - Lexer/Parser: 词法/语法分析                 │
│   - Type System: 类型系统                       │
│   - Validator: 验证器                           │
│   - Executor: 执行器                            │
└─────────────────────────────────────────────────┘
```

### 3.2 Intent DSL - 意图表达语言

**设计目标**：声明式表达用户意图和领域知识

**语法设计**：

```rust
// 意图定义
Intent FileOps::CountPythonLines {
    keywords: ["python", "py", "行数", "count", "lines"],
    patterns: [r"统计.*python.*行数", r"count.*\.py.*lines"],
    entities: {
        file_type: "python",
        operation: "count_lines"
    },
    confidence_threshold: 0.5
}

// 模板定义
Template CountPythonLinesTemplate {
    match: Intent(FileOps::CountPythonLines) AND confidence > 0.7,
    steps: [
        Step {
            description: "查找所有 Python 文件",
            command: Shell("find . -name '*.py' -type f"),
            tool: "execute_shell",
            validation: SandboxCheck
        },
        Step {
            description: "统计总行数",
            command: Shell("find . -name '*.py' -type f -exec wc -l {} +"),
            tool: "execute_shell",
            note: "最后一行显示总行数"
        }
    ]
}

// 组合意图
CompositeIntent AnalyzeErrors {
    primary: DiagnosticOps::ErrorAnalysis,
    secondary: [FileOps::ListFiles, DataOps::Filter],

    preconditions: [
        HasLogFiles,
        ToolAvailable("execute_shell")
    ],

    postconditions: [
        HasResults,
        ResultsValid
    ]
}
```

**关键特性**：
- **声明式**：描述"做什么"，而非"怎么做"
- **可组合**：意图可以嵌套和组合
- **可验证**：内置前置/后置条件
- **类型安全**：静态类型检查

### 3.3 Pipeline IR - 管道中间表示

**设计目标**：统一表示各种命令组合和数据流

**语法设计**：

```rust
// 管道表达式
Pipeline FindAndCount {
    // 节点定义
    nodes: [
        Node<FindFiles> {
            id: "find_py",
            command: "find . -name '*.py' -type f",
            output_type: FileList,
            sandbox_policy: Safe
        },
        Node<CountLines> {
            id: "wc_lines",
            command: "wc -l",
            input_type: FileList,
            output_type: Integer,
            sandbox_policy: Safe
        }
    ],

    // 边（数据流）
    edges: [
        Edge {
            from: "find_py",
            to: "wc_lines",
            transform: PipeData  // 标准管道传递
        }
    ],

    // 元数据
    metadata: {
        max_stages: 10,
        timeout_ms: 30000,
        retries: 1
    }
}

// 并行管道
ParallelPipeline HealthCheck {
    branches: [
        Pipeline { nodes: [CheckLLM, FormatLLMStatus] },
        Pipeline { nodes: [CheckTools, FormatToolStatus] },
        Pipeline { nodes: [CheckMemory, FormatMemoryStatus] }
    ],
    merge_strategy: Concat,  // Concat | Interleave | Custom
    timeout_ms: 5000
}

// 条件管道
ConditionalPipeline ErrorAnalysis {
    condition: HasLogFiles,

    then: Pipeline {
        nodes: [FindLogs, GrepErrors, CountErrors]
    },

    else: Pipeline {
        nodes: [SuggestCreateLog]
    }
}
```

**关键特性**：
- **图结构**：支持复杂的数据流
- **类型化**：输入输出类型明确
- **可分析**：编译期优化和验证
- **可并行**：自动识别可并行节点

### 3.4 Tool DSL - 工具定义语言

**设计目标**：统一工具接口和安全策略

**语法设计**：

```rust
// 工具定义
Tool ExecuteShell {
    name: "execute_shell",
    description: "Execute shell commands with security restrictions",

    // 参数定义
    parameters: {
        command: {
            type: String,
            required: true,
            max_length: 1000,
            pattern: r"^[a-zA-Z0-9\s\-\.\/_\|]*$"  // 基本过滤
        }
    },

    // 返回类型
    returns: Result<String, ExecutionError>,

    // 安全策略
    sandbox: {
        mode: Balanced,  // Strict | Balanced | Permissive

        // 分类规则
        classification: {
            safe: ["ls", "find", "grep", "cat", "head", "tail", "wc"],
            moderate: ["echo", "date", "pwd"],
            dangerous: ["rm", "mv", "chmod", "mkdir", "cp"]
        },

        // 风险评估规则
        risk_rules: [
            Rule {
                name: "prevent_recursive_rm",
                pattern: r"rm\s+-rf?\s+/",
                action: Block("禁止删除根目录")
            },
            Rule {
                name: "prevent_privilege_escalation",
                pattern: r"\bsudo\b",
                action: Block("禁止权限提升")
            }
        ],

        // 资源限制
        limits: {
            max_output_lines: 10000,
            max_pipeline_stages: 10,
            timeout_seconds: 30
        }
    },

    // 执行策略
    execution: {
        mode: Async,
        retry: {
            max_attempts: 1,
            backoff: None
        },
        caching: {
            enabled: false  // Shell 命令不缓存
        }
    }
}

// 工具组合
ToolChain AnalyzeCodebase {
    tools: [
        Tool(ExecuteShell) with {
            command: "find . -name '*.rs' -type f"
        },
        Tool(CountLines),
        Tool(CalculateStats)
    ],

    // 组合策略
    strategy: Sequential,  // Sequential | Parallel | Conditional

    // 失败处理
    on_error: Continue,  // Stop | Continue | Retry

    // 结果聚合
    aggregation: Custom {
        combine: |results| {
            // 自定义聚合逻辑
        }
    }
}
```

**关键特性**：
- **声明式安全**：规则即文档
- **类型系统**：参数和返回值类型化
- **可组合**：工具可以链式组合
- **可测试**：独立的工具单元

### 3.5 类型系统设计

**基础类型**：

```rust
// 原生类型
enum PrimitiveType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
}

// 复合类型
enum CompositeType {
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Optional(Box<Type>),
    Result(Box<Type>, Box<Type>),
}

// 领域类型
enum DomainType {
    FilePath,
    FileList,
    CommandLine,
    PipelineData,
    IntentResult,
    PlanResult,
}

// 类型约束
struct TypeConstraint {
    base_type: Type,
    constraints: Vec<Constraint>
}

enum Constraint {
    Range(min, max),
    Pattern(regex),
    Length(min, max),
    Custom(validator_fn)
}
```

**类型推导示例**：

```rust
// 类型推导链
Pipeline AutoTyped {
    nodes: [
        find_files,  // infer: () -> FileList
        wc_lines,    // infer: FileList -> Integer
        format_num   // infer: Integer -> String
    ]
}
// 编译器自动验证：FileList -> Integer -> String 类型兼容
```

### 3.6 执行模型

**数据流执行模型（Dataflow Execution）**：

```rust
struct DataflowGraph {
    nodes: HashMap<NodeId, Node>,
    edges: Vec<Edge>,

    // 拓扑排序
    execution_order: Vec<NodeId>,

    // 并行组
    parallel_groups: Vec<Vec<NodeId>>
}

impl DataflowGraph {
    // 编译期优化
    fn optimize(&mut self) {
        self.merge_adjacent_nodes();    // 节点合并
        self.eliminate_dead_code();     // 死代码消除
        self.parallelize_independent(); // 并行化分析
    }

    // 运行时执行
    async fn execute(&self, input: Value) -> Result<Value, Error> {
        let mut results = HashMap::new();

        // 按拓扑顺序执行
        for group in &self.parallel_groups {
            // 并行执行组内节点
            let futures: Vec<_> = group.iter()
                .map(|id| self.execute_node(id, &results))
                .collect();

            let group_results = join_all(futures).await;

            for (id, result) in group.iter().zip(group_results) {
                results.insert(*id, result?);
            }
        }

        self.get_final_result(&results)
    }
}
```

## 四、Rust 实现策略

### 4.1 技术栈选择

```rust
// 解析器生成器
pest = "2.7"  // PEG parser generator
// 或者
nom = "7.1"   // Parser combinator

// 类型系统
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

// 异步执行
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"

// 数据流图
petgraph = "0.6"  // Graph data structure

// 表达式求值
evalexpr = "11.0"  // 如果需要动态表达式
```

### 4.2 模块结构

```
src/dsl/
├── mod.rs                    # DSL 模块入口
├── intent/
│   ├── mod.rs
│   ├── parser.rs             # Intent DSL 解析器
│   ├── matcher.rs            # 意图匹配引擎
│   └── types.rs              # 意图类型定义
├── pipeline/
│   ├── mod.rs
│   ├── ir.rs                 # 管道中间表示
│   ├── builder.rs            # 管道构建器
│   ├── optimizer.rs          # 管道优化器
│   └── executor.rs           # 管道执行器
├── tool/
│   ├── mod.rs
│   ├── definition.rs         # 工具定义 DSL
│   ├── registry.rs           # 工具注册表
│   └── sandbox.rs            # 沙箱策略
├── type_system/
│   ├── mod.rs
│   ├── types.rs              # 类型定义
│   ├── checker.rs            # 类型检查器
│   └── inference.rs          # 类型推导
└── runtime/
    ├── mod.rs
    ├── dataflow.rs           # 数据流执行引擎
    ├── scheduler.rs          # 调度器
    └── optimizer.rs          # 运行时优化
```

### 4.3 核心接口设计

```rust
// 统一的 DSL 入口
pub trait DSL {
    type AST;
    type Error;

    fn parse(&self, source: &str) -> Result<Self::AST, Self::Error>;
    fn validate(&self, ast: &Self::AST) -> Result<(), Self::Error>;
    fn compile(&self, ast: Self::AST) -> Result<ExecutablePlan, Self::Error>;
}

// Intent DSL
pub struct IntentDSL {
    matcher: IntentMatcher,
    validator: IntentValidator
}

impl DSL for IntentDSL {
    type AST = IntentDefinition;
    type Error = IntentError;

    fn parse(&self, source: &str) -> Result<Self::AST, Self::Error> {
        // 解析 Intent 定义
    }

    fn validate(&self, ast: &Self::AST) -> Result<(), Self::Error> {
        // 验证语义正确性
    }

    fn compile(&self, ast: Self::AST) -> Result<ExecutablePlan, Self::Error> {
        // 编译为可执行计划
    }
}

// Pipeline DSL
pub struct PipelineDSL {
    type_checker: TypeChecker,
    optimizer: PipelineOptimizer
}

impl DSL for PipelineDSL {
    type AST = PipelineGraph;
    type Error = PipelineError;

    // ...
}

// 统一执行接口
pub trait Executable {
    async fn execute(&self, context: ExecutionContext) -> Result<Value, ExecutionError>;
}

impl Executable for ExecutablePlan {
    async fn execute(&self, context: ExecutionContext) -> Result<Value, ExecutionError> {
        match self {
            ExecutablePlan::Intent(plan) => execute_intent(plan, context).await,
            ExecutablePlan::Pipeline(plan) => execute_pipeline(plan, context).await,
            ExecutablePlan::ToolChain(plan) => execute_tool_chain(plan, context).await,
        }
    }
}
```

## 五、可读性与可计算性平衡策略

### 5.1 可读性优化

**1. 自然语言接近的语法**

```rust
// 优先：类自然语言
Intent "统计 Python 文件行数" {
    匹配关键词: ["python", "行数", "统计"],
    匹配模式: r"统计.*python.*行数"
}

// 而非：过度技术化
Intent {
    id: 0x1234,
    kw: ["python", "lines", "count"],
    pat: [r"\d+"]
}
```

**2. 层次化组织**

```rust
// 优先：清晰的层次结构
Project Analysis {
    Phase("数据收集") {
        Step("查找文件") {
            command: "find . -name '*.rs'"
        }
    },

    Phase("数据处理") {
        Step("统计行数") {
            command: "wc -l"
        }
    }
}

// 而非：扁平化
steps = [find, wc, sort, head]
```

**3. 显式意图声明**

```rust
// 优先：明确意图
Intent FileCount {
    purpose: "统计特定类型文件数量",
    examples: [
        "统计 Python 文件数量",
        "count rust files"
    ]
}

// 而非：隐式推断
int_fc = Int { p: ["cnt", "num"], t: "file" }
```

### 5.2 可计算性优化

**1. 静态类型系统**

```rust
// 编译期类型检查
Pipeline TypeSafe {
    find: () -> FileList,           // 类型明确
    wc: (FileList) -> Integer,      // 类型兼容性检查
    format: (Integer) -> String     // 自动验证
}
// 编译器错误：类型不匹配会在编译期捕获
```

**2. 约束验证**

```rust
// 声明式约束
Tool ExecuteShell {
    parameters: {
        command: String {
            max_length: 1000,                // 长度约束
            pattern: r"^[a-zA-Z0-9\s\-\.]*$", // 格式约束
            not_contains: ["rm -rf /", "sudo"]  // 黑名单约束
        }
    }
}
// 编译器自动生成验证代码
```

**3. 可优化的中间表示**

```rust
// Pipeline IR 支持编译期优化
Pipeline {
    nodes: [A, B, C, D],
    edges: [A->B, B->C, B->D]  // B 的输出同时给 C 和 D
}
// 优化器自动识别：C 和 D 可以并行执行
```

### 5.3 权衡决策表

| 特性 | 可读性优先 | 可计算性优先 | 平衡方案 |
|------|-----------|-------------|---------|
| 变量命名 | 长描述性名称 | 短字母名称 | **驼峰命名 + 类型后缀** |
| 语法简洁性 | 冗长清晰 | 极度简洁 | **关键词 + 简洁参数** |
| 类型声明 | 隐式推导 | 显式声明 | **关键位置显式，其他推导** |
| 错误消息 | 详细解释 | 错误码 | **错误码 + 详细描述** |
| 控制流 | 自然语言 | 符号表达 | **混合：关键词 + 操作符** |

## 六、实施路径

### Phase 1: 基础设施 (Week 1-2)

1. **类型系统实现**
   - 基础类型定义
   - 类型检查器
   - 类型推导引擎

2. **解析器框架**
   - 词法分析器
   - 语法分析器
   - AST 构建器

### Phase 2: Intent DSL (Week 3-4)

1. **意图定义语言**
   - Intent 语法设计
   - 模板系统
   - 匹配引擎

2. **意图识别引擎**
   - 关键词匹配
   - 模式匹配
   - 置信度计算

### Phase 3: Pipeline DSL (Week 5-6)

1. **管道 IR 设计**
   - 图结构表示
   - 类型化节点
   - 边（数据流）定义

2. **管道优化器**
   - 节点合并
   - 并行化分析
   - 死代码消除

### Phase 4: Tool DSL (Week 7-8)

1. **工具定义语言**
   - 工具接口规范
   - 安全策略 DSL
   - 执行策略配置

2. **工具执行引擎**
   - 数据流执行
   - 并行调度
   - 错误处理

### Phase 5: 集成与优化 (Week 9-10)

1. **统一接口**
   - DSL 互操作
   - 上下文管理
   - 状态持久化

2. **性能优化**
   - 编译缓存
   - 运行时优化
   - 内存管理

## 七、示例场景

### 场景 1: 统计代码行数

**用户输入**：
```
"统计 Rust 代码总行数"
```

**Intent DSL 解析**：
```rust
Intent {
    name: "count_rust_lines",
    confidence: 0.9,
    entities: {
        file_type: "rust",
        operation: "count_lines"
    }
}
```

**Pipeline IR 生成**：
```rust
Pipeline {
    nodes: [
        Node<FindRustFiles> {
            command: "find . -name '*.rs' -type f",
            output: FileList
        },
        Node<CountLines> {
            command: "wc -l",
            input: FileList,
            output: Integer
        }
    ],
    edges: [
        Edge { from: "find", to: "wc", mode: Pipe }
    ]
}
```

**执行计划**：
```rust
ExecutionPlan {
    steps: [
        1. execute_shell("find . -name '*.rs' -type f"),
        2. pipe_to("wc -l"),
        3. parse_output(Integer),
        4. format_result("总行数: {}")
    ],
    estimated_time: 500ms,
    safety_level: Safe
}
```

### 场景 2: 错误分析

**用户输入**：
```
"分析最近的错误日志"
```

**复杂意图组合**：
```rust
CompositeIntent {
    primary: DiagnosticOps::ErrorAnalysis,
    secondary: [
        FileOps::FindLogs,
        DataOps::Filter,
        DataOps::Count
    ],

    // 条件执行
    conditionals: [
        If(HasLogFiles) Then FindLogs,
        If(LogsFound) Then GrepErrors,
        Else SuggestCreateLog
    ]
}
```

**动态管道构建**：
```rust
DynamicPipeline {
    // 运行时决定
    condition_check: has_log_files(),

    true_branch: Pipeline {
        nodes: [FindLogs, GrepErrors, CountErrors, SortByTime]
    },

    false_branch: Pipeline {
        nodes: [SuggestAlternatives, ShowHelp]
    }
}
```

## 八、关键创新点

1. **三层 DSL 架构**：Intent-Pipeline-Tool 分层设计
2. **类型化数据流**：编译期类型安全保证
3. **声明式安全**：规则即文档的沙箱系统
4. **工具感知规划**：集成工具可用性和安全性检查
5. **可优化 IR**：支持编译期和运行时优化
6. **可组合抽象**：DSL 元素可自由组合
7. **自然语言友好**：类自然语言的语法设计

## 九、总结

本 DSL 设计方案综合了：

1. **FCToken 框架的 DSL 理论**（六大原则）
2. **Python 版本的实践经验**（execute_shell + planner）
3. **Rust 的类型系统优势**（安全性 + 性能）
4. **现代编译技术**（静态分析 + 运行时优化）

最终实现一个：
- **可读的**：领域专家能理解
- **可计算的**：机器能高效执行
- **可扩展的**：易于添加新能力
- **可验证的**：编译期和运行时双重保证

的领域特定语言系统，为 RealConsole 的智能引擎提供坚实的基础。

---

**下一步行动**：
1. 完善类型系统设计细节
2. 实现 Intent DSL 解析器原型
3. 构建 Pipeline IR 基础结构
4. 集成到现有 Agent 系统
