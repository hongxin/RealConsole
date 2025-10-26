//! 基于上下文的建议生成器
//!
//! 根据项目类型、当前目录等上下文信息生成建议

use super::error_patterns::ErrorPatternMatcher; // ✨ Phase 4.2
use super::types::{
    FileType, Suggestion, SuggestionCategory, SuggestionContext, SuggestionSource,
};
use std::path::Path;

/// 基于上下文的建议生成器
pub struct ContextSuggester {
    /// 是否启用项目类型检测
    enable_project_detection: bool,
    /// ✨ Phase 4.2: 错误模式匹配器
    error_matcher: ErrorPatternMatcher,
}

impl ContextSuggester {
    /// 创建新的上下文建议生成器
    pub fn new() -> Self {
        Self {
            enable_project_detection: true,
            error_matcher: ErrorPatternMatcher::new(), // ✨ Phase 4.2
        }
    }

    /// 生成建议
    pub async fn suggest(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // 1. 基于项目类型的建议
        if self.enable_project_detection {
            if let Some(project_type) = self.detect_project_type(&context.current_dir).await {
                suggestions.extend(self.suggest_for_project_type(&project_type));
            }
        }

        // 2. 基于上一次命令失败的建议
        if context.last_command_failed {
            suggestions.extend(self.suggest_for_failure(context));
        }

        suggestions
    }

    /// 检测项目类型
    async fn detect_project_type(&self, dir: &Path) -> Option<FileType> {
        // 检查 Cargo.toml
        if dir.join("Cargo.toml").exists() {
            return Some(FileType::RustProject);
        }

        // 检查 package.json
        if dir.join("package.json").exists() {
            return Some(FileType::NodeProject);
        }

        // 检查 requirements.txt 或 pyproject.toml
        if dir.join("requirements.txt").exists() || dir.join("pyproject.toml").exists() {
            return Some(FileType::PythonProject);
        }

        // 检查 .git
        if dir.join(".git").exists() {
            return Some(FileType::GitRepository);
        }

        // 检查 Dockerfile
        if dir.join("Dockerfile").exists() {
            return Some(FileType::DockerProject);
        }

        None
    }

    /// 为特定项目类型生成建议
    fn suggest_for_project_type(&self, project_type: &FileType) -> Vec<Suggestion> {
        match project_type {
            FileType::RustProject => self.suggest_for_rust(),
            FileType::NodeProject => self.suggest_for_node(),
            FileType::PythonProject => self.suggest_for_python(),
            FileType::GitRepository => self.suggest_for_git(),
            FileType::DockerProject => self.suggest_for_docker(),
            FileType::Custom(_) => Vec::new(),
        }
    }

    /// Rust 项目建议
    fn suggest_for_rust(&self) -> Vec<Suggestion> {
        vec![
            Suggestion::new(
                "cargo build --release",
                "Build optimized binary",
                0.85,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Building),
            Suggestion::new(
                "cargo test",
                "Run all tests",
                0.82,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Testing),
            Suggestion::new(
                "cargo check",
                "Quick syntax check",
                0.78,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Building),
            Suggestion::new(
                "cargo clippy",
                "Run linter",
                0.75,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Diagnostic),
        ]
    }

    /// Node.js 项目建议
    fn suggest_for_node(&self) -> Vec<Suggestion> {
        vec![
            Suggestion::new(
                "npm install",
                "Install dependencies",
                0.85,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Project),
            Suggestion::new(
                "npm test",
                "Run tests",
                0.80,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Testing),
            Suggestion::new(
                "npm run build",
                "Build project",
                0.75,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Building),
            Suggestion::new(
                "npm start",
                "Start development server",
                0.78,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Project),
        ]
    }

    /// Python 项目建议
    fn suggest_for_python(&self) -> Vec<Suggestion> {
        vec![
            Suggestion::new(
                "pip install -r requirements.txt",
                "Install dependencies",
                0.85,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Project),
            Suggestion::new(
                "python -m pytest",
                "Run tests",
                0.80,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Testing),
            Suggestion::new(
                "python -m venv venv",
                "Create virtual environment",
                0.70,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Project),
        ]
    }

    /// Git 仓库建议
    fn suggest_for_git(&self) -> Vec<Suggestion> {
        vec![
            Suggestion::new(
                "git status",
                "Check repository status",
                0.90,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Git),
            Suggestion::new(
                "git pull",
                "Pull latest changes",
                0.75,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Git),
            Suggestion::new(
                "git log --oneline -10",
                "View recent commits",
                0.70,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Git),
        ]
    }

    /// Docker 项目建议
    fn suggest_for_docker(&self) -> Vec<Suggestion> {
        vec![
            Suggestion::new(
                "docker build -t myapp .",
                "Build Docker image",
                0.85,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Building),
            Suggestion::new(
                "docker compose up",
                "Start services",
                0.80,
                SuggestionSource::Context,
            )
            .with_category(SuggestionCategory::Project),
        ]
    }

    /// 为命令失败生成建议
    fn suggest_for_failure(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // ✨ Phase 4.2: 优先使用错误模式匹配器
        if let Some(ref error_output) = context.last_command_output {
            let failed_cmd = if !context.recent_commands.is_empty() {
                Some(context.recent_commands[0].as_str())
            } else {
                None
            };

            // 使用错误模式匹配器分析错误
            let pattern_suggestions = self.error_matcher.analyze_error(error_output, failed_cmd);
            suggestions.extend(pattern_suggestions);
        }

        // 如果没有错误输出或者没有匹配到模式，使用通用建议
        if suggestions.is_empty() && !context.recent_commands.is_empty() {
            let last_cmd = &context.recent_commands[0];

            // 建议查看帮助
            suggestions.push(
                Suggestion::new(
                    format!("{} --help", last_cmd),
                    "View command help",
                    0.80,
                    SuggestionSource::Context,
                )
                .with_category(SuggestionCategory::Diagnostic),
            );

            // 如果是 cargo，建议详细输出
            if last_cmd.starts_with("cargo") {
                suggestions.push(
                    Suggestion::new(
                        format!("{} --verbose", last_cmd),
                        "Run with verbose output",
                        0.75,
                        SuggestionSource::Context,
                    )
                    .with_category(SuggestionCategory::Diagnostic),
                );
            }
        }

        suggestions
    }
}

impl Default for ContextSuggester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_detect_rust_project() {
        // 使用当前项目目录（应该有 Cargo.toml）
        let current_dir = std::env::current_dir().unwrap();
        let suggester = ContextSuggester::new();

        let project_type = suggester.detect_project_type(&current_dir).await;

        // 当前项目是 Rust 项目
        assert!(matches!(project_type, Some(FileType::RustProject)));
    }

    #[test]
    fn test_rust_project_suggestions() {
        let suggester = ContextSuggester::new();
        let suggestions = suggester.suggest_for_rust();

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command.contains("cargo")));

        // 检查所有建议都有合理的分数
        for suggestion in &suggestions {
            assert!(suggestion.score >= 0.0 && suggestion.score <= 1.0);
        }
    }

    #[test]
    fn test_node_project_suggestions() {
        let suggester = ContextSuggester::new();
        let suggestions = suggester.suggest_for_node();

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command.contains("npm")));
    }

    #[tokio::test]
    async fn test_suggest_with_context() {
        let suggester = ContextSuggester::new();
        let mut context = SuggestionContext::from_env();

        // 当前是 Rust 项目，应该有 Rust 相关建议
        let suggestions = suggester.suggest(&context).await;

        assert!(!suggestions.is_empty());

        // 测试失败建议
        context.last_command_failed = true;
        context.recent_commands.push("cargo build".to_string());

        let suggestions_with_failure = suggester.suggest(&context).await;
        assert!(suggestions_with_failure.len() > suggestions.len());
    }

    #[test]
    fn test_failure_suggestions() {
        let suggester = ContextSuggester::new();
        let mut context = SuggestionContext::new(PathBuf::from("."));

        context.last_command_failed = true;
        context.recent_commands.push("cargo build".to_string());

        let suggestions = suggester.suggest_for_failure(&context);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command.contains("--help")));
    }
}
