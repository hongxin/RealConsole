//! 错误模式定义
//!
//! 定义常见的错误模式和匹配规则

use regex::Regex;
use serde::{Deserialize, Serialize};

/// 错误模式
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    /// 模式名称
    pub name: String,

    /// 匹配正则表达式
    pub regex: Regex,

    /// 错误类型
    pub category: String,

    /// 严重程度 (1-10)
    pub severity: u8,

    /// 建议的修复策略
    pub suggested_fix: String,

    /// 是否可以自动修复
    pub auto_fixable: bool,
}

impl ErrorPattern {
    /// 创建新的错误模式
    pub fn new(
        name: impl Into<String>,
        pattern: &str,
        category: impl Into<String>,
        severity: u8,
        suggested_fix: impl Into<String>,
        auto_fixable: bool,
    ) -> Result<Self, regex::Error> {
        Ok(Self {
            name: name.into(),
            regex: Regex::new(pattern)?,
            category: category.into(),
            severity,
            suggested_fix: suggested_fix.into(),
            auto_fixable,
        })
    }

    /// 匹配错误输出
    pub fn matches(&self, error_output: &str) -> bool {
        self.regex.is_match(error_output)
    }

    /// 提取错误详情
    pub fn extract_details(&self, error_output: &str) -> Option<ErrorDetails> {
        self.regex.captures(error_output).map(|caps| {
            let command = caps.get(1).map(|m| m.as_str().to_string());
            let path = caps.get(2).map(|m| m.as_str().to_string());
            let message = error_output.to_string();

            ErrorDetails {
                command,
                path,
                message,
            }
        })
    }
}

/// 错误详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// 相关命令
    pub command: Option<String>,

    /// 相关路径
    pub path: Option<String>,

    /// 错误消息
    pub message: String,
}

/// 内置错误模式库
pub struct BuiltinPatterns;

impl BuiltinPatterns {
    /// 获取所有内置模式
    pub fn all() -> Vec<ErrorPattern> {
        vec![
            // 命令不存在
            Self::command_not_found(),
            // 权限被拒绝
            Self::permission_denied(),
            // 文件不存在
            Self::file_not_found(),
            // 目录不存在
            Self::directory_not_found(),
            // 语法错误
            Self::syntax_error(),
            // 端口被占用
            Self::port_in_use(),
            // 磁盘空间不足
            Self::disk_full(),
            // 网络连接失败
            Self::connection_refused(),
            // Python 模块未找到
            Self::python_module_not_found(),
            // npm 包未安装
            Self::npm_module_not_found(),
            // Git 错误
            Self::git_error(),
            // Rust 编译错误
            Self::rust_compile_error(),
        ]
    }

    fn command_not_found() -> ErrorPattern {
        ErrorPattern::new(
            "command_not_found",
            r"(?i)(command not found|No such file or directory|not recognized as an internal|'(\w+)'.*not found)",
            "command",
            7,
            "检查命令拼写，或使用包管理器安装",
            true,
        ).unwrap()
    }

    fn permission_denied() -> ErrorPattern {
        ErrorPattern::new(
            "permission_denied",
            r"(?i)(permission denied|access denied|Operation not permitted)",
            "permission",
            8,
            "使用 chmod/sudo 或检查文件权限",
            true,
        ).unwrap()
    }

    fn file_not_found() -> ErrorPattern {
        ErrorPattern::new(
            "file_not_found",
            r#"(?i)(?:(?:no such file|cannot find|does not exist).*['"]?([^'":]+\.\w+)['"]?|([^'":]+\.\w+).*(?:no such file|cannot find|does not exist))"#,
            "file",
            6,
            "检查文件路径和名称是否正确",
            false,
        ).unwrap()
    }

    fn directory_not_found() -> ErrorPattern {
        ErrorPattern::new(
            "directory_not_found",
            r#"(?i)(?:no such (?:file or )?directory|cannot find the path).*['"]?([^'"]+/)['"]?"#,
            "directory",
            6,
            "检查目录路径是否存在，或使用 mkdir 创建",
            true,
        ).unwrap()
    }

    fn syntax_error() -> ErrorPattern {
        ErrorPattern::new(
            "syntax_error",
            r"(?i)(syntax error|unexpected token|invalid syntax)",
            "syntax",
            5,
            "检查命令语法，参考帮助文档",
            false,
        ).unwrap()
    }

    fn port_in_use() -> ErrorPattern {
        ErrorPattern::new(
            "port_in_use",
            r"(?i)(?:(?:port|address).*?(\d+).*?(?:already in use|is being used|in use)|(?:already in use|is being used|in use).*?:(\d+))",
            "network",
            7,
            "使用其他端口或关闭占用进程",
            true,
        ).unwrap()
    }

    fn disk_full() -> ErrorPattern {
        ErrorPattern::new(
            "disk_full",
            r"(?i)(no space left|disk.*full|quota exceeded)",
            "disk",
            9,
            "清理磁盘空间或扩展存储",
            false,
        ).unwrap()
    }

    fn connection_refused() -> ErrorPattern {
        ErrorPattern::new(
            "connection_refused",
            r"(?i)(connection refused|could not connect|failed to connect)",
            "network",
            6,
            "检查网络连接和服务状态",
            false,
        ).unwrap()
    }

    fn python_module_not_found() -> ErrorPattern {
        ErrorPattern::new(
            "python_module_not_found",
            r#"(?i)ModuleNotFoundError.*['"](\w+)['"]|No module named ['"](\w+)['"]"#,
            "python",
            6,
            "使用 pip install 安装缺失的模块",
            true,
        ).unwrap()
    }

    fn npm_module_not_found() -> ErrorPattern {
        ErrorPattern::new(
            "npm_module_not_found",
            r#"(?i)Cannot find module ['"](\w+)['"]|Module not found.*['"](\w+)['"]"#,
            "nodejs",
            6,
            "运行 npm install 安装依赖",
            true,
        ).unwrap()
    }

    fn git_error() -> ErrorPattern {
        ErrorPattern::new(
            "git_error",
            r"(?i)(?:fatal|error).*git.*(?:not a git repository|Your branch is behind)",
            "git",
            5,
            "检查 Git 仓库状态，可能需要 pull/commit",
            true,
        ).unwrap()
    }

    fn rust_compile_error() -> ErrorPattern {
        ErrorPattern::new(
            "rust_compile_error",
            r"(?i)error(?:\[E\d+\])?:.*cannot find.*in.*scope",
            "rust",
            6,
            "检查导入和依赖，运行 cargo check",
            false,
        ).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_not_found_pattern() {
        let pattern = BuiltinPatterns::command_not_found();

        assert!(pattern.matches("bash: foo: command not found"));
        assert!(pattern.matches("'bar' is not recognized as an internal or external command"));
        assert!(pattern.matches("zsh: command not found: baz"));
    }

    #[test]
    fn test_permission_denied_pattern() {
        let pattern = BuiltinPatterns::permission_denied();

        assert!(pattern.matches("Permission denied"));
        assert!(pattern.matches("rm: cannot remove 'file.txt': Permission denied"));
        assert!(pattern.matches("Operation not permitted"));
    }

    #[test]
    fn test_file_not_found_pattern() {
        let pattern = BuiltinPatterns::file_not_found();

        assert!(pattern.matches("No such file: 'config.yaml'"));
        assert!(pattern.matches("cat: test.txt: No such file or directory"));
    }

    #[test]
    fn test_python_module_pattern() {
        let pattern = BuiltinPatterns::python_module_not_found();

        assert!(pattern.matches("ModuleNotFoundError: No module named 'numpy'"));
        assert!(pattern.matches("ImportError: No module named 'pandas'"));
    }

    #[test]
    fn test_port_in_use_pattern() {
        let pattern = BuiltinPatterns::port_in_use();

        assert!(pattern.matches("Error: Port 8080 is already in use"));
        assert!(pattern.matches("Address already in use: bind: :3000"));
    }

    #[test]
    fn test_extract_details() {
        let pattern = BuiltinPatterns::file_not_found();
        let details = pattern.extract_details("No such file: 'config.yaml'");

        assert!(details.is_some());
        let details = details.unwrap();
        assert!(details.message.contains("config.yaml"));
    }

    #[test]
    fn test_builtin_patterns_count() {
        let patterns = BuiltinPatterns::all();
        assert!(patterns.len() >= 12);
    }
}
