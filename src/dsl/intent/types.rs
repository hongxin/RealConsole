//! Intent DSL 核心数据结构
//!
//! 定义了意图识别系统的核心数据类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 意图定义
///
/// Intent 表示用户想要完成的任务，包含了用于识别该意图的关键信息。
///
/// # 示例
///
/// ```rust
/// use simpleconsole::dsl::intent::{Intent, IntentDomain};
/// use std::collections::HashMap;
///
/// let intent = Intent {
///     name: "count_python_lines".to_string(),
///     domain: IntentDomain::FileOps,
///     keywords: vec!["python".to_string(), "行数".to_string()],
///     patterns: vec![r"统计.*python.*行数".to_string()],
///     entities: HashMap::new(),
///     confidence_threshold: 0.5,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// 意图名称（唯一标识）
    pub name: String,

    /// 意图所属领域
    pub domain: IntentDomain,

    /// 关键词列表（用于简单匹配）
    pub keywords: Vec<String>,

    /// 正则表达式模式列表（用于复杂匹配）
    pub patterns: Vec<String>,

    /// 需要提取的实体类型
    pub entities: HashMap<String, EntityType>,

    /// 置信度阈值（0.0 - 1.0）
    pub confidence_threshold: f64,
}

/// 意图领域分类
///
/// 将意图按功能领域分类，便于管理和扩展。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentDomain {
    /// 文件操作（查找、统计、分析等）
    FileOps,

    /// 数据处理（过滤、排序、聚合等）
    DataOps,

    /// 诊断分析（错误分析、健康检查等）
    DiagnosticOps,

    /// 系统管理（清理、配置、监控等）
    SystemOps,

    /// 自定义领域
    Custom(String),
}

/// 实体类型
///
/// 表示从用户输入中提取的结构化信息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    /// 文件类型（如 "python", "rust"）
    FileType(String),

    /// 操作类型（如 "count", "find", "analyze"）
    Operation(String),

    /// 文件路径
    Path(String),

    /// 数值
    Number(f64),

    /// 日期字符串
    Date(String),

    /// 自定义实体（类型名, 值）
    Custom(String, String),
}

/// 意图识别结果
///
/// 包含了识别到的意图、置信度以及提取的实体信息。
///
/// # 示例
///
/// ```rust
/// use simpleconsole::dsl::intent::{IntentMatch, Intent, IntentDomain};
/// use std::collections::HashMap;
///
/// let intent = Intent {
///     name: "count_lines".to_string(),
///     domain: IntentDomain::FileOps,
///     keywords: vec!["统计".to_string()],
///     patterns: vec![],
///     entities: HashMap::new(),
///     confidence_threshold: 0.5,
/// };
///
/// let intent_match = IntentMatch {
///     intent,
///     confidence: 0.85,
///     matched_keywords: vec!["统计".to_string()],
///     extracted_entities: HashMap::new(),
/// };
///
/// assert!(intent_match.confidence > 0.8);
/// ```
#[derive(Debug, Clone)]
pub struct IntentMatch {
    /// 识别到的意图
    pub intent: Intent,

    /// 置信度（0.0 - 1.0）
    pub confidence: f64,

    /// 匹配的关键词列表
    pub matched_keywords: Vec<String>,

    /// 提取的实体信息
    pub extracted_entities: HashMap<String, EntityType>,
}

impl Intent {
    /// 创建一个新的意图
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain};
    ///
    /// let intent = Intent::new(
    ///     "count_lines",
    ///     IntentDomain::FileOps,
    ///     vec!["统计".to_string(), "行数".to_string()],
    ///     vec![r"统计.*行数".to_string()],
    ///     0.5,
    /// );
    /// ```
    pub fn new(
        name: impl Into<String>,
        domain: IntentDomain,
        keywords: Vec<String>,
        patterns: Vec<String>,
        confidence_threshold: f64,
    ) -> Self {
        Self {
            name: name.into(),
            domain,
            keywords,
            patterns,
            entities: HashMap::new(),
            confidence_threshold,
        }
    }

    /// 添加实体定义
    pub fn with_entity(mut self, name: impl Into<String>, entity_type: EntityType) -> Self {
        self.entities.insert(name.into(), entity_type);
        self
    }

    /// 检查置信度是否满足阈值
    pub fn meets_threshold(&self, confidence: f64) -> bool {
        confidence >= self.confidence_threshold
    }
}

impl IntentMatch {
    /// 创建一个新的意图匹配结果
    pub fn new(intent: Intent, confidence: f64) -> Self {
        Self {
            intent,
            confidence,
            matched_keywords: Vec::new(),
            extracted_entities: HashMap::new(),
        }
    }

    /// 添加匹配的关键词
    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.matched_keywords.push(keyword.into());
        self
    }

    /// 添加提取的实体
    pub fn with_entity(mut self, name: impl Into<String>, entity: EntityType) -> Self {
        self.extracted_entities.insert(name.into(), entity);
        self
    }

    /// 检查是否满足意图的置信度阈值
    pub fn meets_threshold(&self) -> bool {
        self.intent.meets_threshold(self.confidence)
    }
}

impl Default for IntentDomain {
    fn default() -> Self {
        Self::Custom("default".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(intent.domain, IntentDomain::FileOps);
        assert_eq!(intent.keywords.len(), 2);
        assert_eq!(intent.confidence_threshold, 0.5);
    }

    #[test]
    fn test_intent_builder() {
        let intent = Intent::new(
            "test_intent",
            IntentDomain::DataOps,
            vec!["test".to_string(), "keyword".to_string()],
            vec![r"test.*pattern".to_string()],
            0.6,
        );

        assert_eq!(intent.name, "test_intent");
        assert_eq!(intent.keywords.len(), 2);
        assert_eq!(intent.patterns.len(), 1);
        assert_eq!(intent.confidence_threshold, 0.6);
    }

    #[test]
    fn test_intent_with_entity() {
        let intent = Intent::new(
            "find_files",
            IntentDomain::FileOps,
            vec!["find".to_string()],
            Vec::new(),
            0.5,
        )
        .with_entity("file_type", EntityType::FileType("python".to_string()));

        assert_eq!(intent.entities.len(), 1);
        assert!(intent.entities.contains_key("file_type"));
    }

    #[test]
    fn test_intent_match_creation() {
        let intent = Intent::new(
            "count_lines",
            IntentDomain::FileOps,
            vec!["统计".to_string(), "行数".to_string()],
            Vec::new(),
            0.5,
        );

        let intent_match = IntentMatch::new(intent.clone(), 0.85);

        assert_eq!(intent_match.intent.name, "count_lines");
        assert_eq!(intent_match.confidence, 0.85);
        assert!(intent_match.matched_keywords.is_empty());
    }

    #[test]
    fn test_intent_match_builder() {
        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            Vec::new(),
            0.5,
        );

        let intent_match = IntentMatch::new(intent, 0.9)
            .with_keyword("test")
            .with_entity("file_type", EntityType::FileType("rust".to_string()));

        assert_eq!(intent_match.matched_keywords.len(), 1);
        assert_eq!(intent_match.extracted_entities.len(), 1);
    }

    #[test]
    fn test_confidence_threshold() {
        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            Vec::new(),
            Vec::new(),
            0.7,
        );

        assert!(intent.meets_threshold(0.7));
        assert!(intent.meets_threshold(0.8));
        assert!(!intent.meets_threshold(0.6));
    }

    #[test]
    fn test_intent_match_threshold() {
        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            Vec::new(),
            Vec::new(),
            0.7,
        );

        let match_high = IntentMatch::new(intent.clone(), 0.9);
        let match_low = IntentMatch::new(intent, 0.5);

        assert!(match_high.meets_threshold());
        assert!(!match_low.meets_threshold());
    }

    #[test]
    fn test_entity_types() {
        let file_type = EntityType::FileType("python".to_string());
        let operation = EntityType::Operation("count".to_string());
        let _path = EntityType::Path("/tmp".to_string());
        let number = EntityType::Number(42.0);
        let _date = EntityType::Date("2025-10-14".to_string());
        let _custom = EntityType::Custom("custom_type".to_string(), "custom_value".to_string());

        match file_type {
            EntityType::FileType(ref ft) => assert_eq!(ft, "python"),
            _ => panic!("Expected FileType"),
        }

        match operation {
            EntityType::Operation(ref op) => assert_eq!(op, "count"),
            _ => panic!("Expected Operation"),
        }

        match number {
            EntityType::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_intent_domain_custom() {
        let custom_domain = IntentDomain::Custom("MyDomain".to_string());

        match custom_domain {
            IntentDomain::Custom(ref name) => assert_eq!(name, "MyDomain"),
            _ => panic!("Expected Custom domain"),
        }
    }

    #[test]
    fn test_serde_serialization() {
        let intent = Intent::new(
            "test_intent",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            vec![r"test.*".to_string()],
            0.5,
        );

        // Test serialization
        let json = serde_json::to_string(&intent).unwrap();
        assert!(json.contains("test_intent"));

        // Test deserialization
        let deserialized: Intent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test_intent");
        assert_eq!(deserialized.domain, IntentDomain::FileOps);
    }
}
