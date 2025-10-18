//! 配置系统
//!
//! 支持：
//! - YAML 配置文件加载
//! - 环境变量扩展 ${VAR} 和 ${VAR:-default}
//! - 默认配置

use crate::display::DisplayMode;
use crate::error::{ErrorCode, FixSuggestion, RealError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

/// 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 命令前缀（默认 "/"）
    #[serde(default = "default_prefix")]
    pub prefix: String,

    /// LLM 配置（统一架构）
    #[serde(default)]
    pub llm: LlmConfig,

    /// 记忆系统配置
    #[serde(default)]
    pub memory: Option<MemoryConfig>,

    /// 功能开关
    #[serde(default)]
    pub features: FeaturesConfig,

    /// Intent DSL 配置
    #[serde(default)]
    pub intent: IntentConfig,

    /// 显示模式配置
    #[serde(default)]
    pub display: DisplayConfig,
}

fn default_prefix() -> String {
    "/".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmConfig {
    pub primary: Option<LlmProvider>,
    pub fallback: Option<LlmProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub provider: String,  // "ollama", "deepseek", "openai"
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 短期记忆容量（默认 100）
    pub capacity: Option<usize>,

    /// 持久化文件路径
    pub persistent_file: Option<String>,

    /// 是否自动保存到文件（默认 false）
    pub auto_save: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default = "default_true")]
    pub shell_enabled: bool,

    #[serde(default = "default_timeout")]
    pub shell_timeout: u64,

    /// 是否启用工具调用（Function Calling）
    #[serde(default)]
    pub tool_calling_enabled: Option<bool>,

    /// 工具调用最大迭代轮数（默认 5）
    #[serde(default = "default_max_tool_iterations")]
    pub max_tool_iterations: usize,

    /// 每轮最多工具数（默认 3）
    #[serde(default = "default_max_tools_per_round")]
    pub max_tools_per_round: usize,

    /// 是否启用 Workflow Intent 系统（Phase 8，默认 false）
    /// 套路化复用，将成功的 LLM 调用模式固化为模板
    #[serde(default = "default_workflow_enabled")]
    pub workflow_enabled: Option<bool>,

    /// 是否启用 Workflow 缓存（默认 true）
    #[serde(default = "default_workflow_cache_enabled")]
    pub workflow_cache_enabled: Option<bool>,

    /// Workflow 缓存默认 TTL（秒，默认 300）
    #[serde(default = "default_workflow_cache_ttl")]
    pub workflow_cache_ttl_default: Option<u64>,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    10
}

fn default_max_tool_iterations() -> usize {
    5
}

fn default_max_tools_per_round() -> usize {
    3
}

fn default_workflow_enabled() -> Option<bool> {
    Some(false)
}

fn default_workflow_cache_enabled() -> Option<bool> {
    Some(true)
}

fn default_workflow_cache_ttl() -> Option<u64> {
    Some(300)
}

/// Intent DSL 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentConfig {
    /// 是否启用 LLM 智能参数提取（默认 false，仅使用 Regex）
    #[serde(default = "default_false")]
    pub llm_extraction_enabled: bool,

    /// 是否启用 LLM 命令验证（默认 false）
    #[serde(default = "default_false")]
    pub llm_validation_enabled: bool,

    /// 命令验证的置信度阈值（0.0-1.0，默认 0.7）
    #[serde(default = "default_validation_threshold")]
    pub validation_threshold: f64,

    /// 验证失败时是否需要用户确认（默认 true）
    #[serde(default = "default_true")]
    pub require_confirmation: bool,

    /// 是否启用 LLM 驱动的 Pipeline 生成（Phase 7，默认 false）
    #[serde(default)]
    pub llm_generation_enabled: Option<bool>,

    /// LLM 生成失败时是否降级到规则匹配（默认 true）
    #[serde(default)]
    pub llm_generation_fallback: Option<bool>,
}

fn default_false() -> bool {
    false
}

fn default_validation_threshold() -> f64 {
    0.7
}

impl Default for IntentConfig {
    fn default() -> Self {
        Self {
            llm_extraction_enabled: false,  // 默认关闭，保持高性能
            llm_validation_enabled: false,  // 默认关闭，保持高性能
            validation_threshold: 0.7,
            require_confirmation: true,
            llm_generation_enabled: Some(false),  // Phase 7: 默认关闭
            llm_generation_fallback: Some(true),  // 默认开启降级
        }
    }
}

/// 显示模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// 显示模式：minimal（默认）、standard、debug
    #[serde(default)]
    pub mode: DisplayMode,

    /// 界面语言（zh-CN, en-US）
    #[serde(default)]
    pub language: Option<String>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Minimal,  // 默认极简模式
            language: None,  // 未指定时从系统环境推断
        }
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            shell_enabled: true,
            shell_timeout: 10,
            tool_calling_enabled: Some(false), // 默认关闭，保持向后兼容
            max_tool_iterations: 5,
            max_tools_per_round: 3,
            workflow_enabled: Some(false), // Phase 8: 默认关闭，保持向后兼容
            workflow_cache_enabled: Some(true), // 启用 Workflow 时默认开启缓存
            workflow_cache_ttl_default: Some(300), // 默认缓存 5 分钟
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: "/".to_string(),
            llm: LlmConfig::default(),
            memory: None,
            features: FeaturesConfig::default(),
            intent: IntentConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl Config {
    /// 从 YAML 文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RealError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                RealError::new(
                    ErrorCode::ConfigNotFound,
                    format!("配置文件不存在: {}", path.display()),
                )
                .with_suggestion(
                    FixSuggestion::new("运行配置向导创建配置文件")
                        .with_command("realconsole wizard"),
                )
                .with_suggestion(
                    FixSuggestion::new("参考示例配置手动创建")
                        .with_command("cp config/minimal.yaml realconsole.yaml"),
                )
            } else {
                RealError::new(
                    ErrorCode::FileReadError,
                    format!("无法读取配置文件: {}", path.display()),
                )
                .with_suggestion(FixSuggestion::new("检查文件权限和路径是否正确"))
                .with_source(e)
            }
        })?;

        // 扩展环境变量
        let expanded = Self::expand_env_vars(&content);

        // 解析 YAML
        let config: Config = serde_yaml::from_str(&expanded).map_err(|e| {
            RealError::new(
                ErrorCode::ConfigParseError,
                format!("配置文件解析失败: {}", path.display()),
            )
            .with_suggestion(FixSuggestion::new("检查 YAML 语法是否正确"))
            .with_suggestion(
                FixSuggestion::new("参考示例配置文件")
                    .with_doc("https://docs.realconsole.com/config"),
            )
            .with_source(e)
        })?;

        Ok(config)
    }

    /// 尝试加载配置，失败则返回默认配置
    #[allow(dead_code)]  // 备用 API，可能在库使用场景中需要
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::from_file(path).unwrap_or_default()
    }

    /// 扩展环境变量
    ///
    /// 支持格式：
    /// - ${VAR}
    /// - ${VAR:-default}
    fn expand_env_vars(content: &str) -> String {
        // ${VAR:-default}
        let re_default = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\:-([^}]*)\}").unwrap();
        let step1 = re_default.replace_all(content, |caps: &regex::Captures| {
            let var = &caps[1];
            let default = &caps[2];
            env::var(var).unwrap_or_else(|_| default.to_string())
        });

        // ${VAR}
        let re_var = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        let step2 = re_var.replace_all(&step1, |caps: &regex::Captures| {
            let var = &caps[1];
            env::var(var).unwrap_or_default()
        });

        step2.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_expansion() {
        env::set_var("TEST_VAR", "hello");
        let input = "value: ${TEST_VAR}";
        let output = Config::expand_env_vars(input);
        assert_eq!(output, "value: hello");
    }

    #[test]
    fn test_env_var_with_default() {
        env::remove_var("MISSING_VAR");
        let input = "value: ${MISSING_VAR:-default_value}";
        let output = Config::expand_env_vars(input);
        assert_eq!(output, "value: default_value");
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.prefix, "/");
        assert!(config.features.shell_enabled);
        assert_eq!(config.features.max_tool_iterations, 5);
        assert_eq!(config.features.max_tools_per_round, 3);
    }

    #[test]
    fn test_custom_tool_limits() {
        // 测试自定义工具限制配置
        let yaml = r#"
prefix: "/"
features:
  shell_enabled: true
  shell_timeout: 10
  tool_calling_enabled: true
  max_tool_iterations: 10
  max_tools_per_round: 5
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.features.max_tool_iterations, 10);
        assert_eq!(config.features.max_tools_per_round, 5);
    }

    #[test]
    fn test_backward_compatibility_without_workflow_fields() {
        // 测试向后兼容：旧配置文件没有 workflow 字段也能正常解析
        let yaml = r#"
prefix: "/"
features:
  shell_enabled: true
  shell_timeout: 10
  tool_calling_enabled: false
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        // 验证旧字段正常工作
        assert_eq!(config.prefix, "/");
        assert!(config.features.shell_enabled);
        assert_eq!(config.features.shell_timeout, 10);
        assert_eq!(config.features.tool_calling_enabled, Some(false));

        // 验证新字段使用默认值（关键：默认禁用以保持向后兼容）
        assert_eq!(config.features.workflow_enabled, Some(false));
        assert_eq!(config.features.workflow_cache_enabled, Some(true));
        assert_eq!(config.features.workflow_cache_ttl_default, Some(300));
    }

    #[test]
    fn test_workflow_config_explicit_enable() {
        // 测试显式启用 Workflow 功能
        let yaml = r#"
prefix: "/"
features:
  shell_enabled: true
  tool_calling_enabled: false
  workflow_enabled: true
  workflow_cache_enabled: true
  workflow_cache_ttl_default: 600
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();

        // 验证 Workflow 配置正确解析
        assert_eq!(config.features.workflow_enabled, Some(true));
        assert_eq!(config.features.workflow_cache_enabled, Some(true));
        assert_eq!(config.features.workflow_cache_ttl_default, Some(600));
    }
}
