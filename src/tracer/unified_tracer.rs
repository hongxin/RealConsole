//! 统一追踪器实现
//!
//! `UnifiedTracer` 聚合四个观测维度，提供统一查询接口

use super::entry::TraceEntry;
use super::types::{Dimension, EntryType, Status};
use crate::conversation::context_manager::ContextManager;
use crate::execution_logger::{CommandType, ExecutionLogger};
use crate::history::{HistoryManager, SortStrategy};
use crate::llm::logger::LlmLogger;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 统一追踪器
///
/// 聚合四个数据源（History, ExecutionLogger, LlmLogger, ContextManager），
/// 提供统一的查询接口
///
/// # 示例
///
/// ```rust,no_run
/// use realconsole::tracer::UnifiedTracer;
/// // ... 创建 tracer
/// let entries = tracer.query_all(20).await?;
/// for entry in entries {
///     println!("{}", entry.preview());
/// }
/// ```
pub struct UnifiedTracer {
    /// 统计维度 - History
    history: Arc<RwLock<HistoryManager>>,

    /// 协同维度 - ExecutionLogger
    exec_logger: Arc<RwLock<ExecutionLogger>>,

    /// 黑盒维度 - LlmLogger (可选)
    llm_logger: Option<Arc<LlmLogger>>,

    /// 记忆维度 - ContextManager
    context: Arc<RwLock<ContextManager>>,

    /// ✨ v1.15.0 Phase 2: 自定义事件存储
    ///
    /// 用于记录自适应优化、炼化等系统内部事件
    /// LRU 策略，最多保留 200 条
    custom_entries: Arc<RwLock<VecDeque<TraceEntry>>>,
}

impl UnifiedTracer {
    /// 创建新的统一追踪器
    ///
    /// # 参数
    ///
    /// - `history`: History 管理器
    /// - `exec_logger`: 执行日志管理器
    /// - `llm_logger`: LLM 日志管理器（可选）
    /// - `context`: Context 管理器
    pub fn new(
        history: Arc<RwLock<HistoryManager>>,
        exec_logger: Arc<RwLock<ExecutionLogger>>,
        llm_logger: Option<Arc<LlmLogger>>,
        context: Arc<RwLock<ContextManager>>,
    ) -> Self {
        Self {
            history,
            exec_logger,
            llm_logger,
            context,
            custom_entries: Arc::new(RwLock::new(VecDeque::with_capacity(200))),
        }
    }

    /// 查询所有维度（默认）
    ///
    /// 并行查询四个数据源，按时间排序，智能去重
    ///
    /// # 参数
    ///
    /// - `limit`: 最大返回条目数
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// let entries = tracer.query_all(20).await?;
    /// ```
    pub async fn query_all(&self, limit: usize) -> Result<Vec<TraceEntry>> {
        // 并行查询四个数据源
        let (history_entries, exec_entries, llm_entries, context_entries) = tokio::join!(
            self.entries_from_history(limit),
            self.entries_from_exec_logger(limit),
            self.entries_from_llm_logger(limit),
            self.entries_from_context(limit)
        );

        // 合并所有条目
        let mut all_entries = Vec::new();
        all_entries.extend(history_entries?);
        all_entries.extend(exec_entries?);
        all_entries.extend(llm_entries?);
        all_entries.extend(context_entries?);

        // ✨ v1.15.0 Phase 2: 合并自定义事件
        let custom = self.custom_entries.read().await;
        all_entries.extend(custom.iter().cloned());

        // 按时间排序（最新优先）
        all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 智能去重
        let deduplicated = Self::deduplicate_entries(all_entries);

        // 限制数量
        Ok(deduplicated.into_iter().take(limit).collect())
    }

    /// 按维度查询
    ///
    /// # 参数
    ///
    /// - `dimension`: 要查询的维度
    /// - `limit`: 最大返回条目数
    pub async fn query_by_dimension(
        &self,
        dimension: Dimension,
        limit: usize,
    ) -> Result<Vec<TraceEntry>> {
        let mut entries = match dimension {
            Dimension::Statistics => self.entries_from_history(limit).await?,
            Dimension::Coordination => self.entries_from_exec_logger(limit).await?,
            Dimension::BlackBox => self.entries_from_llm_logger(limit).await?,
            Dimension::Memory => self.entries_from_context(limit).await?,
        };

        // ✨ v1.15.0 Phase 2: 合并自定义事件（仅匹配维度的）
        let custom = self.custom_entries.read().await;
        entries.extend(
            custom
                .iter()
                .filter(|e| e.dimension == dimension)
                .cloned()
        );

        // 按时间排序（最新优先）
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(entries.into_iter().take(limit).collect())
    }

    /// ✨ v1.15.0 Phase 2: 添加自定义事件
    ///
    /// 用于记录系统内部事件（如自适应优化、炼化过程等）
    ///
    /// # 参数
    ///
    /// - `entry`: 要添加的追踪条目
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use realconsole::tracer::{TraceEntry, Dimension, EntryType, Status};
    ///
    /// let entry = TraceEntry::new(
    ///     Dimension::Statistics,
    ///     EntryType::Custom("adaptive_optimization".to_string()),
    ///     "自动优化生成 7 条建议".to_string(),
    ///     Status::Success,
    /// );
    ///
    /// tracer.add_entry(entry).await;
    /// ```
    pub async fn add_entry(&self, entry: TraceEntry) {
        let mut custom = self.custom_entries.write().await;

        // LRU 策略：超过容量则移除最旧的
        if custom.len() >= 200 {
            custom.pop_front();
        }

        custom.push_back(entry);
    }

    /// ✨ v1.15.0 Phase 2: 获取自定义事件数量
    ///
    /// 用于统计和调试
    pub async fn custom_entries_count(&self) -> usize {
        self.custom_entries.read().await.len()
    }

    /// 按时间范围查询
    ///
    /// # 参数
    ///
    /// - `start`: 开始时间
    /// - `end`: 结束时间
    pub async fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TraceEntry>> {
        // 查询所有维度（使用较大的 limit）
        let all_entries = self.query_all(1000).await?;

        // 按时间范围过滤
        let filtered: Vec<TraceEntry> = all_entries
            .into_iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect();

        Ok(filtered)
    }

    /// 关键词搜索
    ///
    /// 搜索内容中包含关键词的条目
    ///
    /// # 参数
    ///
    /// - `keyword`: 搜索关键词（不区分大小写）
    pub async fn search(&self, keyword: &str) -> Result<Vec<TraceEntry>> {
        let keyword_lower = keyword.to_lowercase();

        // 查询所有维度
        let all_entries = self.query_all(500).await?;

        // 过滤包含关键词的条目
        let results: Vec<TraceEntry> = all_entries
            .into_iter()
            .filter(|entry| entry.content.to_lowercase().contains(&keyword_lower))
            .collect();

        Ok(results)
    }

    /// 获取统计信息
    pub async fn stats(&self) -> Result<TraceStats> {
        let all_entries = self.query_all(1000).await?;

        // 按维度统计
        let mut by_dimension = HashMap::new();
        for entry in &all_entries {
            *by_dimension.entry(entry.dimension).or_insert(0) += 1;
        }

        // 按状态统计
        let mut by_status = HashMap::new();
        for entry in &all_entries {
            let status_key = match &entry.status {
                Status::Success => "Success",
                Status::Failed(_) => "Failed",
                Status::Running => "Running",
                Status::Cancelled => "Cancelled",
            };
            *by_status.entry(status_key.to_string()).or_insert(0) += 1;
        }

        // 时间范围
        let time_range = if all_entries.is_empty() {
            None
        } else {
            let earliest = all_entries.iter().map(|e| e.timestamp).min().unwrap();
            let latest = all_entries.iter().map(|e| e.timestamp).max().unwrap();
            Some((earliest, latest))
        };

        // 平均每小时条目数
        let avg_entries_per_hour = if let Some((earliest, latest)) = time_range {
            let duration_hours = (latest - earliest).num_hours() as f64;
            if duration_hours > 0.0 {
                all_entries.len() as f64 / duration_hours
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(TraceStats {
            total_entries: all_entries.len(),
            by_dimension,
            by_status,
            time_range,
            avg_entries_per_hour,
        })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 数据源适配器
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// 从 History 提取条目
    async fn entries_from_history(&self, limit: usize) -> Result<Vec<TraceEntry>> {
        let history = self.history.read().await;
        let entries = history.recent(limit, SortStrategy::Time);

        Ok(entries
            .into_iter()
            .map(|entry| {
                TraceEntry::with_metadata(
                    Dimension::Statistics,
                    EntryType::ShellCommand,
                    entry.command.clone(),
                    if entry.last_success {
                        Status::Success
                    } else {
                        Status::Failed("".to_string())
                    },
                    HashMap::from([
                        ("count".to_string(), json!(entry.count)),
                        (
                            "first_timestamp".to_string(),
                            json!(entry.first_timestamp.to_rfc3339()),
                        ),
                        (
                            "last_timestamp".to_string(),
                            json!(entry.last_timestamp.to_rfc3339()),
                        ),
                    ]),
                )
            })
            .collect())
    }

    /// 从 ExecutionLogger 提取条目
    async fn entries_from_exec_logger(&self, limit: usize) -> Result<Vec<TraceEntry>> {
        let logger = self.exec_logger.read().await;
        let logs = logger.recent(limit);

        Ok(logs
            .into_iter()
            .map(|log| {
                let entry_type = match log.command_type {
                    CommandType::Shell => EntryType::ShellCommand,
                    CommandType::Command => EntryType::SystemCommand,
                    CommandType::Text => EntryType::TaskExecution,
                };

                let status = if log.success {
                    Status::Success
                } else {
                    Status::Failed(log.result_preview.clone())
                };

                let mut entry = TraceEntry::new(
                    Dimension::Coordination,
                    entry_type,
                    log.command.clone(),
                    status,
                );
                entry.timestamp = log.timestamp;
                entry.add_metadata("duration_ms".to_string(), json!(log.duration_ms));
                entry.add_metadata(
                    "command_type".to_string(),
                    json!(log.command_type.to_string()),
                );
                entry.add_metadata("result_preview".to_string(), json!(log.result_preview));
                entry
            })
            .collect())
    }

    /// 从 LlmLogger 提取条目（如果存在）
    async fn entries_from_llm_logger(&self, _limit: usize) -> Result<Vec<TraceEntry>> {
        // 注意：LlmLogger 的 API 可能不支持直接查询最近的调用
        // 这里返回空数组，Phase 3 后续可以完善
        //
        // TODO: 实现 LlmLogger 的数据提取
        // 可能需要：
        // 1. 读取日志文件
        // 2. 解析 JSONL 格式
        // 3. 转换为 TraceEntry
        Ok(Vec::new())
    }

    /// 从 ContextManager 提取条目
    async fn entries_from_context(&self, limit: usize) -> Result<Vec<TraceEntry>> {
        let context = self.context.read().await;

        // 获取对话轮次
        let turns = context.turns();

        let mut entries = Vec::new();

        // 将每个 Turn 转换为 TraceEntry
        for turn in turns.iter().rev().take(limit) {
            // 用户消息
            let mut user_entry = TraceEntry::new(
                Dimension::Memory,
                EntryType::ContextMessage,
                turn.user_input.clone(),
                if turn.success {
                    Status::Success
                } else {
                    Status::Failed("".to_string())
                },
            );
            user_entry.timestamp = turn.timestamp;
            user_entry.add_metadata("role".to_string(), json!("user"));
            user_entry.add_metadata("turn_id".to_string(), json!(turn.id.to_string()));
            entries.push(user_entry);

            // 助手响应
            let mut assistant_entry = TraceEntry::new(
                Dimension::Memory,
                EntryType::ContextMessage,
                turn.assistant_response.clone(),
                if turn.success {
                    Status::Success
                } else {
                    Status::Failed("".to_string())
                },
            );
            assistant_entry.timestamp = turn.timestamp;
            assistant_entry.add_metadata("role".to_string(), json!("assistant"));
            assistant_entry.add_metadata("turn_id".to_string(), json!(turn.id.to_string()));
            entries.push(assistant_entry);
        }

        Ok(entries)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 辅助方法
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// 智能去重
    ///
    /// 策略：基于内容哈希和时间桶（10秒），识别相同的条目
    pub(crate) fn deduplicate_entries(entries: Vec<TraceEntry>) -> Vec<TraceEntry> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for entry in entries {
            let key = entry.dedup_key();
            if !seen.contains(&key) {
                seen.insert(key);
                result.push(entry);
            }
        }

        result
    }

    /// ✨ v1.6.0: 获取失败的执行日志
    ///
    /// 用于异常检测和错误分析
    ///
    /// # 参数
    ///
    /// - `limit`: 最大返回日志数
    ///
    /// # 返回
    ///
    /// 返回最近的失败执行日志列表
    pub async fn get_failed_logs(&self, limit: usize) -> Result<Vec<crate::execution_logger::ExecutionLog>> {
        let logger = self.exec_logger.read().await;
        let recent_logs = logger.recent(limit * 2); // 获取更多日志，然后过滤失败的

        Ok(recent_logs
            .into_iter()
            .filter(|log| !log.success)
            .take(limit)
            .cloned()
            .collect())
    }
}

/// 追踪统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStats {
    /// 总条目数
    pub total_entries: usize,

    /// 按维度分布
    pub by_dimension: HashMap<Dimension, usize>,

    /// 按状态分布
    pub by_status: HashMap<String, usize>,

    /// 时间范围（最早和最晚的时间戳）
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,

    /// 平均每小时条目数
    pub avg_entries_per_hour: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConversationConfig;
    use crate::history::HistoryEntry;
    use std::path::PathBuf;

    // 辅助函数：创建测试用的 HistoryManager
    fn create_test_history() -> Arc<RwLock<HistoryManager>> {
        let test_path = PathBuf::from("/tmp/realconsole_test_history.json");
        let mut history = HistoryManager::new(test_path, 100);
        history.add("ls -la".to_string(), true);
        history.add("pwd".to_string(), true);
        history.add("cat file.txt".to_string(), false);
        Arc::new(RwLock::new(history))
    }

    // 辅助函数：创建测试用的 ExecutionLogger
    fn create_test_exec_logger() -> Arc<RwLock<ExecutionLogger>> {
        Arc::new(RwLock::new(ExecutionLogger::new(100)))
    }

    // 辅助函数：创建测试用的 ContextManager
    fn create_test_context() -> Arc<RwLock<ContextManager>> {
        let config = ConversationConfig::default();
        Arc::new(RwLock::new(ContextManager::new(config)))
    }

    #[tokio::test]
    async fn test_unified_tracer_new() {
        let history = create_test_history();
        let exec_logger = create_test_exec_logger();
        let context = create_test_context();

        let _tracer = UnifiedTracer::new(history, exec_logger, None, context);

        // 测试基本创建
        assert!(true); // 如果能创建就成功
    }

    #[tokio::test]
    async fn test_query_by_dimension_statistics() {
        let history = create_test_history();
        let exec_logger = create_test_exec_logger();
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        let entries = tracer
            .query_by_dimension(Dimension::Statistics, 10)
            .await
            .unwrap();

        // 应该有来自 history 的条目
        assert!(entries.len() > 0);
        for entry in entries {
            assert_eq!(entry.dimension, Dimension::Statistics);
        }
    }

    #[tokio::test]
    async fn test_search() {
        let history = create_test_history();
        let exec_logger = create_test_exec_logger();
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        // 搜索 "ls"
        let results = tracer.search("ls").await.unwrap();

        // 应该能找到包含 "ls" 的条目
        for entry in results {
            assert!(entry.content.to_lowercase().contains("ls"));
        }
    }

    #[tokio::test]
    async fn test_deduplicate_entries() {
        // 创建重复的条目
        let entry1 = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        let mut entry2 = entry1.clone();
        entry2.id = uuid::Uuid::new_v4(); // 不同 ID
        entry2.timestamp = entry1.timestamp; // 相同时间

        let entries = vec![entry1, entry2];

        // 去重
        let deduplicated = UnifiedTracer::deduplicate_entries(entries.clone());

        // 应该只保留一个
        assert_eq!(deduplicated.len(), 1);
    }

    #[tokio::test]
    async fn test_stats() {
        let history = create_test_history();
        let exec_logger = create_test_exec_logger();
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        let stats = tracer.stats().await.unwrap();

        // 应该有统计数据
        assert!(stats.total_entries > 0);
        assert!(stats.by_dimension.len() > 0);
    }

    // === 边缘情况测试 ===

    #[tokio::test]
    async fn test_empty_data_sources() {
        // 创建空的数据源
        let test_path = PathBuf::from("/tmp/realconsole_test_empty.json");
        let history = Arc::new(RwLock::new(HistoryManager::new(test_path, 100)));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(100)));
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        // 查询空数据应该返回空列表，不应该崩溃
        let entries = tracer.query_all(10).await.unwrap();
        assert_eq!(entries.len(), 0);

        // 搜索空数据应该返回空列表
        let results = tracer.search("anything").await.unwrap();
        assert_eq!(results.len(), 0);

        // 统计空数据应该返回零值
        let stats = tracer.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.by_dimension.len(), 0);
    }

    #[tokio::test]
    async fn test_utf8_content() {
        let test_path = PathBuf::from("/tmp/realconsole_test_utf8.json");
        let mut history = HistoryManager::new(test_path, 100);

        // 添加包含多语言字符的命令
        history.add("请帮我访问纽约时报官网，预估到token限制的前提下，巧妙分析RSS订阅来源".to_string(), true);
        history.add("echo 'Hello 世界 🌍'".to_string(), true);
        history.add("ls -la /путь/到/файл".to_string(), true); // 中俄混合
        history.add("cat 日本語ファイル.txt".to_string(), true);

        let history = Arc::new(RwLock::new(history));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(100)));
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        // 查询不应该崩溃
        let entries = tracer.query_all(10).await.unwrap();
        assert!(entries.len() >= 4);

        // 格式化输出不应该崩溃（测试 preview 方法）
        for entry in entries {
            let preview = entry.preview();
            assert!(!preview.is_empty());
            // 验证 UTF-8 有效性
            assert!(std::str::from_utf8(preview.as_bytes()).is_ok());
        }
    }

    #[tokio::test]
    async fn test_large_dataset() {
        let test_path = PathBuf::from("/tmp/realconsole_test_large.json");
        let mut history = HistoryManager::new(test_path, 10000);

        // 添加大量数据
        for i in 0..5000 {
            history.add(format!("command_{}", i), i % 2 == 0);
        }

        let history = Arc::new(RwLock::new(history));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(10000)));
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        // 查询大数据集应该正常工作
        let entries = tracer.query_all(100).await.unwrap();
        assert_eq!(entries.len(), 100); // 应该限制在 100 条

        // 搜索大数据集（搜索通用词）
        let results = tracer.search("command").await.unwrap();
        assert!(results.len() > 0, "搜索 'command' 应该找到结果");

        // 验证能处理大数据集的统计
        let stats = tracer.stats().await.unwrap();
        assert!(stats.total_entries >= 100, "大数据集统计应该有足够的条目");
    }

    #[tokio::test]
    async fn test_limit_edge_cases() {
        let history = create_test_history();
        let exec_logger = create_test_exec_logger();
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        // limit = 0 应该返回空列表
        let entries = tracer.query_all(0).await.unwrap();
        assert_eq!(entries.len(), 0);

        // limit = 1 应该返回 1 条
        let entries = tracer.query_all(1).await.unwrap();
        assert_eq!(entries.len(), 1);

        // 超大 limit 不应该崩溃
        let entries = tracer.query_all(999999).await.unwrap();
        assert!(entries.len() > 0); // 应该返回所有可用条目
    }

    #[tokio::test]
    async fn test_special_search_keywords() {
        let history = create_test_history();
        let exec_logger = create_test_exec_logger();
        let context = create_test_context();

        let tracer = UnifiedTracer::new(history, exec_logger, None, context);

        // 空搜索关键词 - 不应该崩溃
        let results = tracer.search("").await;
        assert!(results.is_ok(), "空搜索不应该出错");

        // 特殊字符搜索 - 不应该崩溃
        let results = tracer.search("@#$%^&*()").await;
        assert!(results.is_ok(), "特殊字符搜索不应该出错");

        // Unicode 搜索 - 不应该崩溃
        let results = tracer.search("中文").await;
        assert!(results.is_ok(), "Unicode搜索不应该出错");
    }
}
