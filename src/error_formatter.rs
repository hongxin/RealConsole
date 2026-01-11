//! v1.84.0: Enhanced Error Message Formatter
//!
//! 提供用户友好的错误消息格式化，包括：
//! - 原因分析
//! - 可操作的修复建议
//! - 文档链接
//!
//! 将技术性错误转换为用户可理解的诊断信息

use crate::error::{ErrorCode, FixSuggestion, RealError};
use crate::llm::LlmError;

/// 错误格式化器
///
/// 将各种错误类型转换为用户友好的 RealError
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// 格式化 LLM 错误为用户友好的 RealError
    ///
    /// # 参数
    /// - `error`: 原始 LLM 错误
    /// - `provider`: LLM 提供商名称 (如 "Ollama", "Deepseek", "OpenAI")
    /// - `endpoint`: API 端点 URL
    ///
    /// # 返回
    /// 包含原因分析和修复建议的 RealError
    pub fn format_llm_error(error: &LlmError, provider: &str, endpoint: &str) -> RealError {
        match error {
            LlmError::Network(msg) => Self::format_network_error(msg, provider, endpoint),
            LlmError::Http { status, message } => {
                Self::format_http_error(*status, message, provider, endpoint)
            }
            LlmError::RateLimit => Self::format_rate_limit_error(provider),
            LlmError::Timeout => Self::format_timeout_error(provider, endpoint),
            LlmError::Parse(msg) => Self::format_parse_error(msg, provider),
            LlmError::Config(msg) => Self::format_config_error(msg, provider),
            LlmError::Other(msg) => Self::format_other_error(msg, provider),
        }
    }

    /// 格式化网络错误
    fn format_network_error(message: &str, provider: &str, endpoint: &str) -> RealError {
        let (code, analysis, suggestions) = Self::analyze_network_error(message, provider, endpoint);

        RealError::new(code, analysis).with_suggestions(suggestions)
    }

    /// 分析网络错误并提供诊断
    fn analyze_network_error(
        message: &str,
        provider: &str,
        endpoint: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let msg_lower = message.to_lowercase();

        // 连接拒绝
        if msg_lower.contains("connection refused")
            || msg_lower.contains("connect error")
            || msg_lower.contains("couldn't connect")
        {
            return Self::connection_refused_analysis(provider, endpoint);
        }

        // DNS 解析失败
        if msg_lower.contains("dns")
            || msg_lower.contains("resolve")
            || msg_lower.contains("getaddrinfo")
            || msg_lower.contains("no such host")
        {
            return Self::dns_error_analysis(provider, endpoint);
        }

        // SSL/TLS 错误
        if msg_lower.contains("ssl")
            || msg_lower.contains("tls")
            || msg_lower.contains("certificate")
            || msg_lower.contains("handshake")
        {
            return Self::ssl_error_analysis(provider, endpoint);
        }

        // 连接重置
        if msg_lower.contains("reset")
            || msg_lower.contains("broken pipe")
            || msg_lower.contains("connection abort")
        {
            return Self::connection_reset_analysis(provider, endpoint);
        }

        // 通用网络错误
        Self::generic_network_analysis(message, provider, endpoint)
    }

    /// 连接拒绝分析
    fn connection_refused_analysis(
        provider: &str,
        endpoint: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "无法连接到 {} 服务 ({})\n\n可能原因:\n  1. {} 服务未启动\n  2. 端口被防火墙阻止\n  3. 配置的端点地址错误",
            provider, endpoint, provider
        );

        let mut suggestions = vec![];

        // 根据提供商提供特定建议
        match provider.to_lowercase().as_str() {
            "ollama" => {
                suggestions.push(
                    FixSuggestion::new("启动 Ollama 服务")
                        .with_command("ollama serve")
                        .with_doc("https://ollama.ai/download"),
                );
                suggestions.push(
                    FixSuggestion::new("检查 Ollama 是否已安装")
                        .with_command("which ollama || ollama --version"),
                );
                suggestions.push(
                    FixSuggestion::new("验证端点配置")
                        .with_command("curl http://localhost:11434/api/tags"),
                );
            }
            "deepseek" | "openai" | "gemini" => {
                suggestions.push(FixSuggestion::new("检查网络连接").with_command("ping -c 3 api.deepseek.com || ping -c 3 api.openai.com"));
                suggestions.push(FixSuggestion::new("检查是否需要代理设置"));
                suggestions.push(
                    FixSuggestion::new("验证端点 URL 是否正确")
                        .with_command("realconsole wizard"),
                );
            }
            _ => {
                suggestions.push(
                    FixSuggestion::new("检查服务是否运行")
                        .with_command(format!("curl -v {}", endpoint)),
                );
            }
        }

        (ErrorCode::LlmConnectionError, analysis, suggestions)
    }

    /// DNS 解析错误分析
    fn dns_error_analysis(
        provider: &str,
        endpoint: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "DNS 解析失败: 无法解析 {} 的地址 ({})\n\n可能原因:\n  1. 网络连接断开\n  2. DNS 服务器不可用\n  3. 域名拼写错误",
            provider, endpoint
        );

        let suggestions = vec![
            FixSuggestion::new("检查网络连接").with_command("ping -c 3 8.8.8.8"),
            FixSuggestion::new("检查 DNS 配置").with_command("nslookup api.deepseek.com"),
            FixSuggestion::new("尝试使用 IP 地址（如果是本地服务）")
                .with_command("realconsole wizard"),
        ];

        (ErrorCode::DnsError, analysis, suggestions)
    }

    /// SSL/TLS 错误分析
    fn ssl_error_analysis(
        provider: &str,
        endpoint: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "SSL/TLS 连接失败: {} ({})\n\n可能原因:\n  1. 证书验证失败\n  2. TLS 版本不兼容\n  3. 系统时间不正确",
            provider, endpoint
        );

        let suggestions = vec![
            FixSuggestion::new("检查系统时间是否正确").with_command("date"),
            FixSuggestion::new("更新系统 CA 证书"),
            FixSuggestion::new("检查端点是否使用正确的协议 (http vs https)"),
        ];

        (ErrorCode::SslError, analysis, suggestions)
    }

    /// 连接重置分析
    fn connection_reset_analysis(
        provider: &str,
        endpoint: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "连接被重置: {} ({})\n\n可能原因:\n  1. 服务器主动断开连接\n  2. 网络不稳定\n  3. 请求被中间代理拦截",
            provider, endpoint
        );

        let suggestions = vec![
            FixSuggestion::new("稍后重试"),
            FixSuggestion::new("检查是否有代理或 VPN 干扰"),
            FixSuggestion::new("查看服务状态页面"),
        ];

        (ErrorCode::NetworkError, analysis, suggestions)
    }

    /// 通用网络错误分析
    fn generic_network_analysis(
        message: &str,
        provider: &str,
        endpoint: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "网络错误: {} ({})\n\n详情: {}",
            provider, endpoint, message
        );

        let suggestions = vec![
            FixSuggestion::new("检查网络连接"),
            FixSuggestion::new("稍后重试"),
            FixSuggestion::new("运行诊断命令").with_command("realconsole /diag"),
        ];

        (ErrorCode::NetworkError, analysis, suggestions)
    }

    /// 格式化 HTTP 错误
    fn format_http_error(
        status: u16,
        message: &str,
        provider: &str,
        endpoint: &str,
    ) -> RealError {
        let (code, analysis, suggestions) = match status {
            401 => Self::auth_error_analysis(provider),
            403 => Self::forbidden_error_analysis(provider),
            404 => Self::not_found_error_analysis(provider, endpoint),
            429 => return Self::format_rate_limit_error(provider),
            500..=599 => Self::server_error_analysis(status, provider),
            _ => Self::generic_http_error_analysis(status, message, provider),
        };

        RealError::new(code, analysis).with_suggestions(suggestions)
    }

    /// 认证错误分析 (401)
    fn auth_error_analysis(provider: &str) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "{} API 认证失败\n\n可能原因:\n  1. API Key 无效或已过期\n  2. API Key 格式错误\n  3. API Key 权限不足",
            provider
        );

        let suggestions = vec![
            FixSuggestion::new("检查 API Key 是否正确").with_command("cat .env | grep API_KEY"),
            FixSuggestion::new("重新生成 API Key"),
            FixSuggestion::new("使用配置向导重新配置").with_command("realconsole wizard"),
        ];

        (ErrorCode::LlmAuthError, analysis, suggestions)
    }

    /// 禁止访问分析 (403)
    fn forbidden_error_analysis(provider: &str) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "{} 拒绝访问\n\n可能原因:\n  1. 账户被禁用或欠费\n  2. IP 被限制\n  3. 访问了未授权的资源",
            provider
        );

        let suggestions = vec![
            FixSuggestion::new("检查账户状态和余额"),
            FixSuggestion::new("检查 API 使用限制"),
            FixSuggestion::new("联系服务提供商"),
        ];

        (ErrorCode::LlmAuthError, analysis, suggestions)
    }

    /// 资源不存在分析 (404)
    fn not_found_error_analysis(
        provider: &str,
        endpoint: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "{} 请求的资源不存在\n\n可能原因:\n  1. API 端点 URL 错误\n  2. 模型名称不正确\n  3. API 版本已更新",
            provider
        );

        let suggestions = vec![
            FixSuggestion::new("验证端点 URL").with_command(format!("echo 'Current: {}'", endpoint)),
            FixSuggestion::new("检查模型名称是否正确"),
            FixSuggestion::new("查看 API 文档确认正确的端点"),
        ];

        (ErrorCode::LlmModelNotFound, analysis, suggestions)
    }

    /// 服务器错误分析 (5xx)
    fn server_error_analysis(status: u16, provider: &str) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "{} 服务器错误 (HTTP {})\n\n这是服务端问题，通常是临时性的。",
            provider, status
        );

        let suggestions = vec![
            FixSuggestion::new("等待几分钟后重试"),
            FixSuggestion::new("查看服务状态页面"),
            FixSuggestion::new("如果持续出现，联系服务提供商"),
        ];

        (ErrorCode::HttpError, analysis, suggestions)
    }

    /// 通用 HTTP 错误分析
    fn generic_http_error_analysis(
        status: u16,
        message: &str,
        provider: &str,
    ) -> (ErrorCode, String, Vec<FixSuggestion>) {
        let analysis = format!(
            "{} 返回 HTTP 错误 {}\n\n详情: {}",
            provider, status, message
        );

        let suggestions = vec![
            FixSuggestion::new("检查请求参数"),
            FixSuggestion::new("查看 API 文档"),
        ];

        (ErrorCode::HttpError, analysis, suggestions)
    }

    /// 格式化速率限制错误
    fn format_rate_limit_error(provider: &str) -> RealError {
        let analysis = format!(
            "{} API 调用频率超限\n\n您的请求过于频繁，已被暂时限制。",
            provider
        );

        let suggestions = vec![
            FixSuggestion::new("等待 1-2 分钟后重试"),
            FixSuggestion::new("检查账户的 API 配额"),
            FixSuggestion::new("考虑升级 API 套餐以获得更高限额"),
        ];

        RealError::new(ErrorCode::LlmRateLimitError, analysis).with_suggestions(suggestions)
    }

    /// 格式化超时错误
    fn format_timeout_error(provider: &str, endpoint: &str) -> RealError {
        let analysis = format!(
            "{} 请求超时 ({})\n\n可能原因:\n  1. 网络延迟过高\n  2. 服务器响应缓慢\n  3. 请求内容过长",
            provider, endpoint
        );

        let suggestions = vec![
            FixSuggestion::new("稍后重试"),
            FixSuggestion::new("检查网络连接质量").with_command("ping -c 5 api.deepseek.com"),
            FixSuggestion::new("尝试减少输入内容长度"),
        ];

        RealError::new(ErrorCode::LlmTimeoutError, analysis).with_suggestions(suggestions)
    }

    /// 格式化解析错误
    fn format_parse_error(message: &str, provider: &str) -> RealError {
        let analysis = format!(
            "{} 响应格式错误\n\n服务返回的数据无法解析。\n详情: {}",
            provider, message
        );

        let suggestions = vec![
            FixSuggestion::new("这通常是临时问题，请重试"),
            FixSuggestion::new("如果持续出现，可能是 API 版本不兼容"),
            FixSuggestion::new("检查是否使用最新版本的 RealConsole"),
        ];

        RealError::new(ErrorCode::LlmResponseError, analysis).with_suggestions(suggestions)
    }

    /// 格式化配置错误
    fn format_config_error(message: &str, provider: &str) -> RealError {
        let analysis = format!("{} 配置错误: {}", provider, message);

        let suggestions = vec![
            FixSuggestion::new("运行配置向导重新配置").with_command("realconsole wizard"),
            FixSuggestion::new("检查配置文件").with_command("cat realconsole.yaml"),
            FixSuggestion::new("检查环境变量").with_command("env | grep -i api"),
        ];

        RealError::new(ErrorCode::LlmNotConfigured, analysis).with_suggestions(suggestions)
    }

    /// 格式化其他错误
    fn format_other_error(message: &str, provider: &str) -> RealError {
        let analysis = format!("{} 发生未知错误: {}", provider, message);

        let suggestions = vec![
            FixSuggestion::new("稍后重试"),
            FixSuggestion::new("运行诊断命令").with_command("realconsole /diag"),
        ];

        RealError::new(ErrorCode::LlmConnectionError, analysis).with_suggestions(suggestions)
    }

    /// 从 reqwest 错误提取诊断信息
    ///
    /// # 参数
    /// - `error`: reqwest 错误引用
    ///
    /// # 返回
    /// 诊断信息字符串
    pub fn diagnose_reqwest_error(error: &reqwest::Error) -> String {
        let mut diagnosis: Vec<String> = Vec::new();

        if error.is_timeout() {
            diagnosis.push("请求超时".to_string());
        }

        if error.is_connect() {
            diagnosis.push("连接失败".to_string());
        }

        if error.is_request() {
            diagnosis.push("请求构建错误".to_string());
        }

        if error.is_body() {
            diagnosis.push("响应体读取错误".to_string());
        }

        if error.is_decode() {
            diagnosis.push("响应解码错误".to_string());
        }

        if let Some(status) = error.status() {
            diagnosis.push(format!("HTTP 状态码: {}", status.as_u16()));
        }

        if let Some(url) = error.url() {
            diagnosis.push(format!("请求 URL: {}", url));
        }

        if diagnosis.is_empty() {
            format!("未知错误: {}", error)
        } else {
            diagnosis.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_connection_refused() {
        let error = LlmError::Network("connection refused".to_string());
        let real_error = ErrorFormatter::format_llm_error(&error, "Ollama", "http://localhost:11434");

        assert_eq!(real_error.code, ErrorCode::LlmConnectionError);
        assert!(real_error.message.contains("Ollama"));
        assert!(real_error.message.contains("服务未启动"));
        assert!(!real_error.suggestions.is_empty());
        assert!(real_error.suggestions.iter().any(|s| s.command.as_ref().map_or(false, |c| c.contains("ollama serve"))));
    }

    #[test]
    fn test_format_dns_error() {
        let error = LlmError::Network("dns resolution failed".to_string());
        let real_error = ErrorFormatter::format_llm_error(&error, "Deepseek", "https://api.deepseek.com/v1");

        assert_eq!(real_error.code, ErrorCode::DnsError);
        assert!(real_error.message.contains("DNS"));
    }

    #[test]
    fn test_format_ssl_error() {
        let error = LlmError::Network("ssl certificate verify failed".to_string());
        let real_error = ErrorFormatter::format_llm_error(&error, "OpenAI", "https://api.openai.com/v1");

        assert_eq!(real_error.code, ErrorCode::SslError);
        assert!(real_error.message.contains("SSL/TLS"));
    }

    #[test]
    fn test_format_auth_error() {
        let error = LlmError::Http {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        let real_error = ErrorFormatter::format_llm_error(&error, "Deepseek", "https://api.deepseek.com/v1");

        assert_eq!(real_error.code, ErrorCode::LlmAuthError);
        assert!(real_error.message.contains("认证失败"));
        assert!(real_error.suggestions.iter().any(|s| s.description.contains("API Key")));
    }

    #[test]
    fn test_format_rate_limit() {
        let error = LlmError::RateLimit;
        let real_error = ErrorFormatter::format_llm_error(&error, "OpenAI", "https://api.openai.com/v1");

        assert_eq!(real_error.code, ErrorCode::LlmRateLimitError);
        assert!(real_error.message.contains("频率超限"));
    }

    #[test]
    fn test_format_timeout() {
        let error = LlmError::Timeout;
        let real_error = ErrorFormatter::format_llm_error(&error, "Gemini", "https://generativelanguage.googleapis.com");

        assert_eq!(real_error.code, ErrorCode::LlmTimeoutError);
        assert!(real_error.message.contains("超时"));
    }

    #[test]
    fn test_format_server_error() {
        let error = LlmError::Http {
            status: 503,
            message: "Service Unavailable".to_string(),
        };
        let real_error = ErrorFormatter::format_llm_error(&error, "Deepseek", "https://api.deepseek.com/v1");

        assert_eq!(real_error.code, ErrorCode::HttpError);
        assert!(real_error.message.contains("503"));
        assert!(real_error.message.contains("服务器错误"));
    }

    #[test]
    fn test_format_not_found() {
        let error = LlmError::Http {
            status: 404,
            message: "Not Found".to_string(),
        };
        let real_error = ErrorFormatter::format_llm_error(&error, "OpenAI", "https://api.openai.com/v1");

        assert_eq!(real_error.code, ErrorCode::LlmModelNotFound);
        assert!(real_error.message.contains("资源不存在"));
    }

    #[test]
    fn test_format_parse_error() {
        let error = LlmError::Parse("invalid json".to_string());
        let real_error = ErrorFormatter::format_llm_error(&error, "Ollama", "http://localhost:11434");

        assert_eq!(real_error.code, ErrorCode::LlmResponseError);
        assert!(real_error.message.contains("响应格式错误"));
    }

    #[test]
    fn test_format_config_error() {
        let error = LlmError::Config("API key is empty".to_string());
        let real_error = ErrorFormatter::format_llm_error(&error, "Deepseek", "https://api.deepseek.com/v1");

        assert_eq!(real_error.code, ErrorCode::LlmNotConfigured);
        assert!(real_error.suggestions.iter().any(|s| s.command.as_ref().map_or(false, |c| c.contains("wizard"))));
    }

    #[test]
    fn test_ollama_specific_suggestions() {
        let error = LlmError::Network("connection refused".to_string());
        let real_error = ErrorFormatter::format_llm_error(&error, "Ollama", "http://localhost:11434");

        // Ollama 特定建议
        let has_ollama_serve = real_error
            .suggestions
            .iter()
            .any(|s| s.command.as_ref().map_or(false, |c| c.contains("ollama serve")));
        assert!(has_ollama_serve, "Should have 'ollama serve' suggestion for Ollama");
    }

    #[test]
    fn test_user_friendly_output() {
        let error = LlmError::Network("connection refused".to_string());
        let real_error = ErrorFormatter::format_llm_error(&error, "Ollama", "http://localhost:11434");
        let formatted = real_error.format_user_friendly();

        // 应该包含彩色输出的关键元素
        assert!(formatted.contains("E101")); // Error code
        assert!(formatted.contains("建议修复方案")); // Suggestions header
        assert!(formatted.contains("ollama serve")); // Command suggestion
    }
}
