//! 知识系统服务层
//!
//! 整合导入、主题提取、检索功能，提供统一的 API

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::importer::ConversationImporter;
use super::retrieval::{KnowledgeRetrieval, TimelineEntry};
use super::topic::TopicExtractor;
use super::types::*;

/// 知识系统服务
pub struct KnowledgeService {
    /// 检索引擎（使用读写锁保护）
    retrieval: Arc<RwLock<Option<KnowledgeRetrieval>>>,
    /// Claude Code 项目目录
    claude_dir: PathBuf,
    /// 主题提取器
    topic_extractor: TopicExtractor,
}

impl KnowledgeService {
    /// 创建知识服务
    pub fn new(claude_dir: PathBuf) -> Self {
        Self {
            retrieval: Arc::new(RwLock::new(None)),
            claude_dir,
            topic_extractor: TopicExtractor::new(),
        }
    }

    /// 从默认目录创建
    pub fn from_project_path(project_path: &str) -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        let project_dir_name = project_path.replace('/', "-");
        let claude_dir = home.join(".claude/projects").join(project_dir_name);
        Ok(Self::new(claude_dir))
    }

    /// 导入对话历史
    pub async fn import(&self) -> Result<ImportStats> {
        let importer = ConversationImporter::new(self.claude_dir.clone());
        let (mut knowledge_base, stats) = importer.import_from_claude_dir().await?;

        // 提取主题
        let topics = self.topic_extractor.extract_topics(&knowledge_base.messages);
        knowledge_base.topics = topics;
        knowledge_base.metadata.total_topics = knowledge_base.topics.len();

        // 提取决策点
        let decisions = self.topic_extractor.identify_decisions(&knowledge_base.messages);
        knowledge_base.decisions = decisions;
        knowledge_base.metadata.total_decisions = knowledge_base.decisions.len();

        // 创建检索引擎
        let retrieval = KnowledgeRetrieval::new(knowledge_base);

        // 更新服务状态
        let mut lock = self.retrieval.write().await;
        *lock = Some(retrieval);

        Ok(stats)
    }

    /// 从备份文件导入
    pub async fn import_from_backup(&self, backup_path: &Path) -> Result<ImportStats> {
        let importer = ConversationImporter::new(self.claude_dir.clone());
        let (mut knowledge_base, stats) = importer.import_from_backup(backup_path).await?;

        // 提取主题
        let topics = self.topic_extractor.extract_topics(&knowledge_base.messages);
        knowledge_base.topics = topics;
        knowledge_base.metadata.total_topics = knowledge_base.topics.len();

        // 提取决策点
        let decisions = self.topic_extractor.identify_decisions(&knowledge_base.messages);
        knowledge_base.decisions = decisions;
        knowledge_base.metadata.total_decisions = knowledge_base.decisions.len();

        // 创建检索引擎
        let retrieval = KnowledgeRetrieval::new(knowledge_base);

        // 更新服务状态
        let mut lock = self.retrieval.write().await;
        *lock = Some(retrieval);

        Ok(stats)
    }

    /// 搜索
    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.search_keyword(query, limit)),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 按时间范围搜索
    pub async fn search_by_time(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ConversationMessage>> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.search_by_time(start, end)),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 获取主题列表
    pub async fn list_topics(&self) -> Result<Vec<TopicSummary>> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.browse_topics()),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 获取主题详情
    pub async fn get_topic(&self, topic_id: &str) -> Result<Option<Topic>> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.get_topic(topic_id).cloned()),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 获取会话列表
    pub async fn list_sessions(&self) -> Result<Vec<ConversationSession>> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.list_sessions().into_iter().cloned().collect()),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 获取元数据
    pub async fn get_metadata(&self) -> Result<KnowledgeMetadata> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.get_metadata().clone()),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 获取时间线
    pub async fn get_timeline(&self) -> Result<Vec<TimelineEntry>> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.get_timeline()),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 获取决策点列表
    pub async fn list_decisions(&self) -> Result<Vec<Decision>> {
        let lock = self.retrieval.read().await;
        match lock.as_ref() {
            Some(retrieval) => Ok(retrieval.list_decisions().into_iter().cloned().collect()),
            None => Err(anyhow::anyhow!("知识库未初始化，请先导入数据")),
        }
    }

    /// 检查是否已初始化
    pub async fn is_initialized(&self) -> bool {
        self.retrieval.read().await.is_some()
    }

    /// 处理 WebSocket 消息
    pub async fn handle_message(&self, msg: KnowledgeClientMessage) -> KnowledgeServerMessage {
        match msg {
            KnowledgeClientMessage::Import { source_path } => {
                let result = if let Some(path) = source_path {
                    let backup_path = PathBuf::from(path);
                    self.import_from_backup(&backup_path).await
                } else {
                    self.import().await
                };

                match result {
                    Ok(stats) => KnowledgeServerMessage::ImportComplete { stats },
                    Err(e) => KnowledgeServerMessage::Error {
                        message: e.to_string(),
                    },
                }
            }

            KnowledgeClientMessage::Search { query, limit } => match self.search(&query, limit).await {
                Ok(results) => {
                    let total = results.len();
                    KnowledgeServerMessage::SearchResults { results, total }
                }
                Err(e) => KnowledgeServerMessage::Error {
                    message: e.to_string(),
                },
            },

            KnowledgeClientMessage::SearchByTime { start, end } => {
                match self.search_by_time(start, end).await {
                    Ok(messages) => {
                        let results: Vec<SearchResult> = messages
                            .into_iter()
                            .map(|m| SearchResult {
                                message: m,
                                relevance_score: 1.0,
                                matched_keywords: Vec::new(),
                                context: None,
                            })
                            .collect();
                        let total = results.len();
                        KnowledgeServerMessage::SearchResults { results, total }
                    }
                    Err(e) => KnowledgeServerMessage::Error {
                        message: e.to_string(),
                    },
                }
            }

            KnowledgeClientMessage::ListTopics => match self.list_topics().await {
                Ok(topics) => KnowledgeServerMessage::TopicList { topics },
                Err(e) => KnowledgeServerMessage::Error {
                    message: e.to_string(),
                },
            },

            KnowledgeClientMessage::GetTopic { topic_id } => match self.get_topic(&topic_id).await {
                Ok(Some(topic)) => KnowledgeServerMessage::TopicDetail { topic },
                Ok(None) => KnowledgeServerMessage::Error {
                    message: format!("主题不存在: {}", topic_id),
                },
                Err(e) => KnowledgeServerMessage::Error {
                    message: e.to_string(),
                },
            },

            KnowledgeClientMessage::ListSessions => match self.list_sessions().await {
                Ok(sessions) => KnowledgeServerMessage::SessionList { sessions },
                Err(e) => KnowledgeServerMessage::Error {
                    message: e.to_string(),
                },
            },

            KnowledgeClientMessage::GetMetadata => match self.get_metadata().await {
                Ok(metadata) => KnowledgeServerMessage::Metadata { metadata },
                Err(e) => KnowledgeServerMessage::Error {
                    message: e.to_string(),
                },
            },

            KnowledgeClientMessage::IncrementalUpdate => {
                // TODO: 实现增量更新
                match self.import().await {
                    Ok(stats) => KnowledgeServerMessage::UpdateComplete {
                        stats: UpdateStats {
                            new_sessions: stats.sessions_imported,
                            updated_sessions: 0,
                            new_messages: stats.messages_imported,
                            duration_ms: stats.duration_ms,
                        },
                    },
                    Err(e) => KnowledgeServerMessage::Error {
                        message: e.to_string(),
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_service_not_initialized() {
        let service = KnowledgeService::new(PathBuf::from("/nonexistent"));
        assert!(!service.is_initialized().await);

        let result = service.search("test", None).await;
        assert!(result.is_err());
    }
}
