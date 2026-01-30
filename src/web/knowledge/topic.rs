//! 主题提取器
//!
//! 从对话消息中自动提取主题、关键决策点和代码变更

use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

use super::types::*;

/// 主题提取器
pub struct TopicExtractor {
    /// 停用词列表
    stop_words: HashSet<String>,
    /// 最小主题消息数
    min_topic_messages: usize,
    /// 时间聚类窗口（小时）
    time_window_hours: i64,
}

impl Default for TopicExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicExtractor {
    /// 创建主题提取器
    pub fn new() -> Self {
        let stop_words: HashSet<String> = [
            // 中文停用词
            "的", "是", "在", "了", "和", "与", "这", "那", "有", "我", "你", "他",
            "她", "它", "们", "个", "一", "不", "也", "就", "都", "要", "会", "能",
            "可以", "这个", "那个", "什么", "怎么", "为什么", "如何", "好的", "可以",
            // 英文停用词
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "need", "dare",
            "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
            "into", "through", "during", "before", "after", "above", "below",
            "this", "that", "these", "those", "i", "you", "he", "she", "it",
            "we", "they", "what", "which", "who", "when", "where", "why", "how",
            "let", "me", "please", "ok", "okay", "yes", "no", "and", "or", "but",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            stop_words,
            min_topic_messages: 3,
            time_window_hours: 4,
        }
    }

    /// 提取主题
    pub fn extract_topics(&self, messages: &[ConversationMessage]) -> Vec<Topic> {
        if messages.is_empty() {
            return Vec::new();
        }

        // 按时间排序
        let mut sorted_messages: Vec<_> = messages.iter().collect();
        sorted_messages.sort_by_key(|m| m.timestamp);

        // 时间聚类：将时间相近的消息分组
        let clusters = self.cluster_by_time(&sorted_messages);

        // 为每个聚类提取关键词和生成主题
        let mut topics = Vec::new();
        for (cluster_idx, cluster) in clusters.iter().enumerate() {
            if cluster.len() < self.min_topic_messages {
                continue;
            }

            // 提取关键词
            let keywords = self.extract_keywords(cluster);
            if keywords.is_empty() {
                continue;
            }

            // 生成主题名称
            let name = self.generate_topic_name(&keywords, cluster);

            // 获取时间范围
            let timestamps: Vec<_> = cluster.iter().map(|m| m.timestamp).collect();
            let time_range = (
                *timestamps.iter().min().unwrap(),
                *timestamps.iter().max().unwrap(),
            );

            // 生成摘要
            let summary = self.generate_summary(cluster);

            let topic = Topic {
                id: format!("topic-{}", cluster_idx),
                name,
                keywords,
                message_ids: cluster.iter().map(|m| m.id.clone()).collect(),
                time_range,
                summary,
                message_count: cluster.len(),
            };

            topics.push(topic);
        }

        topics
    }

    /// 识别关键决策点
    pub fn identify_decisions(&self, messages: &[ConversationMessage]) -> Vec<Decision> {
        let mut decisions = Vec::new();

        // 决策指示词
        let decision_indicators = [
            // 中文
            "决定", "选择", "采用", "使用", "改为", "重构", "设计",
            "架构", "方案", "策略", "实现", "迁移",
            // 英文
            "decide", "choose", "adopt", "use", "change to", "refactor",
            "design", "architecture", "approach", "strategy", "implement",
            "migrate",
        ];

        for msg in messages {
            if msg.message_type != MessageType::Assistant {
                continue;
            }

            let content_lower = msg.content.to_lowercase();

            // 检查是否包含决策指示词
            for indicator in &decision_indicators {
                if content_lower.contains(indicator) {
                    // 判断决策类型
                    let decision_type = self.classify_decision_type(&content_lower);

                    // 提取受影响的文件
                    let affected_files = self.extract_file_paths(&msg.content);

                    // 生成决策描述
                    let description = self.extract_decision_description(&msg.content, indicator);

                    let decision = Decision {
                        id: format!("decision-{}", msg.id),
                        description,
                        message_id: msg.id.clone(),
                        timestamp: msg.timestamp,
                        decision_type,
                        affected_files,
                    };

                    decisions.push(decision);
                    break; // 每条消息只提取一个决策
                }
            }
        }

        decisions
    }

    /// 提取代码变更关联
    pub fn extract_code_changes(&self, messages: &[ConversationMessage]) -> Vec<CodeChange> {
        let mut changes = Vec::new();

        for msg in messages {
            // 只处理助手消息
            if msg.message_type != MessageType::Assistant {
                continue;
            }

            // 检查工具使用
            for tool in &msg.tools_used {
                let change_type = match tool.as_str() {
                    "Write" => ChangeType::Create,
                    "Edit" => ChangeType::Modify,
                    "Read" => continue, // 读取不算变更
                    _ => continue,
                };

                // 尝试从内容中提取文件路径
                let file_paths = self.extract_file_paths(&msg.content);

                for file_path in file_paths {
                    let change = CodeChange {
                        id: format!("change-{}-{}", msg.id, changes.len()),
                        file_path,
                        change_type: change_type.clone(),
                        message_id: msg.id.clone(),
                        timestamp: msg.timestamp,
                        description: String::new(),
                    };
                    changes.push(change);
                }
            }
        }

        changes
    }

    /// 按时间聚类消息
    fn cluster_by_time<'a>(&self, messages: &[&'a ConversationMessage]) -> Vec<Vec<&'a ConversationMessage>> {
        if messages.is_empty() {
            return Vec::new();
        }

        let window = Duration::hours(self.time_window_hours);
        let mut clusters: Vec<Vec<&ConversationMessage>> = Vec::new();
        let mut current_cluster: Vec<&ConversationMessage> = vec![messages[0]];

        for msg in messages.iter().skip(1) {
            let last_time = current_cluster.last().unwrap().timestamp;
            if msg.timestamp - last_time <= window {
                current_cluster.push(msg);
            } else {
                if !current_cluster.is_empty() {
                    clusters.push(current_cluster);
                }
                current_cluster = vec![msg];
            }
        }

        if !current_cluster.is_empty() {
            clusters.push(current_cluster);
        }

        clusters
    }

    /// 提取关键词
    fn extract_keywords(&self, messages: &[&ConversationMessage]) -> Vec<String> {
        let mut word_freq: HashMap<String, usize> = HashMap::new();

        for msg in messages {
            let words = self.tokenize(&msg.content);
            for word in words {
                if !self.stop_words.contains(&word.to_lowercase()) && word.len() >= 2 {
                    *word_freq.entry(word).or_insert(0) += 1;
                }
            }
        }

        // 按频率排序，取前 10 个
        let mut sorted: Vec<_> = word_freq.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        sorted.into_iter().take(10).map(|(word, _)| word).collect()
    }

    /// 简单分词
    fn tokenize(&self, text: &str) -> Vec<String> {
        let mut words = Vec::new();

        // 英文分词（按空格和标点）
        for word in text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
            let word = word.trim();
            if !word.is_empty() {
                words.push(word.to_string());
            }
        }

        // 中文分词（简单按字符切分，后续可改进）
        // TODO: 使用更好的中文分词库

        words
    }

    /// 生成主题名称
    fn generate_topic_name(&self, keywords: &[String], messages: &[&ConversationMessage]) -> String {
        // 尝试从用户消息中提取简短描述
        for msg in messages {
            if msg.message_type == MessageType::User {
                let content = &msg.content;
                // 取前 50 个字符作为名称
                if content.len() <= 50 {
                    return content.clone();
                } else {
                    return format!("{}...", &content[..50]);
                }
            }
        }

        // 回退到关键词组合
        if keywords.len() >= 2 {
            format!("{} - {}", keywords[0], keywords[1])
        } else if !keywords.is_empty() {
            keywords[0].clone()
        } else {
            "未命名主题".to_string()
        }
    }

    /// 生成摘要
    fn generate_summary(&self, messages: &[&ConversationMessage]) -> String {
        // 简单策略：取第一条用户消息作为摘要
        for msg in messages {
            if msg.message_type == MessageType::User {
                let content = &msg.content;
                if content.len() <= 200 {
                    return content.clone();
                } else {
                    return format!("{}...", &content[..200]);
                }
            }
        }
        String::new()
    }

    /// 分类决策类型
    fn classify_decision_type(&self, content: &str) -> DecisionType {
        if content.contains("架构") || content.contains("architecture") || content.contains("设计") {
            DecisionType::Architecture
        } else if content.contains("重构") || content.contains("refactor") {
            DecisionType::Refactoring
        } else if content.contains("修复") || content.contains("fix") || content.contains("bug") {
            DecisionType::BugFix
        } else if content.contains("选择") || content.contains("采用") || content.contains("use") {
            DecisionType::TechChoice
        } else {
            DecisionType::Implementation
        }
    }

    /// 提取文件路径
    fn extract_file_paths(&self, content: &str) -> Vec<String> {
        let mut paths = Vec::new();

        // 匹配常见的文件路径模式
        // src/xxx.rs, docs/xxx.md, etc.
        let path_pattern = regex::Regex::new(r"(?:src|docs|tests?|lib|bin)/[\w/\-\.]+\.\w+").unwrap();

        for cap in path_pattern.find_iter(content) {
            paths.push(cap.as_str().to_string());
        }

        paths
    }

    /// 提取决策描述
    fn extract_decision_description(&self, content: &str, indicator: &str) -> String {
        // 找到包含指示词的句子
        for sentence in content.split(&['.', '。', '!', '！', '?', '？', '\n'][..]) {
            if sentence.to_lowercase().contains(indicator) {
                let trimmed = sentence.trim();
                if trimmed.len() <= 150 {
                    return trimmed.to_string();
                } else {
                    return format!("{}...", &trimmed[..150]);
                }
            }
        }
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_message(content: &str, msg_type: MessageType) -> ConversationMessage {
        ConversationMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: msg_type,
            content: content.to_string(),
            thinking: None,
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            tools_used: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_extract_keywords() {
        let extractor = TopicExtractor::new();
        let messages = vec![
            create_test_message("实现 Storage Layer 2.0 架构设计", MessageType::User),
            create_test_message("好的，让我设计 Storage Layer 的分层架构", MessageType::Assistant),
        ];

        let refs: Vec<_> = messages.iter().collect();
        let keywords = extractor.extract_keywords(&refs);

        assert!(!keywords.is_empty());
        // Storage, Layer 应该在关键词中
        assert!(keywords.iter().any(|k| k.to_lowercase().contains("storage")));
    }

    #[test]
    fn test_classify_decision_type() {
        let extractor = TopicExtractor::new();

        assert_eq!(
            extractor.classify_decision_type("决定采用分层架构设计"),
            DecisionType::Architecture
        );
        assert_eq!(
            extractor.classify_decision_type("需要重构这个模块"),
            DecisionType::Refactoring
        );
        assert_eq!(
            extractor.classify_decision_type("修复这个 bug"),
            DecisionType::BugFix
        );
    }
}
