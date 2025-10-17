//! 错误自动修复器
//!
//! 基于错误分析生成并应用修复方案

use super::analyzer::{ErrorAnalysis, ErrorCategory};
use crate::llm::{LlmClient, LlmError, Message};
use serde::{Deserialize, Serialize};

/// 修复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixStrategy {
    /// 策略名称
    pub name: String,

    /// 修复命令
    pub command: String,

    /// 策略描述
    pub description: String,

    /// 是否需要用户确认
    pub requires_confirmation: bool,

    /// 风险等级 (1-10)
    pub risk_level: u8,

    /// 预期效果
    pub expected_outcome: String,
}

impl FixStrategy {
    /// 创建新的修复策略
    pub fn new(
        name: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
        risk_level: u8,
    ) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: description.into(),
            requires_confirmation: risk_level >= 5,
            risk_level,
            expected_outcome: String::new(),
        }
    }

    /// 设置预期效果
    pub fn with_outcome(mut self, outcome: impl Into<String>) -> Self {
        self.expected_outcome = outcome.into();
        self
    }

    /// 判断是否为高风险操作
    pub fn is_high_risk(&self) -> bool {
        self.risk_level >= 7
    }
}

/// 修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    /// 是否成功
    pub success: bool,

    /// 应用的策略
    pub strategy: Option<FixStrategy>,

    /// 执行输出
    pub output: String,

    /// 错误信息（如果失败）
    pub error: Option<String>,
}

impl FixResult {
    /// 创建成功结果
    pub fn success(strategy: FixStrategy, output: String) -> Self {
        Self {
            success: true,
            strategy: Some(strategy),
            output,
            error: None,
        }
    }

    /// 创建失败结果
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            strategy: None,
            output: String::new(),
            error: Some(error),
        }
    }
}

/// 错误修复器
pub struct ErrorFixer;

impl ErrorFixer {
    /// 生成修复策略
    ///
    /// # 参数
    /// - `analysis`: 错误分析结果
    ///
    /// # 返回
    /// 修复策略列表（按优先级排序）
    pub fn generate_strategies(analysis: &ErrorAnalysis) -> Vec<FixStrategy> {
        let mut strategies = Vec::new();

        match &analysis.category {
            ErrorCategory::Command => {
                strategies.extend(Self::fix_command_not_found(analysis));
            }
            ErrorCategory::Permission => {
                strategies.extend(Self::fix_permission_denied(analysis));
            }
            ErrorCategory::Directory => {
                strategies.extend(Self::fix_directory_not_found(analysis));
            }
            ErrorCategory::Language(lang) if lang == "Python" => {
                strategies.extend(Self::fix_python_module_missing(analysis));
            }
            ErrorCategory::Language(lang) if lang == "Node.js" => {
                strategies.extend(Self::fix_npm_module_missing(analysis));
            }
            ErrorCategory::Network => {
                strategies.extend(Self::fix_port_in_use(analysis));
            }
            _ => {
                // 通用策略：重试
                strategies.push(
                    FixStrategy::new(
                        "重试命令",
                        &analysis.command,
                        "简单重试原命令",
                        2,
                    )
                    .with_outcome("可能解决临时性问题"),
                );
            }
        }

        strategies
    }

    /// 使用 LLM 生成修复策略
    ///
    /// # 参数
    /// - `analysis`: 错误分析结果
    /// - `llm`: LLM 客户端
    ///
    /// # 返回
    /// LLM 生成的修复策略
    pub async fn generate_strategies_with_llm(
        analysis: &ErrorAnalysis,
        llm: &dyn LlmClient,
    ) -> Result<Vec<FixStrategy>, LlmError> {
        let prompt = format!(
            r#"为以下命令错误生成修复策略。

命令: {}
错误: {}
类别: {}

请返回 JSON 数组，包含多个修复策略：
[
  {{
    "name": "策略名称",
    "command": "修复命令",
    "description": "详细描述",
    "risk_level": 3,
    "expected_outcome": "预期效果"
  }}
]

策略应该：
1. 从低风险到高风险排序
2. 包含具体可执行的命令
3. 说明预期效果和风险

只返回 JSON，不要其他解释。"#,
            analysis.command,
            analysis.raw_error,
            analysis.category
        );

        let messages = vec![Message::user(prompt)];
        let response = llm.chat(messages).await?;

        // 解析响应
        Self::parse_strategies_response(&response)
    }

    /// 解析策略响应
    fn parse_strategies_response(response: &str) -> Result<Vec<FixStrategy>, LlmError> {
        // 提取 JSON
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        let strategies_json: Vec<serde_json::Value> = serde_json::from_str(json_str)
            .map_err(|e| LlmError::Parse(format!("JSON 解析失败: {}", e)))?;

        let mut strategies = Vec::new();

        for item in strategies_json {
            let name = item["name"].as_str().unwrap_or("未命名策略").to_string();
            let command = item["command"].as_str().unwrap_or("").to_string();
            let description = item["description"].as_str().unwrap_or("").to_string();
            let risk_level = item["risk_level"].as_u64().unwrap_or(5) as u8;
            let outcome = item["expected_outcome"].as_str().unwrap_or("").to_string();

            strategies.push(
                FixStrategy::new(name, command, description, risk_level).with_outcome(outcome),
            );
        }

        Ok(strategies)
    }

    // ========== 特定错误类型的修复策略 ==========

    fn fix_command_not_found(analysis: &ErrorAnalysis) -> Vec<FixStrategy> {
        let mut strategies = Vec::new();

        // 提取命令名称
        if let Some(ref details) = analysis.details {
            if let Some(ref cmd) = details.command {
                // 策略 1: 检查常见拼写错误
                strategies.push(
                    FixStrategy::new(
                        "检查拼写",
                        format!("which {} || type {}", cmd, cmd),
                        "检查命令是否存在",
                        1,
                    )
                    .with_outcome("找到正确的命令路径"),
                );

                // 策略 2: 安装常见工具
                if Self::is_common_tool(cmd) {
                    let install_cmd = Self::get_install_command(cmd);
                    strategies.push(
                        FixStrategy::new(
                            "安装工具",
                            install_cmd,
                            format!("使用包管理器安装 {}", cmd),
                            4,
                        )
                        .with_outcome("命令可用"),
                    );
                }
            }
        }

        strategies
    }

    fn fix_permission_denied(analysis: &ErrorAnalysis) -> Vec<FixStrategy> {
        let mut strategies = Vec::new();

        // 策略 1: 添加执行权限
        if analysis.command.starts_with("./") {
            strategies.push(
                FixStrategy::new(
                    "添加执行权限",
                    format!("chmod +x {}", analysis.command.trim_start_matches("!").trim()),
                    "为脚本添加可执行权限",
                    3,
                )
                .with_outcome("脚本可以执行"),
            );
        }

        // 策略 2: 使用 sudo（高风险）
        strategies.push(
            FixStrategy::new(
                "使用管理员权限",
                format!("sudo {}", analysis.command),
                "以管理员身份重新执行",
                8,
            )
            .with_outcome("获得所需权限"),
        );

        strategies
    }

    fn fix_directory_not_found(analysis: &ErrorAnalysis) -> Vec<FixStrategy> {
        let mut strategies = Vec::new();

        if let Some(ref details) = analysis.details {
            if let Some(ref path) = details.path {
                // 策略: 创建目录
                strategies.push(
                    FixStrategy::new(
                        "创建目录",
                        format!("mkdir -p {}", path),
                        format!("创建缺失的目录 {}", path),
                        3,
                    )
                    .with_outcome("目录创建成功"),
                );
            }
        }

        strategies
    }

    fn fix_python_module_missing(analysis: &ErrorAnalysis) -> Vec<FixStrategy> {
        let mut strategies = Vec::new();

        // 从错误消息提取模块名
        if let Some(module) = Self::extract_python_module(&analysis.raw_error) {
            // 策略: pip install
            strategies.push(
                FixStrategy::new(
                    "安装 Python 模块",
                    format!("pip install {}", module),
                    format!("使用 pip 安装 {} 模块", module),
                    4,
                )
                .with_outcome("模块安装成功"),
            );

            // 策略: 使用国内镜像
            strategies.push(
                FixStrategy::new(
                    "使用国内镜像安装",
                    format!("pip install -i https://pypi.tuna.tsinghua.edu.cn/simple {}", module),
                    "使用清华镜像加速安装",
                    4,
                )
                .with_outcome("模块快速安装"),
            );
        }

        strategies
    }

    fn fix_npm_module_missing(_analysis: &ErrorAnalysis) -> Vec<FixStrategy> {
        vec![
            FixStrategy::new(
                "安装依赖",
                "npm install",
                "安装项目所有依赖",
                3,
            )
            .with_outcome("所有依赖安装完成"),
        ]
    }

    fn fix_port_in_use(analysis: &ErrorAnalysis) -> Vec<FixStrategy> {
        let mut strategies = Vec::new();

        // 提取端口号
        if let Some(port) = Self::extract_port(&analysis.raw_error) {
            // 策略 1: 查找占用进程
            strategies.push(
                FixStrategy::new(
                    "查找占用进程",
                    format!("lsof -i :{} || netstat -tuln | grep {}", port, port),
                    format!("查找占用端口 {} 的进程", port),
                    2,
                )
                .with_outcome("找到占用进程"),
            );

            // 策略 2: 使用其他端口
            strategies.push(
                FixStrategy::new(
                    "使用其他端口",
                    Self::modify_port_in_command(&analysis.command, port, port + 1),
                    format!("改用端口 {}", port + 1),
                    3,
                )
                .with_outcome("使用新端口运行"),
            );
        }

        strategies
    }

    // ========== 辅助方法 ==========

    fn is_common_tool(cmd: &str) -> bool {
        matches!(
            cmd,
            "curl" | "wget" | "git" | "python" | "node" | "npm" | "cargo" | "rustc" | "make"
        )
    }

    fn get_install_command(cmd: &str) -> String {
        // 根据操作系统返回对应的安装命令
        #[cfg(target_os = "macos")]
        return format!("brew install {}", cmd);

        #[cfg(target_os = "linux")]
        {
            // 检测 Linux 发行版
            if std::path::Path::new("/etc/debian_version").exists() {
                format!("sudo apt-get install -y {}", cmd)
            } else if std::path::Path::new("/etc/redhat-release").exists() {
                format!("sudo yum install -y {}", cmd)
            } else {
                format!("# 请根据您的系统手动安装 {}", cmd)
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        format!("# 请根据您的系统手动安装 {}", cmd)
    }

    fn extract_python_module(error: &str) -> Option<String> {
        // 使用正则提取模块名
        let re = regex::Regex::new(r#"No module named ['"](\w+)['"]"#).ok()?;
        re.captures(error)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn extract_port(error: &str) -> Option<u16> {
        let re = regex::Regex::new(r":(\d+)").ok()?;
        re.captures(error)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    fn modify_port_in_command(command: &str, old_port: u16, new_port: u16) -> String {
        command.replace(&old_port.to_string(), &new_port.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_strategy_creation() {
        let strategy = FixStrategy::new("test", "echo hello", "测试策略", 3);

        assert_eq!(strategy.name, "test");
        assert_eq!(strategy.command, "echo hello");
        assert!(!strategy.requires_confirmation); // 风险 < 5
    }

    #[test]
    fn test_fix_strategy_high_risk() {
        let strategy = FixStrategy::new("risky", "rm -rf /", "危险操作", 9);

        assert!(strategy.is_high_risk());
        assert!(strategy.requires_confirmation);
    }

    #[test]
    fn test_fix_result_success() {
        let strategy = FixStrategy::new("test", "echo ok", "test", 1);
        let result = FixResult::success(strategy, "ok".to_string());

        assert!(result.success);
        assert!(result.strategy.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_fix_result_failure() {
        let result = FixResult::failure("错误信息".to_string());

        assert!(!result.success);
        assert!(result.strategy.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_extract_python_module() {
        let error = r#"ModuleNotFoundError: No module named 'numpy'"#;
        let module = ErrorFixer::extract_python_module(error);

        assert_eq!(module, Some("numpy".to_string()));
    }

    #[test]
    fn test_extract_port() {
        let error = "Error: Address already in use :8080";
        let port = ErrorFixer::extract_port(error);

        assert_eq!(port, Some(8080));
    }

    #[test]
    fn test_is_common_tool() {
        assert!(ErrorFixer::is_common_tool("git"));
        assert!(ErrorFixer::is_common_tool("curl"));
        assert!(!ErrorFixer::is_common_tool("foobar"));
    }
}
