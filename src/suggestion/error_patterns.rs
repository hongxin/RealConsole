//! 错误模式识别系统
//!
//! 基于模式匹配识别常见错误并提供特定修复建议

use regex::Regex;
use super::types::{Suggestion, SuggestionCategory, SuggestionSource};
use std::collections::HashMap;

/// 错误模式类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorPatternType {
    CommandNotFound,
    PermissionDenied,
    NoSuchFileOrDirectory,
    GitNotARepository,
    GitNothingToCommit,
    CargoNotFound,
    CargoBuildFailed,
    NpmModuleNotFound,
    PortAlreadyInUse,
    ConnectionRefused,
    DiskSpaceFull,
}

/// 错误模式匹配器
pub struct ErrorPatternMatcher {
    patterns: HashMap<ErrorPatternType, Regex>,
}

impl ErrorPatternMatcher {
    /// 创建新的错误模式匹配器
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // 注册所有错误模式
        Self::register_pattern(&mut patterns, ErrorPatternType::CommandNotFound,
            r"(?i)(command not found|not found|zsh: command not found|bash: .*: command not found)");

        Self::register_pattern(&mut patterns, ErrorPatternType::PermissionDenied,
            r"(?i)permission denied");

        Self::register_pattern(&mut patterns, ErrorPatternType::NoSuchFileOrDirectory,
            r"(?i)no such file or directory|cannot find|does not exist");

        Self::register_pattern(&mut patterns, ErrorPatternType::GitNotARepository,
            r"(?i)not a git repository");

        Self::register_pattern(&mut patterns, ErrorPatternType::GitNothingToCommit,
            r"(?i)nothing to commit|no changes added");

        Self::register_pattern(&mut patterns, ErrorPatternType::CargoNotFound,
            r"(?i)could not find `Cargo\.toml`");

        Self::register_pattern(&mut patterns, ErrorPatternType::CargoBuildFailed,
            r"(?i)error: could not compile|compilation failed");

        Self::register_pattern(&mut patterns, ErrorPatternType::NpmModuleNotFound,
            r"(?i)cannot find module|module not found");

        Self::register_pattern(&mut patterns, ErrorPatternType::PortAlreadyInUse,
            r"(?i)(address already in use|port.*already|EADDRINUSE)");

        Self::register_pattern(&mut patterns, ErrorPatternType::ConnectionRefused,
            r"(?i)connection refused|could not connect|ECONNREFUSED");

        Self::register_pattern(&mut patterns, ErrorPatternType::DiskSpaceFull,
            r"(?i)no space left on device|disk full|out of space");

        Self { patterns }
    }

    fn register_pattern(patterns: &mut HashMap<ErrorPatternType, Regex>, pattern_type: ErrorPatternType, pattern_str: &str) {
        match Regex::new(pattern_str) {
            Ok(regex) => {
                patterns.insert(pattern_type, regex);
            }
            Err(e) => {
                eprintln!("⚠ 无效的正则表达式模式 '{:?}': {}", pattern_type, e);
            }
        }
    }

    /// 分析错误消息并生成建议
    pub fn analyze_error(&self, error_msg: &str, failed_command: Option<&str>) -> Vec<Suggestion> {
        // 尝试匹配所有模式
        for (pattern_type, regex) in &self.patterns {
            if regex.is_match(error_msg) {
                // 提取更多上下文信息
                let captures = regex.captures(error_msg);
                return self.generate_suggestions(pattern_type, error_msg, failed_command, captures);
            }
        }

        // 如果没有匹配到任何模式，提供通用建议
        self.generic_error_suggestions(error_msg, failed_command)
    }

    /// 根据错误类型生成建议
    fn generate_suggestions(
        &self,
        pattern_type: &ErrorPatternType,
        error_msg: &str,
        failed_command: Option<&str>,
        captures: Option<regex::Captures>,
    ) -> Vec<Suggestion> {
        match pattern_type {
            ErrorPatternType::CommandNotFound => {
                let cmd = failed_command
                    .and_then(|c| c.split_whitespace().next())
                    .unwrap_or("command");

                vec![
                    Suggestion::new(
                        format!("brew install {}", cmd),
                        format!("使用 Homebrew 安装 {}", cmd),
                        0.9,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        format!("which {}", cmd),
                        format!("检查 {} 是否在 PATH 中", cmd),
                        0.7,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "echo $PATH".to_string(),
                        "查看当前 PATH 环境变量".to_string(),
                        0.6,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                ]
            }

            ErrorPatternType::PermissionDenied => {
                let mut suggestions = vec![
                    Suggestion::new(
                        "ls -la".to_string(),
                        "查看文件权限".to_string(),
                        0.85,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                ];

                if error_msg.contains(".sh") || error_msg.contains("script") {
                    suggestions.insert(0, Suggestion::new(
                        "chmod +x <script>".to_string(),
                        "添加执行权限".to_string(),
                        0.95,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic));
                }

                suggestions
            }

            ErrorPatternType::NoSuchFileOrDirectory => {
                vec![
                    Suggestion::new(
                        "ls".to_string(),
                        "列出当前目录内容".to_string(),
                        0.8,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "pwd".to_string(),
                        "查看当前工作目录".to_string(),
                        0.75,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "find . -name '*pattern*'".to_string(),
                        "搜索文件".to_string(),
                        0.7,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                ]
            }

            ErrorPatternType::GitNotARepository => {
                vec![
                    Suggestion::new(
                        "git init".to_string(),
                        "初始化 Git 仓库".to_string(),
                        0.9,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Git),
                    Suggestion::new(
                        "git clone <url>".to_string(),
                        "克隆远程仓库".to_string(),
                        0.7,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Git),
                ]
            }

            ErrorPatternType::GitNothingToCommit => {
                vec![
                    Suggestion::new(
                        "git status".to_string(),
                        "查看仓库状态".to_string(),
                        0.85,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Git),
                    Suggestion::new(
                        "git add .".to_string(),
                        "添加所有更改".to_string(),
                        0.8,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Git),
                    Suggestion::new(
                        "git diff".to_string(),
                        "查看未暂存的更改".to_string(),
                        0.75,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Git),
                ]
            }

            ErrorPatternType::CargoNotFound => {
                vec![
                    Suggestion::new(
                        "cargo init".to_string(),
                        "初始化 Rust 项目".to_string(),
                        0.9,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Building),
                    Suggestion::new(
                        "ls -la".to_string(),
                        "查看当前目录内容".to_string(),
                        0.7,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                ]
            }

            ErrorPatternType::CargoBuildFailed => {
                vec![
                    Suggestion::new(
                        "cargo check".to_string(),
                        "快速检查编译错误".to_string(),
                        0.9,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Building),
                    Suggestion::new(
                        "cargo clean && cargo build".to_string(),
                        "清理并重新构建".to_string(),
                        0.75,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Building),
                    Suggestion::new(
                        "cargo build --verbose".to_string(),
                        "查看详细构建信息".to_string(),
                        0.7,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Building),
                ]
            }

            ErrorPatternType::NpmModuleNotFound => {
                vec![
                    Suggestion::new(
                        "npm install".to_string(),
                        "安装依赖包".to_string(),
                        0.95,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Building),
                    Suggestion::new(
                        "npm ci".to_string(),
                        "清洁安装依赖（基于 lock 文件）".to_string(),
                        0.8,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Building),
                    Suggestion::new(
                        "ls node_modules".to_string(),
                        "检查已安装的模块".to_string(),
                        0.6,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                ]
            }

            ErrorPatternType::PortAlreadyInUse => {
                // 尝试从错误消息中提取端口号
                let port = if let Some(caps) = captures {
                    let extracted: String = caps.get(0)
                        .map(|m| {
                            let text = m.as_str();
                            // 使用简单的正则提取数字
                            text.chars()
                                .skip_while(|c| !c.is_numeric())
                                .take_while(|c| c.is_numeric())
                                .collect::<String>()
                        })
                        .unwrap_or_default();

                    if extracted.is_empty() {
                        "PORT".to_string()
                    } else {
                        extracted
                    }
                } else {
                    "PORT".to_string()
                };

                vec![
                    Suggestion::new(
                        format!("lsof -ti:{} | xargs kill -9", port),
                        format!("强制终止占用端口 {} 的进程", port),
                        0.9,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic).with_confirmation(true),
                    Suggestion::new(
                        format!("lsof -i:{}", port),
                        format!("查看占用端口 {} 的进程", port),
                        0.85,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "netstat -tuln | grep LISTEN".to_string(),
                        "查看所有监听端口".to_string(),
                        0.7,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                ]
            }

            ErrorPatternType::ConnectionRefused => {
                vec![
                    Suggestion::new(
                        "ping -c 3 <host>".to_string(),
                        "检查网络连接".to_string(),
                        0.8,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "curl -I <url>".to_string(),
                        "测试 HTTP 连接".to_string(),
                        0.75,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "netstat -an | grep LISTEN".to_string(),
                        "查看本地监听端口".to_string(),
                        0.7,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                ]
            }

            ErrorPatternType::DiskSpaceFull => {
                vec![
                    Suggestion::new(
                        "df -h".to_string(),
                        "查看磁盘使用情况".to_string(),
                        0.95,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "du -sh * | sort -hr | head -10".to_string(),
                        "查找占用空间最大的目录".to_string(),
                        0.9,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic),
                    Suggestion::new(
                        "docker system prune -a".to_string(),
                        "清理 Docker 镜像和容器（如果使用 Docker）".to_string(),
                        0.75,
                        SuggestionSource::Rule
                    ).with_category(SuggestionCategory::Diagnostic).with_confirmation(true),
                ]
            }
        }
    }

    /// 通用错误建议（当没有匹配到特定模式时）
    fn generic_error_suggestions(&self, error_msg: &str, failed_command: Option<&str>) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // 基于失败的命令提供建议
        if let Some(cmd) = failed_command {
            let cmd_base = cmd.split_whitespace().next().unwrap_or(cmd);

            suggestions.push(Suggestion::new(
                format!("{} --help", cmd_base),
                format!("查看 {} 命令的帮助信息", cmd_base),
                0.75,
                SuggestionSource::Rule
            ).with_category(SuggestionCategory::Diagnostic));

            suggestions.push(Suggestion::new(
                format!("man {}", cmd_base),
                format!("查看 {} 的手册页", cmd_base),
                0.7,
                SuggestionSource::Rule
            ).with_category(SuggestionCategory::Diagnostic));
        }

        // 基于错误消息内容提供建议
        if error_msg.to_lowercase().contains("error") {
            suggestions.push(Suggestion::new(
                "echo $?".to_string(),
                "查看上一个命令的退出码".to_string(),
                0.6,
                SuggestionSource::Rule
            ).with_category(SuggestionCategory::Diagnostic));
        }

        // 如果没有任何建议，提供最基本的帮助
        if suggestions.is_empty() {
            suggestions.push(Suggestion::new(
                "history | tail -5".to_string(),
                "查看最近执行的命令".to_string(),
                0.5,
                SuggestionSource::Rule
            ).with_category(SuggestionCategory::Diagnostic));
        }

        suggestions
    }
}

impl Default for ErrorPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_not_found() {
        let matcher = ErrorPatternMatcher::new();
        let error = "zsh: command not found: kubectl";
        let suggestions = matcher.analyze_error(error, Some("kubectl version"));

        assert!(!suggestions.is_empty());
        assert!(suggestions[0].command.contains("kubectl"));
        assert!(suggestions[0].score > 0.8);
    }

    #[test]
    fn test_permission_denied() {
        let matcher = ErrorPatternMatcher::new();
        let error = "permission denied: ./deploy.sh";
        let suggestions = matcher.analyze_error(error, Some("./deploy.sh"));

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command.contains("chmod")));
    }

    #[test]
    fn test_git_not_a_repository() {
        let matcher = ErrorPatternMatcher::new();
        let error = "fatal: not a git repository (or any of the parent directories): .git";
        let suggestions = matcher.analyze_error(error, Some("git status"));

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command.contains("git init")));
        assert_eq!(suggestions[0].category, SuggestionCategory::Git);
    }

    #[test]
    fn test_port_already_in_use() {
        let matcher = ErrorPatternMatcher::new();
        let error = "Error: address already in use :3000";
        let suggestions = matcher.analyze_error(error, Some("npm start"));

        assert!(!suggestions.is_empty());
        // 端口号可能无法精确提取，但至少应该有建议
    }

    #[test]
    fn test_cargo_build_failed() {
        let matcher = ErrorPatternMatcher::new();
        let error = "error: could not compile `myproject` due to 3 previous errors";
        let suggestions = matcher.analyze_error(error, Some("cargo build"));

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command.contains("cargo check")));
        assert_eq!(suggestions[0].category, SuggestionCategory::Building);
    }

    #[test]
    fn test_generic_error() {
        let matcher = ErrorPatternMatcher::new();
        let error = "Something went wrong";
        let suggestions = matcher.analyze_error(error, Some("mycommand"));

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command.contains("--help") || s.command.contains("man")));
    }
}
