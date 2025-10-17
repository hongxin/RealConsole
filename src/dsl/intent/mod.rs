//! Intent DSL 模块
//!
//! Intent DSL (意图表达语言) 负责将自然语言输入转换为可执行的操作计划。
//!
//! ## 核心概念
//!
//! - **Intent (意图)**: 表示用户想要完成的任务
//! - **Template (模板)**: 定义如何执行特定意图
//! - **ExecutionPlan (执行计划)**: 意图和模板的组合
//!
//! ## 模块结构
//!
//! ```text
//! intent/
//! ├── types.rs          - 核心数据结构定义 ✅
//! ├── matcher.rs        - 意图匹配引擎 ✅
//! ├── template.rs       - 模板系统 ✅
//! ├── builtin.rs        - 内置意图和模板库 ✅
//! ├── extractor.rs      - 实体提取引擎 ✅ (Phase 3 Week 3 + Phase 2 LLM)
//! ├── validator.rs      - 命令验证器 ✅ (Phase 3 LLM)
//! ├── pipeline_bridge.rs - Intent → Pipeline 转换桥梁 ✅ (Phase 6.3 Step 1)
//! └── optimizer.rs      - 性能优化
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use simpleconsole::dsl::intent::{Intent, IntentMatcher, IntentDomain};
//!
//! let mut matcher = IntentMatcher::new();
//!
//! // 注册意图
//! matcher.register(Intent {
//!     name: "count_lines".to_string(),
//!     domain: IntentDomain::FileOps,
//!     keywords: vec!["统计".to_string(), "行数".to_string()],
//!     patterns: vec![r"统计.*行数".to_string()],
//!     entities: std::collections::HashMap::new(),
//!     confidence_threshold: 0.5,
//! });
//!
//! // 匹配用户输入
//! let matches = matcher.match_intent("统计 Python 代码行数");
//! ```

pub mod types;
pub mod matcher;
pub mod template;
pub mod builtin;
pub mod extractor;
pub mod validator;  // Phase 3: LLM Command Validation
pub mod pipeline_bridge;  // Phase 6.3: Intent → Pipeline Bridge
pub mod llm_bridge;  // Phase 7: LLM → Pipeline Bridge

// Re-export commonly used types
pub use types::{
    EntityType, Intent, IntentDomain, IntentMatch,
};
pub use matcher::IntentMatcher;
pub use template::{Template, TemplateEngine, ExecutionPlan};
pub use builtin::BuiltinIntents;
pub use extractor::EntityExtractor;
pub use validator::{CommandValidator, ValidationResult};
pub use pipeline_bridge::IntentToPipeline;
pub use llm_bridge::LlmToPipeline;
