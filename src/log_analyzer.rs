//! 日志分析工具
//!
//! 提供智能化的日志文件分析能力：
//! - 自动识别日志级别
//! - 错误模式提取
//! - 时间范围分析
//! - 统计聚合
//!
//! **性能优化**：
//! - 使用 once_cell 缓存正则表达式（避免重复编译）
//! - 预期性能提升：5-10倍

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use chrono::{DateTime, NaiveDateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// 正则表达式缓存 - 日志级别检测
static LEVEL_REGEXES: Lazy<Vec<(Regex, LogLevel)>> = Lazy::new(|| {
    vec![
        (Regex::new(r"\[ERROR\]").unwrap(), LogLevel::Error),
        (Regex::new(r"\[WARN\]").unwrap(), LogLevel::Warn),
        (Regex::new(r"\[INFO\]").unwrap(), LogLevel::Info),
        (Regex::new(r"\[DEBUG\]").unwrap(), LogLevel::Debug),
        (Regex::new(r"\[TRACE\]").unwrap(), LogLevel::Trace),
        (Regex::new(r"ERROR:").unwrap(), LogLevel::Error),
        (Regex::new(r"WARN:").unwrap(), LogLevel::Warn),
        (Regex::new(r"INFO:").unwrap(), LogLevel::Info),
        (Regex::new(r"DEBUG:").unwrap(), LogLevel::Debug),
    ]
});

/// 正则表达式缓存 - 时间戳提取
static TIMESTAMP_REGEXES: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        // ISO 8601: 2025-10-16T10:30:45Z
        (Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").unwrap(), "%Y-%m-%dT%H:%M:%S"),
        // 2025-10-16 10:30:45
        (Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap(), "%Y-%m-%d %H:%M:%S"),
        // [2025-10-16 10:30:45]
        (Regex::new(r"\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\]").unwrap(), "%Y-%m-%d %H:%M:%S"),
    ]
});

/// 正则表达式缓存 - 消息提取
static MESSAGE_CLEANUP_REGEXES: Lazy<[Regex; 2]> = Lazy::new(|| {
    [
        // 移除时间戳
        Regex::new(r"^\[?\d{4}-\d{2}-\d{2}[T ]?\d{2}:\d{2}:\d{2}\]?\s*").unwrap(),
        // 移除日志级别
        Regex::new(r"^\[?(ERROR|WARN|INFO|DEBUG|TRACE)\]?:?\s*").unwrap(),
    ]
});

/// 正则表达式缓存 - 错误模式归一化
static PATTERN_NORMALIZE_REGEXES: Lazy<[Regex; 4]> = Lazy::new(|| {
    [
        Regex::new(r"\b\d+\b").unwrap(),           // 数字 → N
        Regex::new(r"/[\w/.-]+").unwrap(),         // 路径 → /PATH
        Regex::new(r#""[^"]*""#).unwrap(),         // 引号内容 → "..."
        Regex::new(r"0x[0-9a-fA-F]+").unwrap(),    // 十六进制地址 → 0xADDR
    ]
});

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Unknown,
}

impl LogLevel {
    /// 从字符串解析日志级别
    pub fn from_str(s: &str) -> Self {
        let s_lower = s.to_lowercase();
        if s_lower.contains("error") || s_lower.contains("err") {
            LogLevel::Error
        } else if s_lower.contains("warn") || s_lower.contains("warning") {
            LogLevel::Warn
        } else if s_lower.contains("info") {
            LogLevel::Info
        } else if s_lower.contains("debug") {
            LogLevel::Debug
        } else if s_lower.contains("trace") {
            LogLevel::Trace
        } else {
            LogLevel::Unknown
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
            LogLevel::Unknown => "UNKNOWN",
        }
    }

    /// 获取优先级（用于排序）
    pub fn priority(&self) -> u8 {
        match self {
            LogLevel::Error => 5,
            LogLevel::Warn => 4,
            LogLevel::Info => 3,
            LogLevel::Debug => 2,
            LogLevel::Trace => 1,
            LogLevel::Unknown => 0,
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 行号
    pub line_number: usize,
    /// 原始内容
    pub raw_content: String,
    /// 日志级别
    pub level: LogLevel,
    /// 时间戳（如果可解析）
    pub timestamp: Option<DateTime<Utc>>,
    /// 消息内容
    pub message: String,
    /// 是否为堆栈跟踪
    pub is_stacktrace: bool,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(line_number: usize, raw_content: String) -> Self {
        let level = Self::detect_level(&raw_content);
        let timestamp = Self::extract_timestamp(&raw_content);
        let message = Self::extract_message(&raw_content);
        let is_stacktrace = Self::is_stacktrace_line(&raw_content);

        Self {
            line_number,
            raw_content,
            level,
            timestamp,
            message,
            is_stacktrace,
        }
    }

    /// 检测日志级别（使用缓存的正则表达式）
    fn detect_level(content: &str) -> LogLevel {
        // 使用预编译的正则表达式进行匹配
        for (regex, level) in LEVEL_REGEXES.iter() {
            if regex.is_match(content) {
                return *level;
            }
        }

        // 如果包含常见错误关键词
        if content.contains("exception") || content.contains("Exception")
            || content.contains("panic") || content.contains("fatal") {
            return LogLevel::Error;
        }

        LogLevel::Unknown
    }

    /// 提取时间戳（使用缓存的正则表达式）
    fn extract_timestamp(content: &str) -> Option<DateTime<Utc>> {
        // 使用预编译的正则表达式进行匹配
        for (regex, format) in TIMESTAMP_REGEXES.iter() {
            if let Some(matched) = regex.find(content) {
                let ts_str = matched.as_str().trim_matches(|c| c == '[' || c == ']');

                // 尝试解析
                if let Ok(dt) = NaiveDateTime::parse_from_str(ts_str, format) {
                    return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
                }
            }
        }

        None
    }

    /// 提取消息内容（使用缓存的正则表达式）
    fn extract_message(content: &str) -> String {
        // 移除时间戳和日志级别，保留核心消息
        let mut message = content.to_string();

        // 使用预编译的正则表达式清理
        for regex in MESSAGE_CLEANUP_REGEXES.iter() {
            message = regex.replace(&message, "").to_string();
        }

        message.trim().to_string()
    }

    /// 判断是否为堆栈跟踪行
    fn is_stacktrace_line(content: &str) -> bool {
        content.trim_start().starts_with("at ")
            || content.contains("Stack trace:")
            || content.contains("Traceback")
            || content.trim_start().starts_with("File \"")
            || content.contains("panicked at")
    }
}

/// 日志分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysis {
    /// 总行数
    pub total_lines: usize,
    /// 各级别统计
    pub level_counts: HashMap<LogLevel, usize>,
    /// 错误条目
    pub errors: Vec<LogEntry>,
    /// 警告条目
    pub warnings: Vec<LogEntry>,
    /// 错误模式（错误消息 -> 出现次数）
    pub error_patterns: HashMap<String, usize>,
    /// 时间范围
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl LogAnalysis {
    /// 创建空的分析结果
    pub fn new() -> Self {
        Self {
            total_lines: 0,
            level_counts: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            error_patterns: HashMap::new(),
            time_range: None,
        }
    }

    /// 获取错误总数
    pub fn error_count(&self) -> usize {
        *self.level_counts.get(&LogLevel::Error).unwrap_or(&0)
    }

    /// 获取警告总数
    pub fn warning_count(&self) -> usize {
        *self.level_counts.get(&LogLevel::Warn).unwrap_or(&0)
    }

    /// 获取信息总数
    pub fn info_count(&self) -> usize {
        *self.level_counts.get(&LogLevel::Info).unwrap_or(&0)
    }

    /// 获取最常见的错误模式（前N个）
    pub fn top_error_patterns(&self, n: usize) -> Vec<(String, usize)> {
        let mut patterns: Vec<_> = self.error_patterns.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1));
        patterns.into_iter().take(n).collect()
    }
}

impl Default for LogAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// 日志分析器
pub struct LogAnalyzer {
    /// 最大读取行数
    max_lines: Option<usize>,
    /// 只分析错误
    errors_only: bool,
    /// 包含堆栈跟踪（预留功能）
    #[allow(dead_code)]
    include_stacktrace: bool,
    /// ✨ Phase 7.2: 最大保存错误数（避免内存占用过大）
    max_errors: Option<usize>,
    /// ✨ Phase 7.2: 最大保存警告数
    max_warnings: Option<usize>,
}

impl LogAnalyzer {
    /// 创建新的日志分析器
    pub fn new() -> Self {
        Self {
            max_lines: None,
            errors_only: false,
            include_stacktrace: true,
            max_errors: None,
            max_warnings: None,
        }
    }

    /// 设置最大读取行数
    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// 只分析错误
    pub fn errors_only(mut self) -> Self {
        self.errors_only = true;
        self
    }

    /// ✨ Phase 7.2: 设置最大保存错误数
    pub fn with_max_errors(mut self, max_errors: usize) -> Self {
        self.max_errors = Some(max_errors);
        self
    }

    /// ✨ Phase 7.2: 设置最大保存警告数
    pub fn with_max_warnings(mut self, max_warnings: usize) -> Self {
        self.max_warnings = Some(max_warnings);
        self
    }

    /// 分析日志文件
    pub fn analyze_file<P: AsRef<Path>>(&self, path: P) -> Result<LogAnalysis, String> {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("无法打开文件: {}", e))?;

        let reader = BufReader::new(file);
        let mut analysis = LogAnalysis::new();

        let mut line_number = 0;
        let mut min_time: Option<DateTime<Utc>> = None;
        let mut max_time: Option<DateTime<Utc>> = None;

        for line_result in reader.lines() {
            line_number += 1;

            // 检查行数限制
            if let Some(max) = self.max_lines {
                if line_number > max {
                    break;
                }
            }

            let line = line_result.map_err(|e| format!("读取行失败: {}", e))?;

            if line.trim().is_empty() {
                continue;
            }

            analysis.total_lines += 1;

            let entry = LogEntry::new(line_number, line);

            // 只分析错误模式
            if self.errors_only && entry.level != LogLevel::Error {
                continue;
            }

            // 更新级别统计
            *analysis.level_counts.entry(entry.level).or_insert(0) += 1;

            // 更新时间范围
            if let Some(ts) = entry.timestamp {
                min_time = Some(min_time.map_or(ts, |t| t.min(ts)));
                max_time = Some(max_time.map_or(ts, |t| t.max(ts)));
            }

            // 收集错误和警告
            match entry.level {
                LogLevel::Error => {
                    // 提取错误模式
                    let pattern = self.extract_error_pattern(&entry.message);
                    *analysis.error_patterns.entry(pattern).or_insert(0) += 1;

                    // ✨ Phase 7.2: 限制保存的错误数量
                    if let Some(max) = self.max_errors {
                        if analysis.errors.len() < max {
                            analysis.errors.push(entry);
                        }
                    } else {
                        analysis.errors.push(entry);
                    }
                }
                LogLevel::Warn => {
                    // ✨ Phase 7.2: 限制保存的警告数量
                    if let Some(max) = self.max_warnings {
                        if analysis.warnings.len() < max {
                            analysis.warnings.push(entry);
                        }
                    } else {
                        analysis.warnings.push(entry);
                    }
                }
                _ => {}
            }
        }

        // 设置时间范围
        if let (Some(min), Some(max)) = (min_time, max_time) {
            analysis.time_range = Some((min, max));
        }

        Ok(analysis)
    }

    /// 分析最近的N行日志
    pub fn analyze_tail<P: AsRef<Path>>(&self, path: P, lines: usize) -> Result<LogAnalysis, String> {
        // 读取最后N行
        let tail_lines = self.read_tail(path.as_ref(), lines)?;

        let mut analysis = LogAnalysis::new();
        let mut line_number = 0;

        for line in tail_lines {
            line_number += 1;

            if line.trim().is_empty() {
                continue;
            }

            analysis.total_lines += 1;

            let entry = LogEntry::new(line_number, line);

            // 只分析错误模式
            if self.errors_only && entry.level != LogLevel::Error {
                continue;
            }

            // 更新级别统计
            *analysis.level_counts.entry(entry.level).or_insert(0) += 1;

            // 收集错误和警告
            match entry.level {
                LogLevel::Error => {
                    let pattern = self.extract_error_pattern(&entry.message);
                    *analysis.error_patterns.entry(pattern).or_insert(0) += 1;

                    // ✨ Phase 7.2: 限制保存的错误数量
                    if let Some(max) = self.max_errors {
                        if analysis.errors.len() < max {
                            analysis.errors.push(entry);
                        }
                    } else {
                        analysis.errors.push(entry);
                    }
                }
                LogLevel::Warn => {
                    // ✨ Phase 7.2: 限制保存的警告数量
                    if let Some(max) = self.max_warnings {
                        if analysis.warnings.len() < max {
                            analysis.warnings.push(entry);
                        }
                    } else {
                        analysis.warnings.push(entry);
                    }
                }
                _ => {}
            }
        }

        Ok(analysis)
    }

    /// 读取文件的最后N行
    fn read_tail<P: AsRef<Path>>(&self, path: P, n: usize) -> Result<Vec<String>, String> {
        use std::collections::VecDeque;

        let file = File::open(path.as_ref())
            .map_err(|e| format!("无法打开文件: {}", e))?;

        let reader = BufReader::new(file);
        let mut buffer: VecDeque<String> = VecDeque::with_capacity(n);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("读取行失败: {}", e))?;

            if buffer.len() >= n {
                buffer.pop_front();
            }
            buffer.push_back(line);
        }

        Ok(buffer.into_iter().collect())
    }

    /// 提取错误模式（移除动态部分，使用缓存的正则表达式）
    fn extract_error_pattern(&self, message: &str) -> String {
        let mut pattern = message.to_string();

        // 使用预编译的正则表达式进行归一化
        // 顺序: 数字 → 路径 → 引号内容 → 十六进制地址
        pattern = PATTERN_NORMALIZE_REGEXES[0].replace_all(&pattern, "N").to_string();
        pattern = PATTERN_NORMALIZE_REGEXES[1].replace_all(&pattern, "/PATH").to_string();
        pattern = PATTERN_NORMALIZE_REGEXES[2].replace_all(&pattern, "\"...\"").to_string();
        pattern = PATTERN_NORMALIZE_REGEXES[3].replace_all(&pattern, "0xADDR").to_string();

        pattern
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("unknown"), LogLevel::Unknown);
    }

    #[test]
    fn test_log_level_priority() {
        assert!(LogLevel::Error.priority() > LogLevel::Warn.priority());
        assert!(LogLevel::Warn.priority() > LogLevel::Info.priority());
    }

    #[test]
    fn test_detect_level() {
        let entry1 = LogEntry::new(1, "[ERROR] Something went wrong".to_string());
        assert_eq!(entry1.level, LogLevel::Error);

        let entry2 = LogEntry::new(2, "INFO: Application started".to_string());
        assert_eq!(entry2.level, LogLevel::Info);

        let entry3 = LogEntry::new(3, "Exception occurred in module".to_string());
        assert_eq!(entry3.level, LogLevel::Error);
    }

    #[test]
    fn test_extract_timestamp() {
        let content = "[2025-10-16 10:30:45] ERROR: Test error";
        let entry = LogEntry::new(1, content.to_string());
        assert!(entry.timestamp.is_some());
    }

    #[test]
    fn test_is_stacktrace() {
        assert!(LogEntry::is_stacktrace_line("  at main.rs:123"));
        assert!(LogEntry::is_stacktrace_line("Stack trace:"));
        assert!(LogEntry::is_stacktrace_line("  File \"/path/to/file.py\", line 42"));
        assert!(!LogEntry::is_stacktrace_line("Regular log message"));
    }

    #[test]
    fn test_extract_error_pattern() {
        let analyzer = LogAnalyzer::new();

        let pattern1 = analyzer.extract_error_pattern("Error at line 123 in /path/to/file.rs");
        assert_eq!(pattern1, "Error at line N in /PATH");

        let pattern2 = analyzer.extract_error_pattern("Failed to parse \"hello world\"");
        assert_eq!(pattern2, "Failed to parse \"...\"");

        let pattern3 = analyzer.extract_error_pattern("Null pointer at 0x7fff5fbff710");
        assert_eq!(pattern3, "Null pointer at 0xADDR");
    }

    #[test]
    fn test_log_analysis_top_patterns() {
        let mut analysis = LogAnalysis::new();
        analysis.error_patterns.insert("Error A".to_string(), 10);
        analysis.error_patterns.insert("Error B".to_string(), 5);
        analysis.error_patterns.insert("Error C".to_string(), 15);

        let top = analysis.top_error_patterns(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "Error C");
        assert_eq!(top[0].1, 15);
        assert_eq!(top[1].0, "Error A");
        assert_eq!(top[1].1, 10);
    }
}
