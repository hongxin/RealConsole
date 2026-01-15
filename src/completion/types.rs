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

/// Git 仓库上下文 (v1.85.0)
#[derive(Debug, Clone, Default)]
pub struct GitContext {
    /// 是否在 Git 仓库中
    pub is_git_repo: bool,

    /// 当前分支名
    pub branch: Option<String>,

    /// 是否有未暂存的更改
    pub has_changes: bool,

    /// 是否有未跟踪的文件
    pub has_untracked: bool,

    /// 是否有暂存的更改
    pub has_staged: bool,
}

impl GitContext {
    /// 检测当前目录的 Git 状态
    pub fn detect() -> Self {
        // 检查是否在 git 仓库中
        let is_git_repo = std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !is_git_repo {
            return Self::default();
        }

        // 获取当前分支
        let branch = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .filter(|s| !s.is_empty());

        // 获取 git status --porcelain
        let status_output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let mut has_changes = false;
        let mut has_untracked = false;
        let mut has_staged = false;

        for line in status_output.lines() {
            if line.starts_with("??") {
                has_untracked = true;
            } else if line.starts_with(' ') {
                has_changes = true; // 工作区有修改
            } else if !line.is_empty() {
                // 第一个字符不是空格且不是 ?? 说明有暂存的更改
                let first_char = line.chars().next().unwrap_or(' ');
                if first_char != ' ' && first_char != '?' {
                    has_staged = true;
                }
                // 第二个字符不是空格说明工作区有修改
                if line.len() > 1 {
                    let second_char = line.chars().nth(1).unwrap_or(' ');
                    if second_char != ' ' {
                        has_changes = true;
                    }
                }
            }
        }

        Self {
            is_git_repo,
            branch,
            has_changes,
            has_untracked,
            has_staged,
        }
    }

    /// 根据 Git 状态推荐相关命令
    pub fn suggest_commands(&self) -> Vec<(&'static str, &'static str)> {
        if !self.is_git_repo {
            return vec![("git init", "初始化 Git 仓库")];
        }

        let mut suggestions = vec![
            ("git status", "查看仓库状态"),
            ("git log --oneline -10", "查看最近提交"),
        ];

        if self.has_untracked {
            suggestions.push(("git add .", "添加所有文件到暂存区"));
            suggestions.push(("git add -p", "交互式添加更改"));
        }

        if self.has_changes {
            suggestions.push(("git diff", "查看未暂存的更改"));
            suggestions.push(("git checkout -- .", "撤销工作区更改"));
        }

        if self.has_staged {
            suggestions.push(("git commit -m \"\"", "提交暂存的更改"));
            suggestions.push(("git diff --cached", "查看已暂存的更改"));
            suggestions.push(("git reset HEAD", "取消暂存"));
        }

        if !self.has_changes && !self.has_untracked && !self.has_staged {
            suggestions.push(("git pull", "拉取远程更新"));
            suggestions.push(("git push", "推送到远程"));
            suggestions.push(("git fetch", "获取远程更新"));
        }

        suggestions
    }
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

    /// Git 仓库上下文 (v1.85.0)
    pub git_context: GitContext,
}

impl Default for CompletionContext {
    fn default() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            recent_commands: Vec::new(),
            conversation_summary: String::new(),
            usage_stats: HashMap::new(),
            git_context: GitContext::default(),
        }
    }
}

impl CompletionContext {
    /// 创建带 Git 上下文检测的补全上下文 (v1.85.0)
    pub fn with_git_detection() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            recent_commands: Vec::new(),
            conversation_summary: String::new(),
            usage_stats: HashMap::new(),
            git_context: GitContext::detect(),
        }
    }

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

    // ===== v1.85.0: Git 上下文测试 =====

    #[test]
    fn test_git_context_default() {
        let context = GitContext::default();
        assert!(!context.is_git_repo);
        assert!(context.branch.is_none());
        assert!(!context.has_changes);
        assert!(!context.has_untracked);
        assert!(!context.has_staged);
    }

    #[test]
    fn test_git_context_detect() {
        // GitContext::detect() should work without panicking
        let context = GitContext::detect();

        // Valid states:
        // 1. In git repo with branch info (normal state)
        // 2. In git repo without branch (detached HEAD, rebasing, fresh init)
        // 3. Not in git repo (CI sandbox, non-git directory)
        //
        // All states are valid - we just verify detect() doesn't panic
        // and returns consistent data
        if !context.is_git_repo {
            // Not in a git repo - should have default values
            assert!(context.branch.is_none());
            assert!(!context.has_changes);
            assert!(!context.has_untracked);
            assert!(!context.has_staged);
        }
        // If is_git_repo is true, branch may or may not be Some
        // depending on git state (detached HEAD, etc.)
    }

    #[test]
    fn test_git_context_suggest_commands_in_repo() {
        let context = GitContext {
            is_git_repo: true,
            branch: Some("main".to_string()),
            has_changes: true,
            has_untracked: true,
            has_staged: false,
        };

        let suggestions = context.suggest_commands();

        // 应该有基本命令
        assert!(suggestions.iter().any(|(cmd, _)| *cmd == "git status"));

        // 有未跟踪文件时应该建议 git add
        assert!(suggestions.iter().any(|(cmd, _)| cmd.starts_with("git add")));

        // 有未暂存更改时应该建议 git diff
        assert!(suggestions.iter().any(|(cmd, _)| *cmd == "git diff"));
    }

    #[test]
    fn test_git_context_suggest_commands_staged() {
        let context = GitContext {
            is_git_repo: true,
            branch: Some("feature".to_string()),
            has_changes: false,
            has_untracked: false,
            has_staged: true,
        };

        let suggestions = context.suggest_commands();

        // 有暂存的更改时应该建议 commit
        assert!(suggestions.iter().any(|(cmd, _)| cmd.starts_with("git commit")));
    }

    #[test]
    fn test_git_context_suggest_commands_clean() {
        let context = GitContext {
            is_git_repo: true,
            branch: Some("main".to_string()),
            has_changes: false,
            has_untracked: false,
            has_staged: false,
        };

        let suggestions = context.suggest_commands();

        // 工作区干净时应该建议 pull/push
        assert!(suggestions.iter().any(|(cmd, _)| *cmd == "git pull"));
        assert!(suggestions.iter().any(|(cmd, _)| *cmd == "git push"));
    }

    #[test]
    fn test_git_context_suggest_commands_not_repo() {
        let context = GitContext {
            is_git_repo: false,
            branch: None,
            has_changes: false,
            has_untracked: false,
            has_staged: false,
        };

        let suggestions = context.suggest_commands();

        // 不在 Git 仓库时应该建议 git init
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].0, "git init");
    }

    #[test]
    fn test_completion_context_with_git_detection() {
        let context = CompletionContext::with_git_detection();

        // Verify the context is created with git detection
        // The actual git_context.is_git_repo depends on the environment
        // We just verify the function doesn't panic and returns valid data

        // Note: Even if is_git_repo is true, branch might be None in
        // detached HEAD state or other edge cases - both are valid

        // Context should have valid current directory
        assert!(!context.current_dir.as_os_str().is_empty());
    }
}
