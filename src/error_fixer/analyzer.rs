//! 错误分析器
//!
//! 分析命令执行错误，识别错误类型和原因

use super::patterns::{BuiltinPatterns, ErrorDetails, ErrorPattern};
use crate::llm::{LlmClient, LlmError, Message};
use serde::{Deserialize, Serialize};

/// 错误类别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorCategory {
    /// 命令相关错误
    Command,
    /// 权限错误
    Permission,
    /// 文件系统错误
    File,
    /// 目录错误
    Directory,
    /// 语法错误
    Syntax,
    /// 网络错误
    Network,
    /// 磁盘错误
    Disk,
    /// 语言特定错误（Python、Node.js、Rust等）
    Language(String),
    /// Git 相关错误
    Git,
    /// 未知错误
    Unknown,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Command => write!(f, "命令错误"),
            ErrorCategory::Permission => write!(f, "权限错误"),
            ErrorCategory::File => write!(f, "文件错误"),
            ErrorCategory::Directory => write!(f, "目录错误"),
            ErrorCategory::Syntax => write!(f, "语法错误"),
            ErrorCategory::Network => write!(f, "网络错误"),
            ErrorCategory::Disk => write!(f, "磁盘错误"),
            ErrorCategory::Language(lang) => write!(f, "{} 错误", lang),
            ErrorCategory::Git => write!(f, "Git 错误"),
            ErrorCategory::Unknown => write!(f, "未知错误"),
        }
    }
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    /// 低 (1-3)
    Low,
    /// 中 (4-6)
    Medium,
    /// 高 (7-9)
    High,
    /// 严重 (10)
    Critical,
}

impl ErrorSeverity {
    /// 从数字转换
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=3 => Self::Low,
            4..=6 => Self::Medium,
            7..=9 => Self::High,
            _ => Self::Critical,
        }
    }
}

/// 错误分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    /// 原始错误输出
    pub raw_error: String,

    /// 原始命令
    pub command: String,

    /// 错误类别
    pub category: ErrorCategory,

    /// 严重程度
    pub severity: ErrorSeverity,

    /// 匹配的模式名称
    pub pattern_name: Option<String>,

    /// 错误详情
    pub details: Option<ErrorDetails>,

    /// 可能的原因
    pub possible_causes: Vec<String>,

    /// 建议的修复方案
    pub suggested_fixes: Vec<String>,

    /// 是否可以自动修复
    pub auto_fixable: bool,

    /// LLM 增强分析（可选）
    pub llm_analysis: Option<String>,
}

impl ErrorAnalysis {
    /// 创建新的分析结果
    pub fn new(raw_error: String, command: String) -> Self {
        Self {
            raw_error,
            command,
            category: ErrorCategory::Unknown,
            severity: ErrorSeverity::Medium,
            pattern_name: None,
            details: None,
            possible_causes: Vec::new(),
            suggested_fixes: Vec::new(),
            auto_fixable: false,
            llm_analysis: None,
        }
    }

    /// 判断是否为严重错误
    pub fn is_severe(&self) -> bool {
        self.severity >= ErrorSeverity::High
    }
}

/// 错误分析器
pub struct ErrorAnalyzer {
    /// 错误模式库
    patterns: Vec<ErrorPattern>,
}

impl ErrorAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            patterns: BuiltinPatterns::all(),
        }
    }

    /// 分析错误
    ///
    /// # 参数
    /// - `command`: 执行的命令
    /// - `error_output`: 错误输出
    ///
    /// # 返回
    /// 错误分析结果
    pub fn analyze(&self, command: &str, error_output: &str) -> ErrorAnalysis {
        let mut analysis = ErrorAnalysis::new(error_output.to_string(), command.to_string());

        // 尝试匹配所有模式
        for pattern in &self.patterns {
            if pattern.matches(error_output) {
                // 更新分析结果
                analysis.pattern_name = Some(pattern.name.clone());
                analysis.category = self.map_category(&pattern.category);
                analysis.severity = ErrorSeverity::from_score(pattern.severity);
                analysis.suggested_fixes.push(pattern.suggested_fix.clone());
                analysis.auto_fixable = pattern.auto_fixable;

                // 提取详情
                if let Some(details) = pattern.extract_details(error_output) {
                    analysis.details = Some(details);
                }

                // 找到第一个匹配就停止（按优先级）
                break;
            }
        }

        // 推断可能的原因
        analysis.possible_causes = self.infer_causes(&analysis);

        analysis
    }

    /// 使用 LLM 增强分析
    ///
    /// # 参数
    /// - `analysis`: 基础分析结果
    /// - `llm`: LLM 客户端
    ///
    /// # 返回
    /// 增强后的分析结果
    pub async fn analyze_with_llm(
        &self,
        mut analysis: ErrorAnalysis,
        llm: &dyn LlmClient,
    ) -> Result<ErrorAnalysis, LlmError> {
        let prompt = format!(
            r#"分析以下命令执行错误，提供详细的诊断和修复建议。

命令: {}
错误输出: {}

请返回 JSON 格式：
{{
  "root_cause": "错误的根本原因",
  "impact": "错误的影响范围",
  "fix_steps": ["步骤1", "步骤2", "步骤3"],
  "alternative_commands": ["替代命令1", "替代命令2"],
  "prevention": "如何预防此类错误"
}}

只返回 JSON，不要其他解释。"#,
            analysis.command, analysis.raw_error
        );

        let messages = vec![Message::user(prompt)];
        let response = llm.chat(messages).await?;

        // 解析 LLM 响应
        if let Ok(llm_data) = self.parse_llm_response(&response) {
            analysis.llm_analysis = Some(serde_json::to_string_pretty(&llm_data).unwrap());

            // 合并 LLM 的修复建议
            if let Some(fix_steps) = llm_data.get("fix_steps").and_then(|v| v.as_array()) {
                for step in fix_steps {
                    if let Some(step_str) = step.as_str() {
                        analysis.suggested_fixes.push(step_str.to_string());
                    }
                }
            }
        }

        Ok(analysis)
    }

    /// 解析 LLM 响应
    fn parse_llm_response(&self, response: &str) -> Result<serde_json::Value, serde_json::Error> {
        // 提取 JSON
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        serde_json::from_str(json_str)
    }

    /// 映射错误类别
    fn map_category(&self, category_str: &str) -> ErrorCategory {
        match category_str {
            "command" => ErrorCategory::Command,
            "permission" => ErrorCategory::Permission,
            "file" => ErrorCategory::File,
            "directory" => ErrorCategory::Directory,
            "syntax" => ErrorCategory::Syntax,
            "network" => ErrorCategory::Network,
            "disk" => ErrorCategory::Disk,
            "python" => ErrorCategory::Language("Python".to_string()),
            "nodejs" => ErrorCategory::Language("Node.js".to_string()),
            "rust" => ErrorCategory::Language("Rust".to_string()),
            "git" => ErrorCategory::Git,
            _ => ErrorCategory::Unknown,
        }
    }

    /// 推断可能的原因
    fn infer_causes(&self, analysis: &ErrorAnalysis) -> Vec<String> {
        let mut causes = Vec::new();

        match analysis.category {
            ErrorCategory::Command => {
                causes.push("命令拼写错误".to_string());
                causes.push("命令未安装".to_string());
                causes.push("PATH 环境变量未配置".to_string());
            }
            ErrorCategory::Permission => {
                causes.push("文件或目录权限不足".to_string());
                causes.push("需要管理员权限".to_string());
            }
            ErrorCategory::File => {
                causes.push("文件路径错误".to_string());
                causes.push("文件已被删除或移动".to_string());
            }
            ErrorCategory::Directory => {
                causes.push("目录不存在".to_string());
                causes.push("路径拼写错误".to_string());
            }
            ErrorCategory::Network => {
                causes.push("网络连接失败".to_string());
                causes.push("服务未启动或端口被占用".to_string());
            }
            ErrorCategory::Language(ref lang) => {
                causes.push(format!("{} 依赖未安装", lang));
                causes.push(format!("{} 版本不兼容", lang));
            }
            _ => {
                causes.push("需要进一步调查".to_string());
            }
        }

        causes
    }
}

impl Default for ErrorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::Command.to_string(), "命令错误");
        assert_eq!(ErrorCategory::Permission.to_string(), "权限错误");
        assert_eq!(
            ErrorCategory::Language("Python".to_string()).to_string(),
            "Python 错误"
        );
    }

    #[test]
    fn test_error_severity_from_score() {
        assert_eq!(ErrorSeverity::from_score(2), ErrorSeverity::Low);
        assert_eq!(ErrorSeverity::from_score(5), ErrorSeverity::Medium);
        assert_eq!(ErrorSeverity::from_score(8), ErrorSeverity::High);
        assert_eq!(ErrorSeverity::from_score(10), ErrorSeverity::Critical);
    }

    #[test]
    fn test_analyzer_command_not_found() {
        let analyzer = ErrorAnalyzer::new();
        let analysis = analyzer.analyze("foo", "bash: foo: command not found");

        assert_eq!(analysis.category, ErrorCategory::Command);
        assert!(analysis.pattern_name.is_some());
        assert!(!analysis.suggested_fixes.is_empty());
    }

    #[test]
    fn test_analyzer_permission_denied() {
        let analyzer = ErrorAnalyzer::new();
        let analysis = analyzer.analyze("rm file.txt", "rm: cannot remove 'file.txt': Permission denied");

        assert_eq!(analysis.category, ErrorCategory::Permission);
        assert!(analysis.severity >= ErrorSeverity::High);
    }

    #[test]
    fn test_analyzer_python_module() {
        let analyzer = ErrorAnalyzer::new();
        let analysis = analyzer.analyze(
            "python script.py",
            "ModuleNotFoundError: No module named 'numpy'",
        );

        assert_eq!(
            analysis.category,
            ErrorCategory::Language("Python".to_string())
        );
        assert!(analysis.auto_fixable);
    }

    #[test]
    fn test_analysis_is_severe() {
        let mut analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());

        analysis.severity = ErrorSeverity::Low;
        assert!(!analysis.is_severe());

        analysis.severity = ErrorSeverity::High;
        assert!(analysis.is_severe());
    }

    #[test]
    fn test_infer_causes_command_error() {
        let analyzer = ErrorAnalyzer::new();
        let mut analysis = ErrorAnalysis::new("error".to_string(), "foo".to_string());
        analysis.category = ErrorCategory::Command;

        let causes = analyzer.infer_causes(&analysis);
        assert!(!causes.is_empty());
        assert!(causes.iter().any(|c| c.contains("拼写")));
    }
}
