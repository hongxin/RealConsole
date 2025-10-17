# RealConsole Phase 3 研发规划：Intent DSL 实现

## 📊 当前状态总结

### 版本信息
- **当前版本**: v0.1.1
- **代码量**: ~11,258 行 Rust 代码
- **测试状态**: 111/111 通过 (100%)
- **模块数**: 20+ 个核心模块

### 已完成功能 (v0.1.0 - v0.1.1)

| 功能模块 | 状态 | 说明 |
|---------|------|------|
| **基础架构** | ✅ | REPL, 配置系统, 命令注册 |
| **LLM 集成** | ✅ | Ollama, Deepseek, 流式输出 |
| **工具调用系统** | ✅ | Tool registry, Tool executor, Function calling |
| **记忆系统** | ✅ | 短期记忆 (ring buffer) + 长期记忆 (JSONL) |
| **执行日志** | ✅ | 命令追踪, 统计分析, 过滤查询 |
| **Shell 执行** | ✅ | 安全沙箱, 危险命令检测 |
| **内置工具** | ✅ | calculator, read_file, list_files, write_file |
| **类型系统** | ✅ | 基础类型, 复合类型, 约束类型, 类型检查 |
| **计算器增强** | ✅ | meval 集成, 支持完整数学表达式 |
| **配置化限制** | ✅ | max_tool_iterations, max_tools_per_round |

### DSL 系统当前状态

根据 `src/dsl/mod.rs`:

```rust
//! DSL 模块
//!
//! RealConsole 领域特定语言系统，包括：
//! - 类型系统 (type_system): 类型定义、类型检查、类型推导 ✅ 已完成
//! - 意图 DSL (intent): 意图识别与分析 - ⏳ 待实现
//! - 管道 IR (pipeline): 中间表示与数据流 - ⏳ 待实现
//! - 工具 DSL (tool): 工具定义与安全策略 - ⏳ 待实现
```

**已完成**: Phase 1 - 类型系统基础设施
- `src/dsl/type_system/types.rs` - 类型定义 (271 行)
- `src/dsl/type_system/checker.rs` - 类型检查器 (285 行)
- `src/dsl/type_system/inference.rs` - 类型推导 (278 行)

**待实现**: Phase 2-4 - DSL 上层应用
- Intent DSL - 意图表达语言
- Pipeline IR - 管道中间表示
- Tool DSL - 工具定义语言

## 🎯 Phase 3 目标：Intent DSL 实现

### 核心目标

实现 **Intent DSL（意图表达语言）**，让系统能够：
1. **理解用户意图** - 从自然语言输入识别用户的真实意图
2. **模板匹配** - 将意图映射到预定义的执行模板
3. **动态规划** - 根据意图生成可执行的步骤计划
4. **工具感知** - 考虑工具可用性和沙箱安全性

### 设计理念

基于 `docs/thinking/realconsole-dsl-design.md` 的设计方案，采用 **声明式 + 可组合** 的架构：

```rust
// Intent 定义示例（目标语法）
Intent FileOps::CountPythonLines {
    keywords: ["python", "py", "行数", "count", "lines"],
    patterns: [r"统计.*python.*行数", r"count.*\.py.*lines"],
    entities: {
        file_type: "python",
        operation: "count_lines"
    },
    confidence_threshold: 0.5
}

// Template 定义示例
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
            command: Shell("wc -l"),
            tool: "execute_shell"
        }
    ]
}
```

### 关键创新点

1. **类型安全的意图系统** - 利用 Rust 类型系统保证意图定义的正确性
2. **工具感知规划** - 集成现有的 tool registry 和 sandbox 系统
3. **可组合的模板** - 意图和模板可以嵌套和组合
4. **编译期验证** - 在编译期检查意图定义的完整性

## 📅 实施计划

### Week 1: 意图核心数据结构 (Day 1-7)

#### Day 1-2: 意图定义数据结构

**目标**: 实现 Intent 和 Entity 的核心数据结构

**文件**: `src/dsl/intent/types.rs` (新建, ~200 lines)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 意图类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub name: String,
    pub domain: IntentDomain,
    pub keywords: Vec<String>,
    pub patterns: Vec<String>,
    pub entities: HashMap<String, EntityType>,
    pub confidence_threshold: f64,
}

/// 意图领域（分类）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentDomain {
    FileOps,        // 文件操作
    DataOps,        // 数据处理
    DiagnosticOps,  // 诊断分析
    SystemOps,      // 系统管理
    Custom(String), // 自定义领域
}

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    FileType(String),      // 文件类型: "python", "rust"
    Operation(String),     // 操作: "count", "find", "analyze"
    Path(String),          // 路径
    Number(f64),           // 数字
    Date(String),          // 日期
    Custom(String, String),// 自定义实体
}

/// 意图识别结果
#[derive(Debug, Clone)]
pub struct IntentMatch {
    pub intent: Intent,
    pub confidence: f64,
    pub matched_keywords: Vec<String>,
    pub extracted_entities: HashMap<String, EntityType>,
}
```

**测试**: `tests/test_intent_types.rs` (新建, ~80 lines)

```rust
#[test]
fn test_intent_creation() {
    let intent = Intent {
        name: "count_python_lines".to_string(),
        domain: IntentDomain::FileOps,
        keywords: vec!["python".to_string(), "行数".to_string()],
        patterns: vec![r"统计.*python.*行数".to_string()],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    };

    assert_eq!(intent.name, "count_python_lines");
    assert_eq!(intent.confidence_threshold, 0.5);
}
```

#### Day 3-4: 意图匹配引擎

**目标**: 实现关键词匹配和正则模式匹配

**文件**: `src/dsl/intent/matcher.rs` (新建, ~300 lines)

```rust
use regex::Regex;
use crate::dsl::intent::types::*;

/// 意图匹配器
pub struct IntentMatcher {
    intents: Vec<Intent>,
    regex_cache: HashMap<String, Regex>,
}

impl IntentMatcher {
    pub fn new() -> Self {
        Self {
            intents: Vec::new(),
            regex_cache: HashMap::new(),
        }
    }

    /// 注册意图
    pub fn register(&mut self, intent: Intent) {
        // 预编译正则表达式
        for pattern in &intent.patterns {
            if !self.regex_cache.contains_key(pattern) {
                if let Ok(regex) = Regex::new(pattern) {
                    self.regex_cache.insert(pattern.clone(), regex);
                }
            }
        }
        self.intents.push(intent);
    }

    /// 匹配用户输入
    pub fn match_intent(&self, input: &str) -> Vec<IntentMatch> {
        let mut matches = Vec::new();

        for intent in &self.intents {
            let mut score = 0.0;
            let mut matched_keywords = Vec::new();

            // 1. 关键词匹配
            for keyword in &intent.keywords {
                if input.contains(keyword) {
                    score += 0.3;
                    matched_keywords.push(keyword.clone());
                }
            }

            // 2. 正则模式匹配
            for pattern in &intent.patterns {
                if let Some(regex) = self.regex_cache.get(pattern) {
                    if regex.is_match(input) {
                        score += 0.7;
                    }
                }
            }

            // 3. 计算置信度
            let confidence = score.min(1.0);

            if confidence >= intent.confidence_threshold {
                matches.push(IntentMatch {
                    intent: intent.clone(),
                    confidence,
                    matched_keywords,
                    extracted_entities: HashMap::new(), // TODO: 实体提取
                });
            }
        }

        // 按置信度排序
        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        matches
    }

    /// 获取最佳匹配
    pub fn best_match(&self, input: &str) -> Option<IntentMatch> {
        self.match_intent(input).into_iter().next()
    }
}
```

**测试**: `tests/test_intent_matcher.rs` (新建, ~120 lines)

```rust
#[test]
fn test_keyword_matching() {
    let mut matcher = IntentMatcher::new();

    let intent = Intent {
        name: "count_files".to_string(),
        domain: IntentDomain::FileOps,
        keywords: vec!["统计".to_string(), "文件".to_string()],
        patterns: vec![],
        entities: HashMap::new(),
        confidence_threshold: 0.3,
    };

    matcher.register(intent);

    let matches = matcher.match_intent("统计 Python 文件数量");
    assert!(!matches.is_empty());
    assert!(matches[0].confidence >= 0.3);
}

#[test]
fn test_pattern_matching() {
    let mut matcher = IntentMatcher::new();

    let intent = Intent {
        name: "count_lines".to_string(),
        domain: IntentDomain::FileOps,
        keywords: vec![],
        patterns: vec![r"统计.*行数".to_string()],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    };

    matcher.register(intent);

    let matches = matcher.match_intent("统计 Python 代码行数");
    assert!(!matches.is_empty());
    assert!(matches[0].confidence >= 0.7);
}
```

#### Day 5-7: 模板系统

**目标**: 实现 Template 和 Step 的数据结构及匹配逻辑

**文件**: `src/dsl/intent/template.rs` (新建, ~350 lines)

```rust
use crate::dsl::intent::types::*;
use serde::{Deserialize, Serialize};

/// 执行步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub number: usize,
    pub description: String,
    pub command: Option<String>,
    pub tool: Option<String>,
    pub validation: StepValidation,
    pub note: Option<String>,
}

/// 步骤验证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepValidation {
    None,
    SandboxCheck,
    ToolAvailable,
    Custom(String),
}

/// 意图模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub intent_name: String,
    pub min_confidence: f64,
    pub steps: Vec<Step>,
    pub preconditions: Vec<Condition>,
    pub postconditions: Vec<Condition>,
}

/// 条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    ToolAvailable(String),
    HasFiles(String),
    HasPermission(String),
    Custom(String),
}

/// 模板匹配器
pub struct TemplateMatcher {
    templates: Vec<Template>,
}

impl TemplateMatcher {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// 注册模板
    pub fn register(&mut self, template: Template) {
        self.templates.push(template);
    }

    /// 根据意图匹配模板
    pub fn match_template(&self, intent_match: &IntentMatch) -> Option<Template> {
        for template in &self.templates {
            if template.intent_name == intent_match.intent.name
                && intent_match.confidence >= template.min_confidence
            {
                return Some(template.clone());
            }
        }
        None
    }

    /// 检查前置条件
    pub fn check_preconditions(&self, template: &Template) -> Result<(), String> {
        for condition in &template.preconditions {
            match condition {
                Condition::ToolAvailable(tool) => {
                    // TODO: 检查工具是否可用
                    // 集成现有的 ToolRegistry
                }
                Condition::HasFiles(pattern) => {
                    // TODO: 检查文件是否存在
                }
                Condition::HasPermission(perm) => {
                    // TODO: 检查权限
                }
                Condition::Custom(desc) => {
                    // TODO: 自定义条件检查
                }
            }
        }
        Ok(())
    }
}

/// 执行计划
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub intent: IntentMatch,
    pub template: Template,
    pub steps: Vec<Step>,
    pub estimated_duration_ms: u64,
}

impl ExecutionPlan {
    pub fn from_template(intent: IntentMatch, template: Template) -> Self {
        let estimated_duration_ms = template.steps.len() as u64 * 500; // 粗略估计

        Self {
            intent,
            steps: template.steps.clone(),
            template,
            estimated_duration_ms,
        }
    }

    /// 生成人类可读的执行计划
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("执行计划: {}\n", self.template.name));
        output.push_str(&format!("意图: {} (置信度: {:.2})\n",
            self.intent.intent.name, self.intent.confidence));
        output.push_str(&format!("预计耗时: {}ms\n\n", self.estimated_duration_ms));

        output.push_str("步骤:\n");
        for step in &self.steps {
            output.push_str(&format!("  {}. {}\n", step.number, step.description));
            if let Some(cmd) = &step.command {
                output.push_str(&format!("     命令: {}\n", cmd));
            }
            if let Some(tool) = &step.tool {
                output.push_str(&format!("     工具: {}\n", tool));
            }
            if let Some(note) = &step.note {
                output.push_str(&format!("     备注: {}\n", note));
            }
        }

        output
    }
}
```

**测试**: `tests/test_intent_template.rs` (新建, ~150 lines)

```rust
#[test]
fn test_template_matching() {
    let mut matcher = TemplateMatcher::new();

    let template = Template {
        name: "CountPythonLines".to_string(),
        intent_name: "count_python_lines".to_string(),
        min_confidence: 0.7,
        steps: vec![
            Step {
                number: 1,
                description: "查找 Python 文件".to_string(),
                command: Some("find . -name '*.py'".to_string()),
                tool: Some("execute_shell".to_string()),
                validation: StepValidation::SandboxCheck,
                note: None,
            },
        ],
        preconditions: vec![],
        postconditions: vec![],
    };

    matcher.register(template.clone());

    let intent_match = IntentMatch {
        intent: Intent {
            name: "count_python_lines".to_string(),
            domain: IntentDomain::FileOps,
            keywords: vec![],
            patterns: vec![],
            entities: HashMap::new(),
            confidence_threshold: 0.5,
        },
        confidence: 0.8,
        matched_keywords: vec![],
        extracted_entities: HashMap::new(),
    };

    let matched = matcher.match_template(&intent_match);
    assert!(matched.is_some());
}

#[test]
fn test_execution_plan_generation() {
    let intent_match = IntentMatch {
        intent: Intent {
            name: "count_python_lines".to_string(),
            domain: IntentDomain::FileOps,
            keywords: vec![],
            patterns: vec![],
            entities: HashMap::new(),
            confidence_threshold: 0.5,
        },
        confidence: 0.9,
        matched_keywords: vec![],
        extracted_entities: HashMap::new(),
    };

    let template = Template {
        name: "CountPythonLines".to_string(),
        intent_name: "count_python_lines".to_string(),
        min_confidence: 0.7,
        steps: vec![
            Step {
                number: 1,
                description: "查找文件".to_string(),
                command: Some("find . -name '*.py'".to_string()),
                tool: Some("execute_shell".to_string()),
                validation: StepValidation::SandboxCheck,
                note: None,
            },
        ],
        preconditions: vec![],
        postconditions: vec![],
    };

    let plan = ExecutionPlan::from_template(intent_match, template);

    assert_eq!(plan.steps.len(), 1);
    assert!(plan.estimated_duration_ms > 0);

    let formatted = plan.format();
    assert!(formatted.contains("执行计划"));
    assert!(formatted.contains("步骤"));
}
```

### Week 2: 内置意图与模板库 (Day 8-14)

#### Day 8-10: 预定义意图库

**目标**: 实现常用的意图定义

**文件**: `src/dsl/intent/builtin.rs` (新建, ~400 lines)

```rust
use crate::dsl::intent::types::*;
use crate::dsl::intent::template::*;

/// 注册内置意图
pub fn register_builtin_intents(matcher: &mut IntentMatcher) {
    // 文件操作意图
    register_file_intents(matcher);

    // 诊断分析意图
    register_diagnostic_intents(matcher);

    // 数据处理意图
    register_data_intents(matcher);

    // 系统管理意图
    register_system_intents(matcher);
}

fn register_file_intents(matcher: &mut IntentMatcher) {
    // 1. 统计文件行数
    matcher.register(Intent {
        name: "count_file_lines".to_string(),
        domain: IntentDomain::FileOps,
        keywords: vec![
            "统计".to_string(),
            "行数".to_string(),
            "count".to_string(),
            "lines".to_string(),
        ],
        patterns: vec![
            r"统计.*行数".to_string(),
            r"count.*lines".to_string(),
            r"有多少行".to_string(),
        ],
        entities: HashMap::from([
            ("file_type".to_string(), EntityType::FileType("".to_string())),
            ("operation".to_string(), EntityType::Operation("count_lines".to_string())),
        ]),
        confidence_threshold: 0.5,
    });

    // 2. 查找文件
    matcher.register(Intent {
        name: "find_files".to_string(),
        domain: IntentDomain::FileOps,
        keywords: vec![
            "查找".to_string(),
            "搜索".to_string(),
            "find".to_string(),
            "search".to_string(),
        ],
        patterns: vec![
            r"查找.*文件".to_string(),
            r"find.*file".to_string(),
            r"搜索.*文件".to_string(),
        ],
        entities: HashMap::from([
            ("file_pattern".to_string(), EntityType::Custom("pattern".to_string(), "".to_string())),
            ("operation".to_string(), EntityType::Operation("find".to_string())),
        ]),
        confidence_threshold: 0.5,
    });

    // 3. 分析代码库
    matcher.register(Intent {
        name: "analyze_codebase".to_string(),
        domain: IntentDomain::FileOps,
        keywords: vec![
            "分析".to_string(),
            "代码".to_string(),
            "analyze".to_string(),
            "codebase".to_string(),
        ],
        patterns: vec![
            r"分析.*代码".to_string(),
            r"analyze.*code".to_string(),
        ],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    });
}

fn register_diagnostic_intents(matcher: &mut IntentMatcher) {
    // 1. 错误分析
    matcher.register(Intent {
        name: "analyze_errors".to_string(),
        domain: IntentDomain::DiagnosticOps,
        keywords: vec![
            "错误".to_string(),
            "error".to_string(),
            "分析".to_string(),
            "analyze".to_string(),
        ],
        patterns: vec![
            r"分析.*错误".to_string(),
            r"analyze.*error".to_string(),
            r"查看.*错误".to_string(),
        ],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    });

    // 2. 系统健康检查
    matcher.register(Intent {
        name: "health_check".to_string(),
        domain: IntentDomain::DiagnosticOps,
        keywords: vec![
            "健康".to_string(),
            "检查".to_string(),
            "状态".to_string(),
            "health".to_string(),
            "status".to_string(),
        ],
        patterns: vec![
            r"健康.*检查".to_string(),
            r"health.*check".to_string(),
            r"系统.*状态".to_string(),
        ],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    });
}

fn register_data_intents(matcher: &mut IntentMatcher) {
    // 1. 数据过滤
    matcher.register(Intent {
        name: "filter_data".to_string(),
        domain: IntentDomain::DataOps,
        keywords: vec![
            "过滤".to_string(),
            "筛选".to_string(),
            "filter".to_string(),
        ],
        patterns: vec![
            r"过滤.*数据".to_string(),
            r"filter.*data".to_string(),
        ],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    });

    // 2. 数据排序
    matcher.register(Intent {
        name: "sort_data".to_string(),
        domain: IntentDomain::DataOps,
        keywords: vec![
            "排序".to_string(),
            "sort".to_string(),
        ],
        patterns: vec![
            r"排序.*数据".to_string(),
            r"sort.*data".to_string(),
        ],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    });
}

fn register_system_intents(matcher: &mut IntentMatcher) {
    // 1. 清理缓存
    matcher.register(Intent {
        name: "clean_cache".to_string(),
        domain: IntentDomain::SystemOps,
        keywords: vec![
            "清理".to_string(),
            "缓存".to_string(),
            "clean".to_string(),
            "cache".to_string(),
        ],
        patterns: vec![
            r"清理.*缓存".to_string(),
            r"clean.*cache".to_string(),
        ],
        entities: HashMap::new(),
        confidence_threshold: 0.5,
    });
}

/// 注册内置模板
pub fn register_builtin_templates(matcher: &mut TemplateMatcher) {
    // 文件操作模板
    register_file_templates(matcher);

    // 诊断分析模板
    register_diagnostic_templates(matcher);
}

fn register_file_templates(matcher: &mut TemplateMatcher) {
    // 1. 统计文件行数模板
    matcher.register(Template {
        name: "CountFileLinesTemplate".to_string(),
        intent_name: "count_file_lines".to_string(),
        min_confidence: 0.5,
        steps: vec![
            Step {
                number: 1,
                description: "查找指定类型的文件".to_string(),
                command: Some("find . -name '*.rs' -type f".to_string()),
                tool: Some("execute_shell".to_string()),
                validation: StepValidation::SandboxCheck,
                note: Some("根据用户指定的文件类型调整".to_string()),
            },
            Step {
                number: 2,
                description: "统计总行数".to_string(),
                command: Some("wc -l".to_string()),
                tool: Some("execute_shell".to_string()),
                validation: StepValidation::SandboxCheck,
                note: Some("最后一行显示总行数".to_string()),
            },
        ],
        preconditions: vec![
            Condition::ToolAvailable("execute_shell".to_string()),
        ],
        postconditions: vec![],
    });

    // 2. 查找文件模板
    matcher.register(Template {
        name: "FindFilesTemplate".to_string(),
        intent_name: "find_files".to_string(),
        min_confidence: 0.5,
        steps: vec![
            Step {
                number: 1,
                description: "使用 find 命令搜索文件".to_string(),
                command: Some("find . -name '*.rs'".to_string()),
                tool: Some("execute_shell".to_string()),
                validation: StepValidation::SandboxCheck,
                note: Some("根据用户输入调整文件模式".to_string()),
            },
        ],
        preconditions: vec![
            Condition::ToolAvailable("execute_shell".to_string()),
        ],
        postconditions: vec![],
    });
}

fn register_diagnostic_templates(matcher: &mut TemplateMatcher) {
    // 1. 系统健康检查模板
    matcher.register(Template {
        name: "HealthCheckTemplate".to_string(),
        intent_name: "health_check".to_string(),
        min_confidence: 0.5,
        steps: vec![
            Step {
                number: 1,
                description: "检查 LLM 状态".to_string(),
                command: None,
                tool: Some("llm_status".to_string()),
                validation: StepValidation::None,
                note: None,
            },
            Step {
                number: 2,
                description: "检查工具可用性".to_string(),
                command: None,
                tool: Some("tool_status".to_string()),
                validation: StepValidation::None,
                note: None,
            },
            Step {
                number: 3,
                description: "检查记忆系统".to_string(),
                command: None,
                tool: Some("memory_status".to_string()),
                validation: StepValidation::None,
                note: None,
            },
        ],
        preconditions: vec![],
        postconditions: vec![],
    });
}
```

#### Day 11-12: 与 Agent 集成

**目标**: 将 Intent DSL 集成到现有的 Agent 系统

**文件**: `src/agent.rs` (修改, +80 lines)

```rust
use crate::dsl::intent::{IntentMatcher, TemplateMatcher, builtin};

pub struct Agent {
    // ... 现有字段

    // 新增：Intent DSL 组件
    pub intent_matcher: Arc<RwLock<IntentMatcher>>,
    pub template_matcher: Arc<RwLock<TemplateMatcher>>,
}

impl Agent {
    pub fn new(config: Config, registry: CommandRegistry) -> Self {
        // ... 现有初始化代码

        // 初始化 Intent DSL
        let mut intent_matcher = IntentMatcher::new();
        let mut template_matcher = TemplateMatcher::new();

        // 注册内置意图和模板
        builtin::register_builtin_intents(&mut intent_matcher);
        builtin::register_builtin_templates(&mut template_matcher);

        Self {
            // ... 现有字段
            intent_matcher: Arc::new(RwLock::new(intent_matcher)),
            template_matcher: Arc::new(RwLock::new(template_matcher)),
        }
    }

    /// 处理自由文本（支持意图识别）
    fn handle_text(&self, text: &str) -> String {
        // 1. 尝试意图识别
        let intent_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let matcher = self.intent_matcher.read().await;
                matcher.best_match(text)
            })
        });

        // 2. 如果识别到意图，尝试生成执行计划
        if let Some(intent_match) = intent_result {
            if intent_match.confidence > 0.7 {
                // 高置信度：直接执行
                return self.execute_intent_plan(intent_match);
            } else if intent_match.confidence > 0.5 {
                // 中等置信度：询问用户确认
                return format!(
                    "检测到意图: {} (置信度: {:.2})\n是否执行? (yes/no)",
                    intent_match.intent.name,
                    intent_match.confidence
                );
            }
        }

        // 3. 无法识别意图，回退到 LLM
        self.handle_text_with_llm(text)
    }

    /// 执行意图计划
    fn execute_intent_plan(&self, intent_match: IntentMatch) -> String {
        // 1. 匹配模板
        let template = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let matcher = self.template_matcher.read().await;
                matcher.match_template(&intent_match)
            })
        });

        let template = match template {
            Some(t) => t,
            None => {
                return format!(
                    "意图识别成功: {}\n但未找到对应的执行模板",
                    intent_match.intent.name
                );
            }
        };

        // 2. 生成执行计划
        let plan = ExecutionPlan::from_template(intent_match, template);

        // 3. 显示计划
        let mut output = plan.format();
        output.push_str("\n正在执行...\n\n");

        // 4. 执行步骤
        for step in &plan.steps {
            output.push_str(&format!("执行步骤 {}: {}\n", step.number, step.description));

            match self.execute_step(step) {
                Ok(result) => {
                    output.push_str(&format!("结果: {}\n\n", result));
                }
                Err(e) => {
                    output.push_str(&format!("错误: {}\n\n", e));
                    break;
                }
            }
        }

        output
    }

    /// 执行单个步骤
    fn execute_step(&self, step: &Step) -> Result<String, String> {
        // 检查验证
        match step.validation {
            StepValidation::SandboxCheck => {
                // TODO: 集成 shell_executor 的沙箱检查
            }
            StepValidation::ToolAvailable => {
                // TODO: 检查工具是否可用
            }
            _ => {}
        }

        // 执行命令或工具
        if let Some(cmd) = &step.command {
            // 使用 execute_shell
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    crate::shell_executor::execute_shell(cmd).await
                })
            })
        } else if let Some(tool) = &step.tool {
            // 使用 tool executor
            Err("工具执行暂未实现".to_string())
        } else {
            Err("步骤既无命令也无工具".to_string())
        }
    }
}
```

#### Day 13-14: 端到端测试

**目标**: 完整的意图识别 → 计划生成 → 执行流程测试

**文件**: `tests/test_intent_e2e.rs` (新建, ~200 lines)

```rust
use realconsole::agent::Agent;
use realconsole::config::Config;
use realconsole::command::CommandRegistry;

#[test]
fn test_intent_recognition_e2e() {
    // 1. 创建 Agent
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 2. 测试意图识别
    let inputs = vec![
        "统计 Rust 代码行数",
        "查找所有 Python 文件",
        "分析错误日志",
        "检查系统状态",
    ];

    for input in inputs {
        println!("测试输入: {}", input);
        let result = agent.handle(input);
        println!("结果:\n{}\n", result);

        // 验证结果包含预期内容
        assert!(!result.contains("LLM 调用失败"));
        assert!(result.len() > 0);
    }
}

#[test]
fn test_count_python_lines_intent() {
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    let result = agent.handle("统计 Python 文件总行数");

    // 应该识别为 count_file_lines 意图
    assert!(result.contains("执行计划") || result.contains("步骤"));
}

#[tokio::test]
async fn test_intent_with_low_confidence() {
    let config = Config::default();
    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 模糊的输入，置信度应该较低
    let result = agent.handle("做一些事情");

    // 应该回退到 LLM 或询问用户
    assert!(result.len() > 0);
}
```

### Week 3: 优化与文档 (Day 15-21)

#### Day 15-17: 性能优化

**优化点**:
1. 正则表达式缓存（已在 matcher 中实现）
2. 意图匹配并行化
3. 模板预编译
4. 结果缓存

**文件**: `src/dsl/intent/optimizer.rs` (新建, ~150 lines)

```rust
use std::collections::HashMap;
use lru::LruCache;

/// 意图识别缓存
pub struct IntentCache {
    cache: LruCache<String, Vec<IntentMatch>>,
    max_size: usize,
}

impl IntentCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: LruCache::new(max_size),
            max_size,
        }
    }

    pub fn get(&mut self, input: &str) -> Option<&Vec<IntentMatch>> {
        self.cache.get(input)
    }

    pub fn insert(&mut self, input: String, matches: Vec<IntentMatch>) {
        self.cache.put(input, matches);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}
```

#### Day 18-19: 文档编写

**创建文档**:
1. `docs/features/INTENT_DSL.md` - Intent DSL 功能文档
2. `docs/guides/INTENT_DSL_GUIDE.md` - 使用指南
3. `CHANGELOG.md` - 更新版本记录

**文件**: `docs/features/INTENT_DSL.md` (新建, ~800 lines)

```markdown
# Intent DSL - 意图表达语言

## 概述

Intent DSL 是 RealConsole 的核心组件之一，负责将自然语言输入转换为可执行的操作计划。

## 核心概念

### Intent (意图)

意图表示用户想要完成的任务。每个意图包含：
- **名称**: 意图的唯一标识
- **领域**: 意图所属的领域（FileOps, DataOps, etc.）
- **关键词**: 用于匹配的关键词列表
- **模式**: 正则表达式模式
- **实体**: 需要提取的实体类型
- **置信度阈值**: 最低匹配置信度

### Template (模板)

模板定义了如何执行特定意图。每个模板包含：
- **步骤**: 执行步骤列表
- **前置条件**: 执行前需要满足的条件
- **后置条件**: 执行后的预期状态

### ExecutionPlan (执行计划)

执行计划是意图和模板的组合，包含：
- **意图匹配结果**: 识别的意图和置信度
- **执行步骤**: 具体的执行步骤
- **预计耗时**: 估计的执行时间

## 使用示例

### 示例 1: 统计代码行数

**输入**: "统计 Rust 代码行数"

**识别结果**:
```
意图: count_file_lines
置信度: 0.9
匹配关键词: ["统计", "行数"]
```

**执行计划**:
```
执行计划: CountFileLinesTemplate
步骤:
  1. 查找指定类型的文件
     命令: find . -name '*.rs' -type f
     工具: execute_shell
  2. 统计总行数
     命令: wc -l
     工具: execute_shell
```

### 示例 2: 系统健康检查

**输入**: "检查系统状态"

**识别结果**:
```
意图: health_check
置信度: 0.85
匹配关键词: ["检查", "状态"]
```

**执行计划**:
```
执行计划: HealthCheckTemplate
步骤:
  1. 检查 LLM 状态
     工具: llm_status
  2. 检查工具可用性
     工具: tool_status
  3. 检查记忆系统
     工具: memory_status
```

## API 参考

### IntentMatcher

```rust
pub struct IntentMatcher {
    intents: Vec<Intent>,
    regex_cache: HashMap<String, Regex>,
}

impl IntentMatcher {
    pub fn new() -> Self;
    pub fn register(&mut self, intent: Intent);
    pub fn match_intent(&self, input: &str) -> Vec<IntentMatch>;
    pub fn best_match(&self, input: &str) -> Option<IntentMatch>;
}
```

### TemplateMatcher

```rust
pub struct TemplateMatcher {
    templates: Vec<Template>,
}

impl TemplateMatcher {
    pub fn new() -> Self;
    pub fn register(&mut self, template: Template);
    pub fn match_template(&self, intent_match: &IntentMatch) -> Option<Template>;
    pub fn check_preconditions(&self, template: &Template) -> Result<(), String>;
}
```

## 扩展指南

### 添加自定义意图

```rust
let mut matcher = IntentMatcher::new();

matcher.register(Intent {
    name: "my_custom_intent".to_string(),
    domain: IntentDomain::Custom("MyDomain".to_string()),
    keywords: vec!["custom".to_string(), "keyword".to_string()],
    patterns: vec![r"my.*pattern".to_string()],
    entities: HashMap::new(),
    confidence_threshold: 0.5,
});
```

### 添加自定义模板

```rust
let mut matcher = TemplateMatcher::new();

matcher.register(Template {
    name: "MyCustomTemplate".to_string(),
    intent_name: "my_custom_intent".to_string(),
    min_confidence: 0.7,
    steps: vec![
        Step {
            number: 1,
            description: "执行自定义操作".to_string(),
            command: Some("my_command".to_string()),
            tool: Some("execute_shell".to_string()),
            validation: StepValidation::SandboxCheck,
            note: None,
        },
    ],
    preconditions: vec![],
    postconditions: vec![],
});
```
```

#### Day 20-21: 集成测试与Bug修复

**任务**:
1. 运行完整的测试套件
2. 修复发现的问题
3. 性能测试和优化
4. 准备 v0.2.0 发布

## 📈 成功指标

### 功能完成度

- ✅ Intent 数据结构定义
- ✅ IntentMatcher 实现（关键词 + 正则）
- ✅ Template 和 Step 数据结构
- ✅ TemplateMatcher 实现
- ✅ ExecutionPlan 生成
- ✅ 内置意图库（10+ 意图）
- ✅ 内置模板库（5+ 模板）
- ✅ 与 Agent 集成
- ✅ 端到端测试
- ✅ 性能优化
- ✅ 文档完善

### 测试覆盖

- **目标**: 80% 代码覆盖率
- **单元测试**: 每个公共函数有测试
- **集成测试**: 端到端流程测试
- **性能测试**: 意图识别延迟 < 50ms

### 性能指标

| 指标 | 目标 | 实测 |
|------|------|------|
| 意图识别延迟 | < 50ms | - |
| 模板匹配延迟 | < 10ms | - |
| 内存占用增加 | < 5MB | - |
| 缓存命中率 | > 70% | - |

### 用户体验

- **识别准确率**: > 85% （在常见场景下）
- **错误提示**: 清晰友好
- **文档完整性**: 90%+

## 🔄 与现有系统的集成点

### 1. Agent 系统

```rust
Agent {
    // 现有组件
    llm_manager: Arc<RwLock<LlmManager>>,
    tool_executor: Arc<ToolExecutor>,
    memory: Arc<RwLock<Memory>>,

    // 新增组件
    intent_matcher: Arc<RwLock<IntentMatcher>>,     // ← 新增
    template_matcher: Arc<RwLock<TemplateMatcher>>, // ← 新增
}
```

### 2. Tool System

Intent 执行计划将调用现有的工具系统：
- `execute_shell` - Shell 命令执行
- `read_file` - 文件读取
- `list_files` - 文件列表
- `calculator` - 计算器工具

### 3. LLM System

对于低置信度的意图或无法识别的输入，回退到 LLM：
```rust
if intent_match.confidence < 0.5 {
    // 回退到 LLM
    return self.handle_text_with_llm(text);
}
```

### 4. Memory System

Intent 执行过程可以访问记忆系统：
```rust
Condition::Custom("has_recent_errors") => {
    let memory = self.memory.read().await;
    memory.search("error").len() > 0
}
```

## 🎯 后续阶段预览

### Phase 4: Pipeline IR (Week 4-6)

在 Intent DSL 基础上，实现管道中间表示：
- **目标**: 将执行步骤转换为数据流图
- **优化**: 节点合并、并行化、死代码消除
- **类型检查**: 编译期类型安全验证

### Phase 5: Tool DSL (Week 7-8)

增强工具定义语言：
- **目标**: 声明式工具定义和安全策略
- **沙箱增强**: 细粒度的安全控制
- **工具组合**: 工具链式调用

## 📊 项目统计预测

| 指标 | v0.1.1 (当前) | v0.2.0 (目标) | 增长 |
|------|--------------|---------------|------|
| 总代码量 | ~11,258 行 | ~13,500 行 | +20% |
| 模块数 | 20 个 | 25 个 | +5 |
| 测试数 | 111 个 | 140+ 个 | +25% |
| 文档页数 | ~30 页 | ~50 页 | +67% |

## 🔧 开发环境要求

- **Rust**: 1.70+
- **依赖**:
  - `regex = "1.10"` - 正则表达式
  - `lru = "0.12"` - LRU 缓存
  - `serde = { version = "1.0", features = ["derive"] }`
  - `serde_json = "1.0"`
  - `tokio = { version = "1.0", features = ["full"] }`

## 📝 开发规范

### 代码风格

1. **命名规范**
   - Intent: 使用下划线命名 `count_file_lines`
   - Template: 使用驼峰命名 `CountFileLinesTemplate`
   - 结构体: 使用驼峰命名 `IntentMatcher`

2. **文档注释**
   - 每个公共函数必须有文档注释
   - 包含参数说明和示例

3. **错误处理**
   - 使用 `Result<T, E>` 而非 `panic!`
   - 提供清晰的错误信息

### 测试要求

1. **单元测试**
   - 每个公共函数必须有测试
   - 测试边界情况和错误场景

2. **集成测试**
   - 端到端流程测试
   - 真实场景模拟

3. **性能测试**
   - 使用 `criterion` 进行性能基准测试
   - 确保性能回归不超过 5%

### 提交规范

```bash
# 格式: <type>(<scope>): <subject>

# 示例
feat(intent): add intent matcher implementation
fix(intent): fix regex pattern matching bug
test(intent): add end-to-end tests for intent system
docs(intent): update Intent DSL documentation
perf(intent): optimize intent matching performance
```

## 🚀 下一步行动

### 立即开始

1. **Day 1** - 创建 `src/dsl/intent/types.rs`
2. **Day 1** - 创建 `tests/test_intent_types.rs`
3. **Day 2** - 实现 Intent 和 Entity 数据结构
4. **Day 2** - 运行测试: `cargo test test_intent_types`

### 本周目标

完成 Week 1 的所有任务：
- Intent 核心数据结构
- IntentMatcher 实现
- TemplateMatcher 实现
- 基础测试覆盖

---

**规划日期**: 2025-10-14
**目标版本**: v0.2.0
**预计完成**: 2025-11-04 (3 周)
**状态**: 📋 计划中

让我们开始 Phase 3 的开发吧！🎯
