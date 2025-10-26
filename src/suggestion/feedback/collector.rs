//! 反馈收集器
//!
//! 负责收集用户对建议的反馈行为

use super::storage::FeedbackStorage;
use super::types::{FeedbackContext, FeedbackType, SuggestionFeedback};
use crate::suggestion::Suggestion;
use anyhow::{Context as _, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 反馈会话
///
/// 记录一次建议展示的上下文，等待用户响应
#[derive(Debug, Clone)]
struct FeedbackSession {
    /// 展示的所有建议及其对应的反馈记录
    feedbacks: Vec<SuggestionFeedback>,

    /// 建议总数
    total_count: usize,
}

/// 反馈收集器
///
/// 收集并记录用户对建议的反馈行为
///
/// ## 工作流程
///
/// ```text
/// 1. record_suggestion_shown()  →  创建反馈会话，返回 session_id
/// 2. record_selection()         →  标记选中的建议为 Accepted，其他为 Skipped
///    或 record_skip()           →  标记所有建议为 Skipped
/// 3. 自动保存到 FeedbackStorage
/// ```
pub struct FeedbackCollector {
    /// 存储后端
    storage: Arc<RwLock<FeedbackStorage>>,

    /// 待确认的反馈会话（session_id -> FeedbackSession）
    ///
    /// 当建议展示时创建会话，用户选择或跳过后移除会话并持久化
    pending_sessions: Arc<RwLock<HashMap<String, FeedbackSession>>>,
}

impl FeedbackCollector {
    /// 创建新的收集器
    pub fn new(storage: FeedbackStorage) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            pending_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 从默认位置创建收集器
    ///
    /// 默认路径：`~/.realconsole/feedback`
    pub async fn from_default_location() -> Result<Self> {
        let storage = FeedbackStorage::from_default_location().await?;
        Ok(Self::new(storage))
    }

    /// 记录建议展示
    ///
    /// 创建反馈会话，为每个建议生成反馈记录（状态待定）
    ///
    /// # 参数
    /// - `suggestions`: 展示给用户的建议列表
    /// - `context`: 建议生成时的上下文
    ///
    /// # 返回
    /// - `session_id`: 反馈会话 ID，用于后续记录用户选择
    ///
    /// # 示例
    /// ```no_run
    /// # use realconsole::suggestion::feedback::{FeedbackCollector, FeedbackContext};
    /// # use realconsole::suggestion::Suggestion;
    /// # async fn example() -> anyhow::Result<()> {
    /// let collector = FeedbackCollector::from_default_location().await?;
    /// let suggestions = vec![/* ... */];
    /// let context = FeedbackContext::new("/home/user/project".to_string());
    ///
    /// let session_id = collector.record_suggestion_shown(&suggestions, &context).await?;
    /// println!("Session created: {}", session_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record_suggestion_shown(
        &self,
        suggestions: &[Suggestion],
        context: &FeedbackContext,
    ) -> Result<String> {
        if suggestions.is_empty() {
            anyhow::bail!("Cannot record empty suggestion list");
        }

        // 生成会话 ID
        let session_id = Self::generate_session_id();

        // 为每个建议创建反馈记录
        let feedbacks: Vec<SuggestionFeedback> = suggestions
            .iter()
            .map(|suggestion| {
                SuggestionFeedback::new(
                    suggestion.command.clone(),
                    format!("{:?}", suggestion.source), // 建议来源
                    suggestion.score,
                    FeedbackType::Skipped, // 默认为 Skipped，等待用户响应
                    context.clone(),
                )
            })
            .collect();

        // 创建会话
        let session = FeedbackSession {
            feedbacks,
            total_count: suggestions.len(),
        };

        // 保存会话
        let mut sessions = self.pending_sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// 记录用户选择
    ///
    /// 将选中的建议标记为 Accepted，其他建议标记为 Skipped
    ///
    /// # 参数
    /// - `session_id`: 反馈会话 ID
    /// - `selected_index`: 用户选择的建议索引（0-based）
    ///
    /// # 示例
    /// ```no_run
    /// # use realconsole::suggestion::feedback::FeedbackCollector;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let collector = FeedbackCollector::from_default_location().await?;
    /// # let session_id = "session_123".to_string();
    /// collector.record_selection(&session_id, 0).await?; // 选择第一个建议
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record_selection(&self, session_id: &str, selected_index: usize) -> Result<()> {
        // 获取并移除会话
        let session = {
            let mut sessions = self.pending_sessions.write().await;
            sessions
                .remove(session_id)
                .context("Feedback session not found")?
        };

        if selected_index >= session.total_count {
            anyhow::bail!(
                "Invalid selection index: {} (total: {})",
                selected_index,
                session.total_count
            );
        }

        // 处理所有反馈
        for (index, mut feedback) in session.feedbacks.into_iter().enumerate() {
            if index == selected_index {
                // 选中的建议
                feedback.feedback_type = FeedbackType::Accepted;
                feedback.selected_index = Some(index);
                feedback.total_suggestions = session.total_count;
            } else {
                // 未选中的建议
                feedback.feedback_type = FeedbackType::Skipped;
            }

            // 保存反馈并更新统计
            self.save_feedback(&feedback).await?;
        }

        Ok(())
    }

    /// 记录用户跳过
    ///
    /// 将所有建议标记为 Skipped
    ///
    /// # 示例
    /// ```no_run
    /// # use realconsole::suggestion::feedback::FeedbackCollector;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let collector = FeedbackCollector::from_default_location().await?;
    /// # let session_id = "session_123".to_string();
    /// collector.record_skip(&session_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record_skip(&self, session_id: &str) -> Result<()> {
        // 获取并移除会话
        let session = {
            let mut sessions = self.pending_sessions.write().await;
            sessions
                .remove(session_id)
                .context("Feedback session not found")?
        };

        // 所有建议都标记为 Skipped
        for feedback in session.feedbacks {
            self.save_feedback(&feedback).await?;
        }

        Ok(())
    }

    /// 清理过期会话
    ///
    /// 移除超过指定时间（默认 5 分钟）未响应的会话
    ///
    /// # 返回
    /// 被清理的会话数量
    pub async fn cleanup_stale_sessions(&self) -> Result<usize> {
        use chrono::Utc;

        let mut sessions = self.pending_sessions.write().await;
        let now = Utc::now();

        let mut to_remove = Vec::new();

        for (session_id, session) in sessions.iter() {
            // 检查会话中第一个反馈的时间戳
            if let Some(first_feedback) = session.feedbacks.first() {
                let age = now.signed_duration_since(first_feedback.timestamp);
                if age.num_minutes() > 5 {
                    to_remove.push(session_id.clone());
                }
            }
        }

        let count = to_remove.len();
        for session_id in to_remove {
            sessions.remove(&session_id);
        }

        Ok(count)
    }

    /// 获取待确认的会话数量
    pub async fn pending_sessions_count(&self) -> usize {
        self.pending_sessions.read().await.len()
    }

    /// 获取存储实例（用于直接访问统计数据）
    pub fn storage(&self) -> Arc<RwLock<FeedbackStorage>> {
        self.storage.clone()
    }

    /// 保存反馈到存储
    async fn save_feedback(&self, feedback: &SuggestionFeedback) -> Result<()> {
        let storage = self.storage.write().await;
        storage.save_feedback(feedback).await?;
        storage.update_stats(feedback).await?;
        Ok(())
    }

    /// 生成唯一会话 ID
    fn generate_session_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::sync::atomic::{AtomicU64, Ordering};

        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("session_{}_{}", timestamp, counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggestion::{Suggestion, SuggestionSource};
    use tempfile::TempDir;

    async fn create_test_collector() -> (FeedbackCollector, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FeedbackStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();
        let collector = FeedbackCollector::new(storage);
        (collector, temp_dir)
    }

    fn create_test_suggestions(count: usize) -> Vec<Suggestion> {
        use crate::suggestion::SuggestionCategory;
        (0..count)
            .map(|i| Suggestion {
                command: format!("cmd_{}", i),
                description: format!("Description {}", i),
                score: 0.8,
                source: SuggestionSource::Context,
                category: SuggestionCategory::General,
                needs_confirmation: false,
            })
            .collect()
    }

    #[tokio::test]
    async fn test_record_suggestion_shown() {
        let (collector, _temp) = create_test_collector().await;

        let suggestions = create_test_suggestions(3);
        let context = FeedbackContext::new("/test".to_string());

        let session_id = collector
            .record_suggestion_shown(&suggestions, &context)
            .await
            .unwrap();

        assert!(session_id.starts_with("session_"));
        assert_eq!(collector.pending_sessions_count().await, 1);
    }

    #[tokio::test]
    async fn test_record_selection() {
        let (collector, _temp) = create_test_collector().await;

        let suggestions = create_test_suggestions(3);
        let context = FeedbackContext::new("/test".to_string());

        let session_id = collector
            .record_suggestion_shown(&suggestions, &context)
            .await
            .unwrap();

        // 选择第二个建议
        collector.record_selection(&session_id, 1).await.unwrap();

        // 会话应该被移除
        assert_eq!(collector.pending_sessions_count().await, 0);

        // 验证统计数据
        let storage = collector.storage.read().await;
        let stats = storage.load_stats().await.unwrap();

        // cmd_1 应该被接受
        let cmd1_stats = stats.get("cmd_1").unwrap();
        assert_eq!(cmd1_stats.accepted_count, 1);
        assert_eq!(cmd1_stats.shown_count, 1);

        // cmd_0 和 cmd_2 应该被跳过
        let cmd0_stats = stats.get("cmd_0").unwrap();
        assert_eq!(cmd0_stats.accepted_count, 0);
        assert_eq!(cmd0_stats.skipped_count, 1);
    }

    #[tokio::test]
    async fn test_record_skip() {
        let (collector, _temp) = create_test_collector().await;

        let suggestions = create_test_suggestions(3);
        let context = FeedbackContext::new("/test".to_string());

        let session_id = collector
            .record_suggestion_shown(&suggestions, &context)
            .await
            .unwrap();

        collector.record_skip(&session_id).await.unwrap();

        // 会话应该被移除
        assert_eq!(collector.pending_sessions_count().await, 0);

        // 验证统计数据
        let storage = collector.storage.read().await;
        let stats = storage.load_stats().await.unwrap();

        // 所有建议都应该被跳过
        for i in 0..3 {
            let cmd = format!("cmd_{}", i);
            let cmd_stats = stats.get(&cmd).unwrap();
            assert_eq!(cmd_stats.accepted_count, 0);
            assert_eq!(cmd_stats.skipped_count, 1);
        }
    }

    #[tokio::test]
    async fn test_invalid_session_id() {
        let (collector, _temp) = create_test_collector().await;

        let result = collector.record_selection("invalid_session", 0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("session not found"));
    }

    #[tokio::test]
    async fn test_invalid_selection_index() {
        let (collector, _temp) = create_test_collector().await;

        let suggestions = create_test_suggestions(3);
        let context = FeedbackContext::new("/test".to_string());

        let session_id = collector
            .record_suggestion_shown(&suggestions, &context)
            .await
            .unwrap();

        let result = collector.record_selection(&session_id, 10).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid selection index"));
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let (collector, _temp) = create_test_collector().await;

        let suggestions1 = create_test_suggestions(2);
        let suggestions2 = create_test_suggestions(3);
        let context = FeedbackContext::new("/test".to_string());

        let session_id1 = collector
            .record_suggestion_shown(&suggestions1, &context)
            .await
            .unwrap();

        let session_id2 = collector
            .record_suggestion_shown(&suggestions2, &context)
            .await
            .unwrap();

        assert_eq!(collector.pending_sessions_count().await, 2);
        assert_ne!(session_id1, session_id2);

        // 完成第一个会话
        collector.record_selection(&session_id1, 0).await.unwrap();
        assert_eq!(collector.pending_sessions_count().await, 1);

        // 跳过第二个会话
        collector.record_skip(&session_id2).await.unwrap();
        assert_eq!(collector.pending_sessions_count().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_stale_sessions() {
        let (collector, _temp) = create_test_collector().await;

        let suggestions = create_test_suggestions(2);
        let context = FeedbackContext::new("/test".to_string());

        collector
            .record_suggestion_shown(&suggestions, &context)
            .await
            .unwrap();

        // 立即清理不应该移除会话（未超时）
        let removed = collector.cleanup_stale_sessions().await.unwrap();
        assert_eq!(removed, 0);
        assert_eq!(collector.pending_sessions_count().await, 1);
    }

    #[tokio::test]
    async fn test_empty_suggestions() {
        let (collector, _temp) = create_test_collector().await;

        let suggestions: Vec<Suggestion> = vec![];
        let context = FeedbackContext::new("/test".to_string());

        let result = collector
            .record_suggestion_shown(&suggestions, &context)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot record empty"));
    }

    #[tokio::test]
    async fn test_feedback_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_path_buf();

        // 创建第一个收集器，记录反馈
        {
            let storage = FeedbackStorage::new(storage_path.clone()).await.unwrap();
            let collector = FeedbackCollector::new(storage);

            let suggestions = create_test_suggestions(2);
            let context = FeedbackContext::new("/test".to_string());

            let session_id = collector
                .record_suggestion_shown(&suggestions, &context)
                .await
                .unwrap();

            collector.record_selection(&session_id, 0).await.unwrap();
        }

        // 创建第二个收集器，验证数据持久化
        {
            let storage = FeedbackStorage::new(storage_path).await.unwrap();
            let stats = storage.load_stats().await.unwrap();

            assert_eq!(stats.len(), 2);
            assert!(stats.contains_key("cmd_0"));
            assert!(stats.contains_key("cmd_1"));

            let cmd0_stats = stats.get("cmd_0").unwrap();
            assert_eq!(cmd0_stats.accepted_count, 1);
        }
    }
}
