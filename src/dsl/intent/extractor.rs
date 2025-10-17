//! Entity Extraction Module
//!
//! Extracts structured entities from natural language input.
//!
//! # Entity Types
//!
//! - **FileType**: File extensions (py, rs, js, etc.)
//! - **Operation**: Action verbs (count, find, analyze, etc.)
//! - **Path**: File system paths (., ./src, /tmp, etc.)
//! - **Number**: Numeric values (42, 100, 1.5, etc.)
//! - **Date**: Date strings (2025-10-14, yesterday, etc.)
//!
//! # Design Philosophy
//!
//! "大道至简" (The Great Way is Simple) - Use simple, robust patterns
//! that work reliably for common use cases.
//!
//! # LLM Enhancement (Phase 2)
//!
//! When regex extraction fails, optionally use LLM for intelligent extraction.

use crate::dsl::intent::types::EntityType;
use crate::llm::{LlmClient, Message};
use regex::Regex;
use std::collections::HashMap;

/// Entity Extractor
///
/// Extracts structured information from natural language input.
///
/// # Example
///
/// ```rust
/// use simpleconsole::dsl::intent::extractor::EntityExtractor;
///
/// let extractor = EntityExtractor::new();
/// let entities = extractor.extract("统计 Python 代码行数", &Default::default());
/// ```
#[derive(Debug)]
pub struct EntityExtractor {
    /// Cached regex patterns for performance
    file_type_pattern: Regex,
    number_pattern: Regex,
    path_pattern: Regex,
}

impl EntityExtractor {
    /// Create a new entity extractor
    pub fn new() -> Self {
        Self {
            // File types: python, py, rust, rs, js, etc.
            file_type_pattern: Regex::new(
                r"(?i)(python|py|rust|rs|javascript|js|typescript|ts|go|java|cpp|c\+\+|c|shell|sh|yaml|yml|json|xml|html|css|md|markdown|txt|log)"
            ).unwrap(),

            // Numbers: integers or decimals
            number_pattern: Regex::new(r"\b(\d+(?:\.\d+)?)\b").unwrap(),

            // Paths: starting with . or /, just ., or simple directory names
            // Supports: ./src, /tmp, ., doc, docs/, test-dir
            path_pattern: Regex::new(r"(\./[^\s]+|/[^\s]+|\.|[a-zA-Z0-9_-]+/?)").unwrap(),
        }
    }

    /// Extract all entities from input based on expected entity definitions
    ///
    /// # Arguments
    ///
    /// * `input` - User's natural language input
    /// * `expected` - Expected entity types defined by the intent
    ///
    /// # Returns
    ///
    /// HashMap of extracted entities (entity name → EntityType)
    ///
    /// # Example
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::extractor::EntityExtractor;
    /// use simpleconsole::dsl::intent::EntityType;
    /// use std::collections::HashMap;
    ///
    /// let extractor = EntityExtractor::new();
    /// let mut expected = HashMap::new();
    /// expected.insert("file_type".to_string(), EntityType::FileType(String::new()));
    /// expected.insert("path".to_string(), EntityType::Path(String::new()));
    ///
    /// let entities = extractor.extract("统计 Python 代码行数在 ./src 目录", &expected);
    /// assert!(entities.contains_key("file_type"));
    /// assert!(entities.contains_key("path"));
    /// ```
    pub fn extract(
        &self,
        input: &str,
        expected: &HashMap<String, EntityType>,
    ) -> HashMap<String, EntityType> {
        let mut extracted = HashMap::new();

        // Extract each expected entity type
        for (name, entity_type) in expected {
            match entity_type {
                EntityType::FileType(_) => {
                    if let Some(file_type) = self.extract_file_type(input) {
                        extracted.insert(name.clone(), file_type);
                    }
                }
                EntityType::Operation(_) => {
                    if let Some(operation) = self.extract_operation(input) {
                        extracted.insert(name.clone(), operation);
                    }
                }
                EntityType::Path(_) => {
                    if let Some(path) = self.extract_path(input) {
                        extracted.insert(name.clone(), path);
                    }
                }
                EntityType::Number(_) => {
                    if let Some(number) = self.extract_number(input) {
                        extracted.insert(name.clone(), number);
                    }
                }
                EntityType::Date(_) => {
                    if let Some(date) = self.extract_date(input) {
                        extracted.insert(name.clone(), date);
                    }
                }
                EntityType::Custom(type_name, _) => {
                    // For custom entities, try generic extraction
                    if let Some(custom) = self.extract_custom(input, type_name) {
                        extracted.insert(name.clone(), custom);
                    }
                }
            }
        }

        extracted
    }

    /// Extract file type from input
    ///
    /// Recognizes common file extensions and language names.
    ///
    /// # Example
    ///
    /// ```
    /// use simpleconsole::dsl::intent::extractor::EntityExtractor;
    /// use simpleconsole::dsl::intent::EntityType;
    ///
    /// let extractor = EntityExtractor::new();
    ///
    /// if let Some(EntityType::FileType(ft)) = extractor.extract_file_type("统计 Python 代码") {
    ///     assert_eq!(ft, "py");
    /// }
    /// ```
    pub fn extract_file_type(&self, input: &str) -> Option<EntityType> {
        if let Some(captures) = self.file_type_pattern.captures(input) {
            if let Some(matched) = captures.get(1) {
                let file_type = matched.as_str().to_lowercase();
                // Normalize to standard extensions
                let normalized = match file_type.as_str() {
                    "python" => "py",
                    "rust" => "rs",
                    "javascript" => "js",
                    "typescript" => "ts",
                    "c++" | "cpp" => "cpp",
                    "shell" => "sh",
                    "yaml" => "yaml",
                    "markdown" => "md",
                    other => other,
                };
                return Some(EntityType::FileType(normalized.to_string()));
            }
        }
        None
    }

    /// Extract operation from input
    ///
    /// Recognizes common action verbs.
    fn extract_operation(&self, input: &str) -> Option<EntityType> {
        let input_lower = input.to_lowercase();

        // Common operations (Chinese and English)
        let operations = [
            ("统计", "count"),
            ("查找", "find"),
            ("搜索", "search"),
            ("分析", "analyze"),
            ("检查", "check"),
            ("列出", "list"),
            ("显示", "show"),
            ("排序", "sort"),
            ("grep", "grep"),
            ("count", "count"),
            ("find", "find"),
            ("search", "search"),
            ("analyze", "analyze"),
            ("check", "check"),
            ("list", "list"),
            ("sort", "sort"),
        ];

        for (keyword, operation) in &operations {
            if input_lower.contains(keyword) {
                return Some(EntityType::Operation(operation.to_string()));
            }
        }

        None
    }

    /// Extract path from input
    ///
    /// Recognizes file system paths.
    ///
    /// # Example
    ///
    /// ```
    /// use simpleconsole::dsl::intent::extractor::EntityExtractor;
    /// use simpleconsole::dsl::intent::EntityType;
    ///
    /// let extractor = EntityExtractor::new();
    ///
    /// if let Some(EntityType::Path(path)) = extractor.extract_path("查找 ./src 目录") {
    ///     assert_eq!(path, "./src");
    /// }
    /// ```
    pub fn extract_path(&self, input: &str) -> Option<EntityType> {
        // Try to find all path matches and return the first valid one
        for captures in self.path_pattern.captures_iter(input) {
            if let Some(matched) = captures.get(1) {
                let path = matched.as_str();
                // Filter out command keywords and file types to avoid false positives
                if !self.is_command_keyword(path) {
                    return Some(EntityType::Path(path.to_string()));
                }
            }
        }

        // Default path extraction: look for common keywords
        if input.contains("当前目录") || input.contains("这里") {
            return Some(EntityType::Path(".".to_string()));
        }

        None
    }

    /// Extract number from input
    ///
    /// Recognizes integers and floating-point numbers.
    ///
    /// # Example
    ///
    /// ```
    /// use simpleconsole::dsl::intent::extractor::EntityExtractor;
    /// use simpleconsole::dsl::intent::EntityType;
    ///
    /// let extractor = EntityExtractor::new();
    ///
    /// if let Some(EntityType::Number(n)) = extractor.extract_number("查找大于 100 MB 的文件") {
    ///     assert_eq!(n, 100.0);
    /// }
    /// ```
    pub fn extract_number(&self, input: &str) -> Option<EntityType> {
        if let Some(captures) = self.number_pattern.captures(input) {
            if let Some(matched) = captures.get(1) {
                if let Ok(num) = matched.as_str().parse::<f64>() {
                    return Some(EntityType::Number(num));
                }
            }
        }
        None
    }

    /// Extract date from input
    ///
    /// Recognizes date patterns and relative time expressions.
    fn extract_date(&self, input: &str) -> Option<EntityType> {
        let input_lower = input.to_lowercase();

        // Relative time expressions
        if input_lower.contains("今天") || input_lower.contains("today") {
            return Some(EntityType::Date("today".to_string()));
        }
        if input_lower.contains("昨天") || input_lower.contains("yesterday") {
            return Some(EntityType::Date("yesterday".to_string()));
        }
        if input_lower.contains("最近") || input_lower.contains("recent") {
            return Some(EntityType::Date("recent".to_string()));
        }

        // ISO date pattern: 2025-10-14
        let date_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}").ok()?;
        if let Some(captures) = date_pattern.captures(input) {
            if let Some(matched) = captures.get(0) {
                return Some(EntityType::Date(matched.as_str().to_string()));
            }
        }

        None
    }

    /// Check if a word is a command keyword or file type (not a path)
    ///
    /// This prevents false positives when extracting paths.
    /// For example, in "查看文件", we don't want to extract "查看" as a path.
    /// Similarly, in "统计 Python 代码", we don't want to extract "Python" as a path.
    fn is_command_keyword(&self, word: &str) -> bool {
        let word_lower = word.to_lowercase();

        matches!(
            word_lower.as_str(),
            // Chinese command keywords
            "查看" | "显示" | "列出" | "检查" | "统计" | "查找" | "搜索" |
            "分析" | "排序" | "计数" | "执行" | "运行" |
            // English command keywords
            "ls" | "list" | "find" | "grep" | "search" | "check" | "show" |
            "count" | "sort" | "analyze" | "run" | "execute" |
            // File type keywords (to avoid extracting as paths)
            "python" | "py" | "rust" | "rs" | "javascript" | "js" |
            "typescript" | "ts" | "go" | "java" | "cpp" | "c" |
            "shell" | "sh" | "yaml" | "yml" | "json" | "xml" |
            "html" | "css" | "md" | "markdown" | "txt" | "log"
        )
    }

    /// Extract custom entity
    ///
    /// Generic extraction for custom entity types.
    ///
    /// **Phase 6.2.1**: 支持排序方向识别
    fn extract_custom(&self, input: &str, type_name: &str) -> Option<EntityType> {
        match type_name {
            // Phase 6.2.1: 识别排序方向
            "sort" => self.extract_sort_direction(input),
            _ => {
                // 其他自定义实体暂不支持
                eprintln!("警告: 自定义实体类型 '{}' 无法自动提取", type_name);
                None
            }
        }
    }

    /// Extract sort direction from input (Phase 6.2.1)
    ///
    /// **哲学体现**：
    /// - "最大" vs "最小" 不是两个独立的操作
    /// - 而是同一维度（爻）的两端
    /// - Ascending ⇄ Descending 的连续变化
    ///
    /// # Example
    ///
    /// ```
    /// use simpleconsole::dsl::intent::extractor::EntityExtractor;
    /// use simpleconsole::dsl::intent::EntityType;
    ///
    /// let extractor = EntityExtractor::new();
    ///
    /// // 最大 → 降序
    /// if let Some(EntityType::Custom(_, dir)) = extractor.extract_sort_direction("显示最大的文件") {
    ///     assert_eq!(dir, "-hr");
    /// }
    ///
    /// // 最小 → 升序
    /// if let Some(EntityType::Custom(_, dir)) = extractor.extract_sort_direction("显示最小的文件") {
    ///     assert_eq!(dir, "-h");
    /// }
    /// ```
    fn extract_sort_direction(&self, input: &str) -> Option<EntityType> {
        let input_lower = input.to_lowercase();

        // 降序关键词 (Descending) - 用于"最大"、"大于"
        let descending_keywords = [
            "最大", "大于", "大的", "largest", "bigger", "greater",
            "top", "最多", "降序", "descending", "desc"
        ];

        // 升序关键词 (Ascending) - 用于"最小"、"小于"
        let ascending_keywords = [
            "最小", "小于", "小的", "smallest", "smaller", "less",
            "bottom", "最少", "升序", "ascending", "asc"
        ];

        // 检查降序关键词
        for keyword in &descending_keywords {
            if input_lower.contains(keyword) {
                return Some(EntityType::Custom(
                    "sort".to_string(),
                    "-hr".to_string(),  // sort -k5 -hr (降序)
                ));
            }
        }

        // 检查升序关键词
        for keyword in &ascending_keywords {
            if input_lower.contains(keyword) {
                return Some(EntityType::Custom(
                    "sort".to_string(),
                    "-h".to_string(),  // sort -k5 -h (升序)
                ));
            }
        }

        // 默认：如果没有明确指定，返回降序（符合find_large_files的默认行为）
        Some(EntityType::Custom(
            "sort".to_string(),
            "-hr".to_string(),
        ))
    }

    /// Extract all entities without type constraints
    ///
    /// This method extracts all detectable entities regardless of expectations.
    /// Useful for exploratory analysis or when entity types are unknown.
    #[allow(dead_code)]
    pub fn extract_all(&self, input: &str) -> HashMap<String, EntityType> {
        let mut entities = HashMap::new();

        // Extract file type
        if let Some(file_type) = self.extract_file_type(input) {
            entities.insert("file_type".to_string(), file_type);
        }

        // Extract operation
        if let Some(operation) = self.extract_operation(input) {
            entities.insert("operation".to_string(), operation);
        }

        // Extract path
        if let Some(path) = self.extract_path(input) {
            entities.insert("path".to_string(), path);
        }

        // Extract number
        if let Some(number) = self.extract_number(input) {
            entities.insert("number".to_string(), number);
        }

        // Extract date
        if let Some(date) = self.extract_date(input) {
            entities.insert("date".to_string(), date);
        }

        entities
    }

    // ========== LLM 增强提取 (Phase 2) ==========

    /// 使用 LLM 智能提取实体 (Phase 2)
    ///
    /// 当正则提取失败时的 fallback 机制，使用 LLM 理解语义
    ///
    /// # Arguments
    ///
    /// * `input` - 用户输入
    /// * `expected` - 期望提取的实体类型
    /// * `llm` - LLM 客户端
    ///
    /// # Returns
    ///
    /// 完整的实体集合（Regex 提取 + LLM 补充）
    pub async fn extract_with_llm(
        &self,
        input: &str,
        expected: &HashMap<String, EntityType>,
        llm: &dyn LlmClient,
    ) -> HashMap<String, EntityType> {
        let mut extracted = HashMap::new();

        // Step 1: 先尝试正则提取
        let regex_extracted = self.extract(input, expected);

        // Step 2: 检查是否有缺失的实体
        let missing_entities: Vec<_> = expected
            .keys()
            .filter(|k| !regex_extracted.contains_key(*k))
            .collect();

        if missing_entities.is_empty() {
            // 正则已提取完整，无需 LLM
            return regex_extracted;
        }

        // Step 3: 构造 LLM 提取 prompt
        let prompt = self.build_extraction_prompt(input, &missing_entities, expected);

        // Step 4: 调用 LLM
        let messages = vec![Message::user(prompt)];
        match llm.chat(messages).await {
            Ok(response) => {
                // 解析 LLM 返回的 JSON
                if let Ok(llm_entities) = self.parse_llm_response(&response, expected) {
                    extracted.extend(llm_entities);
                }
            }
            Err(e) => {
                eprintln!("⚠ LLM 提取失败: {}", e);
            }
        }

        // Step 5: 合并正则和 LLM 提取结果
        extracted.extend(regex_extracted);
        extracted
    }

    /// 构造 LLM 提取 prompt
    fn build_extraction_prompt(
        &self,
        input: &str,
        missing: &[&String],
        expected: &HashMap<String, EntityType>,
    ) -> String {
        let entity_descriptions: Vec<String> = missing
            .iter()
            .map(|name| {
                let entity_type = expected.get(*name).unwrap();
                format!("  - {}: {}", name, self.describe_entity_type(entity_type))
            })
            .collect();

        format!(
            r#"从以下用户输入中提取指定的参数：

用户输入: "{}"

需要提取的参数:
{}

请以 JSON 格式返回提取结果，格式为:
{{
  "param_name": "value"
}}

如果无法提取某个参数，请忽略它。只返回 JSON，不要包含其他解释。"#,
            input,
            entity_descriptions.join("\n")
        )
    }

    /// 描述实体类型（用于 prompt）
    fn describe_entity_type(&self, entity_type: &EntityType) -> String {
        match entity_type {
            EntityType::Path(_) => "文件路径或目录名 (如: ./src, doc, /tmp)".to_string(),
            EntityType::FileType(_) => "文件类型 (如: py, rs, js)".to_string(),
            EntityType::Number(_) => "数字 (如: 100, 3.14)".to_string(),
            EntityType::Operation(_) => "操作名称 (如: count, find)".to_string(),
            EntityType::Date(_) => "日期或时间 (如: 2025-10-14, today)".to_string(),
            EntityType::Custom(name, _) => name.clone(),
        }
    }

    /// 解析 LLM 响应的 JSON
    fn parse_llm_response(
        &self,
        response: &str,
        expected: &HashMap<String, EntityType>,
    ) -> Result<HashMap<String, EntityType>, String> {
        // 提取 JSON 块（支持 ```json ... ``` 格式）
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                return Err("未找到完整的 JSON".to_string());
            }
        } else {
            return Err("未找到 JSON 响应".to_string());
        };

        // 解析 JSON
        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        let mut entities = HashMap::new();

        // 将 JSON 转换为 EntityType
        for (name, entity_type) in expected {
            if let Some(value) = parsed.get(name) {
                if let Some(value_str) = value.as_str() {
                    if value_str.is_empty() {
                        continue; // 跳过空值
                    }
                    let entity = match entity_type {
                        EntityType::Path(_) => EntityType::Path(value_str.to_string()),
                        EntityType::FileType(_) => EntityType::FileType(value_str.to_string()),
                        EntityType::Operation(_) => EntityType::Operation(value_str.to_string()),
                        EntityType::Date(_) => EntityType::Date(value_str.to_string()),
                        EntityType::Number(_) => {
                            if let Ok(n) = value_str.parse::<f64>() {
                                EntityType::Number(n)
                            } else {
                                continue;
                            }
                        }
                        EntityType::Custom(t, _) => {
                            EntityType::Custom(t.clone(), value_str.to_string())
                        }
                    };
                    entities.insert(name.clone(), entity);
                } else if let Some(n) = value.as_f64() {
                    // 直接是数字
                    if matches!(entity_type, EntityType::Number(_)) {
                        entities.insert(name.clone(), EntityType::Number(n));
                    }
                }
            }
        }

        Ok(entities)
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let extractor = EntityExtractor::new();
        assert!(extractor.file_type_pattern.is_match("python"));
    }

    #[test]
    fn test_extract_file_type_python() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::FileType(ft)) = extractor.extract_file_type("统计 Python 代码") {
            assert_eq!(ft, "py");
        } else {
            panic!("Expected FileType entity");
        }
    }

    #[test]
    fn test_extract_file_type_rust() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::FileType(ft)) = extractor.extract_file_type("查找 Rust 文件") {
            assert_eq!(ft, "rs");
        } else {
            panic!("Expected FileType entity");
        }
    }

    #[test]
    fn test_extract_file_type_extension() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::FileType(ft)) = extractor.extract_file_type("统计 .py 文件") {
            assert_eq!(ft, "py");
        } else {
            panic!("Expected FileType entity");
        }
    }

    #[test]
    fn test_extract_operation_count() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Operation(op)) = extractor.extract_operation("统计代码行数") {
            assert_eq!(op, "count");
        } else {
            panic!("Expected Operation entity");
        }
    }

    #[test]
    fn test_extract_operation_find() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Operation(op)) = extractor.extract_operation("查找大文件") {
            assert_eq!(op, "find");
        } else {
            panic!("Expected Operation entity");
        }
    }

    #[test]
    fn test_extract_path_relative() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Path(path)) = extractor.extract_path("查找 ./src 目录下的文件") {
            assert_eq!(path, "./src");
        } else {
            panic!("Expected Path entity");
        }
    }

    #[test]
    fn test_extract_path_absolute() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Path(path)) = extractor.extract_path("查找 /tmp 目录下的文件") {
            assert_eq!(path, "/tmp");
        } else {
            panic!("Expected Path entity");
        }
    }

    #[test]
    fn test_extract_path_current() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Path(path)) = extractor.extract_path("统计当前目录的文件") {
            assert_eq!(path, ".");
        } else {
            panic!("Expected Path entity");
        }
    }

    #[test]
    fn test_extract_path_simple_directory_name() {
        let extractor = EntityExtractor::new();

        // Test simple directory name "doc"
        if let Some(EntityType::Path(path)) = extractor.extract_path("查看子目录 doc") {
            assert_eq!(path, "doc");
        } else {
            panic!("Expected to extract 'doc' as path");
        }

        // Test simple directory name "src"
        if let Some(EntityType::Path(path)) = extractor.extract_path("列出 src 的内容") {
            assert_eq!(path, "src");
        } else {
            panic!("Expected to extract 'src' as path");
        }

        // Test simple directory name "tests"
        if let Some(EntityType::Path(path)) = extractor.extract_path("查找 tests 目录") {
            assert_eq!(path, "tests");
        } else {
            panic!("Expected to extract 'tests' as path");
        }
    }

    #[test]
    fn test_extract_path_with_trailing_slash() {
        let extractor = EntityExtractor::new();

        // Test directory name with trailing slash
        if let Some(EntityType::Path(path)) = extractor.extract_path("列出 docs/ 的内容") {
            assert_eq!(path, "docs/");
        } else {
            panic!("Expected to extract 'docs/' as path");
        }
    }

    #[test]
    fn test_extract_path_filters_keywords() {
        let extractor = EntityExtractor::new();

        // Should NOT extract command keywords as paths
        // "查看" is a command keyword, should be filtered out
        let result = extractor.extract_path("查看当前目录");
        if let Some(EntityType::Path(path)) = result {
            // Should return "." (from keyword detection), not "查看"
            assert_eq!(path, ".");
        } else {
            panic!("Expected to return '.' for current directory");
        }
    }

    #[test]
    fn test_extract_path_hyphenated_directory() {
        let extractor = EntityExtractor::new();

        // Test directory names with hyphens and underscores
        if let Some(EntityType::Path(path)) = extractor.extract_path("查看 test-dir 目录") {
            assert_eq!(path, "test-dir");
        } else {
            panic!("Expected to extract 'test-dir' as path");
        }

        if let Some(EntityType::Path(path)) = extractor.extract_path("列出 my_folder 的内容") {
            assert_eq!(path, "my_folder");
        } else {
            panic!("Expected to extract 'my_folder' as path");
        }
    }

    #[test]
    fn test_extract_number_integer() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Number(n)) = extractor.extract_number("查找大于 100 MB 的文件") {
            assert_eq!(n, 100.0);
        } else {
            panic!("Expected Number entity");
        }
    }

    #[test]
    fn test_extract_number_decimal() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Number(n)) = extractor.extract_number("阈值设置为 0.95") {
            assert_eq!(n, 0.95);
        } else {
            panic!("Expected Number entity");
        }
    }

    #[test]
    fn test_extract_date_today() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Date(d)) = extractor.extract_date("查找今天修改的文件") {
            assert_eq!(d, "today");
        } else {
            panic!("Expected Date entity");
        }
    }

    #[test]
    fn test_extract_date_recent() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Date(d)) = extractor.extract_date("查找最近修改的文件") {
            assert_eq!(d, "recent");
        } else {
            panic!("Expected Date entity");
        }
    }

    #[test]
    fn test_extract_date_iso() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Date(d)) = extractor.extract_date("查找 2025-10-14 修改的文件") {
            assert_eq!(d, "2025-10-14");
        } else {
            panic!("Expected Date entity");
        }
    }

    #[test]
    fn test_extract_with_expected() {
        let extractor = EntityExtractor::new();

        let mut expected = HashMap::new();
        expected.insert(
            "file_type".to_string(),
            EntityType::FileType(String::new()),
        );
        expected.insert("path".to_string(), EntityType::Path(String::new()));

        let entities = extractor.extract("统计 Python 代码在 ./src 目录", &expected);

        assert_eq!(entities.len(), 2);
        assert!(entities.contains_key("file_type"));
        assert!(entities.contains_key("path"));

        if let Some(EntityType::FileType(ft)) = entities.get("file_type") {
            assert_eq!(ft, "py");
        }

        if let Some(EntityType::Path(path)) = entities.get("path") {
            assert_eq!(path, "./src");
        }
    }

    #[test]
    fn test_extract_all() {
        let extractor = EntityExtractor::new();

        let entities = extractor.extract_all("统计 Python 代码在 ./src 目录，查找大于 100 行的文件");

        assert!(entities.contains_key("file_type"));
        assert!(entities.contains_key("operation"));
        assert!(entities.contains_key("path"));
        assert!(entities.contains_key("number"));
    }

    #[test]
    fn test_no_entities_found() {
        let extractor = EntityExtractor::new();

        let mut expected = HashMap::new();
        expected.insert(
            "file_type".to_string(),
            EntityType::FileType(String::new()),
        );

        let entities = extractor.extract("这是一段不包含任何实体的文本", &expected);

        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_case_insensitive_file_type() {
        let extractor = EntityExtractor::new();

        // Test uppercase
        if let Some(EntityType::FileType(ft)) = extractor.extract_file_type("统计 PYTHON 代码") {
            assert_eq!(ft, "py");
        }

        // Test mixed case
        if let Some(EntityType::FileType(ft)) = extractor.extract_file_type("统计 PyThOn 代码") {
            assert_eq!(ft, "py");
        }
    }

    #[test]
    fn test_multiple_numbers() {
        let extractor = EntityExtractor::new();

        // Should extract the first number
        if let Some(EntityType::Number(n)) = extractor.extract_number("查找 100 到 200 MB 的文件") {
            assert_eq!(n, 100.0);
        } else {
            panic!("Expected Number entity");
        }
    }

    #[test]
    fn test_extract_operation_english() {
        let extractor = EntityExtractor::new();

        if let Some(EntityType::Operation(op)) = extractor.extract_operation("count lines of code") {
            assert_eq!(op, "count");
        } else {
            panic!("Expected Operation entity");
        }
    }

    // ========== Phase 6.2.1: 排序方向提取测试 ==========

    #[test]
    fn test_extract_sort_direction_descending_chinese() {
        let extractor = EntityExtractor::new();

        // 测试"最大"关键词 → 降序
        if let Some(EntityType::Custom(type_name, dir)) =
            extractor.extract_sort_direction("显示当前目录下体积最大的rs文件")
        {
            assert_eq!(type_name, "sort");
            assert_eq!(dir, "-hr");
        } else {
            panic!("Expected Custom(sort, -hr) entity");
        }
    }

    #[test]
    fn test_extract_sort_direction_ascending_chinese() {
        let extractor = EntityExtractor::new();

        // 测试"最小"关键词 → 升序
        if let Some(EntityType::Custom(type_name, dir)) =
            extractor.extract_sort_direction("显示当前目录下体积最小的rs文件")
        {
            assert_eq!(type_name, "sort");
            assert_eq!(dir, "-h");
        } else {
            panic!("Expected Custom(sort, -h) entity");
        }
    }

    #[test]
    fn test_extract_sort_direction_descending_english() {
        let extractor = EntityExtractor::new();

        // 测试英文"largest"关键词 → 降序
        if let Some(EntityType::Custom(type_name, dir)) =
            extractor.extract_sort_direction("find the largest files")
        {
            assert_eq!(type_name, "sort");
            assert_eq!(dir, "-hr");
        } else {
            panic!("Expected Custom(sort, -hr) entity");
        }
    }

    #[test]
    fn test_extract_sort_direction_ascending_english() {
        let extractor = EntityExtractor::new();

        // 测试英文"smallest"关键词 → 升序
        if let Some(EntityType::Custom(type_name, dir)) =
            extractor.extract_sort_direction("find the smallest files")
        {
            assert_eq!(type_name, "sort");
            assert_eq!(dir, "-h");
        } else {
            panic!("Expected Custom(sort, -h) entity");
        }
    }

    #[test]
    fn test_extract_sort_direction_default() {
        let extractor = EntityExtractor::new();

        // 没有明确的排序关键词 → 默认降序
        if let Some(EntityType::Custom(type_name, dir)) =
            extractor.extract_sort_direction("显示rs文件")
        {
            assert_eq!(type_name, "sort");
            assert_eq!(dir, "-hr");
        } else {
            panic!("Expected Custom(sort, -hr) entity");
        }
    }

    #[test]
    fn test_extract_custom_sort_entity() {
        let extractor = EntityExtractor::new();

        // 测试通过 extract_custom 方法调用
        if let Some(EntityType::Custom(type_name, dir)) =
            extractor.extract_custom("显示最小的文件", "sort")
        {
            assert_eq!(type_name, "sort");
            assert_eq!(dir, "-h");
        } else {
            panic!("Expected Custom(sort, -h) entity");
        }
    }
}
