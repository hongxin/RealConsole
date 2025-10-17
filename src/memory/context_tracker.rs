//! 上下文追踪器 - Phase 9.1 智能化基础
//!
//! 功能：
//! - 记录最近提到的实体（文件、目录、命令等）
//! - 指代消解（"它"、"这个文件"等）
//! - 工作上下文管理
//! - 跨对话的实体跟踪
//!
//! 设计理念（一分为三）：
//! - 提取层：从用户输入中提取实体
//! - 存储层：LRU 缓存管理最近实体
//! - 消解层：将指代词映射到具体实体

use chrono::{DateTime, Utc};
use lru::LruCache;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;

// LLM 增强功能
use crate::llm::{LlmClient, Message, LlmError};
use serde_json::Value as JsonValue;

/// 实体类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    /// 文件路径
    File(PathBuf),

    /// 目录路径
    Directory(PathBuf),

    /// 命令（Shell 命令或系统命令）
    Command(String),

    /// 变量（名称-值对）
    Variable { name: String, value: String },

    /// 概念（抽象概念，如"日志"、"错误"）
    Concept(String),

    /// 数字值
    Number(f64),

    /// URL
    Url(String),
}

impl EntityType {
    /// 获取实体的显示名称
    pub fn display_name(&self) -> String {
        match self {
            EntityType::File(path) => path.display().to_string(),
            EntityType::Directory(path) => path.display().to_string(),
            EntityType::Command(cmd) => cmd.clone(),
            EntityType::Variable { name, value } => format!("{} = {}", name, value),
            EntityType::Concept(concept) => concept.clone(),
            EntityType::Number(n) => n.to_string(),
            EntityType::Url(url) => url.clone(),
        }
    }

    /// 获取实体的类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            EntityType::File(_) => "文件",
            EntityType::Directory(_) => "目录",
            EntityType::Command(_) => "命令",
            EntityType::Variable { .. } => "变量",
            EntityType::Concept(_) => "概念",
            EntityType::Number(_) => "数字",
            EntityType::Url(_) => "URL",
        }
    }
}

/// 实体记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体类型
    pub entity_type: EntityType,

    /// 提及时间
    pub mentioned_at: DateTime<Utc>,

    /// 提及次数
    pub mention_count: u32,

    /// 上下文信息（用户输入的完整句子）
    pub context: String,

    /// 置信度（0.0-1.0）
    pub confidence: f32,
}

impl Entity {
    /// 创建新实体
    pub fn new(entity_type: EntityType, context: String, confidence: f32) -> Self {
        Self {
            entity_type,
            mentioned_at: Utc::now(),
            mention_count: 1,
            context,
            confidence,
        }
    }

    /// 更新提及信息
    pub fn update_mention(&mut self, context: String) {
        self.mention_count += 1;
        self.mentioned_at = Utc::now();
        self.context = context;
    }
}

/// 指代记录
#[derive(Debug, Clone)]
pub struct ReferenceRecord {
    /// 指代词（如"它"、"这个"）
    pub pronoun: String,

    /// 解析到的实体
    pub resolved_entity: Entity,

    /// 解析时间
    pub resolved_at: DateTime<Utc>,
}

/// 工作上下文
#[derive(Debug, Clone, Default)]
pub struct WorkingContext {
    /// 当前工作目录
    pub current_directory: Option<PathBuf>,

    /// 最近操作的文件
    pub last_file: Option<PathBuf>,

    /// 最近执行的命令
    pub last_command: Option<String>,

    /// 活跃的变量
    pub active_variables: HashMap<String, String>,

    /// 当前任务描述
    pub current_task: Option<String>,
}

/// 上下文追踪器
pub struct ContextTracker {
    /// 最近提到的实体（LRU 缓存，最多保留 50 个）
    recent_entities: LruCache<String, Entity>,

    /// 工作上下文
    working_context: WorkingContext,

    /// 指代历史（最近 20 条）
    reference_history: Vec<ReferenceRecord>,

    /// 最大指代历史数量
    max_reference_history: usize,

    /// 实体提取器（正则表达式）
    extractor: EntityExtractor,
}

impl ContextTracker {
    /// 创建新的上下文追踪器
    pub fn new() -> Self {
        Self {
            recent_entities: LruCache::new(NonZeroUsize::new(50).unwrap()),
            working_context: WorkingContext::default(),
            reference_history: Vec::new(),
            max_reference_history: 20,
            extractor: EntityExtractor::new(),
        }
    }

    /// 从用户输入中提取实体
    pub fn extract_entities(&mut self, user_input: &str) -> Vec<Entity> {
        self.extractor.extract(user_input)
    }

    /// 记录实体
    pub fn record_entity(&mut self, entity: Entity) {
        let key = entity.entity_type.display_name();

        // 检查是否已存在
        if let Some(existing) = self.recent_entities.get_mut(&key) {
            existing.update_mention(entity.context.clone());
        } else {
            self.recent_entities.put(key, entity);
        }
    }

    /// 批量记录实体
    pub fn record_entities(&mut self, entities: Vec<Entity>) {
        for entity in entities {
            self.record_entity(entity);
        }
    }

    /// 解析指代词
    ///
    /// # 参数
    /// - `pronoun`: 指代词（如"它"、"这个"、"那个"）
    ///
    /// # 返回
    /// - Some(Entity): 解析成功
    /// - None: 无法解析
    pub fn resolve_reference(&mut self, pronoun: &str) -> Option<Entity> {
        // 简单策略：返回最近提到的实体
        let pronoun_lower = pronoun.to_lowercase();

        // 特定指代词映射
        let entity = match pronoun_lower.as_str() {
            "它" | "it" | "this" | "that" => {
                // 返回最近的实体
                self.get_most_recent_entity()
            }
            "这个文件" | "该文件" | "this file" => {
                // 返回最近的文件
                self.get_most_recent_entity_by_type("文件")
            }
            "这个目录" | "该目录" | "this directory" => {
                // 返回最近的目录
                self.get_most_recent_entity_by_type("目录")
            }
            "上一个命令" | "刚才的命令" | "last command" => {
                // 返回最近的命令
                self.get_most_recent_entity_by_type("命令")
            }
            _ => {
                // 尝试从工作上下文获取
                self.resolve_from_working_context(pronoun)
            }
        };

        // 记录指代解析历史
        if let Some(ref resolved) = entity {
            let record = ReferenceRecord {
                pronoun: pronoun.to_string(),
                resolved_entity: resolved.clone(),
                resolved_at: Utc::now(),
            };

            self.reference_history.push(record);

            // 限制历史数量
            if self.reference_history.len() > self.max_reference_history {
                self.reference_history.remove(0);
            }
        }

        entity
    }

    /// 获取最近的实体
    fn get_most_recent_entity(&mut self) -> Option<Entity> {
        // LruCache 会自动把最近访问的移到前面
        self.recent_entities.iter().next().map(|(_, e)| e.clone())
    }

    /// 按类型获取最近的实体
    fn get_most_recent_entity_by_type(&mut self, type_name: &str) -> Option<Entity> {
        self.recent_entities
            .iter()
            .find(|(_, e)| e.entity_type.type_name() == type_name)
            .map(|(_, e)| e.clone())
    }

    /// 从工作上下文解析
    fn resolve_from_working_context(&self, _pronoun: &str) -> Option<Entity> {
        // 尝试从工作上下文获取
        if let Some(ref file) = self.working_context.last_file {
            return Some(Entity::new(
                EntityType::File(file.clone()),
                "工作上下文".to_string(),
                0.8,
            ));
        }

        None
    }

    /// 更新工作上下文
    pub fn update_working_context(&mut self, context_update: WorkingContextUpdate) {
        match context_update {
            WorkingContextUpdate::CurrentDirectory(dir) => {
                self.working_context.current_directory = Some(dir);
            }
            WorkingContextUpdate::LastFile(file) => {
                self.working_context.last_file = Some(file);
            }
            WorkingContextUpdate::LastCommand(cmd) => {
                self.working_context.last_command = Some(cmd);
            }
            WorkingContextUpdate::Variable { name, value } => {
                self.working_context.active_variables.insert(name, value);
            }
            WorkingContextUpdate::CurrentTask(task) => {
                self.working_context.current_task = Some(task);
            }
        }
    }

    /// 获取所有实体（按最近使用顺序）
    pub fn get_all_entities(&mut self) -> Vec<Entity> {
        self.recent_entities.iter().map(|(_, e)| e.clone()).collect()
    }

    /// 清除所有实体
    pub fn clear(&mut self) {
        self.recent_entities.clear();
        self.reference_history.clear();
        self.working_context = WorkingContext::default();
    }

    /// 获取统计信息
    pub fn stats(&self) -> ContextStats {
        ContextStats {
            total_entities: self.recent_entities.len(),
            total_references: self.reference_history.len(),
            working_context_active: self.working_context.current_task.is_some(),
        }
    }

    // ============================================================================
    // LLM 增强功能 (Phase 9.1 Day 3-4)
    // ============================================================================

    /// 使用 LLM 增强的实体提取
    ///
    /// 相比正则表达式，LLM 可以识别：
    /// - 抽象概念（如"性能问题"、"内存泄漏"）
    /// - 任务描述（如"优化数据库查询"）
    /// - 复杂的上下文相关实体
    ///
    /// # 参数
    /// - `user_input`: 用户输入
    /// - `llm`: LLM 客户端
    ///
    /// # 返回
    /// - `Ok(Vec<Entity>)`: 提取的实体列表
    /// - `Err(LlmError)`: LLM 调用失败
    pub async fn extract_entities_with_llm(
        &self,
        user_input: &str,
        llm: &dyn LlmClient,
    ) -> Result<Vec<Entity>, LlmError> {
        let prompt = format!(
            r#"从下面的用户输入中提取关键实体。返回 JSON 格式的实体列表。

用户输入："{}"

请提取以下类型的实体：
- 文件路径（如 "src/main.rs"）
- 目录路径（如 "/tmp/"）
- 命令（如 "cargo build"）
- 概念（如 "性能问题"、"内存泄漏"）
- URL（如 "https://example.com"）
- 数字（如 "100", "3.14"）

返回格式（JSON 数组）：
[
  {{"type": "file", "value": "src/main.rs", "confidence": 0.9}},
  {{"type": "concept", "value": "性能问题", "confidence": 0.8}}
]

只返回 JSON，不要其他解释。"#,
            user_input
        );

        let messages = vec![Message::user(prompt)];
        let response = llm.chat(messages).await?;

        // 解析 JSON 响应
        self.parse_llm_entities_response(&response, user_input)
    }

    /// 解析 LLM 返回的实体 JSON
    fn parse_llm_entities_response(
        &self,
        response: &str,
        context: &str,
    ) -> Result<Vec<Entity>, LlmError> {
        // 尝试提取 JSON 数组（可能被包裹在其他文本中）
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        let entities_json: Vec<JsonValue> = serde_json::from_str(json_str)
            .map_err(|e| LlmError::Parse(format!("JSON 解析失败: {}", e)))?;

        let mut entities = Vec::new();

        for entity_json in entities_json {
            let entity_type = entity_json["type"].as_str().unwrap_or("unknown");
            let value = entity_json["value"].as_str().unwrap_or("");
            let confidence = entity_json["confidence"].as_f64().unwrap_or(0.5) as f32;

            let entity_type = match entity_type {
                "file" => EntityType::File(PathBuf::from(value)),
                "directory" | "dir" => EntityType::Directory(PathBuf::from(value)),
                "command" | "cmd" => EntityType::Command(value.to_string()),
                "concept" => EntityType::Concept(value.to_string()),
                "url" => EntityType::Url(value.to_string()),
                "number" => {
                    if let Ok(num) = value.parse::<f64>() {
                        EntityType::Number(num)
                    } else {
                        continue;
                    }
                }
                _ => EntityType::Concept(value.to_string()),
            };

            entities.push(Entity::new(entity_type, context.to_string(), confidence));
        }

        Ok(entities)
    }

    /// 使用 LLM 分析上下文相关性
    ///
    /// 分析当前用户输入与历史实体的相关性，返回最相关的实体
    ///
    /// # 参数
    /// - `user_input`: 用户输入
    /// - `llm`: LLM 客户端
    ///
    /// # 返回
    /// - `Ok(Vec<(Entity, f32)>)`: 实体及其相关性分数（0.0-1.0）
    /// - `Err(LlmError)`: LLM 调用失败
    pub async fn analyze_context_relevance(
        &mut self,
        user_input: &str,
        llm: &dyn LlmClient,
    ) -> Result<Vec<(Entity, f32)>, LlmError> {
        if self.recent_entities.is_empty() {
            return Ok(Vec::new());
        }

        // 构建历史实体列表
        let entities_list: Vec<String> = self
            .recent_entities
            .iter()
            .map(|(_, e)| format!("- {} ({})", e.entity_type.display_name(), e.entity_type.type_name()))
            .collect();

        let prompt = format!(
            r#"分析用户当前输入与历史实体的相关性。

用户当前输入："{}"

历史实体：
{}

请为每个实体评估相关性（0.0-1.0），并返回 JSON 格式：
[
  {{"entity": "src/main.rs", "relevance": 0.9}},
  {{"entity": "/tmp/", "relevance": 0.3}}
]

只返回相关性 > 0.5 的实体。只返回 JSON，不要其他解释。"#,
            user_input,
            entities_list.join("\n")
        );

        let messages = vec![Message::user(prompt)];
        let response = llm.chat(messages).await?;

        // 解析响应并匹配实体
        self.parse_relevance_response(&response)
    }

    /// 解析相关性分析响应
    fn parse_relevance_response(&mut self, response: &str) -> Result<Vec<(Entity, f32)>, LlmError> {
        // 提取 JSON
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        let relevance_json: Vec<JsonValue> = serde_json::from_str(json_str)
            .map_err(|e| LlmError::Parse(format!("JSON 解析失败: {}", e)))?;

        let mut results = Vec::new();

        for item in relevance_json {
            let entity_name = item["entity"].as_str().unwrap_or("");
            let relevance = item["relevance"].as_f64().unwrap_or(0.0) as f32;

            // 在缓存中查找匹配的实体
            if let Some(entity) = self.recent_entities.get(entity_name) {
                results.push((entity.clone(), relevance));
            }
        }

        Ok(results)
    }

    /// 使用 LLM 增强的指代消解
    ///
    /// 对于复杂的指代关系，使用 LLM 理解上下文进行消解
    ///
    /// # 参数
    /// - `user_input`: 包含指代词的用户输入
    /// - `llm`: LLM 客户端
    ///
    /// # 返回
    /// - `Ok(Option<Entity>)`: 解析的实体
    /// - `Err(LlmError)`: LLM 调用失败
    pub async fn resolve_reference_with_llm(
        &mut self,
        user_input: &str,
        llm: &dyn LlmClient,
    ) -> Result<Option<Entity>, LlmError> {
        if self.recent_entities.is_empty() {
            return Ok(None);
        }

        // 构建上下文
        let entities_context: Vec<String> = self
            .recent_entities
            .iter()
            .take(10) // 只取最近 10 个
            .map(|(_, e)| {
                format!(
                    "- {} ({}): {}",
                    e.entity_type.display_name(),
                    e.entity_type.type_name(),
                    e.context
                )
            })
            .collect();

        let prompt = format!(
            r#"用户输入包含指代词，请根据对话历史识别指代的实体。

用户输入："{}"

最近提到的实体：
{}

请识别用户输入中的指代词指向哪个实体，返回 JSON 格式：
{{
  "pronoun": "它",
  "refers_to": "src/main.rs",
  "confidence": 0.9
}}

如果无法确定，返回：{{"refers_to": null}}

只返回 JSON，不要其他解释。"#,
            user_input,
            entities_context.join("\n")
        );

        let messages = vec![Message::user(prompt)];
        let response = llm.chat(messages).await?;

        // 解析响应
        self.parse_reference_response(&response, user_input)
    }

    /// 解析指代消解响应
    fn parse_reference_response(
        &mut self,
        response: &str,
        context: &str,
    ) -> Result<Option<Entity>, LlmError> {
        // 提取 JSON
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        let result: JsonValue = serde_json::from_str(json_str)
            .map_err(|e| LlmError::Parse(format!("JSON 解析失败: {}", e)))?;

        if let Some(refers_to) = result["refers_to"].as_str() {
            // 查找匹配的实体
            if let Some(entity) = self.recent_entities.get(refers_to) {
                // 更新提及次数
                let mut updated_entity = entity.clone();
                updated_entity.update_mention(context.to_string());
                self.recent_entities.put(refers_to.to_string(), updated_entity.clone());

                return Ok(Some(updated_entity));
            }
        }

        Ok(None)
    }
}

impl Default for ContextTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 工作上下文更新
#[derive(Debug, Clone)]
pub enum WorkingContextUpdate {
    CurrentDirectory(PathBuf),
    LastFile(PathBuf),
    LastCommand(String),
    Variable { name: String, value: String },
    CurrentTask(String),
}

/// 上下文统计
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub total_entities: usize,
    pub total_references: usize,
    pub working_context_active: bool,
}

/// 实体提取器
struct EntityExtractor {
    /// 文件路径正则
    file_regex: Regex,

    /// 目录路径正则
    dir_regex: Regex,

    /// URL 正则
    url_regex: Regex,

    /// 数字正则
    number_regex: Regex,
}

impl EntityExtractor {
    fn new() -> Self {
        Self {
            // 匹配文件路径（绝对路径或相对路径，带扩展名）
            file_regex: Regex::new(r"(?:^|[\s,;])((?:[./~])?(?:[a-zA-Z0-9_-]+/)*[a-zA-Z0-9_-]+\.[a-zA-Z0-9]+)").unwrap(),

            // 匹配目录路径（以 / 结尾或常见目录名）
            dir_regex: Regex::new(r"(?:^|[\s,;])((?:[./~])?(?:[a-zA-Z0-9_-]+/)+)").unwrap(),

            // 匹配 URL
            url_regex: Regex::new(r"https?://[^\s]+").unwrap(),

            // 匹配数字
            number_regex: Regex::new(r"\b\d+(?:\.\d+)?\b").unwrap(),
        }
    }

    fn extract(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        // 提取文件
        for cap in self.file_regex.captures_iter(text) {
            if let Some(path_str) = cap.get(1) {
                let path = PathBuf::from(path_str.as_str());
                entities.push(Entity::new(
                    EntityType::File(path),
                    text.to_string(),
                    0.9,
                ));
            }
        }

        // 提取目录
        for cap in self.dir_regex.captures_iter(text) {
            if let Some(path_str) = cap.get(1) {
                let path = PathBuf::from(path_str.as_str());
                // 避免重复（文件路径可能包含目录）
                if !entities.iter().any(|e| matches!(&e.entity_type, EntityType::File(p) if p.starts_with(&path))) {
                    entities.push(Entity::new(
                        EntityType::Directory(path),
                        text.to_string(),
                        0.8,
                    ));
                }
            }
        }

        // 提取 URL
        for cap in self.url_regex.captures_iter(text) {
            entities.push(Entity::new(
                EntityType::Url(cap.get(0).unwrap().as_str().to_string()),
                text.to_string(),
                0.95,
            ));
        }

        // 提取数字（置信度较低）
        for cap in self.number_regex.captures_iter(text) {
            if let Ok(num) = cap.get(0).unwrap().as_str().parse::<f64>() {
                entities.push(Entity::new(
                    EntityType::Number(num),
                    text.to_string(),
                    0.6,
                ));
            }
        }

        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_display_name() {
        let file = EntityType::File(PathBuf::from("/tmp/test.txt"));
        assert_eq!(file.display_name(), "/tmp/test.txt");

        let var = EntityType::Variable {
            name: "name".to_string(),
            value: "Alice".to_string(),
        };
        assert_eq!(var.display_name(), "name = Alice");
    }

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(
            EntityType::File(PathBuf::from("test.txt")),
            "查看 test.txt".to_string(),
            0.9,
        );

        assert_eq!(entity.mention_count, 1);
        assert_eq!(entity.confidence, 0.9);
    }

    #[test]
    fn test_context_tracker_record_entity() {
        let mut tracker = ContextTracker::new();

        let entity = Entity::new(
            EntityType::File(PathBuf::from("main.rs")),
            "查看 main.rs".to_string(),
            0.9,
        );

        tracker.record_entity(entity);

        assert_eq!(tracker.stats().total_entities, 1);
    }

    #[test]
    fn test_context_tracker_duplicate_entity() {
        let mut tracker = ContextTracker::new();

        let entity1 = Entity::new(
            EntityType::File(PathBuf::from("test.txt")),
            "第一次提及".to_string(),
            0.9,
        );

        let entity2 = Entity::new(
            EntityType::File(PathBuf::from("test.txt")),
            "第二次提及".to_string(),
            0.9,
        );

        tracker.record_entity(entity1);
        tracker.record_entity(entity2);

        // 应该只有一个实体，但提及次数为 2
        assert_eq!(tracker.stats().total_entities, 1);

        let entities = tracker.get_all_entities();
        assert_eq!(entities[0].mention_count, 2);
    }

    #[test]
    fn test_resolve_reference() {
        let mut tracker = ContextTracker::new();

        // 记录一个文件
        let file_entity = Entity::new(
            EntityType::File(PathBuf::from("config.yaml")),
            "查看 config.yaml".to_string(),
            0.9,
        );
        tracker.record_entity(file_entity);

        // 解析"它"
        let resolved = tracker.resolve_reference("它");
        assert!(resolved.is_some());

        let entity = resolved.unwrap();
        assert!(matches!(entity.entity_type, EntityType::File(_)));

        // 检查指代历史
        assert_eq!(tracker.reference_history.len(), 1);
        assert_eq!(tracker.reference_history[0].pronoun, "它");
    }

    #[test]
    fn test_resolve_specific_reference() {
        let mut tracker = ContextTracker::new();

        // 记录多个实体
        tracker.record_entity(Entity::new(
            EntityType::File(PathBuf::from("test.txt")),
            "test".to_string(),
            0.9,
        ));

        tracker.record_entity(Entity::new(
            EntityType::Directory(PathBuf::from("/tmp/")),
            "dir".to_string(),
            0.9,
        ));

        // 解析"这个文件"应该返回文件
        let resolved = tracker.resolve_reference("这个文件");
        assert!(resolved.is_some());

        let entity = resolved.unwrap();
        assert!(matches!(entity.entity_type, EntityType::File(_)));
    }

    #[test]
    fn test_entity_extractor() {
        let extractor = EntityExtractor::new();

        let text = "请查看 src/main.rs 文件，位于 /home/user/ 目录";
        let entities = extractor.extract(text);

        // 应该提取到文件和目录
        assert!(!entities.is_empty());

        // 检查是否有文件实体
        let has_file = entities.iter().any(|e| matches!(&e.entity_type, EntityType::File(_)));
        assert!(has_file);
    }

    #[test]
    fn test_entity_extractor_url() {
        let extractor = EntityExtractor::new();

        let text = "访问 https://example.com 获取更多信息";
        let entities = extractor.extract(text);

        let has_url = entities.iter().any(|e| matches!(&e.entity_type, EntityType::Url(_)));
        assert!(has_url);
    }

    #[test]
    fn test_working_context_update() {
        let mut tracker = ContextTracker::new();

        tracker.update_working_context(WorkingContextUpdate::CurrentDirectory(
            PathBuf::from("/home/user"),
        ));

        tracker.update_working_context(WorkingContextUpdate::Variable {
            name: "PATH".to_string(),
            value: "/usr/bin".to_string(),
        });

        assert!(tracker.working_context.current_directory.is_some());
        assert_eq!(
            tracker.working_context.active_variables.get("PATH"),
            Some(&"/usr/bin".to_string())
        );
    }

    #[test]
    fn test_context_tracker_clear() {
        let mut tracker = ContextTracker::new();

        tracker.record_entity(Entity::new(
            EntityType::File(PathBuf::from("test.txt")),
            "test".to_string(),
            0.9,
        ));

        tracker.clear();

        assert_eq!(tracker.stats().total_entities, 0);
        assert_eq!(tracker.stats().total_references, 0);
    }
}
