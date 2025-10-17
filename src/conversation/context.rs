//! 对话上下文
//!
//! 管理单个对话的状态、参数和生命周期

use super::state::ConversationState;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

// Re-export for convenience
pub use super::parameter::{ParameterType, ParameterValue};

/// 对话上下文
pub struct ConversationContext {
    /// 对话 ID
    pub id: String,

    /// 对话意图
    pub intent: String,

    /// 对话状态
    pub state: ConversationState,

    /// 待收集的参数
    pub pending_parameters: Vec<ParameterSpec>,

    /// 已收集的参数
    pub parameters: HashMap<String, ParameterValue>,

    /// 超时时间（秒）
    #[allow(dead_code)]
    pub timeout: u64,

    /// 最后活跃时间
    pub last_active: DateTime<Utc>,

    /// 创建时间
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl ConversationContext {
    /// 创建新的对话上下文
    pub fn new(intent: impl Into<String>, timeout: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            intent: intent.into(),
            state: ConversationState::Initializing,
            pending_parameters: Vec::new(),
            parameters: HashMap::new(),
            timeout,
            last_active: Utc::now(),
            created_at: Utc::now(),
        }
    }

    /// 标记参数已收集
    pub fn mark_parameter_collected(&mut self, name: &str) {
        self.pending_parameters.retain(|p| p.name != name);
        self.last_active = Utc::now();
    }

    /// 获取下一个待收集的参数
    pub fn next_pending_parameter(&self) -> Option<&ParameterSpec> {
        self.pending_parameters.first()
    }

    /// 检查是否超时
    #[allow(dead_code)]
    pub fn is_timeout(&self) -> bool {
        let elapsed = (Utc::now() - self.last_active).num_seconds() as u64;
        elapsed > self.timeout
    }

    /// 更新活跃时间
    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.last_active = Utc::now();
    }
}

/// 参数规格
///
/// 定义对话需要收集的参数
#[derive(Debug, Clone)]
pub struct ParameterSpec {
    /// 参数名称
    pub name: String,

    /// 参数类型
    pub param_type: ParameterType,

    /// 参数描述
    pub description: String,

    /// 是否可选
    pub is_optional: bool,

    /// 提示信息
    pub hint: Option<String>,

    /// 默认值
    pub default: Option<ParameterValue>,

    /// 示例值
    pub example: Option<String>,
}

impl ParameterSpec {
    /// 创建新的参数规格（默认必需）
    pub fn new(name: &str, param_type: ParameterType, description: &str) -> Self {
        Self {
            name: name.to_string(),
            param_type,
            description: description.to_string(),
            is_optional: false,
            hint: None,
            default: None,
            example: None,
        }
    }

    /// 标记为可选参数
    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    /// 设置提示信息
    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_string());
        self
    }

    /// 设置默认值
    #[allow(dead_code)]
    pub fn with_default(mut self, default: ParameterValue) -> Self {
        self.default = Some(default);
        self
    }

    /// 设置示例值
    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ConversationContext::new("test_intent", 300);

        assert!(!ctx.id.is_empty());
        assert_eq!(ctx.intent, "test_intent");
        assert_eq!(ctx.timeout, 300);
        assert_eq!(ctx.state, ConversationState::Initializing);
        assert!(ctx.pending_parameters.is_empty());
        assert!(ctx.parameters.is_empty());
    }

    #[test]
    fn test_parameter_collection() {
        let mut ctx = ConversationContext::new("test", 300);

        let spec1 = ParameterSpec::new("param1", ParameterType::String, "First param");
        let spec2 = ParameterSpec::new("param2", ParameterType::String, "Second param");

        ctx.pending_parameters.push(spec1);
        ctx.pending_parameters.push(spec2);

        assert_eq!(ctx.pending_parameters.len(), 2);
        assert_eq!(ctx.next_pending_parameter().unwrap().name, "param1");

        ctx.mark_parameter_collected("param1");

        assert_eq!(ctx.pending_parameters.len(), 1);
        assert_eq!(ctx.next_pending_parameter().unwrap().name, "param2");
    }

    #[test]
    fn test_timeout_detection() {
        let ctx = ConversationContext::new("test", 1);  // 1秒超时

        assert!(!ctx.is_timeout());  // 刚创建不应该超时
    }

    #[test]
    fn test_parameter_spec_builder() {
        let spec = ParameterSpec::new("test_param", ParameterType::String, "Test description")
            .with_hint("This is a hint")
            .with_example("example_value");

        assert_eq!(spec.name, "test_param");
        assert_eq!(spec.description, "Test description");
        assert_eq!(spec.hint, Some("This is a hint".to_string()));
        assert_eq!(spec.example, Some("example_value".to_string()));
    }
}
