//! RealConsole 统一错误系统
//!
//! 提供：
//! - 统一的错误代码系统
//! - 友好的错误消息
//! - 建议性修复方案
//! - 国际化准备

use colored::Colorize;
use std::fmt;
use thiserror::Error;

/// RealConsole 错误代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 配置错误 (E001-E099)
    ConfigNotFound,
    ConfigParseError,
    ConfigValidationError,
    EnvFileNotFound,
    EnvVarMissing,
    ApiKeyInvalid,
    ApiKeyEmpty,

    // LLM 错误 (E100-E199)
    LlmNotConfigured,
    LlmConnectionError,
    LlmAuthError,
    LlmRateLimitError,
    LlmTimeoutError,
    LlmResponseError,
    LlmModelNotFound,

    // 工具错误 (E200-E299)
    ToolNotFound,
    ToolParameterError,
    ToolExecutionError,
    ToolTimeoutError,
    ToolPermissionDenied,

    // Shell 错误 (E300-E399)
    ShellDisabled,
    ShellCommandEmpty,
    ShellDangerousCommand,
    ShellTimeoutError,
    ShellExecutionError,

    // 网络错误 (E600-E699)
    NetworkError,
    HttpError,
    DnsError,
    SslError,

    // 文件系统错误 (E700-E799)
    FileNotFound,
    FileReadError,
    FileWriteError,
    DirectoryNotFound,
}

impl ErrorCode {
    /// 获取错误代码编号
    pub fn code(&self) -> &'static str {
        match self {
            // 配置错误
            Self::ConfigNotFound => "E001",
            Self::ConfigParseError => "E002",
            Self::ConfigValidationError => "E003",
            Self::EnvFileNotFound => "E004",
            Self::EnvVarMissing => "E005",
            Self::ApiKeyInvalid => "E006",
            Self::ApiKeyEmpty => "E007",

            // LLM 错误
            Self::LlmNotConfigured => "E100",
            Self::LlmConnectionError => "E101",
            Self::LlmAuthError => "E102",
            Self::LlmRateLimitError => "E103",
            Self::LlmTimeoutError => "E104",
            Self::LlmResponseError => "E105",
            Self::LlmModelNotFound => "E106",

            // 工具错误
            Self::ToolNotFound => "E200",
            Self::ToolParameterError => "E201",
            Self::ToolExecutionError => "E202",
            Self::ToolTimeoutError => "E203",
            Self::ToolPermissionDenied => "E204",

            // Shell 错误
            Self::ShellDisabled => "E300",
            Self::ShellCommandEmpty => "E301",
            Self::ShellDangerousCommand => "E302",
            Self::ShellTimeoutError => "E303",
            Self::ShellExecutionError => "E304",

            // 网络错误
            Self::NetworkError => "E600",
            Self::HttpError => "E601",
            Self::DnsError => "E602",
            Self::SslError => "E603",

            // 文件系统错误
            Self::FileNotFound => "E700",
            Self::FileReadError => "E701",
            Self::FileWriteError => "E702",
            Self::DirectoryNotFound => "E703",
        }
    }

    /// 获取友好的错误名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConfigNotFound => "配置文件不存在",
            Self::ConfigParseError => "配置文件解析失败",
            Self::ConfigValidationError => "配置验证失败",
            Self::EnvFileNotFound => "环境变量文件不存在",
            Self::EnvVarMissing => "缺少必需的环境变量",
            Self::ApiKeyInvalid => "API Key 格式无效",
            Self::ApiKeyEmpty => "API Key 为空",

            Self::LlmNotConfigured => "LLM 未配置",
            Self::LlmConnectionError => "无法连接到 LLM 服务",
            Self::LlmAuthError => "LLM 认证失败",
            Self::LlmRateLimitError => "超过 API 调用限额",
            Self::LlmTimeoutError => "LLM 请求超时",
            Self::LlmResponseError => "LLM 响应格式错误",
            Self::LlmModelNotFound => "模型不存在",

            Self::ToolNotFound => "工具不存在",
            Self::ToolParameterError => "工具参数错误",
            Self::ToolExecutionError => "工具执行失败",
            Self::ToolTimeoutError => "工具执行超时",
            Self::ToolPermissionDenied => "工具权限不足",

            Self::ShellDisabled => "Shell 功能未启用",
            Self::ShellCommandEmpty => "Shell 命令为空",
            Self::ShellDangerousCommand => "检测到危险命令",
            Self::ShellTimeoutError => "Shell 命令超时",
            Self::ShellExecutionError => "Shell 命令执行失败",

            Self::NetworkError => "网络错误",
            Self::HttpError => "HTTP 请求失败",
            Self::DnsError => "DNS 解析失败",
            Self::SslError => "SSL/TLS 错误",

            Self::FileNotFound => "文件不存在",
            Self::FileReadError => "文件读取失败",
            Self::FileWriteError => "文件写入失败",
            Self::DirectoryNotFound => "目录不存在",
        }
    }
}

/// 修复建议
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    /// 建议描述
    pub description: String,
    /// 示例命令（可选）
    pub command: Option<String>,
    /// 文档链接（可选）
    pub doc_link: Option<String>,
}

impl FixSuggestion {
    /// 创建新的修复建议
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            command: None,
            doc_link: None,
        }
    }

    /// 添加示例命令
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// 添加文档链接
    pub fn with_doc(mut self, link: impl Into<String>) -> Self {
        self.doc_link = Some(link.into());
        self
    }
}

/// RealConsole 统一错误类型
#[derive(Error, Debug)]
pub struct RealError {
    /// 错误代码
    pub code: ErrorCode,
    /// 错误消息
    pub message: String,
    /// 修复建议
    pub suggestions: Vec<FixSuggestion>,
    /// 底层错误（可选）
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl RealError {
    /// 创建新的错误
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            suggestions: Vec::new(),
            source: None,
        }
    }

    /// 添加单个修复建议
    pub fn with_suggestion(mut self, suggestion: FixSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// 添加多个修复建议
    pub fn with_suggestions(mut self, suggestions: Vec<FixSuggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// 添加底层错误
    pub fn with_source(
        mut self,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 格式化为用户友好的错误消息
    pub fn format_user_friendly(&self) -> String {
        let mut output = String::new();

        // 错误标题
        output.push_str(&format!(
            "\n{} [{}] {}\n",
            "✗",
            self.code.code().yellow(),
            self.code.name().red().bold()
        ));

        // 错误详情
        output.push_str(&format!("\n{}\n", self.message));

        // 修复建议
        if !self.suggestions.is_empty() {
            output.push_str(&format!("\n{}\n", "建议修复方案:".green().bold()));
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                output.push_str(&format!("\n  {}. {}", i + 1, suggestion.description));

                if let Some(cmd) = &suggestion.command {
                    output.push_str(&format!("\n     {}: {}", "命令".dimmed(), cmd.cyan()));
                }

                if let Some(link) = &suggestion.doc_link {
                    output.push_str(&format!(
                        "\n     {}: {}",
                        "文档".dimmed(),
                        link.blue().underline()
                    ));
                }
            }
            output.push('\n');
        }

        // 底层错误
        if let Some(source) = &self.source {
            output.push_str(&format!(
                "\n{} {}\n",
                "详细信息:".dimmed(),
                source.to_string().dimmed()
            ));
        }

        output
    }
}

impl fmt::Display for RealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.code.code(),
            self.code.name(),
            self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        assert_eq!(ErrorCode::ConfigNotFound.code(), "E001");
        assert_eq!(ErrorCode::ConfigNotFound.name(), "配置文件不存在");

        assert_eq!(ErrorCode::LlmAuthError.code(), "E102");
        assert_eq!(ErrorCode::ShellDangerousCommand.code(), "E302");
    }

    #[test]
    fn test_fix_suggestion() {
        let suggestion = FixSuggestion::new("运行配置向导")
            .with_command("realconsole wizard")
            .with_doc("https://docs.realconsole.com");

        assert_eq!(suggestion.description, "运行配置向导");
        assert_eq!(suggestion.command, Some("realconsole wizard".to_string()));
        assert!(suggestion.doc_link.is_some());
    }

    #[test]
    fn test_real_error_creation() {
        let error = RealError::new(ErrorCode::ConfigNotFound, "配置文件缺失");

        assert_eq!(error.code.code(), "E001");
        assert_eq!(error.message, "配置文件缺失");
        assert!(error.suggestions.is_empty());
        assert!(error.source.is_none());
    }

    #[test]
    fn test_real_error_with_suggestions() {
        let error = RealError::new(ErrorCode::ConfigNotFound, "配置文件缺失")
            .with_suggestion(FixSuggestion::new("运行向导").with_command("realconsole wizard"))
            .with_suggestion(FixSuggestion::new("手动创建配置"));

        assert_eq!(error.suggestions.len(), 2);
        assert_eq!(error.suggestions[0].description, "运行向导");
        assert_eq!(error.suggestions[1].description, "手动创建配置");
    }

    #[test]
    fn test_error_display() {
        let error = RealError::new(ErrorCode::LlmAuthError, "API Key 无效");

        let display = error.to_string();
        assert!(display.contains("E102"));
        assert!(display.contains("LLM 认证失败"));
        assert!(display.contains("API Key 无效"));
    }

    #[test]
    fn test_format_user_friendly() {
        let error = RealError::new(
            ErrorCode::ConfigNotFound,
            "配置文件 'realconsole.yaml' 不存在",
        )
        .with_suggestion(
            FixSuggestion::new("运行配置向导创建配置")
                .with_command("realconsole wizard"),
        );

        let formatted = error.format_user_friendly();

        // 应包含错误代码
        assert!(formatted.contains("E001"));
        // 应包含错误名称
        assert!(formatted.contains("配置文件不存在"));
        // 应包含错误消息
        assert!(formatted.contains("realconsole.yaml"));
        // 应包含建议
        assert!(formatted.contains("建议修复方案"));
        assert!(formatted.contains("运行配置向导"));
        // 应包含命令
        assert!(formatted.contains("realconsole wizard"));
    }

    #[test]
    fn test_error_code_uniqueness() {
        // 确保所有错误代码唯一
        let codes = vec![
            ErrorCode::ConfigNotFound.code(),
            ErrorCode::ConfigParseError.code(),
            ErrorCode::LlmAuthError.code(),
            ErrorCode::ShellDangerousCommand.code(),
            ErrorCode::FileNotFound.code(),
        ];

        let unique_count = codes.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, codes.len());
    }
}
