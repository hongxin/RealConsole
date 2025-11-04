//! 配置系统
//!
//! 支持：
//! - YAML 配置文件加载
//! - 多路径搜索（当前目录 + ~/.realconsole/）
//! - 环境变量扩展 ${VAR} 和 ${VAR:-default}
//! - 默认配置

use crate::display::DisplayMode;
use crate::error::{ErrorCode, FixSuggestion, RealError};
use crate::path_resolver::PathResolver;
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

    /// 对话上下文配置
    #[serde(default)]
    pub conversation: ConversationConfig,

    /// 语音播报配置
    #[serde(default)]
    pub voice: VoiceConfig,

    /// ✨ v1.21.0: 任务系统配置
    #[serde(default)]
    pub task: TaskConfig,

    /// 离坎炼化炉配置
    #[serde(default)]
    pub likan: Option<crate::likan::FurnaceConfig>,

    /// 八卦记忆宫配置
    #[serde(default)]
    pub bagua: Option<BaguaConfig>,

    /// ✨ v1.9.1: 两仪演化系统配置
    #[serde(default)]
    pub liangyyi: Option<LiangyyiConfig>,

    /// ✨ v1.23.0: Web 终端配置
    #[serde(default)]
    pub web: WebConfig,
}

fn default_prefix() -> String {
    "/".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmConfig {
    pub primary: Option<LlmProvider>,
    pub fallback: Option<LlmProvider>,

    /// LLM 交互日志配置
    #[serde(default)]
    pub logging: LlmLoggingConfig,

    /// 系统提示词（可选，用于指导 LLM 行为）
    /// 如果未配置，将使用内置默认提示词
    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub provider: String, // "ollama", "deepseek", "openai"
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}

/// LLM 交互日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmLoggingConfig {
    /// 是否启用日志（默认 false）
    #[serde(default)]
    pub enabled: bool,

    /// 日志目录（默认 ~/.realconsole/llm_logs）
    pub log_dir: Option<String>,

    /// 是否记录完整内容（默认 true）
    #[serde(default = "default_true")]
    pub include_content: bool,

    /// 日志保留天数（默认 30）
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    /// 最大日志大小 MB（默认 100）
    #[serde(default = "default_max_size_mb")]
    pub max_size_mb: u32,
}

fn default_retention_days() -> u32 {
    30
}

fn default_max_size_mb() -> u32 {
    100
}

impl Default for LlmLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_dir: None,
            include_content: true,
            retention_days: 30,
            max_size_mb: 100,
        }
    }
}

impl LlmConfig {
    /// ✨ 从环境变量智能检测 LLM 配置
    ///
    /// 检测优先级：
    /// 1. DEEPSEEK_API_KEY → Deepseek (推荐，性价比高)
    /// 2. OPENAI_API_KEY → OpenAI
    /// 3. ANTHROPIC_API_KEY → Claude
    ///
    /// # 示例
    /// ```ignore
    /// export DEEPSEEK_API_KEY="sk-xxx"
    /// let config = LlmConfig::detect_from_env();
    /// ```
    pub fn detect_from_env() -> Self {
        // 检测 Deepseek
        if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
            if !key.is_empty() {
                return Self {
                    primary: Some(LlmProvider {
                        provider: "deepseek".to_string(),
                        model: Some("deepseek-chat".to_string()),
                        endpoint: Some("https://api.deepseek.com/v1".to_string()),
                        api_key: Some(key),
                    }),
                    fallback: None,
                    logging: LlmLoggingConfig::default(),
                    system_prompt: None,
                };
            }
        }

        // 检测 OpenAI
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            if !key.is_empty() {
                return Self {
                    primary: Some(LlmProvider {
                        provider: "openai".to_string(),
                        model: Some("gpt-4".to_string()),
                        endpoint: Some("https://api.openai.com/v1".to_string()),
                        api_key: Some(key),
                    }),
                    fallback: None,
                    logging: LlmLoggingConfig::default(),
                    system_prompt: None,
                };
            }
        }

        // 检测 Claude
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                return Self {
                    primary: Some(LlmProvider {
                        provider: "claude".to_string(),
                        model: Some("claude-3-sonnet".to_string()),
                        endpoint: Some("https://api.anthropic.com/v1".to_string()),
                        api_key: Some(key),
                    }),
                    fallback: None,
                    logging: LlmLoggingConfig::default(),
                    system_prompt: None,
                };
            }
        }

        // 未检测到任何 API key，返回默认配置
        Self::default()
    }
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

    /// ✨ Phase 4.1: 是否启用命令失败时自动建议（默认 true）
    #[serde(default = "default_auto_suggest")]
    pub auto_suggest: Option<bool>,
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

fn default_auto_suggest() -> Option<bool> {
    Some(true)
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
            llm_extraction_enabled: false, // 默认关闭，保持高性能
            llm_validation_enabled: false, // 默认关闭，保持高性能
            validation_threshold: 0.7,
            require_confirmation: true,
            llm_generation_enabled: Some(false), // Phase 7: 默认关闭
            llm_generation_fallback: Some(true), // 默认开启降级
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

    /// 是否显示对话轮次详情（仅在 debug 模式下生效，默认 true）
    /// 即使是正常成功的对话也会显示 LLM 多轮次来回的详细信息
    #[serde(default = "default_show_conversation_rounds")]
    pub show_conversation_rounds: bool,

    /// 是否使用 emoji（默认 false，推荐关闭以避免终端兼容性问题）
    #[serde(default)]
    pub use_emoji: bool,

    /// 是否使用颜色（默认 true）
    #[serde(default = "default_true")]
    pub use_colors: bool,
}

fn default_show_conversation_rounds() -> bool {
    true
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Minimal,     // 默认极简模式
            language: None,                 // 未指定时从系统环境推断
            show_conversation_rounds: true, // debug 模式下默认显示对话轮次
            use_emoji: false,               // 默认关闭 emoji（避免终端兼容性问题）
            use_colors: true,               // 默认启用颜色
        }
    }
}

/// 对话上下文配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfig {
    /// 上下文模式（旧配置，保留向后兼容）
    #[serde(default)]
    pub mode: ContextMode,

    /// 上下文感知场（新配置，可选）
    ///
    /// 如果未指定，会从 `mode` 自动转换
    /// 优先级：awareness_field > mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awareness_field: Option<ContextAwarenessField>,

    /// 最大轮次（保留最近 N 轮对话）
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,

    /// 最大上下文长度（字符数，超过则自动裁剪）
    #[serde(default = "default_max_context_length")]
    pub max_context_length: usize,

    /// 自动清除策略
    #[serde(default)]
    pub auto_clear: AutoClearConfig,

    /// 上下文包含内容
    #[serde(default)]
    pub include: ContextIncludeConfig,
}

fn default_max_turns() -> usize {
    10
}

fn default_max_context_length() -> usize {
    8000
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            mode: ContextMode::Disabled, // 默认关闭，保持向后兼容
            awareness_field: None,       // 默认从 mode 转换
            max_turns: 10,
            max_context_length: 8000,
            auto_clear: AutoClearConfig::default(),
            include: ContextIncludeConfig::default(),
        }
    }
}

impl ConversationConfig {
    /// 获取有效的上下文感知场配置
    ///
    /// 优先使用显式配置的 `awareness_field`，
    /// 否则从 `mode` 自动转换（向后兼容）
    pub fn effective_field(&self) -> ContextAwarenessField {
        self.awareness_field
            .clone()
            .unwrap_or_else(|| self.mode.into())
    }

    /// 检查是否使用连续场模式
    pub fn is_continuous_mode(&self) -> bool {
        self.awareness_field.is_some()
    }
}

/// 上下文模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextMode {
    /// 关闭（默认）：单命令执行，无上下文
    Disabled,
    /// 手动：用户显式控制上下文
    Manual,
    /// 自动：智能识别需要上下文的场景
    Auto,
}

impl Default for ContextMode {
    fn default() -> Self {
        Self::Disabled
    }
}

impl std::fmt::Display for ContextMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextMode::Disabled => write!(f, "Disabled"),
            ContextMode::Manual => write!(f, "Manual"),
            ContextMode::Auto => write!(f, "Auto"),
        }
    }
}

/// 上下文感知场配置（连续化重构）
///
/// 基于 [docs/00-core/think.md](../../docs/00-core/think.md) 哲学：
/// 将离散的三态（Disabled/Manual/Auto）演化为**连续的势能场**。
///
/// 核心理念：
/// - 上下文感知不是"开/关"，而是 0%-100% 的连续强度
/// - 支持"部分激活"、"渐变触发"、"平滑衰减"
/// - 向后兼容：可从旧的 ContextMode 自动转换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwarenessField {
    /// 基础敏感度 (0.0 - 1.0)
    ///
    /// - 0.0 = 完全关闭（等价于 Disabled）
    /// - 0.5 = 中等感知（等价于 Manual）
    /// - 1.0 = 全自动感知（等价于 Auto）
    #[serde(default = "default_sensitivity")]
    pub sensitivity: f64,

    /// 自动触发阈值 (0.0 - 1.0)
    ///
    /// 当"需要上下文"的置信度 >= 此值时自动启用
    #[serde(default = "default_auto_threshold")]
    pub auto_threshold: f64,

    /// 上下文衰减速率（每秒衰减百分比）
    ///
    /// 控制上下文强度如何随时间衰减
    #[serde(default = "default_decay_rate")]
    pub decay_rate: f64,

    /// 最大上下文强度 (0.0 - 1.0)
    ///
    /// 限制上下文的最大影响力
    #[serde(default = "default_max_strength")]
    pub max_strength: f64,
}

fn default_sensitivity() -> f64 {
    0.0 // 默认关闭（向后兼容）
}

fn default_auto_threshold() -> f64 {
    0.6 // 60% 置信度触发
}

fn default_decay_rate() -> f64 {
    0.001 // 每秒衰减 0.1%
}

fn default_max_strength() -> f64 {
    1.0 // 允许全强度
}

impl Default for ContextAwarenessField {
    fn default() -> Self {
        Self {
            sensitivity: 0.0,
            auto_threshold: 0.6,
            decay_rate: 0.001,
            max_strength: 1.0,
        }
    }
}

/// 向后兼容：从旧的 ContextMode 转换为连续场配置
impl From<ContextMode> for ContextAwarenessField {
    fn from(mode: ContextMode) -> Self {
        match mode {
            ContextMode::Disabled => Self {
                sensitivity: 0.0,
                auto_threshold: 1.0, // 永不自动触发
                decay_rate: 0.01,    // 快速衰减
                max_strength: 0.0,   // 零强度
            },
            ContextMode::Manual => Self {
                sensitivity: 0.5,
                auto_threshold: 1.0, // 不自动触发（需手动启动）
                decay_rate: 0.001,
                max_strength: 0.8, // 中等强度
            },
            ContextMode::Auto => Self {
                sensitivity: 1.0,
                auto_threshold: 0.6, // 较低阈值，容易触发
                decay_rate: 0.0005,  // 缓慢衰减
                max_strength: 1.0,   // 全强度
            },
        }
    }
}

/// 自动清除配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoClearConfig {
    /// 是否启用自动清除
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// 空闲多久后清除（秒）
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    /// 任务完成后是否清除
    #[serde(default = "default_false")]
    pub on_task_complete: bool,
}

fn default_idle_timeout() -> u64 {
    600 // 10 分钟
}

impl Default for AutoClearConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_timeout: 600,
            on_task_complete: false,
        }
    }
}

/// 上下文包含配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextIncludeConfig {
    /// 是否包含工具调用历史
    #[serde(default = "default_true")]
    pub tool_calls: bool,

    /// 是否包含 Shell 执行结果
    #[serde(default = "default_false")]
    pub shell_output: bool,

    /// 是否包含错误信息
    #[serde(default = "default_true")]
    pub errors: bool,
}

impl Default for ContextIncludeConfig {
    fn default() -> Self {
        Self {
            tool_calls: true,
            shell_output: false,
            errors: true,
        }
    }
}

/// 语音播报配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// 是否启用语音播报（默认 false）
    #[serde(default = "default_false")]
    pub enabled: bool,

    /// 语音名称（可选，使用系统默认）
    /// macOS: Ting-Ting (中文), Samantha (英文)
    pub voice: Option<String>,

    /// 最大队列长度（默认 10）
    #[serde(default = "default_max_queue_size")]
    pub max_queue_size: usize,

    /// 是否自动播报 LLM 响应（默认 true）
    /// 当 enabled=true 时，自动播报所有 LLM 的回复
    #[serde(default = "default_true")]
    pub auto_broadcast: bool,

    /// 最大播报长度（字符数，默认 200）
    /// 超过此长度会被截断并添加省略提示
    #[serde(default = "default_max_broadcast_length")]
    pub max_broadcast_length: usize,

    /// 是否过滤代码块（默认 true）
    /// 自动跳过 markdown 代码块内容
    #[serde(default = "default_true")]
    pub filter_code_blocks: bool,
}

fn default_max_queue_size() -> usize {
    10
}

fn default_max_broadcast_length() -> usize {
    200
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            voice: None,
            max_queue_size: 10,
            auto_broadcast: true,
            max_broadcast_length: 200,
            filter_code_blocks: true,
        }
    }
}

/// ✨ v1.21.0: 任务系统配置
///
/// 控制任务执行和显示的行为
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskConfig {
    /// 任务显示配置
    #[serde(default)]
    pub display: TaskDisplayConfig,

    /// 任务执行配置
    #[serde(default)]
    pub execution: TaskExecutionConfig,
}

/// 任务显示配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDisplayConfig {
    /// 是否在 Standard 模式下显示任务输出（默认 true）
    #[serde(default = "default_true")]
    pub show_task_output: bool,

    /// 输出最大行数（Standard 模式，默认 50）
    /// 0 表示不限制（等同于 Debug 模式）
    #[serde(default = "default_max_output_lines")]
    pub max_output_lines: usize,

    /// 是否高亮数字（识别计算结果，默认 true）
    #[serde(default = "default_true")]
    pub highlight_numbers: bool,

    /// 是否显示任务执行时间（默认 true）
    #[serde(default = "default_true")]
    pub show_task_duration: bool,
}

/// 任务执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionConfig {
    /// 是否合并 Stage 执行（支持环境变量共享，默认 true）
    #[serde(default = "default_true")]
    pub merge_stages: bool,

    /// 合并执行的最大任务数（默认 20）
    /// 超过此数量的任务计划将不合并（防止命令过长）
    #[serde(default = "default_max_merged_tasks")]
    pub max_merged_tasks: usize,
}

fn default_max_output_lines() -> usize {
    50
}

fn default_max_merged_tasks() -> usize {
    20
}

impl Default for TaskDisplayConfig {
    fn default() -> Self {
        Self {
            show_task_output: true,
            max_output_lines: 50,
            highlight_numbers: true,
            show_task_duration: true,
        }
    }
}

impl Default for TaskExecutionConfig {
    fn default() -> Self {
        Self {
            merge_stages: true,
            max_merged_tasks: 20,
        }
    }
}

/// ✨ 八卦记忆宫配置（v1.8.4+）
///
/// 基于易经八卦哲学的多维记忆系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaguaConfig {
    /// 是否启用八卦记忆宫（默认 false）
    #[serde(default = "default_false")]
    pub enabled: bool,

    /// 存储位置（默认 ~/.realconsole/bagua）
    pub storage_path: Option<String>,

    /// 每个维度的最大容量（默认 1000）
    #[serde(default = "default_dimension_capacity")]
    pub dimension_capacity: usize,

    /// 数据保留天数（默认 30）
    #[serde(default = "default_bagua_retention_days")]
    pub retention_days: u64,

    /// 是否启用跨维度查询（默认 true）
    #[serde(default = "default_true")]
    pub cross_dimension_query: bool,
}

fn default_dimension_capacity() -> usize {
    1000
}

fn default_bagua_retention_days() -> u64 {
    30
}

impl Default for BaguaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            storage_path: None, // 使用智能默认路径
            dimension_capacity: 1000,
            retention_days: 30,
            cross_dimension_query: true,
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
            auto_suggest: Some(true), // ✨ Phase 4.1: 默认开启自动建议
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
            conversation: ConversationConfig::default(),
            voice: VoiceConfig::default(),
            task: TaskConfig::default(), // ✨ v1.21.0: 任务系统配置
            likan: None, // 默认使用 None，从配置文件加载
            bagua: None, // ✨ 八卦记忆宫，默认关闭
            liangyyi: None, // ✨ v1.9.1: 两仪演化系统，默认使用默认配置
            web: WebConfig::default(), // ✨ v1.23.0: Web 终端，默认关闭
        }
    }
}

impl Config {
    /// ✨ 创建智能默认配置
    ///
    /// 从环境变量智能检测 LLM 配置，其他全部使用合理默认值
    ///
    /// # 环境变量检测
    /// - `DEEPSEEK_API_KEY` → Deepseek
    /// - `OPENAI_API_KEY` → OpenAI
    /// - `ANTHROPIC_API_KEY` → Claude
    ///
    /// # 示例
    /// ```ignore
    /// // 只需设置环境变量
    /// export DEEPSEEK_API_KEY="sk-xxx"
    ///
    /// // 创建配置（自动检测）
    /// let config = Config::smart_defaults();
    /// ```
    pub fn smart_defaults() -> Self {
        let default = Self::default();

        // 智能检测 LLM 配置
        let llm = LlmConfig::detect_from_env();

        // 构建配置
        Self {
            llm,
            features: FeaturesConfig {
                tool_calling_enabled: Some(true),
                auto_suggest: Some(true),
                ..default.features
            },
            likan: Some(crate::likan::FurnaceConfig::default()),
            ..default
        }
    }

    /// 从 YAML 文件加载配置（支持多路径搜索）
    ///
    /// # 搜索策略
    /// 1. 如果提供的是绝对路径：直接使用
    /// 2. 如果是相对路径/文件名：按以下顺序搜索
    ///    - 当前工作目录
    ///    - ~/.realconsole/ 目录
    ///
    /// # 示例
    /// ```ignore
    /// // 显式路径
    /// Config::from_file("/etc/realconsole.yaml");
    ///
    /// // 自动搜索
    /// Config::from_file("realconsole.yaml");  // 在 ./ 和 ~/.realconsole/ 中搜索
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RealError> {
        let path_str = path.as_ref().to_str().unwrap_or("realconsole.yaml");

        // 使用 PathResolver 进行路径解析
        let resolved_path = PathResolver::resolve_config(path_str).ok_or_else(|| {
            let search_paths = PathResolver::search_paths(path_str);
            let search_locations = search_paths
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n");

            RealError::new(
                ErrorCode::ConfigNotFound,
                format!(
                    "配置文件未找到: {}\n\n搜索位置：\n{}",
                    path_str, search_locations
                ),
            )
            .with_suggestion(
                FixSuggestion::new("运行配置向导创建配置文件")
                    .with_command("realconsole wizard"),
            )
            .with_suggestion(
                FixSuggestion::new("手动复制示例配置到用户目录")
                    .with_command("cp config/minimal.yaml ~/.realconsole/realconsole.yaml"),
            )
        })?;

        let content = fs::read_to_string(&resolved_path).map_err(|e| {
            RealError::new(
                ErrorCode::FileReadError,
                format!("无法读取配置文件: {}", resolved_path.display()),
            )
            .with_suggestion(FixSuggestion::new("检查文件权限和路径是否正确"))
            .with_source(e)
        })?;

        // 扩展环境变量
        let expanded = Self::expand_env_vars(&content);

        // 解析 YAML
        let config: Config = serde_yml::from_str(&expanded).map_err(|e| {
            RealError::new(
                ErrorCode::ConfigParseError,
                format!("配置文件解析失败: {}", resolved_path.display()),
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
    #[allow(dead_code)] // 备用 API，可能在库使用场景中需要
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::from_file(path).unwrap_or_default()
    }

    /// ✨ 验证配置并自动修复问题
    ///
    /// 返回警告信息列表，同时修复配置中的常见问题
    ///
    /// # 示例
    /// ```ignore
    /// let mut config = Config::from_file("realconsole.yaml")?;
    /// let warnings = config.validate_and_fix();
    /// for warning in warnings {
    ///     eprintln!("{}", warning);
    /// }
    /// ```
    pub fn validate_and_fix(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        // 检查 LLM 配置
        if self.llm.primary.is_none() {
            warnings.push("⚠️ 未配置 LLM，部分功能将受限".to_string());
            warnings.push("💡 提示：设置环境变量 DEEPSEEK_API_KEY 或运行 `realconsole wizard`".to_string());
        }

        // 检查离坎炼化炉配置
        if let Some(ref mut likan) = self.likan {
            // 检查循环间隔
            if likan.cycle_interval_secs < 60 {
                warnings.push(format!(
                    "⚠️ 炼化炉循环间隔过短（{}秒），自动调整为 60 秒",
                    likan.cycle_interval_secs
                ));
                likan.cycle_interval_secs = 60;
            }

            // 检查配置一致性
            use crate::likan::NotificationMode;
            if likan.notification_mode == NotificationMode::Prompt && !likan.show_in_prompt {
                warnings.push("💡 已自动启用 show_in_prompt（notification_mode=prompt）".to_string());
                likan.show_in_prompt = true;
            }
        }

        // 检查内存配置
        if let Some(ref mut mem) = self.memory {
            if let Some(capacity) = mem.capacity {
                if capacity < 10 {
                    warnings.push(format!(
                        "⚠️ 内存容量过小（{}），自动调整为 10",
                        capacity
                    ));
                    mem.capacity = Some(10);
                }
            }
        }

        warnings
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

// ========================================
// ✨ v1.9.1: 两仪演化系统配置
// ========================================

/// 两仪演化系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiangyyiConfig {
    /// 是否启用两仪系统（默认: true）
    #[serde(default = "default_liangyyi_enabled")]
    pub enabled: bool,

    /// 状态追踪器配置
    #[serde(default)]
    pub state_tracker: crate::liangyyi::StateTrackerConfig,
}

fn default_liangyyi_enabled() -> bool {
    true
}

impl Default for LiangyyiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            state_tracker: crate::liangyyi::StateTrackerConfig::default(),
        }
    }
}

/// ✨ v1.23.0: Web 终端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    /// 是否启用 Web 服务（默认: false）
    #[serde(default)]
    pub enabled: bool,

    /// 绑定地址（默认: 127.0.0.1，仅本地访问）
    #[serde(default = "default_web_bind")]
    pub bind: String,

    /// 端口（默认: 7788）
    #[serde(default = "default_web_port")]
    pub port: u16,

    /// CORS 允许的源（默认: ["*"]）
    #[serde(default = "default_web_allowed_origins")]
    pub allowed_origins: Vec<String>,
}

fn default_web_bind() -> String {
    "127.0.0.1".to_string()
}

fn default_web_port() -> u16 {
    7788
}

fn default_web_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: default_web_bind(),
            port: 7788,
            allowed_origins: default_web_allowed_origins(),
        }
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
        let config: Config = serde_yml::from_str(yaml).unwrap();
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
        let config: Config = serde_yml::from_str(yaml).unwrap();

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
        let config: Config = serde_yml::from_str(yaml).unwrap();

        // 验证 Workflow 配置正确解析
        assert_eq!(config.features.workflow_enabled, Some(true));
        assert_eq!(config.features.workflow_cache_enabled, Some(true));
        assert_eq!(config.features.workflow_cache_ttl_default, Some(600));
    }

    #[test]
    fn test_conversation_config_default() {
        // 测试对话上下文默认配置（向后兼容：默认关闭）
        let config = Config::default();

        assert_eq!(config.conversation.mode, ContextMode::Disabled);
        assert_eq!(config.conversation.max_turns, 10);
        assert_eq!(config.conversation.max_context_length, 8000);
        assert!(config.conversation.auto_clear.enabled);
        assert_eq!(config.conversation.auto_clear.idle_timeout, 600);
        assert!(!config.conversation.auto_clear.on_task_complete);
        assert!(config.conversation.include.tool_calls);
        assert!(!config.conversation.include.shell_output);
        assert!(config.conversation.include.errors);
    }

    #[test]
    fn test_conversation_config_manual_mode() {
        // 测试手动模式配置
        let yaml = r#"
prefix: "/"
conversation:
  mode: manual
  max_turns: 20
  max_context_length: 16000
  auto_clear:
    enabled: false
"#;
        let config: Config = serde_yml::from_str(yaml).unwrap();

        assert_eq!(config.conversation.mode, ContextMode::Manual);
        assert_eq!(config.conversation.max_turns, 20);
        assert_eq!(config.conversation.max_context_length, 16000);
        assert!(!config.conversation.auto_clear.enabled);
    }

    #[test]
    fn test_conversation_config_auto_mode() {
        // 测试自动模式配置
        let yaml = r#"
prefix: "/"
conversation:
  mode: auto
  max_turns: 5
  max_context_length: 8000
  auto_clear:
    enabled: true
    idle_timeout: 300
    on_task_complete: true
  include:
    tool_calls: true
    shell_output: false
    errors: true
"#;
        let config: Config = serde_yml::from_str(yaml).unwrap();

        assert_eq!(config.conversation.mode, ContextMode::Auto);
        assert_eq!(config.conversation.max_turns, 5);
        assert_eq!(config.conversation.max_context_length, 8000);
        assert!(config.conversation.auto_clear.enabled);
        assert_eq!(config.conversation.auto_clear.idle_timeout, 300);
        assert!(config.conversation.auto_clear.on_task_complete);
        assert!(config.conversation.include.tool_calls);
        assert!(!config.conversation.include.shell_output);
        assert!(config.conversation.include.errors);
    }

    #[test]
    fn test_conversation_backward_compatibility() {
        // 测试向后兼容：旧配置文件没有 conversation 字段
        let yaml = r#"
prefix: "/"
features:
  shell_enabled: true
  shell_timeout: 10
"#;
        let config: Config = serde_yml::from_str(yaml).unwrap();

        // 验证使用默认值（关闭模式）
        assert_eq!(config.conversation.mode, ContextMode::Disabled);
        assert_eq!(config.conversation.max_turns, 10);
    }

    #[test]
    fn test_task_config_default() {
        // 测试 TaskConfig 默认值
        let config = TaskConfig::default();

        assert!(config.display.show_task_output);
        assert_eq!(config.display.max_output_lines, 50);
        assert!(config.display.highlight_numbers);
        assert!(config.display.show_task_duration);

        assert!(config.execution.merge_stages);
        assert_eq!(config.execution.max_merged_tasks, 20);
    }

    #[test]
    fn test_task_config_from_yaml() {
        // 测试从 YAML 加载 TaskConfig
        let yaml = r#"
prefix: "/"
task:
  display:
    show_task_output: false
    max_output_lines: 100
    highlight_numbers: false
    show_task_duration: false
  execution:
    merge_stages: false
    max_merged_tasks: 50
"#;
        let config: Config = serde_yml::from_str(yaml).unwrap();

        assert!(!config.task.display.show_task_output);
        assert_eq!(config.task.display.max_output_lines, 100);
        assert!(!config.task.display.highlight_numbers);
        assert!(!config.task.display.show_task_duration);

        assert!(!config.task.execution.merge_stages);
        assert_eq!(config.task.execution.max_merged_tasks, 50);
    }

    #[test]
    fn test_task_config_partial() {
        // 测试部分配置（其他使用默认值）
        let yaml = r#"
prefix: "/"
task:
  display:
    max_output_lines: 0
"#;
        let config: Config = serde_yml::from_str(yaml).unwrap();

        // 明确配置的值
        assert_eq!(config.task.display.max_output_lines, 0); // 0 表示不限制

        // 使用默认值的字段
        assert!(config.task.display.show_task_output); // 默认 true
        assert!(config.task.display.highlight_numbers); // 默认 true
        assert!(config.task.display.show_task_duration); // 默认 true
        assert!(config.task.execution.merge_stages); // 默认 true
        assert_eq!(config.task.execution.max_merged_tasks, 20); // 默认 20
    }
}
