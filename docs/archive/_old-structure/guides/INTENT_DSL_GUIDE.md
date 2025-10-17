# Intent DSL 使用指南

Intent DSL (Intent Domain Specific Language) 是 RealConsole 的自然语言理解核心，能够将用户的自然语言输入转换为可执行的 Shell 命令。

## 📖 目录

- [核心概念](#核心概念)
- [快速开始](#快速开始)
- [Intent 定义](#intent-定义)
- [Entity Extraction (实体提取)](#entity-extraction-实体提取)
- [Template 模板系统](#template-模板系统)
- [IntentMatcher 匹配引擎](#intentmatcher-匹配引擎)
- [完整示例](#完整示例)
- [最佳实践](#最佳实践)

---

## 核心概念

### Intent (意图)

表示用户想要完成的任务。每个 Intent 包含：

- **名称** (name): 唯一标识符
- **领域** (domain): 所属领域（FileOps, DiagnosticOps 等）
- **关键词** (keywords): 用于关键词匹配
- **模式** (patterns): 正则表达式模式
- **实体** (entities): 需要提取的参数
- **置信度阈值** (confidence_threshold): 匹配的最低置信度

### Template (模板)

定义如何执行特定意图。模板使用 `{variable}` 语法进行变量替换。

### EntityType (实体类型)

支持以下实体类型：
- `FileType`: 文件类型（如 py, rs, js）
- `Path`: 文件路径
- `Number`: 数值（大小、数量、时间等）
- `Date`: 日期时间
- `Operation`: 操作类型（count, find, check 等）

### ExecutionPlan (执行计划)

Intent 和 Template 的组合，包含最终要执行的命令。

---

## 快速开始

### 1. 导入依赖

```rust
use realconsole::dsl::intent::{
    BuiltinIntents, EntityType, Intent, IntentDomain,
    IntentMatcher, Template, TemplateEngine
};
```

### 2. 使用内置意图

最简单的方式是使用内置的 10 个意图：

```rust
use realconsole::dsl::intent::BuiltinIntents;

// 创建内置意图系统
let builtin = BuiltinIntents::new();

// 获取 IntentMatcher 和 TemplateEngine
let matcher = builtin.create_matcher();
let engine = builtin.create_engine();

// 匹配用户输入
let user_input = "统计当前目录下有多少个 Python 文件";
if let Some(intent_match) = matcher.best_match(user_input) {
    // 生成执行计划
    let plan = engine.generate_from_intent(&intent_match)?;
    println!("执行命令: {}", plan.command);
    // 输出: find . -name '*.py' -type f | wc -l
}
```

### 3. 内置意图列表

| Intent 名称 | 功能 | 示例输入 |
|------------|------|---------|
| `count_python_lines` | 统计 Python 代码行数 | "统计 Python 代码行数" |
| `count_files` | 统计文件数量 | "统计当前目录下有多少个 py 文件" |
| `find_large_files` | 查找大文件 | "查找大于 100MB 的文件" |
| `find_recent_files` | 查找最近修改的文件 | "查找最近 60 分钟修改的文件" |
| `check_disk_usage` | 检查磁盘使用 | "检查当前目录的磁盘使用情况" |
| `list_running_processes` | 列出运行进程 | "显示 CPU 占用最高的 10 个进程" |
| `show_environment` | 显示环境变量 | "显示环境变量 PATH" |
| `count_code_lines` | 统计代码行数 | "统计项目代码总行数" |
| `archive_logs` | 归档日志文件 | "打包最近 7 天的日志" |
| `monitor_resources` | 监控系统资源 | "每 2 秒显示系统资源使用情况" |

---

## Intent 定义

### 基础 Intent

```rust
use realconsole::dsl::intent::{Intent, IntentDomain};

let intent = Intent::new(
    "count_files",                           // 名称
    IntentDomain::FileOps,                   // 领域
    vec![
        "统计".to_string(),
        "文件".to_string(),
        "数量".to_string()
    ],                                       // 关键词
    vec![r"(?i)统计.*文件.*(数量|个数)".to_string()],  // 正则模式
    0.5,                                     // 置信度阈值
);
```

### 带实体的 Intent

使用 `.with_entity()` 方法添加实体定义：

```rust
let intent = Intent::new(
    "count_files",
    IntentDomain::FileOps,
    vec!["统计".to_string(), "文件".to_string()],
    vec![r"(?i)统计.*文件".to_string()],
    0.5,
)
.with_entity("path", EntityType::Path(".".to_string()))      // 路径实体（默认值: "."）
.with_entity("ext", EntityType::FileType("*".to_string()));  // 文件类型实体（默认值: "*"）
```

### IntentDomain (意图领域)

```rust
pub enum IntentDomain {
    FileOps,         // 文件操作
    DiagnosticOps,   // 诊断操作
    DataProcessing,  // 数据处理
    SystemOps,       // 系统操作
    General,         // 通用操作
}
```

---

## Entity Extraction (实体提取)

实体提取是 Phase 3 Week 3 的核心功能，能够自动从用户输入中提取结构化信息。

### EntityType 定义

```rust
pub enum EntityType {
    FileType(String),    // 文件类型: "py", "rs", "js" 等
    Path(String),        // 路径: "./src", "/tmp", "." 等
    Number(f64),         // 数值: 文件大小、数量、时间等
    Date(String),        // 日期: "today", "2025-10-15" 等
    Operation(String),   // 操作: "count", "find", "check" 等
}
```

### 自动实体提取

Intent DSL 系统会自动提取实体，无需手动解析：

```rust
let matcher = builtin.create_matcher();

// 用户输入包含路径和文件类型
let matches = matcher.match_intent("统计 ./src 目录下有多少个 Python 文件");

if let Some(best_match) = matches.first() {
    // 自动提取的实体
    println!("{:?}", best_match.extracted_entities);
    // 输出: {"path": Path("./src"), "ext": FileType("py")}
}
```

### 支持的提取模式

#### 1. FileType (文件类型)

**支持的类型**:
- Python: `python`, `py`
- Rust: `rust`, `rs`
- JavaScript: `javascript`, `js`
- TypeScript: `typescript`, `ts`
- Go: `go`
- Java: `java`
- C++: `cpp`, `c++`
- C: `c`
- Shell: `shell`, `sh`
- YAML: `yaml`, `yml`
- JSON: `json`
- XML: `xml`
- HTML: `html`
- CSS: `css`
- Markdown: `md`, `markdown`
- Text: `txt`
- Log: `log`

**示例**:
```rust
// 输入: "统计 Python 文件数量"
// 提取: FileType("py")

// 输入: "查找 Rust 源代码"
// 提取: FileType("rs")
```

#### 2. Path (路径)

**支持的格式**:
- 相对路径: `./path`, `../path`
- 绝对路径: `/path/to/dir`
- 当前目录: `.`
- 智能识别: "当前目录" → `.`

**示例**:
```rust
// 输入: "统计 ./src 目录下的文件"
// 提取: Path("./src")

// 输入: "检查 /var/log 的磁盘使用"
// 提取: Path("/var/log")

// 输入: "统计当前目录下的文件"
// 提取: Path(".")  // 智能推断
```

#### 3. Number (数值)

**支持的格式**:
- 整数: `100`, `500`
- 小数: `1.5`, `3.14`
- 带单位的自动识别

**示例**:
```rust
// 输入: "查找大于 500 MB 的文件"
// 提取: Number(500.0)

// 输入: "查找最近 30 分钟修改的文件"
// 提取: Number(30.0)

// 输入: "显示前 10 个进程"
// 提取: Number(10.0)
```

#### 4. Date (日期)

**支持的格式**:
- 相对时间: `今天`, `昨天`, `最近`
- ISO 格式: `2025-10-15`
- 描述性: `上周`, `本月`

**示例**:
```rust
// 输入: "查找今天修改的文件"
// 提取: Date("today")

// 输入: "统计 2025-10-15 的日志"
// 提取: Date("2025-10-15")
```

#### 5. Operation (操作)

**支持的操作**:
- 统计: `count`, `统计`
- 查找: `find`, `查找`, `搜索`
- 检查: `check`, `检查`
- 列出: `list`, `列出`, `显示`
- 删除: `delete`, `删除`

### 实体默认值 (Smart Fallback)

当用户未提供某些实体时，系统会使用智能默认值：

```rust
// Intent 定义时指定默认值
let intent = Intent::new(...)
    .with_entity("path", EntityType::Path(".".to_string()))      // 默认当前目录
    .with_entity("ext", EntityType::FileType("*".to_string()))   // 默认所有类型
    .with_entity("limit", EntityType::Number(10.0));             // 默认显示 10 个

// 用户输入: "统计文件数量"（未指定路径和类型）
// 提取结果: {"path": Path("."), "ext": FileType("*")}
// 使用默认值填充缺失的实体
```

### EntityExtractor 直接使用

如果需要手动提取实体：

```rust
use realconsole::dsl::intent::{EntityExtractor, EntityType};
use std::collections::HashMap;

let extractor = EntityExtractor::new();

// 1. 提取文件类型
let file_type = extractor.extract_file_type("查找 Python 文件");
// 返回: Some(EntityType::FileType("py"))

// 2. 提取路径
let path = extractor.extract_path("检查 ./src 目录");
// 返回: Some(EntityType::Path("./src"))

// 3. 提取数值
let number = extractor.extract_number("大于 500 MB");
// 返回: Some(EntityType::Number(500.0))

// 4. 批量提取
let mut expected = HashMap::new();
expected.insert("path".to_string(), EntityType::Path(".".to_string()));
expected.insert("ext".to_string(), EntityType::FileType("*".to_string()));

let extracted = extractor.extract("统计 ./src 目录下的 Python 文件", &expected);
// 返回: {"path": Path("./src"), "ext": FileType("py")}
```

---

## Template 模板系统

### 创建模板

```rust
use realconsole::dsl::intent::Template;

let template = Template::new(
    "count_python_lines",                         // 模板名称（通常与 Intent 名称一致）
    "find {path} -name '*.py' -type f | xargs wc -l | tail -1",  // 命令模板
    vec!["path".to_string()],                     // 需要的变量
);
```

### 变量替换

模板使用 `{variable}` 语法，变量会从提取的实体中获取：

```rust
// 模板: "find {path} -name '*.{ext}' -type f | wc -l"
// 实体: {"path": Path("./src"), "ext": FileType("py")}
// 结果: "find ./src -name '*.py' -type f | wc -l"
```

### TemplateEngine

```rust
use realconsole::dsl::intent::TemplateEngine;

let mut engine = TemplateEngine::new();

// 注册模板
engine.register(Template::new(
    "count_files",
    "find {path} -name '*.{ext}' -type f | wc -l",
    vec!["path".to_string(), "ext".to_string()],
));

// 从 IntentMatch 生成执行计划
let plan = engine.generate_from_intent(&intent_match)?;
println!("命令: {}", plan.command);
```

---

## IntentMatcher 匹配引擎

### 创建 IntentMatcher

```rust
use realconsole::dsl::intent::IntentMatcher;

let mut matcher = IntentMatcher::new();

// 注册 Intent
matcher.register(intent1);
matcher.register(intent2);
```

### 匹配用户输入

```rust
// 获取所有匹配结果
let matches = matcher.match_intent("统计当前目录下的 Python 文件");

// 获取最佳匹配
if let Some(best_match) = matcher.best_match("统计 Python 文件") {
    println!("意图: {}", best_match.intent.name);
    println!("置信度: {:.2}", best_match.confidence);
    println!("实体: {:?}", best_match.extracted_entities);
}
```

### 匹配算法

置信度计算基于：
1. **关键词匹配** (40%): 用户输入包含的关键词数量
2. **正则模式匹配** (60%): 是否匹配正则表达式模式

```rust
// 伪代码
confidence = (matched_keywords / total_keywords) * 0.4
           + (pattern_match ? 1.0 : 0.0) * 0.6
```

### IntentMatch 结构

```rust
pub struct IntentMatch {
    pub intent: Intent,                                 // 匹配的意图
    pub confidence: f64,                                // 置信度 (0.0 ~ 1.0)
    pub matched_keywords: Vec<String>,                  // 匹配的关键词
    pub extracted_entities: HashMap<String, EntityType>, // 提取的实体
}
```

---

## 完整示例

### 示例 1: 统计文件数量

```rust
use realconsole::dsl::intent::{
    BuiltinIntents, EntityType, Intent, IntentDomain
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建内置意图系统
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();
    let engine = builtin.create_engine();

    // 2. 用户输入
    let user_input = "统计 ./src 目录下有多少个 Python 文件";

    // 3. 匹配意图
    if let Some(best_match) = matcher.best_match(user_input) {
        println!("✅ 匹配意图: {}", best_match.intent.name);
        println!("   置信度: {:.2}", best_match.confidence);

        // 4. 查看提取的实体
        println!("   提取实体:");
        for (key, value) in &best_match.extracted_entities {
            println!("     - {}: {:?}", key, value);
        }
        // 输出:
        //   - path: Path("./src")
        //   - ext: FileType("py")

        // 5. 生成执行计划
        let plan = engine.generate_from_intent(&best_match)?;
        println!("📝 生成命令: {}", plan.command);
        // 输出: find ./src -name '*.py' -type f | wc -l

        // 6. 执行命令（可选）
        // let output = std::process::Command::new("sh")
        //     .arg("-c")
        //     .arg(&plan.command)
        //     .output()?;
        // println!("📊 结果: {}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}
```

### 示例 2: 自定义 Intent

```rust
use realconsole::dsl::intent::{
    EntityType, Intent, IntentDomain, IntentMatcher,
    Template, TemplateEngine
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 定义自定义 Intent
    let my_intent = Intent::new(
        "find_config_files",
        IntentDomain::FileOps,
        vec!["查找".to_string(), "配置".to_string(), "文件".to_string()],
        vec![r"(?i)查找.*配置.*文件".to_string()],
        0.5,
    )
    .with_entity("path", EntityType::Path(".".to_string()));

    // 2. 创建 IntentMatcher 并注册
    let mut matcher = IntentMatcher::new();
    matcher.register(my_intent);

    // 3. 创建 Template
    let my_template = Template::new(
        "find_config_files",
        "find {path} -type f \\( -name '*.yaml' -o -name '*.yml' -o -name '*.toml' -o -name '*.json' \\)",
        vec!["path".to_string()],
    );

    // 4. 创建 TemplateEngine 并注册
    let mut engine = TemplateEngine::new();
    engine.register(my_template);

    // 5. 用户输入
    let user_input = "查找 ./config 目录下的配置文件";

    // 6. 匹配和生成
    if let Some(best_match) = matcher.best_match(user_input) {
        let plan = engine.generate_from_intent(&best_match)?;
        println!("命令: {}", plan.command);
        // 输出: find ./config -type f \( -name '*.yaml' -o -name '*.yml' -o -name '*.toml' -o -name '*.json' \)
    }

    Ok(())
}
```

### 示例 3: 查找大文件

```rust
use realconsole::dsl::intent::BuiltinIntents;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();
    let engine = builtin.create_engine();

    // 用户输入包含路径和大小
    let user_input = "查找 /var/log 目录下大于 500 MB 的文件";

    if let Some(best_match) = matcher.best_match(user_input) {
        println!("意图: {}", best_match.intent.name);
        // 输出: find_large_files

        // 提取的实体
        for (key, value) in &best_match.extracted_entities {
            println!("{}: {:?}", key, value);
        }
        // 输出:
        //   path: Path("/var/log")
        //   size: Number(500.0)

        // 生成命令
        let plan = engine.generate_from_intent(&best_match)?;
        println!("命令: {}", plan.command);
        // 输出: find /var/log -type f -size +500M
    }

    Ok(())
}
```

### 示例 4: 查找最近修改的文件

```rust
use realconsole::dsl::intent::BuiltinIntents;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();
    let engine = builtin.create_engine();

    let user_input = "查找 . 目录下最近 30 分钟修改的文件";

    if let Some(best_match) = matcher.best_match(user_input) {
        println!("提取实体: {:?}", best_match.extracted_entities);
        // 输出: {"path": Path("."), "minutes": Number(30.0)}

        let plan = engine.generate_from_intent(&best_match)?;
        println!("命令: {}", plan.command);
        // 输出: find . -type f -mmin -30
    }

    Ok(())
}
```

---

## 最佳实践

### 1. Intent 设计原则

**关键词选择**:
- 使用高频核心词（如"统计"、"查找"、"文件"）
- 避免过于通用的词（如"的"、"是"）
- 中文和英文关键词混合使用

**正则模式编写**:
- 使用 `(?i)` 忽略大小写
- 使用 `.*` 匹配中间的任意内容
- 使用 `(选项1|选项2)` 匹配多个可能的词

示例：
```rust
vec![
    r"(?i)统计.*文件.*(数量|个数)".to_string(),  // 匹配: "统计文件数量" 或 "统计文件个数"
    r"(?i)查找.*大于.*文件".to_string(),        // 匹配: "查找大于 100MB 的文件"
]
```

### 2. 实体设计原则

**提供合理的默认值**:
```rust
.with_entity("path", EntityType::Path(".".to_string()))      // 默认当前目录
.with_entity("limit", EntityType::Number(10.0))              // 默认显示 10 个
.with_entity("ext", EntityType::FileType("*".to_string()))   // 默认所有类型
```

**实体命名规范**:
- 使用清晰的名称: `path`, `size`, `limit`, `ext`
- 避免缩写: 使用 `extension` 而非 `ext` (如果更清晰)
- 保持一致性: 所有 Intent 中相同含义的实体使用相同名称

### 3. 模板设计原则

**命令安全性**:
- 避免使用危险命令（如 `rm -rf`）
- 对路径参数进行引号包裹: `'{path}'`
- 验证输入参数

**命令可读性**:
```rust
// ❌ 不好的模板（难以理解）
"find {p} -t f -n '*.{e}' | wc"

// ✅ 好的模板（清晰明确）
"find {path} -type f -name '*.{ext}' | wc -l"
```

### 4. 置信度阈值

根据 Intent 的复杂度调整阈值：

```rust
// 简单、明确的 Intent - 较高阈值
Intent::new(..., 0.7)  // 70% 置信度

// 复杂、模糊的 Intent - 较低阈值
Intent::new(..., 0.4)  // 40% 置信度

// 推荐默认值
Intent::new(..., 0.5)  // 50% 置信度
```

### 5. 错误处理

```rust
// 1. 检查是否有匹配
if let Some(best_match) = matcher.best_match(user_input) {
    // 2. 检查置信度
    if best_match.confidence < 0.6 {
        println!("⚠️ 置信度较低，可能不准确");
    }

    // 3. 生成执行计划（可能失败）
    match engine.generate_from_intent(&best_match) {
        Ok(plan) => {
            println!("✅ 命令: {}", plan.command);
        }
        Err(e) => {
            eprintln!("❌ 生成失败: {}", e);
        }
    }
} else {
    println!("❌ 无法识别意图，请重新描述");
}
```

### 6. 性能优化

**正则表达式缓存**:
IntentMatcher 自动缓存编译后的正则表达式，无需手动优化。

**减少不必要的匹配**:
```rust
// 使用 best_match() 而不是 match_intent() 获取所有结果
let best = matcher.best_match(input);  // ✅ 只返回最佳匹配

let all_matches = matcher.match_intent(input);  // ⚠️ 返回所有匹配（可能较慢）
```

### 7. 测试建议

编写单元测试验证 Intent 行为：

```rust
#[test]
fn test_count_files_intent() {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试匹配
    let matches = matcher.match_intent("统计当前目录下有多少个 Python 文件");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].intent.name, "count_files");

    // 测试实体提取
    if let Some(EntityType::FileType(ft)) = matches[0].extracted_entities.get("ext") {
        assert_eq!(ft, "py");
    }
}

#[test]
fn test_template_generation() {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();
    let engine = builtin.create_engine();

    if let Some(best_match) = matcher.best_match("查找大于 500 MB 的文件") {
        let plan = engine.generate_from_intent(&best_match).unwrap();
        assert!(plan.command.contains("500"));
        assert!(plan.command.contains("find"));
    }
}
```

---

## 附录

### A. 完整代码示例

详见 `examples/intent_dsl_demo.rs`

### B. 内置意图源码

详见 `src/dsl/intent/builtin.rs`

### C. EntityExtractor 实现

详见 `src/dsl/intent/extractor.rs`

### D. 集成测试

详见 `tests/test_intent_integration.rs`

---

## 常见问题 (FAQ)

### Q1: 为什么我的 Intent 没有匹配？

**可能原因**:
1. 置信度低于阈值 - 检查 `confidence_threshold`
2. 关键词不匹配 - 添加更多相关关键词
3. 正则模式不匹配 - 调整正则表达式

**调试方法**:
```rust
let matches = matcher.match_intent(user_input);
for m in &matches {
    println!("Intent: {}, 置信度: {:.2}", m.intent.name, m.confidence);
}
```

### Q2: 如何提高匹配准确性？

1. **增加关键词**: 添加更多同义词和相关词
2. **优化正则模式**: 使用更精确的表达式
3. **调整置信度阈值**: 根据实际情况调整
4. **添加更多 Intent**: 细化意图分类

### Q3: Entity Extraction 提取不准确怎么办？

**解决方法**:
1. 检查正则表达式是否匹配
2. 添加更多模式到 EntityExtractor
3. 使用更具体的用户输入
4. 依赖 Smart Fallback 的默认值

### Q4: 如何支持新的实体类型？

1. 在 `EntityType` enum 中添加新类型
2. 在 `EntityExtractor` 中添加提取方法
3. 在 `extract()` 方法中添加匹配逻辑

示例：
```rust
// 1. 添加新类型
pub enum EntityType {
    // ... 现有类型
    User(String),  // 新增用户类型
}

// 2. 添加提取方法
impl EntityExtractor {
    pub fn extract_user(&self, input: &str) -> Option<EntityType> {
        let pattern = Regex::new(r"@(\w+)").unwrap();
        if let Some(captures) = pattern.captures(input) {
            return Some(EntityType::User(captures[1].to_string()));
        }
        None
    }
}
```

### Q5: 如何与 Agent 集成？

Intent DSL 已集成到 Agent 中，参见 `src/agent.rs`:

```rust
// Agent 自动使用 Intent DSL
let agent = Agent::new(config, registry);

// 用户输入会先尝试 Intent 匹配
let result = agent.handle("统计 Python 文件");
```

---

**RealConsole Intent DSL** - 让自然语言理解变得简单而强大 🚀
