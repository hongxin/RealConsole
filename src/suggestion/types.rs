//! 建议系统核心数据类型
//!
//! 定义建议、触发器、来源等基础数据结构

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// 建议项
///
/// 表示系统给用户的一条建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// 建议的命令或操作
    pub command: String,

    /// 建议的描述/理由
    pub description: String,

    /// 建议的分数（0.0-1.0，越高越相关）
    pub score: f64,

    /// 建议来源
    pub source: SuggestionSource,

    /// 建议类别
    pub category: SuggestionCategory,

    /// 是否需要确认（对于危险操作）
    pub needs_confirmation: bool,
}

impl Suggestion {
    /// 创建新的建议
    pub fn new(
        command: impl Into<String>,
        description: impl Into<String>,
        score: f64,
        source: SuggestionSource,
    ) -> Self {
        Self {
            command: command.into(),
            description: description.into(),
            score: score.clamp(0.0, 1.0),
            source,
            category: SuggestionCategory::General,
            needs_confirmation: false,
        }
    }

    /// 设置建议类别
    pub fn with_category(mut self, category: SuggestionCategory) -> Self {
        self.category = category;
        self
    }

    /// 设置是否需要确认
    pub fn with_confirmation(mut self, needs: bool) -> Self {
        self.needs_confirmation = needs;
        self
    }

    /// 判断是否为高质量建议（分数 > 0.7）
    pub fn is_high_quality(&self) -> bool {
        self.score > 0.7
    }
}

/// 建议来源
///
/// 表示建议从哪里生成的
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SuggestionSource {
    /// 基于上下文（项目类型、当前目录）
    Context,

    /// 基于历史（常用命令）
    History,

    /// 基于 LLM 推理
    Llm,

    /// 基于规则/模板
    Rule,
}

impl SuggestionSource {
    /// 获取来源的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Context => "Context",
            Self::History => "History",
            Self::Llm => "AI",
            Self::Rule => "Rule",
        }
    }
}

/// 建议类别
///
/// 用于分类和组织建议
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SuggestionCategory {
    /// 通用建议
    General,

    /// 项目相关（如 cargo build, npm test）
    Project,

    /// Git 操作
    Git,

    /// 部署相关
    Deployment,

    /// 测试相关
    Testing,

    /// 构建相关
    Building,

    /// 诊断/调试
    Diagnostic,

    /// 自定义类别
    Custom(String),
}

impl SuggestionCategory {
    /// 获取类别的图标
    pub fn icon(&self) -> &'static str {
        match self {
            Self::General => "📋",
            Self::Project => "📦",
            Self::Git => "🔀",
            Self::Deployment => "🚀",
            Self::Testing => "🧪",
            Self::Building => "🔨",
            Self::Diagnostic => "🔍",
            Self::Custom(_) => "⚙️",
        }
    }

    /// 获取类别的显示名称
    pub fn display_name(&self) -> String {
        match self {
            Self::General => "General".to_string(),
            Self::Project => "Project".to_string(),
            Self::Git => "Git".to_string(),
            Self::Deployment => "Deployment".to_string(),
            Self::Testing => "Testing".to_string(),
            Self::Building => "Building".to_string(),
            Self::Diagnostic => "Diagnostic".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }
}

/// 建议触发器
///
/// 定义什么时候触发建议系统
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionTrigger {
    /// 用户进入新目录
    DirectoryChange(PathBuf),

    /// 用户闲置一段时间
    Idle(Duration),

    /// 命令执行失败
    CommandFailed {
        command: String,
        exit_code: i32,
        error: String,
    },

    /// 检测到特定文件（如 package.json, Cargo.toml）
    FileDetected(FileType),

    /// 用户显式请求（如 /suggest 命令）
    Explicit,

    /// REPL 启动时
    Startup,

    /// 命令执行成功后
    CommandSuccess { command: String },
}

/// 文件类型检测
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    /// Rust 项目（Cargo.toml）
    RustProject,

    /// Node.js 项目（package.json）
    NodeProject,

    /// Python 项目（requirements.txt, pyproject.toml）
    PythonProject,

    /// Git 仓库（.git/）
    GitRepository,

    /// Docker 项目（Dockerfile）
    DockerProject,

    /// 自定义文件
    Custom(String),
}

/// 建议上下文
///
/// 用于生成建议的上下文信息
#[derive(Debug, Clone)]
pub struct SuggestionContext {
    /// 当前工作目录
    pub current_dir: PathBuf,

    /// 检测到的项目类型
    pub project_type: Option<FileType>,

    /// 最近执行的命令（最多 5 条）
    pub recent_commands: Vec<String>,

    /// 上一次命令是否失败
    pub last_command_failed: bool,

    /// 上一次命令的输出（如果有）
    pub last_command_output: Option<String>,

    /// 用户的常用命令（按频率排序）
    pub frequent_commands: Vec<(String, usize)>,
}

impl SuggestionContext {
    /// 创建新的建议上下文
    pub fn new(current_dir: PathBuf) -> Self {
        Self {
            current_dir,
            project_type: None,
            recent_commands: Vec::new(),
            last_command_failed: false,
            last_command_output: None,
            frequent_commands: Vec::new(),
        }
    }

    /// 从当前环境创建上下文
    pub fn from_env() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(current_dir)
    }
}

/// 建议配置
#[derive(Debug, Clone)]
pub struct SuggestionConfig {
    /// 是否启用上下文建议
    pub enable_context: bool,

    /// 是否启用历史建议
    pub enable_history: bool,

    /// 是否启用 LLM 建议
    pub enable_llm: bool,

    /// 最大建议数量
    pub max_suggestions: usize,

    /// 最低分数阈值（低于此分数的建议会被过滤）
    pub min_score: f64,

    /// LLM 调用超时（毫秒）
    pub llm_timeout_ms: u64,

    /// 是否自动触发建议（在特定事件时）
    pub auto_trigger: bool,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        Self {
            enable_context: true,
            enable_history: true,
            enable_llm: true,
            max_suggestions: 5,
            min_score: 0.3,
            llm_timeout_ms: 2000,
            auto_trigger: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion::new(
            "cargo test",
            "Run tests for Rust project",
            0.85,
            SuggestionSource::Context,
        );

        assert_eq!(suggestion.command, "cargo test");
        assert_eq!(suggestion.score, 0.85);
        assert!(suggestion.is_high_quality());
        assert_eq!(suggestion.source, SuggestionSource::Context);
    }

    #[test]
    fn test_suggestion_score_clamping() {
        let suggestion = Suggestion::new(
            "test",
            "Test",
            1.5, // 超出范围
            SuggestionSource::History,
        );

        assert_eq!(suggestion.score, 1.0); // 应该被限制到 1.0
    }

    #[test]
    fn test_suggestion_with_category() {
        let suggestion = Suggestion::new("git status", "Check status", 0.9, SuggestionSource::History)
            .with_category(SuggestionCategory::Git)
            .with_confirmation(false);

        assert_eq!(suggestion.category, SuggestionCategory::Git);
        assert!(!suggestion.needs_confirmation);
    }

    #[test]
    fn test_suggestion_source_display() {
        assert_eq!(SuggestionSource::Context.display_name(), "Context");
        assert_eq!(SuggestionSource::Llm.display_name(), "AI");
    }

    #[test]
    fn test_suggestion_category_icon() {
        assert_eq!(SuggestionCategory::Git.icon(), "🔀");
        assert_eq!(SuggestionCategory::Testing.icon(), "🧪");
    }

    #[test]
    fn test_suggestion_context_from_env() {
        let context = SuggestionContext::from_env();
        assert!(context.current_dir.exists() || !context.current_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_suggestion_config_default() {
        let config = SuggestionConfig::default();
        assert!(config.enable_context);
        assert!(config.enable_history);
        assert!(config.enable_llm);
        assert_eq!(config.max_suggestions, 5);
    }
}
