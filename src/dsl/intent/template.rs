//! Template 系统
//!
//! **设计哲学**：大道至简（道德经）
//!
//! Template 是连接意图与执行的桥梁，遵循以下原则：
//! - 上善若水：适配任何命令格式，无形而有力
//! - 少则得，多则惑：只做变量替换，不引入复杂逻辑
//! - 返璞归真：使用最简单的 {variable} 语法

use crate::dsl::intent::types::{EntityType, IntentMatch};
use std::collections::HashMap;

/// 命令模板
///
/// Template 定义了如何将意图转换为可执行的命令。
///
/// # 设计原则
///
/// - **静态定义**：模板在编译时或初始化时定义
/// - **简单替换**：使用 `{variable}` 占位符，运行时替换
/// - **无副作用**：模板本身不执行任何操作
///
/// # 示例
///
/// ```rust
/// use simpleconsole::dsl::intent::Template;
///
/// let template = Template::new(
///     "count_files",
///     "find {path} -name '*.{ext}' | wc -l",
///     vec!["path".to_string(), "ext".to_string()],
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Template {
    /// 模板名称（对应意图名称）
    pub name: String,

    /// 命令模板字符串
    /// 使用 {variable} 作为占位符
    pub template: String,

    /// 需要的变量列表
    pub variables: Vec<String>,

    /// 模板描述
    pub description: String,
}

/// 执行计划
///
/// ExecutionPlan 是模板和实际参数的组合，表示一个可以直接执行的命令。
///
/// # 设计原则
///
/// - **不可变性**：一旦创建，不可修改
/// - **完整性**：包含执行所需的所有信息
/// - **可追溯**：保留原始意图和模板信息
///
/// # 示例
///
/// ```rust
/// use simpleconsole::dsl::intent::ExecutionPlan;
/// use std::collections::HashMap;
///
/// // ExecutionPlan 通常由 TemplateEngine 生成
/// let plan = ExecutionPlan {
///     command: "find . -name '*.py' | wc -l".to_string(),
///     template_name: "count_files".to_string(),
///     bindings: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 生成的命令（已完成变量替换）
    pub command: String,

    /// 使用的模板名称
    pub template_name: String,

    /// 变量绑定（变量名 -> 值）
    pub bindings: HashMap<String, String>,
}

/// 模板引擎
///
/// TemplateEngine 负责管理模板和生成执行计划。
///
/// # 设计原则（《易经》：易则易知，简则易从）
///
/// - **简单注册**：register() 添加模板
/// - **简单生成**：generate() 生成执行计划
/// - **简单替换**：substitute() 完成变量替换
///
/// # 示例
///
/// ```rust
/// use simpleconsole::dsl::intent::{Template, TemplateEngine};
/// use std::collections::HashMap;
///
/// let mut engine = TemplateEngine::new();
///
/// // 注册模板
/// let template = Template::new(
///     "count_files",
///     "find {path} -name '*.{ext}' | wc -l",
///     vec!["path".to_string(), "ext".to_string()],
/// );
/// engine.register(template);
///
/// // 生成执行计划
/// let mut bindings = HashMap::new();
/// bindings.insert("path".to_string(), ".".to_string());
/// bindings.insert("ext".to_string(), "py".to_string());
///
/// let plan = engine.generate("count_files", bindings).unwrap();
/// assert_eq!(plan.command, "find . -name '*.py' | wc -l");
/// ```
#[derive(Debug)]
pub struct TemplateEngine {
    /// 已注册的模板（模板名 -> 模板）
    templates: HashMap<String, Template>,
}

impl Template {
    /// 创建一个新的模板
    ///
    /// # 参数
    ///
    /// - `name`: 模板名称
    /// - `template`: 命令模板字符串（使用 `{variable}` 作为占位符）
    /// - `variables`: 变量名列表
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::Template;
    ///
    /// let template = Template::new(
    ///     "grep_files",
    ///     "grep -r '{pattern}' {path}",
    ///     vec!["pattern".to_string(), "path".to_string()],
    /// );
    /// ```
    pub fn new(
        name: impl Into<String>,
        template: impl Into<String>,
        variables: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            template: template.into(),
            variables,
            description: String::new(),
        }
    }

    /// 添加描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 检查是否包含某个变量
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.iter().any(|v| v == name)
    }

    /// 提取模板中的所有变量占位符
    ///
    /// 使用简单的正则匹配提取 `{variable}` 格式的占位符
    pub fn extract_placeholders(&self) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut chars = self.template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut var_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume '}'
                        break;
                    }
                    var_name.push(next_ch);
                    chars.next();
                }
                if !var_name.is_empty() {
                    placeholders.push(var_name);
                }
            }
        }

        placeholders
    }
}

impl TemplateEngine {
    /// 创建一个新的模板引擎
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::TemplateEngine;
    ///
    /// let engine = TemplateEngine::new();
    /// ```
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// 注册一个模板
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Template, TemplateEngine};
    ///
    /// let mut engine = TemplateEngine::new();
    ///
    /// let template = Template::new(
    ///     "list_files",
    ///     "ls -la {path}",
    ///     vec!["path".to_string()],
    /// );
    ///
    /// engine.register(template);
    /// ```
    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// 获取模板
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// 生成执行计划
    ///
    /// # 参数
    ///
    /// - `template_name`: 模板名称
    /// - `bindings`: 变量绑定（变量名 -> 值）
    ///
    /// # 返回
    ///
    /// 成功返回 `ExecutionPlan`，失败返回错误信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Template, TemplateEngine};
    /// use std::collections::HashMap;
    ///
    /// let mut engine = TemplateEngine::new();
    ///
    /// let template = Template::new(
    ///     "count_lines",
    ///     "wc -l {file}",
    ///     vec!["file".to_string()],
    /// );
    /// engine.register(template);
    ///
    /// let mut bindings = HashMap::new();
    /// bindings.insert("file".to_string(), "test.txt".to_string());
    ///
    /// let plan = engine.generate("count_lines", bindings).unwrap();
    /// assert_eq!(plan.command, "wc -l test.txt");
    /// ```
    pub fn generate(
        &self,
        template_name: &str,
        bindings: HashMap<String, String>,
    ) -> Result<ExecutionPlan, String> {
        // 1. 查找模板
        let template = self
            .templates
            .get(template_name)
            .ok_or_else(|| format!("模板不存在: {}", template_name))?;

        // 2. 检查必需变量
        for var in &template.variables {
            if !bindings.contains_key(var) {
                return Err(format!("缺少必需变量: {}", var));
            }
        }

        // 3. 执行变量替换
        let command = Self::substitute(&template.template, &bindings);

        // 4. 创建执行计划
        Ok(ExecutionPlan {
            command,
            template_name: template_name.to_string(),
            bindings,
        })
    }

    /// 从意图匹配生成执行计划
    ///
    /// 自动从 IntentMatch 的 extracted_entities 中提取变量
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentMatch, IntentDomain, Template, TemplateEngine};
    /// use std::collections::HashMap;
    ///
    /// let mut engine = TemplateEngine::new();
    ///
    /// let template = Template::new(
    ///     "count_files",
    ///     "find . -name '*.{ext}' | wc -l",
    ///     vec!["ext".to_string()],
    /// );
    /// engine.register(template);
    ///
    /// // 创建意图匹配
    /// let intent = Intent::new(
    ///     "count_files",
    ///     IntentDomain::FileOps,
    ///     vec![],
    ///     vec![],
    ///     0.5,
    /// );
    ///
    /// let intent_match = IntentMatch::new(intent, 0.9);
    ///
    /// // 注意：此示例中 extracted_entities 为空，实际使用需要包含实体
    /// ```
    pub fn generate_from_intent(
        &self,
        intent_match: &IntentMatch,
    ) -> Result<ExecutionPlan, String> {
        // 从实体中提取变量绑定
        let mut bindings = HashMap::new();

        // 步骤 1: 先使用 Intent 中定义的默认值（道德经：有无相生）
        for (name, entity) in &intent_match.intent.entities {
            let value = match entity {
                EntityType::FileType(v) => v.clone(),
                EntityType::Operation(v) => v.clone(),
                EntityType::Path(v) => v.clone(),
                EntityType::Number(n) => n.to_string(),
                EntityType::Date(v) => v.clone(),
                EntityType::Custom(_, v) => v.clone(),
            };
            bindings.insert(name.clone(), value);
        }

        // 步骤 2: 用提取的实体覆盖默认值（道德经：后其身而身先）
        for (name, entity) in &intent_match.extracted_entities {
            let value = match entity {
                EntityType::FileType(v) => v.clone(),
                EntityType::Operation(v) => v.clone(),
                EntityType::Path(v) => v.clone(),
                EntityType::Number(n) => n.to_string(),
                EntityType::Date(v) => v.clone(),
                EntityType::Custom(_, v) => v.clone(),
            };
            bindings.insert(name.clone(), value);
        }

        self.generate(&intent_match.intent.name, bindings)
    }

    /// 变量替换（核心算法）
    ///
    /// **道德经**：「天下难事，必作于易；天下大事，必作于细」
    ///
    /// 使用最简单的字符串替换算法：
    /// - 遍历所有变量绑定
    /// - 将 `{variable}` 替换为对应的值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::TemplateEngine;
    /// use std::collections::HashMap;
    ///
    /// let mut bindings = HashMap::new();
    /// bindings.insert("name".to_string(), "Alice".to_string());
    /// bindings.insert("age".to_string(), "30".to_string());
    ///
    /// let result = TemplateEngine::substitute(
    ///     "Hello {name}, you are {age} years old",
    ///     &bindings
    /// );
    ///
    /// assert_eq!(result, "Hello Alice, you are 30 years old");
    /// ```
    pub fn substitute(template: &str, bindings: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (var_name, value) in bindings {
            let placeholder = format!("{{{}}}", var_name);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// 获取已注册的模板数量
    pub fn len(&self) -> usize {
        self.templates.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    /// 清空所有模板
    pub fn clear(&mut self) {
        self.templates.clear();
    }

    /// 获取所有模板名称
    pub fn template_names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionPlan {
    /// 创建一个新的执行计划
    pub fn new(
        command: impl Into<String>,
        template_name: impl Into<String>,
        bindings: HashMap<String, String>,
    ) -> Self {
        Self {
            command: command.into(),
            template_name: template_name.into(),
            bindings,
        }
    }

    /// 获取绑定的变量值
    pub fn get_binding(&self, name: &str) -> Option<&String> {
        self.bindings.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = Template::new(
            "count_files",
            "find {path} -name '*.{ext}' | wc -l",
            vec!["path".to_string(), "ext".to_string()],
        );

        assert_eq!(template.name, "count_files");
        assert_eq!(template.template, "find {path} -name '*.{ext}' | wc -l");
        assert_eq!(template.variables.len(), 2);
    }

    #[test]
    fn test_template_with_description() {
        let template = Template::new(
            "test",
            "echo {msg}",
            vec!["msg".to_string()],
        )
        .with_description("Test template");

        assert_eq!(template.description, "Test template");
    }

    #[test]
    fn test_template_has_variable() {
        let template = Template::new(
            "test",
            "echo {msg}",
            vec!["msg".to_string()],
        );

        assert!(template.has_variable("msg"));
        assert!(!template.has_variable("other"));
    }

    #[test]
    fn test_template_extract_placeholders() {
        let template = Template::new(
            "test",
            "find {path} -name '*.{ext}' -type {type}",
            vec![],
        );

        let placeholders = template.extract_placeholders();
        assert_eq!(placeholders.len(), 3);
        assert!(placeholders.contains(&"path".to_string()));
        assert!(placeholders.contains(&"ext".to_string()));
        assert!(placeholders.contains(&"type".to_string()));
    }

    #[test]
    fn test_engine_creation() {
        let engine = TemplateEngine::new();
        assert_eq!(engine.len(), 0);
        assert!(engine.is_empty());
    }

    #[test]
    fn test_engine_register() {
        let mut engine = TemplateEngine::new();

        let template = Template::new(
            "test",
            "echo {msg}",
            vec!["msg".to_string()],
        );

        engine.register(template);

        assert_eq!(engine.len(), 1);
        assert!(!engine.is_empty());
        assert!(engine.get("test").is_some());
    }

    #[test]
    fn test_substitute_simple() {
        let mut bindings = HashMap::new();
        bindings.insert("name".to_string(), "Alice".to_string());

        let result = TemplateEngine::substitute("Hello {name}", &bindings);
        assert_eq!(result, "Hello Alice");
    }

    #[test]
    fn test_substitute_multiple() {
        let mut bindings = HashMap::new();
        bindings.insert("path".to_string(), ".".to_string());
        bindings.insert("ext".to_string(), "py".to_string());

        let result = TemplateEngine::substitute(
            "find {path} -name '*.{ext}'",
            &bindings,
        );
        assert_eq!(result, "find . -name '*.py'");
    }

    #[test]
    fn test_substitute_no_match() {
        let bindings = HashMap::new();
        let result = TemplateEngine::substitute("echo hello", &bindings);
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_generate_success() {
        let mut engine = TemplateEngine::new();

        let template = Template::new(
            "count_lines",
            "wc -l {file}",
            vec!["file".to_string()],
        );
        engine.register(template);

        let mut bindings = HashMap::new();
        bindings.insert("file".to_string(), "test.txt".to_string());

        let plan = engine.generate("count_lines", bindings).unwrap();
        assert_eq!(plan.command, "wc -l test.txt");
        assert_eq!(plan.template_name, "count_lines");
    }

    #[test]
    fn test_generate_template_not_found() {
        let engine = TemplateEngine::new();
        let bindings = HashMap::new();

        let result = engine.generate("nonexistent", bindings);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("模板不存在"));
    }

    #[test]
    fn test_generate_missing_variable() {
        let mut engine = TemplateEngine::new();

        let template = Template::new(
            "test",
            "echo {msg}",
            vec!["msg".to_string()],
        );
        engine.register(template);

        let bindings = HashMap::new(); // 没有提供 msg

        let result = engine.generate("test", bindings);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("缺少必需变量"));
    }

    #[test]
    fn test_generate_complex_template() {
        let mut engine = TemplateEngine::new();

        // 道德经：「大道至简」
        // 避免转义复杂性，直接使用简单明了的模板
        let template = Template::new(
            "complex",
            "find {path} -name '*.{ext}' -mtime {days}",
            vec![
                "path".to_string(),
                "ext".to_string(),
                "days".to_string(),
            ],
        );
        engine.register(template);

        let mut bindings = HashMap::new();
        bindings.insert("path".to_string(), "/tmp".to_string());
        bindings.insert("ext".to_string(), "log".to_string());
        bindings.insert("days".to_string(), "-7".to_string());

        let plan = engine.generate("complex", bindings).unwrap();
        assert_eq!(
            plan.command,
            "find /tmp -name '*.log' -mtime -7"
        );
    }

    #[test]
    fn test_execution_plan_get_binding() {
        let mut bindings = HashMap::new();
        bindings.insert("file".to_string(), "test.txt".to_string());

        let plan = ExecutionPlan::new("wc -l test.txt", "count_lines", bindings);

        assert_eq!(plan.get_binding("file"), Some(&"test.txt".to_string()));
        assert_eq!(plan.get_binding("nonexistent"), None);
    }

    #[test]
    fn test_template_names() {
        let mut engine = TemplateEngine::new();

        engine.register(Template::new("t1", "cmd1", vec![]));
        engine.register(Template::new("t2", "cmd2", vec![]));

        let names = engine.template_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"t1".to_string()));
        assert!(names.contains(&"t2".to_string()));
    }

    #[test]
    fn test_engine_clear() {
        let mut engine = TemplateEngine::new();

        engine.register(Template::new("test", "echo hello", vec![]));
        assert!(!engine.is_empty());

        engine.clear();
        assert!(engine.is_empty());
    }

    #[test]
    fn test_generate_from_intent_uses_default_entities() {
        use crate::dsl::intent::types::{Intent, IntentDomain, IntentMatch};

        // 创建模板引擎和模板
        let mut engine = TemplateEngine::new();
        let template = Template::new(
            "check_disk_usage",
            "du -sh {path}/* | sort -hr | head -n {limit}",
            vec!["path".to_string(), "limit".to_string()],
        );
        engine.register(template);

        // 创建带默认实体的意图
        let intent = Intent::new(
            "check_disk_usage",
            IntentDomain::DiagnosticOps,
            vec!["检查".to_string(), "磁盘".to_string()],
            vec![],
            0.5,
        )
        .with_entity("path", EntityType::Path(".".to_string()))
        .with_entity("limit", EntityType::Number(10.0));

        // 创建意图匹配（没有提取任何实体）
        let intent_match = IntentMatch {
            intent,
            confidence: 0.9,
            matched_keywords: vec![],
            extracted_entities: std::collections::HashMap::new(), // 空的提取实体
        };

        // 生成执行计划 - 应该使用默认值
        let plan = engine.generate_from_intent(&intent_match).unwrap();

        // 验证命令使用了默认值
        assert_eq!(plan.command, "du -sh ./* | sort -hr | head -n 10");
        assert_eq!(plan.template_name, "check_disk_usage");
    }

    #[test]
    fn test_generate_from_intent_extracted_overrides_defaults() {
        use crate::dsl::intent::types::{Intent, IntentDomain, IntentMatch};

        // 创建模板引擎和模板
        let mut engine = TemplateEngine::new();
        let template = Template::new(
            "check_disk_usage",
            "du -sh {path}/* | sort -hr | head -n {limit}",
            vec!["path".to_string(), "limit".to_string()],
        );
        engine.register(template);

        // 创建带默认实体的意图
        let intent = Intent::new(
            "check_disk_usage",
            IntentDomain::DiagnosticOps,
            vec!["检查".to_string(), "磁盘".to_string()],
            vec![],
            0.5,
        )
        .with_entity("path", EntityType::Path(".".to_string()))
        .with_entity("limit", EntityType::Number(10.0));

        // 创建意图匹配（提取了部分实体）
        let mut extracted = std::collections::HashMap::new();
        extracted.insert("path".to_string(), EntityType::Path("/home".to_string()));
        // 注意：limit 没有被提取，应该使用默认值

        let intent_match = IntentMatch {
            intent,
            confidence: 0.9,
            matched_keywords: vec![],
            extracted_entities: extracted,
        };

        // 生成执行计划 - path 使用提取值，limit 使用默认值
        let plan = engine.generate_from_intent(&intent_match).unwrap();

        // 验证命令：path=/home（提取值），limit=10（默认值）
        assert_eq!(plan.command, "du -sh /home/* | sort -hr | head -n 10");
        assert_eq!(plan.template_name, "check_disk_usage");
    }
}
