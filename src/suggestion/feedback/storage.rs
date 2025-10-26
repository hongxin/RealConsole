//! 反馈数据持久化存储
//!
//! 负责将反馈记录和统计数据持久化到本地文件系统

use super::types::{SuggestionFeedback, SuggestionStats};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// 反馈存储
///
/// 管理两个持久化文件：
/// - `feedbacks.json`: 原始反馈记录（最近 1000 条）
/// - `stats.json`: 聚合统计数据
pub struct FeedbackStorage {
    /// 存储目录路径
    storage_dir: PathBuf,

    /// 反馈记录文件路径
    feedbacks_path: PathBuf,

    /// 统计数据文件路径
    stats_path: PathBuf,

    /// 最大反馈记录数（超过则清理最老的）
    max_feedbacks: usize,
}

impl FeedbackStorage {
    /// 创建新的存储实例
    ///
    /// # 参数
    /// - `storage_dir`: 存储目录路径（通常为 `~/.realconsole/feedback`）
    ///
    /// # 示例
    /// ```no_run
    /// use realconsole::suggestion::feedback::FeedbackStorage;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = FeedbackStorage::new(
    ///     PathBuf::from("/home/user/.realconsole/feedback")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(storage_dir: PathBuf) -> Result<Self> {
        // 确保存储目录存在
        fs::create_dir_all(&storage_dir)
            .await
            .context("Failed to create feedback storage directory")?;

        let feedbacks_path = storage_dir.join("feedbacks.json");
        let stats_path = storage_dir.join("stats.json");

        Ok(Self {
            storage_dir,
            feedbacks_path,
            stats_path,
            max_feedbacks: 1000,
        })
    }

    /// 从默认位置创建存储实例
    ///
    /// 默认路径：`~/.realconsole/feedback`
    pub async fn from_default_location() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let storage_dir = home.join(".realconsole").join("feedback");
        Self::new(storage_dir).await
    }

    /// 保存反馈记录
    ///
    /// 将新的反馈记录追加到文件中，如果超过最大数量则清理最老的记录
    pub async fn save_feedback(&self, feedback: &SuggestionFeedback) -> Result<()> {
        // 读取现有记录
        let mut feedbacks = self.load_feedbacks().await.unwrap_or_default();

        // 追加新记录
        feedbacks.push(feedback.clone());

        // 如果超过最大数量，清理最老的记录
        if feedbacks.len() > self.max_feedbacks {
            feedbacks.drain(0..(feedbacks.len() - self.max_feedbacks));
        }

        // 保存到文件
        self.write_feedbacks(&feedbacks).await
    }

    /// 加载所有反馈记录
    pub async fn load_feedbacks(&self) -> Result<Vec<SuggestionFeedback>> {
        if !self.feedbacks_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.feedbacks_path)
            .await
            .context("Failed to read feedbacks file")?;

        let feedbacks: Vec<SuggestionFeedback> =
            serde_json::from_str(&content).context("Failed to parse feedbacks JSON")?;

        Ok(feedbacks)
    }

    /// 保存统计数据
    pub async fn save_stats(&self, stats: &HashMap<String, SuggestionStats>) -> Result<()> {
        let json = serde_json::to_string_pretty(stats).context("Failed to serialize stats")?;

        let mut file = fs::File::create(&self.stats_path)
            .await
            .context("Failed to create stats file")?;

        file.write_all(json.as_bytes())
            .await
            .context("Failed to write stats file")?;

        file.sync_all().await.context("Failed to sync stats file")?;

        Ok(())
    }

    /// 加载统计数据
    pub async fn load_stats(&self) -> Result<HashMap<String, SuggestionStats>> {
        if !self.stats_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.stats_path)
            .await
            .context("Failed to read stats file")?;

        let stats: HashMap<String, SuggestionStats> =
            serde_json::from_str(&content).context("Failed to parse stats JSON")?;

        Ok(stats)
    }

    /// 更新统计数据
    ///
    /// 根据反馈记录更新对应建议的统计信息
    pub async fn update_stats(&self, feedback: &SuggestionFeedback) -> Result<()> {
        // 加载现有统计
        let mut stats = self.load_stats().await.unwrap_or_default();

        // 获取或创建统计记录
        let stat = stats
            .entry(feedback.suggestion.clone())
            .or_insert_with(|| SuggestionStats::new(feedback.suggestion.clone()));

        // 更新统计
        stat.update(feedback);

        // 保存回文件
        self.save_stats(&stats).await
    }

    /// 获取指定建议的统计信息
    pub async fn get_stats(&self, command_pattern: &str) -> Result<Option<SuggestionStats>> {
        let stats = self.load_stats().await?;
        Ok(stats.get(command_pattern).cloned())
    }

    /// 获取所有高质量建议
    ///
    /// 返回质量分数 > 0.7 的建议列表
    pub async fn get_high_quality_suggestions(&self) -> Result<Vec<SuggestionStats>> {
        let stats = self.load_stats().await?;
        let mut high_quality: Vec<SuggestionStats> = stats
            .values()
            .filter(|s| s.is_high_quality())
            .cloned()
            .collect();

        // 按质量分数降序排列
        high_quality.sort_by(|a, b| {
            b.quality_score()
                .partial_cmp(&a.quality_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(high_quality)
    }

    /// 获取所有低质量建议
    ///
    /// 返回质量分数 < 0.3 且展示次数 >= 5 的建议列表
    pub async fn get_low_quality_suggestions(&self) -> Result<Vec<SuggestionStats>> {
        let stats = self.load_stats().await?;
        let mut low_quality: Vec<SuggestionStats> = stats
            .values()
            .filter(|s| s.is_low_quality())
            .cloned()
            .collect();

        // 按质量分数升序排列
        low_quality.sort_by(|a, b| {
            a.quality_score()
                .partial_cmp(&b.quality_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(low_quality)
    }

    /// 清理低质量建议的统计数据
    ///
    /// 删除质量分数过低的建议统计（可选操作，用于维护）
    pub async fn cleanup_low_quality(&self) -> Result<usize> {
        let mut stats = self.load_stats().await?;
        let original_count = stats.len();

        // 移除低质量建议
        stats.retain(|_, s| !s.is_low_quality());

        // 保存更新后的统计
        self.save_stats(&stats).await?;

        Ok(original_count - stats.len())
    }

    /// 获取最近 N 条反馈记录
    pub async fn get_recent_feedbacks(&self, count: usize) -> Result<Vec<SuggestionFeedback>> {
        let feedbacks = self.load_feedbacks().await?;
        let start = feedbacks.len().saturating_sub(count);
        Ok(feedbacks[start..].to_vec())
    }

    /// 清空所有反馈数据（危险操作，仅用于测试）
    #[cfg(test)]
    pub async fn clear_all(&self) -> Result<()> {
        if self.feedbacks_path.exists() {
            fs::remove_file(&self.feedbacks_path).await?;
        }
        if self.stats_path.exists() {
            fs::remove_file(&self.stats_path).await?;
        }
        Ok(())
    }

    /// 获取存储统计信息
    pub async fn storage_info(&self) -> Result<StorageInfo> {
        let feedbacks_count = self.load_feedbacks().await?.len();
        let stats_count = self.load_stats().await?.len();

        let feedbacks_size = if self.feedbacks_path.exists() {
            fs::metadata(&self.feedbacks_path).await?.len()
        } else {
            0
        };

        let stats_size = if self.stats_path.exists() {
            fs::metadata(&self.stats_path).await?.len()
        } else {
            0
        };

        Ok(StorageInfo {
            storage_dir: self.storage_dir.clone(),
            feedbacks_count,
            stats_count,
            feedbacks_size_bytes: feedbacks_size,
            stats_size_bytes: stats_size,
            total_size_bytes: feedbacks_size + stats_size,
        })
    }

    /// 写入反馈记录到文件
    async fn write_feedbacks(&self, feedbacks: &[SuggestionFeedback]) -> Result<()> {
        let json =
            serde_json::to_string_pretty(feedbacks).context("Failed to serialize feedbacks")?;

        let mut file = fs::File::create(&self.feedbacks_path)
            .await
            .context("Failed to create feedbacks file")?;

        file.write_all(json.as_bytes())
            .await
            .context("Failed to write feedbacks file")?;

        file.sync_all()
            .await
            .context("Failed to sync feedbacks file")?;

        Ok(())
    }
}

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// 存储目录
    pub storage_dir: PathBuf,

    /// 反馈记录数量
    pub feedbacks_count: usize,

    /// 统计记录数量
    pub stats_count: usize,

    /// 反馈文件大小（字节）
    pub feedbacks_size_bytes: u64,

    /// 统计文件大小（字节）
    pub stats_size_bytes: u64,

    /// 总大小（字节）
    pub total_size_bytes: u64,
}

impl StorageInfo {
    /// 格式化文件大小
    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;

        if bytes < KB {
            format!("{} B", bytes)
        } else if bytes < MB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        }
    }

    /// 获取友好的描述
    pub fn description(&self) -> String {
        format!(
            "Storage: {} feedbacks, {} stats | Size: {} total ({} feedbacks + {} stats)",
            self.feedbacks_count,
            self.stats_count,
            Self::format_size(self.total_size_bytes),
            Self::format_size(self.feedbacks_size_bytes),
            Self::format_size(self.stats_size_bytes)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggestion::feedback::types::{FeedbackContext, FeedbackType};
    use tempfile::TempDir;

    async fn create_test_storage() -> (FeedbackStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FeedbackStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();
        (storage, temp_dir)
    }

    fn create_test_feedback(cmd: &str, feedback_type: FeedbackType) -> SuggestionFeedback {
        let context = FeedbackContext::new("/test".to_string());
        SuggestionFeedback::new(
            cmd.to_string(),
            "Test".to_string(),
            0.8,
            feedback_type,
            context,
        )
    }

    #[tokio::test]
    async fn test_save_and_load_feedback() {
        let (storage, _temp) = create_test_storage().await;

        let feedback = create_test_feedback("cargo build", FeedbackType::Accepted);
        storage.save_feedback(&feedback).await.unwrap();

        let loaded = storage.load_feedbacks().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].suggestion, "cargo build");
    }

    #[tokio::test]
    async fn test_max_feedbacks_cleanup() {
        let (mut storage, _temp) = create_test_storage().await;
        storage.max_feedbacks = 5; // 设置较小的最大值

        // 添加 10 条记录
        for i in 0..10 {
            let feedback = create_test_feedback(&format!("cmd_{}", i), FeedbackType::Accepted);
            storage.save_feedback(&feedback).await.unwrap();
        }

        let loaded = storage.load_feedbacks().await.unwrap();
        assert_eq!(loaded.len(), 5);
        // 应该保留最后 5 条
        assert_eq!(loaded[0].suggestion, "cmd_5");
        assert_eq!(loaded[4].suggestion, "cmd_9");
    }

    #[tokio::test]
    async fn test_update_stats() {
        let (storage, _temp) = create_test_storage().await;

        let feedback = create_test_feedback("cargo test", FeedbackType::Accepted).with_selection(0, 3);
        storage.update_stats(&feedback).await.unwrap();

        let stats = storage.get_stats("cargo test").await.unwrap();
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.shown_count, 1);
        assert_eq!(stats.accepted_count, 1);
        assert_eq!(stats.acceptance_rate, 1.0);
    }

    #[tokio::test]
    async fn test_high_quality_suggestions() {
        let (storage, _temp) = create_test_storage().await;

        // 创建高质量建议
        for _ in 0..10 {
            let feedback =
                create_test_feedback("cargo build", FeedbackType::Accepted).with_selection(0, 3);
            storage.update_stats(&feedback).await.unwrap();
        }

        // 创建低质量建议
        for _ in 0..10 {
            let feedback =
                create_test_feedback("bad_cmd", FeedbackType::Skipped).with_selection(2, 3);
            storage.update_stats(&feedback).await.unwrap();
        }

        let high_quality = storage.get_high_quality_suggestions().await.unwrap();
        assert_eq!(high_quality.len(), 1);
        assert_eq!(high_quality[0].command_pattern, "cargo build");
    }

    #[tokio::test]
    async fn test_low_quality_suggestions() {
        let (storage, _temp) = create_test_storage().await;

        // 创建低质量建议（展示10次，只接受1次）
        for i in 0..10 {
            let feedback_type = if i == 0 {
                FeedbackType::Accepted
            } else {
                FeedbackType::Skipped
            };
            let feedback = create_test_feedback("bad_cmd", feedback_type).with_selection(2, 3);
            storage.update_stats(&feedback).await.unwrap();
        }

        let low_quality = storage.get_low_quality_suggestions().await.unwrap();
        assert_eq!(low_quality.len(), 1);
        assert_eq!(low_quality[0].command_pattern, "bad_cmd");
    }

    #[tokio::test]
    async fn test_cleanup_low_quality() {
        let (storage, _temp) = create_test_storage().await;

        // 创建高质量建议
        for _ in 0..5 {
            let feedback =
                create_test_feedback("good_cmd", FeedbackType::Accepted).with_selection(0, 3);
            storage.update_stats(&feedback).await.unwrap();
        }

        // 创建低质量建议
        for i in 0..10 {
            let feedback_type = if i == 0 {
                FeedbackType::Accepted
            } else {
                FeedbackType::Skipped
            };
            let feedback = create_test_feedback("bad_cmd", feedback_type);
            storage.update_stats(&feedback).await.unwrap();
        }

        let removed = storage.cleanup_low_quality().await.unwrap();
        assert_eq!(removed, 1);

        let stats = storage.load_stats().await.unwrap();
        assert_eq!(stats.len(), 1);
        assert!(stats.contains_key("good_cmd"));
        assert!(!stats.contains_key("bad_cmd"));
    }

    #[tokio::test]
    async fn test_get_recent_feedbacks() {
        let (storage, _temp) = create_test_storage().await;

        // 添加 10 条记录
        for i in 0..10 {
            let feedback = create_test_feedback(&format!("cmd_{}", i), FeedbackType::Accepted);
            storage.save_feedback(&feedback).await.unwrap();
        }

        let recent = storage.get_recent_feedbacks(3).await.unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].suggestion, "cmd_7");
        assert_eq!(recent[2].suggestion, "cmd_9");
    }

    #[tokio::test]
    async fn test_storage_info() {
        let (storage, _temp) = create_test_storage().await;

        // 添加一些数据
        let feedback = create_test_feedback("test", FeedbackType::Accepted);
        storage.save_feedback(&feedback).await.unwrap();
        storage.update_stats(&feedback).await.unwrap();

        let info = storage.storage_info().await.unwrap();
        assert_eq!(info.feedbacks_count, 1);
        assert_eq!(info.stats_count, 1);
        assert!(info.total_size_bytes > 0);

        // 测试格式化
        let desc = info.description();
        assert!(desc.contains("1 feedbacks"));
        assert!(desc.contains("1 stats"));
    }
}
