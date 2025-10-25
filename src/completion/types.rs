//! Tab 补全系统核心数据结构

use std::collections::HashMap;
use std::path::PathBuf;

/// 补全候选
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// 补全文本
    pub text: String,

    /// 候选描述（显示给用户）
    pub description: String,

    /// 初始评分（由各 Completer 计算）
    pub score: f64,

    /// 补全源类型
    pub source: CompletionSource,
}

impl Candidate {
    /// 创建新的候选
    pub fn new(text: impl Into<String>, description: impl Into<String>, source: CompletionSource) -> Self {
        Self {
            text: text.into(),
            description: description.into(),
            score: 1.0,
            source,
        }
    }

    /// 创建带评分的候选
    pub fn with_score(
        text: impl Into<String>,
        description: impl Into<String>,
        score: f64,
        source: CompletionSource,
    ) -> Self {
        Self {
            text: text.into(),
            description: description.into(),
            score,
            source,
        }
    }
}

/// 补全源类型（体现"一分为三"思想）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionSource {
    /// 静态补全（系统命令、文件路径、历史）
    /// - 确定性：> 0.8
    /// - 速度：< 10ms
    Static,

    /// 语义补全（Intent DSL、模糊匹配）
    /// - 确定性：0.4-0.8
    /// - 速度：10-50ms
    #[allow(dead_code)] // Phase 2 将使用
    Semantic,

    /// 智能补全（LLM 预测）
    /// - 确定性：< 0.4
    /// - 速度：50-300ms
    #[allow(dead_code)] // Phase 3 将使用
    Intelligent,
}

/// 补全上下文（用于多维评分）
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// 当前工作目录
    pub current_dir: PathBuf,

    /// 最近命令（最近 5 条）
    pub recent_commands: Vec<String>,

    /// 对话上下文摘要
    pub conversation_summary: String,

    /// 命令使用频率统计
    pub usage_stats: HashMap<String, usize>,
}

impl Default for CompletionContext {
    fn default() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            recent_commands: Vec::new(),
            conversation_summary: String::new(),
            usage_stats: HashMap::new(),
        }
    }
}

impl CompletionContext {
    /// 获取命令使用频率（归一化到 0.0-1.0）
    #[allow(dead_code)] // Phase 2 多维评分时使用
    pub fn get_usage_frequency(&self, command: &str) -> f64 {
        let max_count = self.usage_stats.values().max().copied().unwrap_or(1);
        let count = self.usage_stats.get(command).copied().unwrap_or(0);

        count as f64 / max_count as f64
    }

    /// 计算上下文相关性（简单实现）
    #[allow(dead_code)] // Phase 2 多维评分时使用
    pub fn relevance_score(&self, text: &str) -> f64 {
        let mut score: f64 = 0.0;

        // 如果最近命令中包含相似文本，加分
        for recent in &self.recent_commands {
            if recent.contains(text) || text.contains(recent) {
                score += 0.3;
            }
        }

        // 如果当前目录名出现在文本中，加分
        if let Some(dir_name) = self.current_dir.file_name() {
            if let Some(dir_str) = dir_name.to_str() {
                if text.contains(dir_str) {
                    score += 0.2;
                }
            }
        }

        score.min(1.0)
    }
}

/// 补全配置
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    // ===== Phase 1: 静态补全 =====
    /// 启用静态补全
    pub enable_static: bool,

    /// 启用路径补全
    pub enable_path_completion: bool,

    /// 启用历史命令补全
    pub enable_history_completion: bool,

    // ===== Phase 2: 语义补全 =====
    /// 启用语义补全
    #[allow(dead_code)]
    pub enable_semantic: bool,

    /// 启用模糊匹配
    #[allow(dead_code)]
    pub enable_fuzzy: bool,

    /// 模糊匹配阈值
    #[allow(dead_code)]
    pub fuzzy_threshold: f64,

    // ===== Phase 3: 智能补全 =====
    /// 启用智能补全
    #[allow(dead_code)]
    pub enable_intelligent: bool,

    /// LLM 预测
    #[allow(dead_code)]
    pub llm_prediction: bool,

    /// LLM 超时（毫秒）
    #[allow(dead_code)]
    pub llm_timeout_ms: u64,

    // ===== 交互配置 =====
    /// 最多显示候选数
    pub max_candidates: usize,

    /// 补全类型
    pub completion_type: CompletionType,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            // Phase 1
            enable_static: true,
            enable_path_completion: true,
            enable_history_completion: true,

            // Phase 2
            enable_semantic: false, // Phase 2 实现后启用
            enable_fuzzy: false,
            fuzzy_threshold: 0.6,

            // Phase 3
            enable_intelligent: false,
            llm_prediction: false,
            llm_timeout_ms: 500,

            // 交互
            max_candidates: 10,
            completion_type: CompletionType::List,
        }
    }
}

/// 补全交互类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionType {
    /// 即时补全（唯一候选直接补全）
    #[allow(dead_code)]
    Instant,

    /// 列表模式（显示候选列表供选择）
    List,

    /// 循环模式（Tab 键循环选择）
    #[allow(dead_code)]
    Cyclic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_creation() {
        let candidate = Candidate::new("test", "description", CompletionSource::Static);
        assert_eq!(candidate.text, "test");
        assert_eq!(candidate.description, "description");
        assert_eq!(candidate.score, 1.0);
        assert_eq!(candidate.source, CompletionSource::Static);
    }

    #[test]
    fn test_candidate_with_score() {
        let candidate = Candidate::with_score("test", "desc", 0.8, CompletionSource::Static);
        assert_eq!(candidate.score, 0.8);
    }

    #[test]
    fn test_completion_context_default() {
        let context = CompletionContext::default();
        assert!(!context.current_dir.as_os_str().is_empty());
        assert_eq!(context.recent_commands.len(), 0);
    }

    #[test]
    fn test_usage_frequency() {
        let mut context = CompletionContext::default();
        context.usage_stats.insert("git status".to_string(), 10);
        context.usage_stats.insert("git commit".to_string(), 5);

        assert_eq!(context.get_usage_frequency("git status"), 1.0);
        assert_eq!(context.get_usage_frequency("git commit"), 0.5);
        assert_eq!(context.get_usage_frequency("unknown"), 0.0);
    }

    #[test]
    fn test_relevance_score() {
        let mut context = CompletionContext::default();
        context.recent_commands = vec!["git status".to_string()];

        let score = context.relevance_score("git");
        assert!(score > 0.0);
    }
}
