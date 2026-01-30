//! 知识检索引擎
//!
//! 支持关键词搜索、时间范围查询、主题浏览等

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::types::*;

/// 知识检索引擎
pub struct KnowledgeRetrieval {
    /// 知识库数据
    knowledge_base: KnowledgeBase,
    /// 消息索引（按 ID）
    message_index: HashMap<String, usize>,
    /// 会话索引（按 ID）
    session_index: HashMap<String, usize>,
}

impl KnowledgeRetrieval {
    /// 创建检索引擎
    pub fn new(knowledge_base: KnowledgeBase) -> Self {
        // 构建消息索引
        let message_index: HashMap<String, usize> = knowledge_base
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id.clone(), i))
            .collect();

        // 构建会话索引
        let session_index: HashMap<String, usize> = knowledge_base
            .sessions
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.clone(), i))
            .collect();

        Self {
            knowledge_base,
            message_index,
            session_index,
        }
    }

    /// 关键词搜索
    pub fn search_keyword(&self, query: &str, limit: Option<usize>) -> Vec<SearchResult> {
        let limit = limit.unwrap_or(50);
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<SearchResult> = self
            .knowledge_base
            .messages
            .iter()
            .filter_map(|msg| {
                let content_lower = msg.content.to_lowercase();

                // 计算匹配的关键词
                let matched_keywords: Vec<String> = query_terms
                    .iter()
                    .filter(|term| content_lower.contains(*term))
                    .map(|s| s.to_string())
                    .collect();

                if matched_keywords.is_empty() {
                    return None;
                }

                // 计算相关性分数
                let relevance_score = self.calculate_relevance(&content_lower, &query_terms);

                Some(SearchResult {
                    message: msg.clone(),
                    relevance_score,
                    matched_keywords,
                    context: self.get_message_context(&msg.id),
                })
            })
            .collect();

        // 按相关性排序
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // 限制结果数量
        results.truncate(limit);

        results
    }

    /// 按时间范围查询
    pub fn search_by_time(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<ConversationMessage> {
        self.knowledge_base
            .messages
            .iter()
            .filter(|msg| msg.timestamp >= start && msg.timestamp <= end)
            .cloned()
            .collect()
    }

    /// 获取主题列表
    pub fn browse_topics(&self) -> Vec<TopicSummary> {
        self.knowledge_base
            .topics
            .iter()
            .map(|topic| TopicSummary {
                id: topic.id.clone(),
                name: topic.name.clone(),
                message_count: topic.message_count,
                time_range: topic.time_range,
            })
            .collect()
    }

    /// 获取主题详情
    pub fn get_topic(&self, topic_id: &str) -> Option<&Topic> {
        self.knowledge_base.topics.iter().find(|t| t.id == topic_id)
    }

    /// 获取主题的所有消息
    pub fn get_topic_messages(&self, topic_id: &str) -> Vec<&ConversationMessage> {
        let topic = match self.get_topic(topic_id) {
            Some(t) => t,
            None => return Vec::new(),
        };

        topic
            .message_ids
            .iter()
            .filter_map(|id| self.get_message(id))
            .collect()
    }

    /// 获取会话列表
    pub fn list_sessions(&self) -> Vec<&ConversationSession> {
        self.knowledge_base.sessions.iter().collect()
    }

    /// 获取会话详情
    pub fn get_session(&self, session_id: &str) -> Option<&ConversationSession> {
        self.session_index
            .get(session_id)
            .map(|&i| &self.knowledge_base.sessions[i])
    }

    /// 获取会话的所有消息
    pub fn get_session_messages(&self, session_id: &str) -> Vec<&ConversationMessage> {
        self.knowledge_base
            .messages
            .iter()
            .filter(|m| m.session_id == session_id)
            .collect()
    }

    /// 获取知识库元数据
    pub fn get_metadata(&self) -> &KnowledgeMetadata {
        &self.knowledge_base.metadata
    }

    /// 获取决策点列表
    pub fn list_decisions(&self) -> Vec<&Decision> {
        self.knowledge_base.decisions.iter().collect()
    }

    /// 按类型筛选决策点
    pub fn get_decisions_by_type(&self, decision_type: DecisionType) -> Vec<&Decision> {
        self.knowledge_base
            .decisions
            .iter()
            .filter(|d| d.decision_type == decision_type)
            .collect()
    }

    /// 推荐相关对话
    pub fn recommend_related(&self, context: &str, limit: usize) -> Vec<SearchResult> {
        // 使用关键词搜索作为简单的相关推荐
        // TODO: 未来可以使用向量相似度
        self.search_keyword(context, Some(limit))
    }

    /// 获取时间线视图（按天分组）
    pub fn get_timeline(&self) -> Vec<TimelineEntry> {
        let mut day_groups: HashMap<String, Vec<&ConversationMessage>> = HashMap::new();

        for msg in &self.knowledge_base.messages {
            let day = msg.timestamp.format("%Y-%m-%d").to_string();
            day_groups.entry(day).or_default().push(msg);
        }

        let mut timeline: Vec<TimelineEntry> = day_groups
            .into_iter()
            .map(|(date, messages)| {
                // 提取当天的主要活动
                let activities: Vec<String> = messages
                    .iter()
                    .filter(|m| m.message_type == MessageType::User)
                    .take(5)
                    .map(|m| {
                        if m.content.len() <= 50 {
                            m.content.clone()
                        } else {
                            format!("{}...", &m.content[..50])
                        }
                    })
                    .collect();

                TimelineEntry {
                    date,
                    message_count: messages.len(),
                    activities,
                }
            })
            .collect();

        // 按日期降序排序
        timeline.sort_by(|a, b| b.date.cmp(&a.date));

        timeline
    }

    /// 获取消息
    fn get_message(&self, id: &str) -> Option<&ConversationMessage> {
        self.message_index
            .get(id)
            .map(|&i| &self.knowledge_base.messages[i])
    }

    /// 获取消息上下文（前后消息）
    fn get_message_context(&self, id: &str) -> Option<SearchContext> {
        let idx = self.message_index.get(id)?;

        // 获取同会话的消息
        let msg = &self.knowledge_base.messages[*idx];
        let session_id = &msg.session_id;

        let session_messages: Vec<_> = self
            .knowledge_base
            .messages
            .iter()
            .filter(|m| m.session_id == *session_id)
            .collect();

        // 找到当前消息在会话中的位置
        let pos = session_messages.iter().position(|m| m.id == *id)?;

        let previous = if pos > 0 {
            Some(Box::new(session_messages[pos - 1].clone()))
        } else {
            None
        };

        let next = if pos < session_messages.len() - 1 {
            Some(Box::new(session_messages[pos + 1].clone()))
        } else {
            None
        };

        Some(SearchContext { previous, next })
    }

    /// 计算相关性分数
    fn calculate_relevance(&self, content: &str, query_terms: &[&str]) -> f64 {
        let mut score = 0.0;
        let content_len = content.len() as f64;

        for term in query_terms {
            // 计算词频
            let count = content.matches(term).count() as f64;
            if count > 0.0 {
                // TF-IDF 简化版
                score += (1.0 + count.ln()) * (1.0 / (1.0 + content_len / 1000.0));
            }
        }

        // 归一化
        score / query_terms.len() as f64
    }

    /// 更新知识库（用于增量更新）
    pub fn update_knowledge_base(&mut self, new_data: KnowledgeBase) {
        // 合并消息
        for msg in new_data.messages {
            if !self.message_index.contains_key(&msg.id) {
                let idx = self.knowledge_base.messages.len();
                self.message_index.insert(msg.id.clone(), idx);
                self.knowledge_base.messages.push(msg);
            }
        }

        // 合并会话
        for session in new_data.sessions {
            if !self.session_index.contains_key(&session.id) {
                let idx = self.knowledge_base.sessions.len();
                self.session_index.insert(session.id.clone(), idx);
                self.knowledge_base.sessions.push(session);
            }
        }

        // 更新元数据
        self.knowledge_base.metadata.updated_at = Utc::now();
        self.knowledge_base.metadata.total_messages = self.knowledge_base.messages.len();
        self.knowledge_base.metadata.total_sessions = self
            .knowledge_base
            .sessions
            .iter()
            .filter(|s| !s.is_agent)
            .count();
        self.knowledge_base.metadata.total_agents = self
            .knowledge_base
            .sessions
            .iter()
            .filter(|s| s.is_agent)
            .count();

        // 更新主题和决策
        self.knowledge_base.metadata.total_topics = self.knowledge_base.topics.len();
        self.knowledge_base.metadata.total_decisions = self.knowledge_base.decisions.len();
    }

    /// 设置主题
    pub fn set_topics(&mut self, topics: Vec<Topic>) {
        self.knowledge_base.topics = topics;
        self.knowledge_base.metadata.total_topics = self.knowledge_base.topics.len();
    }

    /// 设置决策点
    pub fn set_decisions(&mut self, decisions: Vec<Decision>) {
        self.knowledge_base.decisions = decisions;
        self.knowledge_base.metadata.total_decisions = self.knowledge_base.decisions.len();
    }

    /// 获取知识库的可变引用（用于直接修改）
    pub fn knowledge_base_mut(&mut self) -> &mut KnowledgeBase {
        &mut self.knowledge_base
    }

    /// 获取知识库的引用
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        &self.knowledge_base
    }
}

/// 时间线条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineEntry {
    /// 日期（YYYY-MM-DD）
    pub date: String,
    /// 消息数量
    pub message_count: usize,
    /// 主要活动摘要
    pub activities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_knowledge_base() -> KnowledgeBase {
        let messages = vec![
            ConversationMessage {
                id: "msg1".to_string(),
                message_type: MessageType::User,
                content: "实现 Storage Layer 2.0 架构".to_string(),
                thinking: None,
                timestamp: Utc::now(),
                session_id: "session1".to_string(),
                tools_used: Vec::new(),
                metadata: HashMap::new(),
            },
            ConversationMessage {
                id: "msg2".to_string(),
                message_type: MessageType::Assistant,
                content: "好的，我来设计 Storage Layer 的分层架构".to_string(),
                thinking: None,
                timestamp: Utc::now(),
                session_id: "session1".to_string(),
                tools_used: Vec::new(),
                metadata: HashMap::new(),
            },
        ];

        let sessions = vec![ConversationSession {
            id: "session1".to_string(),
            summary: "Storage Layer 开发".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            message_count: 2,
            filename: "session1.jsonl".to_string(),
            file_size_mb: 0.1,
            is_agent: false,
            parent_session_id: None,
        }];

        KnowledgeBase {
            metadata: KnowledgeMetadata {
                project: "Test".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                total_sessions: 1,
                total_agents: 0,
                total_messages: 2,
                total_topics: 0,
                total_decisions: 0,
                source_path: "/tmp".to_string(),
            },
            sessions,
            messages,
            topics: Vec::new(),
            decisions: Vec::new(),
        }
    }

    #[test]
    fn test_search_keyword() {
        let kb = create_test_knowledge_base();
        let retrieval = KnowledgeRetrieval::new(kb);

        let results = retrieval.search_keyword("Storage", None);
        assert!(!results.is_empty());
        assert!(results[0].matched_keywords.iter().any(|k| k.to_lowercase().contains("storage")));
    }

    #[test]
    fn test_list_sessions() {
        let kb = create_test_knowledge_base();
        let retrieval = KnowledgeRetrieval::new(kb);

        let sessions = retrieval.list_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "session1");
    }
}
