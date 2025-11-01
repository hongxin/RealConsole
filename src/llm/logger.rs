//! LLM 交互日志系统
//!
//! 记录完整的 LLM 请求和响应，用于：
//! - 问题排查和 debug
//! - 性能分析和优化
//! - 数据复盘和分析
//! - 成本统计
//!
//! 设计哲学：三态日志（请求/响应/元数据）

use super::{Message, MessageRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// LLM 交互日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInteractionLog {
    /// 会话 ID（用于关联同一次交互）
    pub session_id: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 模型名称
    pub model: String,

    /// 请求内容
    pub request: LlmRequest,

    /// 响应内容（可能为空，如果请求失败）
    pub response: Option<LlmResponse>,

    /// 元数据（性能、错误等）
    pub meta: LlmMetadata,
}

/// LLM 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// 消息数量
    pub message_count: usize,

    /// 消息摘要（用于隐私保护）
    pub summary: String,

    /// 完整消息（可选，根据配置决定是否记录）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,

    /// 请求参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// LLM 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// 响应内容长度（字符数）
    pub content_length: usize,

    /// 响应内容摘要
    pub summary: String,

    /// 完整内容（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Token 使用量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,

    /// 结束原因
    pub finish_reason: String,
}

/// Token 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 提示词 Token 数
    pub prompt_tokens: u32,

    /// 补全 Token 数
    pub completion_tokens: u32,

    /// 总 Token 数
    pub total_tokens: u32,
}

/// LLM 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMetadata {
    /// 延迟（毫秒）
    pub latency_ms: u64,

    /// 状态（success/error/timeout）
    pub status: String,

    /// 错误信息（如果有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// 是否为流式
    pub is_streaming: bool,

    /// 请求开始时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    /// 请求结束时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// 上下文信息（新增）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<CallContext>,
}

/// 调用上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallContext {
    /// 原始用户输入
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input: Option<String>,

    /// Intent 识别结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,

    /// 使用的工具列表
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools_used: Vec<String>,

    /// 工具调用结果摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results_summary: Option<String>,
}

/// log_interaction 函数的参数封装
pub struct LogInteractionParams<'a> {
    pub session_id: String,
    pub model: String,
    pub messages: &'a [Message],
    pub response_content: Option<String>,
    pub start_time: Instant,
    pub is_streaming: bool,
    pub error: Option<String>,
    pub context: Option<CallContext>,
}

/// LLM 日志配置
#[derive(Debug, Clone)]
pub struct LlmLoggerConfig {
    /// 是否启用日志
    pub enabled: bool,

    /// 日志目录
    pub log_dir: PathBuf,

    /// 是否记录完整消息内容
    pub include_content: bool,

    /// 敏感词过滤模式（正则表达式）
    pub sensitive_patterns: Vec<String>,

    /// 自动清理（天数）
    pub retention_days: u32,

    /// 最大日志大小（MB）
    pub max_size_mb: u32,
}

impl Default for LlmLoggerConfig {
    fn default() -> Self {
        // 默认日志目录：~/.realconsole/llm_logs
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let log_dir = PathBuf::from(home)
            .join(".realconsole")
            .join("llm_logs");

        Self {
            enabled: false, // 默认关闭
            log_dir,
            include_content: true,
            sensitive_patterns: vec![
                r"api[_-]?key".to_string(),
                r"password".to_string(),
                r"token".to_string(),
            ],
            retention_days: 30,
            max_size_mb: 100,
        }
    }
}

/// LLM 日志记录器
pub struct LlmLogger {
    /// 配置
    config: LlmLoggerConfig,

    /// 当前日志文件路径
    current_log_file: Arc<RwLock<Option<PathBuf>>>,
}

impl LlmLogger {
    /// 创建新的 LLM 日志记录器
    pub fn new(config: LlmLoggerConfig) -> Self {
        Self {
            config,
            current_log_file: Arc::new(RwLock::new(None)),
        }
    }

    /// 创建默认配置的日志记录器
    pub fn with_defaults() -> Self {
        Self::new(LlmLoggerConfig::default())
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// 开始记录一次交互（返回会话 ID 和开始时间）
    pub fn start_logging(&self, _model: &str) -> (String, Instant) {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();

        (session_id, start_time)
    }

    /// 记录 LLM 交互
    ///
    /// # 参数
    /// - `params`: 日志交互参数（封装了所有必要的参数）
    pub async fn log_interaction(&self, params: LogInteractionParams<'_>) {
        if !self.config.enabled {
            return;
        }

        let LogInteractionParams {
            session_id,
            model,
            messages,
            response_content,
            start_time,
            is_streaming,
            error,
            context,
        } = params;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let now = Utc::now();

        // 构建请求
        let request = self.build_request(messages);

        // 构建响应
        let response = response_content.as_ref().map(|content| {
            self.build_response(content)
        });

        // 构建元数据
        let status = if error.is_some() {
            "error".to_string()
        } else if response.is_some() {
            "success".to_string()
        } else {
            "timeout".to_string()
        };

        let meta = LlmMetadata {
            latency_ms,
            status,
            error,
            is_streaming,
            started_at: Some(now - chrono::Duration::milliseconds(latency_ms as i64)),
            completed_at: Some(now),
            context,
        };

        // 构建完整日志
        let log = LlmInteractionLog {
            session_id,
            timestamp: now,
            model,
            request,
            response,
            meta,
        };

        // 写入日志文件
        self.write_log(&log).await;
    }

    /// 构建请求对象
    fn build_request(&self, messages: &[Message]) -> LlmRequest {
        let message_count = messages.len();

        // 生成摘要（取最后一条用户消息的前 50 字符）
        let summary = messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .and_then(|m| m.content.as_ref())
            .map(|content| {
                let chars: String = content.chars().take(50).collect();
                if content.chars().count() > 50 {
                    format!("{}...", chars)
                } else {
                    chars
                }
            })
            .unwrap_or_else(|| "[无内容]".to_string());

        // 根据配置决定是否包含完整消息
        let messages_opt = if self.config.include_content {
            Some(messages.to_vec())
        } else {
            None
        };

        LlmRequest {
            message_count,
            summary,
            messages: messages_opt,
            temperature: None,
            max_tokens: None,
        }
    }

    /// 构建响应对象
    fn build_response(&self, content: &str) -> LlmResponse {
        let content_length = content.chars().count();

        // 生成摘要（前 100 字符）
        let summary = if content_length > 100 {
            let chars: String = content.chars().take(100).collect();
            format!("{}...", chars)
        } else {
            content.to_string()
        };

        // 根据配置决定是否包含完整内容
        let content_opt = if self.config.include_content {
            Some(content.to_string())
        } else {
            None
        };

        LlmResponse {
            content_length,
            summary,
            content: content_opt,
            usage: None, // TODO: 从 API 响应中提取
            finish_reason: "stop".to_string(),
        }
    }

    /// 写入日志到文件
    async fn write_log(&self, log: &LlmInteractionLog) {
        // 确保日志目录存在
        if let Err(e) = create_dir_all(&self.config.log_dir) {
            eprintln!("创建日志目录失败: {}", e);
            return;
        }

        // 生成日志文件名（按日期）
        let date_str = log.timestamp.format("%Y-%m-%d").to_string();
        let log_file = self.config.log_dir.join(format!("llm_{}.jsonl", date_str));

        // 序列化为 JSON
        let json_str = match serde_json::to_string(log) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("序列化日志失败: {}", e);
                return;
            }
        };

        // 追加写入文件
        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("打开日志文件失败: {}", e);
                return;
            }
        };

        if let Err(e) = writeln!(file, "{}", json_str) {
            eprintln!("写入日志失败: {}", e);
        }

        // 更新当前日志文件路径
        *self.current_log_file.write().await = Some(log_file);
    }

    /// 获取日志目录
    pub fn log_dir(&self) -> &Path {
        &self.config.log_dir
    }

    /// 获取最近的日志文件
    pub async fn get_current_log_file(&self) -> Option<PathBuf> {
        self.current_log_file.read().await.clone()
    }

    /// 搜索日志（支持关键词和时间范围）
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词（在摘要和完整内容中搜索）
    /// - `days`: 搜索最近 N 天的日志（None 表示全部）
    pub fn search_logs(&self, keyword: &str, days: Option<u32>) -> Vec<LlmInteractionLog> {
        let mut results = Vec::new();

        // 计算时间范围
        let cutoff_time = days.map(|d| {
            Utc::now() - chrono::Duration::days(d as i64)
        });

        // 读取所有日志文件
        if let Ok(entries) = std::fs::read_dir(&self.config.log_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "jsonl" {
                                // 读取文件内容
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    for line in content.lines() {
                                        if let Ok(log) = serde_json::from_str::<LlmInteractionLog>(line) {
                                            // 时间过滤
                                            if let Some(cutoff) = cutoff_time {
                                                if log.timestamp < cutoff {
                                                    continue;
                                                }
                                            }

                                            // 关键词过滤
                                            let keyword_lower = keyword.to_lowercase();
                                            let matches = log.request.summary.to_lowercase().contains(&keyword_lower)
                                                || log.response.as_ref().is_some_and(|r|
                                                    r.summary.to_lowercase().contains(&keyword_lower)
                                                )
                                                || log.model.to_lowercase().contains(&keyword_lower);

                                            if matches {
                                                results.push(log);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 按时间排序（最新的在前）
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results
    }

    /// 获取统计信息
    ///
    /// # 参数
    /// - `days`: 统计最近 N 天的日志（None 表示全部）
    pub fn get_statistics(&self, days: Option<u32>) -> LogStatistics {
        let mut stats = LogStatistics::default();

        // 计算时间范围
        let cutoff_time = days.map(|d| {
            Utc::now() - chrono::Duration::days(d as i64)
        });

        let mut latencies = Vec::new();

        // 读取所有日志文件
        if let Ok(entries) = std::fs::read_dir(&self.config.log_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "jsonl" {
                                // 读取文件内容
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    for line in content.lines() {
                                        if let Ok(log) = serde_json::from_str::<LlmInteractionLog>(line) {
                                            // 时间过滤
                                            if let Some(cutoff) = cutoff_time {
                                                if log.timestamp < cutoff {
                                                    continue;
                                                }
                                            }

                                            stats.total_requests += 1;

                                            // 状态统计
                                            match log.meta.status.as_str() {
                                                "success" => stats.successful_requests += 1,
                                                "error" => stats.failed_requests += 1,
                                                _ => {}
                                            }

                                            // 模型统计
                                            *stats.model_usage.entry(log.model.clone()).or_insert(0) += 1;

                                            // 延迟统计
                                            latencies.push(log.meta.latency_ms);

                                            // Token 统计
                                            if let Some(ref response) = log.response {
                                                if let Some(ref usage) = response.usage {
                                                    stats.total_prompt_tokens += usage.prompt_tokens as u64;
                                                    stats.total_completion_tokens += usage.completion_tokens as u64;
                                                    stats.total_tokens += usage.total_tokens as u64;
                                                }
                                            }

                                            // 流式统计
                                            if log.meta.is_streaming {
                                                stats.streaming_requests += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 计算延迟统计
        if !latencies.is_empty() {
            latencies.sort_unstable();
            let len = latencies.len();

            stats.avg_latency_ms = latencies.iter().sum::<u64>() / len as u64;
            stats.min_latency_ms = *latencies.first().unwrap();
            stats.max_latency_ms = *latencies.last().unwrap();
            stats.p50_latency_ms = latencies[len / 2];
            stats.p95_latency_ms = latencies[(len * 95) / 100];
            stats.p99_latency_ms = latencies[(len * 99) / 100];
        }

        stats
    }

    /// 清理旧日志
    ///
    /// # 参数
    /// - `days`: 删除 N 天前的日志
    ///
    /// # 返回
    /// (删除的文件数, 释放的字节数)
    pub fn clean_old_logs(&self, days: u32) -> (usize, u64) {
        let mut deleted_files = 0;
        let mut freed_bytes = 0;

        if let Ok(entries) = std::fs::read_dir(&self.config.log_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "jsonl" {
                                // 检查文件修改时间
                                if let Ok(modified) = metadata.modified() {
                                    if let Ok(modified_dt) = modified.elapsed() {
                                        let file_age_days = modified_dt.as_secs() / 86400;
                                        if file_age_days as u32 > days {
                                            // 删除文件
                                            if std::fs::remove_file(&path).is_ok() {
                                                deleted_files += 1;
                                                freed_bytes += metadata.len();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        (deleted_files, freed_bytes)
    }

    /// 按大小清理日志
    ///
    /// # 参数
    /// - `max_size_mb`: 最大日志大小（MB）
    ///
    /// # 返回
    /// (删除的文件数, 释放的字节数)
    pub fn clean_by_size(&self, max_size_mb: u32) -> (usize, u64) {
        let max_size_bytes = max_size_mb as u64 * 1024 * 1024;
        let mut total_size = 0u64;
        let mut deleted_files = 0;
        let mut freed_bytes = 0;

        // 收集所有日志文件及其元数据
        let mut files: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.config.log_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "jsonl" {
                                if let Ok(modified) = metadata.modified() {
                                    files.push((path, metadata.len(), modified));
                                    total_size += metadata.len();
                                }
                            }
                        }
                    }
                }
            }
        }

        // 如果总大小超过限制，删除最旧的文件
        if total_size > max_size_bytes {
            // 按修改时间排序（最旧的在前）
            files.sort_by(|a, b| a.2.cmp(&b.2));

            for (path, size, _) in files {
                if total_size <= max_size_bytes {
                    break;
                }

                if std::fs::remove_file(&path).is_ok() {
                    deleted_files += 1;
                    freed_bytes += size;
                    total_size -= size;
                }
            }
        }

        (deleted_files, freed_bytes)
    }

    /// 获取所有日志文件的总大小
    pub fn get_total_size(&self) -> u64 {
        let mut total_size = 0u64;

        if let Ok(entries) = std::fs::read_dir(&self.config.log_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "jsonl" {
                                total_size += metadata.len();
                            }
                        }
                    }
                }
            }
        }

        total_size
    }

    /// 根据 session_id 获取日志
    ///
    /// # 参数
    /// - `session_id`: 会话 ID
    ///
    /// # 返回
    /// 找到的日志记录（如果存在）
    pub fn get_log_by_session_id(&self, session_id: &str) -> Option<LlmInteractionLog> {
        // 读取所有日志文件
        if let Ok(entries) = std::fs::read_dir(&self.config.log_dir) {
            // 收集所有日志文件并按修改时间排序（最新的在前）
            let mut files: Vec<PathBuf> = Vec::new();
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "jsonl" {
                                files.push(path);
                            }
                        }
                    }
                }
            }

            // 按修改时间排序（最新的在前）
            files.sort_by(|a, b| {
                let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
                let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
                b_time.cmp(&a_time)
            });

            // 从最新的文件开始搜索
            for file in files {
                if let Ok(content) = std::fs::read_to_string(&file) {
                    for line in content.lines().rev() {  // 反向搜索（最新的在后）
                        if let Ok(log) = serde_json::from_str::<LlmInteractionLog>(line) {
                            if log.session_id == session_id {
                                return Some(log);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 获取最近的会话列表
    ///
    /// # 参数
    /// - `limit`: 返回的最大数量
    ///
    /// # 返回
    /// (session_id, timestamp, model, summary) 列表
    pub fn list_recent_sessions(&self, limit: usize) -> Vec<(String, DateTime<Utc>, String, String)> {
        let mut sessions = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // 读取所有日志文件
        if let Ok(entries) = std::fs::read_dir(&self.config.log_dir) {
            // 收集所有日志文件并按修改时间排序（最新的在前）
            let mut files: Vec<PathBuf> = Vec::new();
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "jsonl" {
                                files.push(path);
                            }
                        }
                    }
                }
            }

            // 按修改时间排序（最新的在前）
            files.sort_by(|a, b| {
                let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
                let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
                b_time.cmp(&a_time)
            });

            // 从最新的文件开始读取
            for file in files {
                if sessions.len() >= limit {
                    break;
                }

                if let Ok(content) = std::fs::read_to_string(&file) {
                    // 反向读取（最新的在后）
                    for line in content.lines().rev() {
                        if sessions.len() >= limit {
                            break;
                        }

                        if let Ok(log) = serde_json::from_str::<LlmInteractionLog>(line) {
                            // 去重（只保留每个 session_id 的第一次出现）
                            if !seen_ids.contains(&log.session_id) {
                                seen_ids.insert(log.session_id.clone());
                                sessions.push((
                                    log.session_id,
                                    log.timestamp,
                                    log.model,
                                    log.request.summary,
                                ));
                            }
                        }
                    }
                }
            }
        }

        sessions
    }
}

/// 日志统计信息
#[derive(Debug, Default, Clone)]
pub struct LogStatistics {
    /// 总请求数
    pub total_requests: u64,

    /// 成功请求数
    pub successful_requests: u64,

    /// 失败请求数
    pub failed_requests: u64,

    /// 流式请求数
    pub streaming_requests: u64,

    /// 模型使用统计
    pub model_usage: std::collections::HashMap<String, u64>,

    /// 平均延迟（毫秒）
    pub avg_latency_ms: u64,

    /// 最小延迟（毫秒）
    pub min_latency_ms: u64,

    /// 最大延迟（毫秒）
    pub max_latency_ms: u64,

    /// P50 延迟（毫秒）
    pub p50_latency_ms: u64,

    /// P95 延迟（毫秒）
    pub p95_latency_ms: u64,

    /// P99 延迟（毫秒）
    pub p99_latency_ms: u64,

    /// 总 Token 数
    pub total_tokens: u64,

    /// 总 Prompt Token 数
    pub total_prompt_tokens: u64,

    /// 总 Completion Token 数
    pub total_completion_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_config_default() {
        let config = LlmLoggerConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.max_size_mb, 100);
        assert!(config.include_content);
    }

    #[test]
    fn test_logger_creation() {
        let logger = LlmLogger::with_defaults();
        assert!(!logger.is_enabled());
    }

    #[test]
    fn test_start_logging() {
        let logger = LlmLogger::with_defaults();
        let (session_id, start_time) = logger.start_logging("test-model");

        assert!(!session_id.is_empty());
        assert!(start_time.elapsed().as_millis() < 10); // 应该非常快
    }

    #[tokio::test]
    async fn test_log_interaction() {
        let mut config = LlmLoggerConfig::default();
        config.enabled = true;
        config.log_dir = PathBuf::from("/tmp/realconsole_test_logs");

        let logger = LlmLogger::new(config);

        let messages = vec![Message::user("Hello, world!")];
        let (session_id, start_time) = logger.start_logging("test-model");

        logger
            .log_interaction(LogInteractionParams {
                session_id,
                model: "test-model".to_string(),
                messages: &messages,
                response_content: Some("Hi there!".to_string()),
                start_time,
                is_streaming: false,
                error: None,
                context: None,
            })
            .await;

        // 验证日志文件是否创建
        let log_file = logger.get_current_log_file().await;
        assert!(log_file.is_some());
    }

    #[test]
    fn test_build_request() {
        let logger = LlmLogger::with_defaults();
        let messages = vec![
            Message::user("What is Rust?"),
            Message::assistant("Rust is a systems programming language."),
        ];

        let request = logger.build_request(&messages);

        assert_eq!(request.message_count, 2);
        assert!(request.summary.contains("What is Rust?"));
    }

    #[test]
    fn test_build_response() {
        let logger = LlmLogger::with_defaults();
        let content = "This is a test response";

        let response = logger.build_response(content);

        assert_eq!(response.content_length, 23);
        assert_eq!(response.summary, content);
        assert!(response.content.is_some());
    }

    #[test]
    fn test_build_response_long_content() {
        let logger = LlmLogger::with_defaults();
        let content = "a".repeat(200);

        let response = logger.build_response(&content);

        assert_eq!(response.content_length, 200);
        assert!(response.summary.ends_with("..."));
        assert_eq!(response.summary.chars().count(), 103); // 100 + "..."
    }

    #[test]
    fn test_search_logs_empty() {
        let mut config = LlmLoggerConfig::default();
        config.log_dir = PathBuf::from("/tmp/realconsole_test_search_empty");
        let logger = LlmLogger::new(config);

        let results = logger.search_logs("test", None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_statistics_empty() {
        let mut config = LlmLoggerConfig::default();
        config.log_dir = PathBuf::from("/tmp/realconsole_test_stats_empty");
        let logger = LlmLogger::new(config);

        let stats = logger.get_statistics(None);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
    }

    #[test]
    fn test_clean_old_logs_no_files() {
        let mut config = LlmLoggerConfig::default();
        config.log_dir = PathBuf::from("/tmp/realconsole_test_clean_empty");
        let logger = LlmLogger::new(config);

        let (deleted, freed) = logger.clean_old_logs(30);
        assert_eq!(deleted, 0);
        assert_eq!(freed, 0);
    }

    #[test]
    fn test_clean_by_size_no_files() {
        let mut config = LlmLoggerConfig::default();
        config.log_dir = PathBuf::from("/tmp/realconsole_test_size_empty");
        let logger = LlmLogger::new(config);

        let (deleted, freed) = logger.clean_by_size(100);
        assert_eq!(deleted, 0);
        assert_eq!(freed, 0);
    }

    #[test]
    fn test_get_total_size_empty() {
        let mut config = LlmLoggerConfig::default();
        config.log_dir = PathBuf::from("/tmp/realconsole_test_size_check");
        let logger = LlmLogger::new(config);

        let size = logger.get_total_size();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_log_statistics_default() {
        let stats = LogStatistics::default();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.avg_latency_ms, 0);
        assert!(stats.model_usage.is_empty());
    }

    #[test]
    fn test_get_log_by_session_id_not_found() {
        let mut config = LlmLoggerConfig::default();
        config.log_dir = PathBuf::from("/tmp/realconsole_test_replay_empty");
        let logger = LlmLogger::new(config);

        let result = logger.get_log_by_session_id("non-existent-id");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_recent_sessions_empty() {
        let mut config = LlmLoggerConfig::default();
        config.log_dir = PathBuf::from("/tmp/realconsole_test_sessions_empty");
        let logger = LlmLogger::new(config);

        let sessions = logger.list_recent_sessions(10);
        assert!(sessions.is_empty());
    }
}
