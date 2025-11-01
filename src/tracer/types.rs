//! 统一追踪系统的核心类型定义
//!
//! 提供四维观测体系的类型抽象

use serde::{Deserialize, Serialize};
use std::fmt;

/// 四个观测维度
///
/// 基于四象理论（太阳、少阴、少阳、太阴），提供互补的观测视角
///
/// # 哲学映射
///
/// ```text
/// 太阳 (Taiyang) → Statistics  - 统计维度，宏观规律
/// 少阴 (Shaoyin) → Coordination - 协同维度，执行追踪
/// 少阳 (Shaoyang) → BlackBox    - 黑盒维度，LLM透视
/// 太阴 (Taiyin)  → Memory       - 记忆维度，对话连贯
/// ```
///
/// 详见: `docs/04-reports/four-dimensions-philosophy.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dimension {
    /// 统计维度 - History (太阳/Taiyang)
    ///
    /// 关注：命令频率、使用模式、统计规律
    Statistics,

    /// 协同维度 - log (少阴/Shaoyin)
    ///
    /// 关注：端到端执行、任务协同、完整链路
    Coordination,

    /// 黑盒维度 - llm-log (少阳/Shaoyang)
    ///
    /// 关注：LLM API 调用、模型行为、token 使用
    BlackBox,

    /// 记忆维度 - Context (太阴/Taiyin)
    ///
    /// 关注：对话上下文、状态延续、记忆连贯
    Memory,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dimension::Statistics => write!(f, "Statistics"),
            Dimension::Coordination => write!(f, "Coordination"),
            Dimension::BlackBox => write!(f, "BlackBox"),
            Dimension::Memory => write!(f, "Memory"),
        }
    }
}

impl Dimension {
    /// 获取维度对应的图标
    pub fn icon(&self) -> &'static str {
        match self {
            Dimension::Statistics => "📊",
            Dimension::Coordination => "🔗",
            Dimension::BlackBox => "🤖",
            Dimension::Memory => "💭",
        }
    }

    /// 获取维度对应的命令名称
    pub fn command_name(&self) -> &'static str {
        match self {
            Dimension::Statistics => "history",
            Dimension::Coordination => "log",
            Dimension::BlackBox => "llm-log",
            Dimension::Memory => "context",
        }
    }

    /// 获取维度的中文名称
    pub fn chinese_name(&self) -> &'static str {
        match self {
            Dimension::Statistics => "统计维度",
            Dimension::Coordination => "协同维度",
            Dimension::BlackBox => "黑盒维度",
            Dimension::Memory => "记忆维度",
        }
    }

    /// 获取所有维度
    pub fn all() -> Vec<Dimension> {
        vec![
            Dimension::Statistics,
            Dimension::Coordination,
            Dimension::BlackBox,
            Dimension::Memory,
        ]
    }
}

/// 条目类型
///
/// 定义不同维度中的具体条目类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryType {
    // ━━━ 统计维度 ━━━
    /// Shell 命令（如 !ls, !cat）
    ShellCommand,

    /// 系统命令（如 /help, /history）
    SystemCommand,

    // ━━━ 协同维度 ━━━
    /// 任务执行记录
    TaskExecution,

    /// 工具调用记录
    ToolInvocation,

    // ━━━ 黑盒维度 ━━━
    /// LLM 请求
    LlmRequest,

    /// LLM 响应
    LlmResponse,

    /// LLM 完整对话（请求+响应）
    LlmConversation,

    // ━━━ 记忆维度 ━━━
    /// 对话消息
    ContextMessage,

    /// 上下文切换
    ContextSwitch,

    /// 上下文状态变更
    ContextStateChange,

    // ━━━ v1.15.0 Phase 2: 系统内部事件 ━━━
    /// 自适应优化事件
    AdaptiveOptimization,

    /// Bagua 炼化事件
    BaguaRefinement,

    /// 通用系统事件
    SystemEvent,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryType::ShellCommand => write!(f, "ShellCommand"),
            EntryType::SystemCommand => write!(f, "SystemCommand"),
            EntryType::TaskExecution => write!(f, "TaskExecution"),
            EntryType::ToolInvocation => write!(f, "ToolInvocation"),
            EntryType::LlmRequest => write!(f, "LlmRequest"),
            EntryType::LlmResponse => write!(f, "LlmResponse"),
            EntryType::LlmConversation => write!(f, "LlmConversation"),
            EntryType::ContextMessage => write!(f, "ContextMessage"),
            EntryType::ContextSwitch => write!(f, "ContextSwitch"),
            EntryType::ContextStateChange => write!(f, "ContextStateChange"),
            EntryType::AdaptiveOptimization => write!(f, "AdaptiveOptimization"),
            EntryType::BaguaRefinement => write!(f, "BaguaRefinement"),
            EntryType::SystemEvent => write!(f, "SystemEvent"),
        }
    }
}

impl EntryType {
    /// 获取条目类型的图标
    pub fn icon(&self) -> &'static str {
        match self {
            EntryType::ShellCommand => "🐚",
            EntryType::SystemCommand => "⚙️",
            EntryType::TaskExecution => "▶️",
            EntryType::ToolInvocation => "🔧",
            EntryType::LlmRequest => "📤",
            EntryType::LlmResponse => "📥",
            EntryType::LlmConversation => "💬",
            EntryType::ContextMessage => "💭",
            EntryType::ContextSwitch => "🔄",
            EntryType::ContextStateChange => "🔀",
            EntryType::AdaptiveOptimization => "🎯",
            EntryType::BaguaRefinement => "🌊",
            EntryType::SystemEvent => "⚡",
        }
    }

    /// 获取条目类型的中文名称
    pub fn chinese_name(&self) -> &'static str {
        match self {
            EntryType::ShellCommand => "Shell 命令",
            EntryType::SystemCommand => "系统命令",
            EntryType::TaskExecution => "任务执行",
            EntryType::ToolInvocation => "工具调用",
            EntryType::LlmRequest => "LLM 请求",
            EntryType::LlmResponse => "LLM 响应",
            EntryType::LlmConversation => "LLM 对话",
            EntryType::ContextMessage => "对话消息",
            EntryType::ContextSwitch => "上下文切换",
            EntryType::ContextStateChange => "状态变更",
            EntryType::AdaptiveOptimization => "自适应优化",
            EntryType::BaguaRefinement => "八卦炼化",
            EntryType::SystemEvent => "系统事件",
        }
    }
}

/// 记忆重要性级别（Memory 维度专用）
///
/// 用于标记 Memory 维度条目的重要程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Importance {
    /// 低重要性 - 可以快速淡忘
    Low,

    /// 普通重要性 - 默认级别
    Normal,

    /// 重要 - 需要长期保留
    Important,

    /// 关键 - 永久保留
    Critical,
}

impl fmt::Display for Importance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Importance::Low => write!(f, "Low"),
            Importance::Normal => write!(f, "Normal"),
            Importance::Important => write!(f, "Important"),
            Importance::Critical => write!(f, "Critical"),
        }
    }
}

impl Importance {
    /// 获取重要性标记符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Importance::Low => "",
            Importance::Normal => "",
            Importance::Important => "[*]",
            Importance::Critical => "[**]",
        }
    }

    /// 获取重要性对应的图标
    pub fn icon(&self) -> &'static str {
        match self {
            Importance::Low => "·",
            Importance::Normal => "○",
            Importance::Important => "●",
            Importance::Critical => "⭐",
        }
    }

    /// 获取重要性的中文名称
    pub fn chinese_name(&self) -> &'static str {
        match self {
            Importance::Low => "低",
            Importance::Normal => "普通",
            Importance::Important => "重要",
            Importance::Critical => "关键",
        }
    }
}

impl Default for Importance {
    fn default() -> Self {
        Importance::Normal
    }
}

/// 执行状态
///
/// 记录条目的执行结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// 成功
    Success,

    /// 失败（包含错误信息）
    Failed(String),

    /// 运行中
    Running,

    /// 已取消
    Cancelled,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Success => write!(f, "Success"),
            Status::Failed(err) => write!(f, "Failed({})", err),
            Status::Running => write!(f, "Running"),
            Status::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl Status {
    /// 获取状态图标
    pub fn icon(&self) -> &'static str {
        match self {
            Status::Success => "✓",
            Status::Failed(_) => "✗",
            Status::Running => "⟳",
            Status::Cancelled => "⊘",
        }
    }

    /// 判断是否为成功状态
    pub fn is_success(&self) -> bool {
        matches!(self, Status::Success)
    }

    /// 判断是否为失败状态
    pub fn is_failed(&self) -> bool {
        matches!(self, Status::Failed(_))
    }

    /// 获取错误信息（如果存在）
    pub fn error_message(&self) -> Option<&str> {
        if let Status::Failed(err) = self {
            Some(err)
        } else {
            None
        }
    }
}

// ━━━━━ v1.17.0 Phase 5: 查询过滤 ━━━━━

/// 查询过滤条件
///
/// 用于在 UnifiedTracer 层进行过滤下推，避免全量加载后过滤
///
/// # 示例
///
/// ```rust,no_run
/// use realconsole::tracer::types::{QueryFilter, EntryType, Importance};
///
/// // 查询重要的用户消息
/// let filter = QueryFilter {
///     entry_type: Some(EntryType::ContextMessage),
///     importance: Some(Importance::Important),
///     tags: None,
///     context_id: None,
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QueryFilter {
    /// 按条目类型过滤
    pub entry_type: Option<EntryType>,

    /// 按重要性过滤
    pub importance: Option<Importance>,

    /// 按标签过滤（包含任一标签即匹配）
    pub tags: Option<Vec<String>>,

    /// 按上下文 ID 过滤
    pub context_id: Option<String>,
}

impl QueryFilter {
    /// 创建新的空过滤器
    pub fn new() -> Self {
        Self::default()
    }

    /// 按条目类型过滤
    pub fn with_entry_type(mut self, entry_type: EntryType) -> Self {
        self.entry_type = Some(entry_type);
        self
    }

    /// 按重要性过滤
    pub fn with_importance(mut self, importance: Importance) -> Self {
        self.importance = Some(importance);
        self
    }

    /// 按标签过滤
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// 按上下文 ID 过滤
    pub fn with_context_id(mut self, context_id: String) -> Self {
        self.context_id = Some(context_id);
        self
    }

    /// 判断是否为空过滤器（无任何条件）
    pub fn is_empty(&self) -> bool {
        self.entry_type.is_none()
            && self.importance.is_none()
            && self.tags.is_none()
            && self.context_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_display() {
        assert_eq!(Dimension::Statistics.to_string(), "Statistics");
        assert_eq!(Dimension::Coordination.to_string(), "Coordination");
        assert_eq!(Dimension::BlackBox.to_string(), "BlackBox");
        assert_eq!(Dimension::Memory.to_string(), "Memory");
    }

    #[test]
    fn test_dimension_icon() {
        assert_eq!(Dimension::Statistics.icon(), "📊");
        assert_eq!(Dimension::Coordination.icon(), "🔗");
        assert_eq!(Dimension::BlackBox.icon(), "🤖");
        assert_eq!(Dimension::Memory.icon(), "💭");
    }

    #[test]
    fn test_dimension_command_name() {
        assert_eq!(Dimension::Statistics.command_name(), "history");
        assert_eq!(Dimension::Coordination.command_name(), "log");
        assert_eq!(Dimension::BlackBox.command_name(), "llm-log");
        assert_eq!(Dimension::Memory.command_name(), "context");
    }

    #[test]
    fn test_dimension_all() {
        let all = Dimension::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&Dimension::Statistics));
        assert!(all.contains(&Dimension::Coordination));
        assert!(all.contains(&Dimension::BlackBox));
        assert!(all.contains(&Dimension::Memory));
    }

    #[test]
    fn test_entry_type_icon() {
        assert_eq!(EntryType::ShellCommand.icon(), "🐚");
        assert_eq!(EntryType::LlmRequest.icon(), "📤");
        assert_eq!(EntryType::ContextMessage.icon(), "💭");
    }

    #[test]
    fn test_status_icon() {
        assert_eq!(Status::Success.icon(), "✓");
        assert_eq!(Status::Failed("error".to_string()).icon(), "✗");
        assert_eq!(Status::Running.icon(), "⟳");
        assert_eq!(Status::Cancelled.icon(), "⊘");
    }

    #[test]
    fn test_status_is_success() {
        assert!(Status::Success.is_success());
        assert!(!Status::Failed("error".to_string()).is_success());
        assert!(!Status::Running.is_success());
    }

    #[test]
    fn test_status_is_failed() {
        assert!(!Status::Success.is_failed());
        assert!(Status::Failed("error".to_string()).is_failed());
        assert!(!Status::Running.is_failed());
    }

    #[test]
    fn test_status_error_message() {
        assert_eq!(Status::Success.error_message(), None);
        assert_eq!(
            Status::Failed("test error".to_string()).error_message(),
            Some("test error")
        );
        assert_eq!(Status::Running.error_message(), None);
    }

    #[test]
    fn test_dimension_serialization() {
        let dim = Dimension::Statistics;
        let json = serde_json::to_string(&dim).unwrap();
        let deserialized: Dimension = serde_json::from_str(&json).unwrap();
        assert_eq!(dim, deserialized);
    }

    #[test]
    fn test_entry_type_serialization() {
        let entry_type = EntryType::ShellCommand;
        let json = serde_json::to_string(&entry_type).unwrap();
        let deserialized: EntryType = serde_json::from_str(&json).unwrap();
        assert_eq!(entry_type, deserialized);
    }

    #[test]
    fn test_status_serialization() {
        let status = Status::Failed("test".to_string());
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: Status = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
