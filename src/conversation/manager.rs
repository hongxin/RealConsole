//! 对话管理器
//!
//! 管理所有活跃对话会话的生命周期

use super::analyzer::ParameterAnalyzer;
use super::context::{ConversationContext, ParameterSpec, ParameterValue};
use super::state::StateEvent;
use crate::llm::LlmClient;
use std::collections::HashMap;
use std::fmt;

/// 对话管理器
///
/// 负责创建、追踪和管理多个对话会话
pub struct ConversationManager {
    /// 活跃的对话
    active_conversations: HashMap<String, ConversationContext>,

    /// 默认超时（秒）
    default_timeout: u64,

    /// 参数分析器
    analyzer: ParameterAnalyzer,
}

impl ConversationManager {
    /// 创建新的对话管理器
    pub fn new(default_timeout: u64) -> Self {
        Self {
            active_conversations: HashMap::new(),
            default_timeout,
            analyzer: ParameterAnalyzer::new(),
        }
    }

    /// 启动新对话
    pub fn start_conversation(&mut self, intent: impl Into<String>) -> Result<String, ConversationError> {
        let mut context = ConversationContext::new(intent, self.default_timeout);

        // 转换到初始化状态
        context.state.transition(StateEvent::IntentRecognized)
            .map_err(|e| ConversationError::StateTransition(e.to_string()))?;

        let id = context.id.clone();
        self.active_conversations.insert(id.clone(), context);

        Ok(id)
    }

    /// 获取对话上下文
    pub fn get_context(&self, conversation_id: &str) -> Result<&ConversationContext, ConversationError> {
        self.active_conversations
            .get(conversation_id)
            .ok_or(ConversationError::NotFound)
    }

    /// 获取可变的对话上下文
    pub fn get_context_mut(&mut self, conversation_id: &str) -> Result<&mut ConversationContext, ConversationError> {
        self.active_conversations
            .get_mut(conversation_id)
            .ok_or(ConversationError::NotFound)
    }

    /// 添加参数规格到对话
    pub fn add_parameter_spec(
        &mut self,
        conversation_id: &str,
        spec: ParameterSpec,
    ) -> Result<(), ConversationError> {
        let context = self.get_context_mut(conversation_id)?;
        context.pending_parameters.push(spec);
        Ok(())
    }

    /// 收集参数
    pub fn collect_parameter(
        &mut self,
        conversation_id: &str,
        param_name: &str,
        value: ParameterValue,
    ) -> Result<Response, ConversationError> {
        let context = self.get_context_mut(conversation_id)?;

        // 保存参数
        context.parameters.insert(param_name.to_string(), value);
        context.mark_parameter_collected(param_name);

        // 检查是否还有待收集的参数
        let next_param_info = context.next_pending_parameter().map(|p| (
            p.name.clone(),
            p.description.clone(),
            p.hint.clone(),
            p.default.clone(),
        ));

        if let Some((name, description, hint, default)) = next_param_info {
            // 转换状态
            context.state.transition(StateEvent::ParameterProvided {
                name: name.clone(),
            }).map_err(|e| ConversationError::StateTransition(e.to_string()))?;

            Ok(Response::AskForParameter {
                name,
                description,
                hint,
                default,
            })
        } else {
            // 所有参数已收集
            context.state.transition(StateEvent::AllParametersCollected)
                .map_err(|e| ConversationError::StateTransition(e.to_string()))?;

            Ok(Response::AllParametersCollected)
        }
    }

    /// 确认执行
    pub fn confirm_execution(
        &mut self,
        conversation_id: &str,
        confirmed: bool,
    ) -> Result<Response, ConversationError> {
        let context = self.get_context_mut(conversation_id)?;

        if confirmed {
            context.state.transition(StateEvent::UserConfirmed)
                .map_err(|e| ConversationError::StateTransition(e.to_string()))?;
            Ok(Response::ReadyToExecute)
        } else {
            context.state.transition(StateEvent::UserRejected)
                .map_err(|e| ConversationError::StateTransition(e.to_string()))?;
            Ok(Response::Cancelled)
        }
    }

    /// 完成执行
    pub fn complete_execution(
        &mut self,
        conversation_id: &str,
        success: bool,
        message: String,
    ) -> Result<Response, ConversationError> {
        let context = self.get_context_mut(conversation_id)?;

        context.state.transition(StateEvent::ExecutionCompleted {
            success,
            message: message.clone(),
        }).map_err(|e| ConversationError::StateTransition(e.to_string()))?;

        Ok(Response::ExecutionResult { success, output: message })
    }

    /// 取消对话
    pub fn cancel_conversation(
        &mut self,
        conversation_id: &str,
        reason: impl Into<String>,
    ) -> Result<(), ConversationError> {
        let context = self.get_context_mut(conversation_id)?;

        context.state.transition(StateEvent::UserCancelled {
            reason: reason.into(),
        }).map_err(|e| ConversationError::StateTransition(e.to_string()))?;

        Ok(())
    }

    /// 检查并处理超时对话
    pub fn check_timeouts(&mut self) -> Vec<String> {
        let mut timeout_ids = Vec::new();

        for (id, context) in &mut self.active_conversations {
            if context.is_timeout() && !context.state.is_terminal() {
                let _ = context.state.transition(StateEvent::Timeout);
                timeout_ids.push(id.clone());
            }
        }

        timeout_ids
    }

    /// 清理已完成的对话
    pub fn cleanup_completed(&mut self) {
        self.active_conversations.retain(|_, ctx| !ctx.state.is_terminal());
    }

    /// 获取活跃对话数量
    pub fn active_count(&self) -> usize {
        self.active_conversations
            .values()
            .filter(|ctx| !ctx.state.is_terminal())
            .count()
    }

    /// 使用 LLM 智能提取参数
    ///
    /// 从用户输入中自动提取参数值
    pub async fn extract_parameters_with_llm(
        &mut self,
        conversation_id: &str,
        user_input: &str,
        llm: &dyn LlmClient,
    ) -> Result<Vec<(String, ParameterValue)>, ConversationError> {
        let context = self.get_context(conversation_id)?;
        let specs = context.pending_parameters.clone();

        if specs.is_empty() {
            return Ok(Vec::new());
        }

        self.analyzer
            .extract_parameters(user_input, &specs, llm)
            .await
            .map_err(|e| ConversationError::ParameterError(e))
    }

    /// 生成智能提问
    ///
    /// 使用 LLM 为下一个待收集的参数生成自然的提问
    pub async fn generate_smart_question(
        &self,
        conversation_id: &str,
        llm: &dyn LlmClient,
    ) -> Result<String, ConversationError> {
        let context = self.get_context(conversation_id)?;

        let next_param = context
            .next_pending_parameter()
            .ok_or(ConversationError::ParameterError(
                "没有待收集的参数".to_string(),
            ))?;

        let context_str = format!("用户意图：{}", context.intent);

        self.analyzer
            .generate_question(&next_param, &context_str, llm)
            .await
            .map_err(|e| ConversationError::ParameterError(e))
    }

    /// 智能收集参数
    ///
    /// 结合 LLM 分析和验证的参数收集
    pub async fn collect_parameter_smart(
        &mut self,
        conversation_id: &str,
        param_name: &str,
        value: ParameterValue,
        llm: &dyn LlmClient,
    ) -> Result<Response, ConversationError> {
        // 验证参数
        let context = self.get_context(conversation_id)?;
        if let Some(spec) = context
            .pending_parameters
            .iter()
            .find(|s| s.name == param_name)
        {
            self.analyzer
                .validate_parameter(spec, &value)
                .map_err(|e| ConversationError::ParameterError(e))?;
        }

        // 收集参数
        let result = self.collect_parameter(conversation_id, param_name, value)?;

        // 如果还有待收集的参数，生成智能提问
        if matches!(result, Response::AskForParameter { .. }) {
            if let Ok(smart_question) = self.generate_smart_question(conversation_id, llm).await {
                // 获取当前参数的其他信息
                let context = self.get_context(conversation_id)?;
                if let Some(next_param) = context.next_pending_parameter() {
                    return Ok(Response::AskForParameter {
                        name: next_param.name.clone(),
                        description: smart_question, // 使用 LLM 生成的智能提问
                        hint: next_param.hint.clone(),
                        default: next_param.default.clone(),
                    });
                }
            }
        }

        Ok(result)
    }

    /// 检测缺失的参数
    ///
    /// 返回当前对话中还未收集的必需参数
    pub fn detect_missing_parameters(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<ParameterSpec>, ConversationError> {
        let context = self.get_context(conversation_id)?;
        let specs = context.pending_parameters.clone();
        let collected = context.parameters.clone();

        Ok(self.analyzer.detect_missing_parameters(&specs, &collected))
    }
}

/// 响应类型
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// 询问参数
    AskForParameter {
        name: String,
        description: String,
        hint: Option<String>,
        default: Option<ParameterValue>,
    },

    /// 所有参数已收集，准备验证
    AllParametersCollected,

    /// 准备执行
    ReadyToExecute,

    /// 执行结果
    ExecutionResult {
        success: bool,
        output: String,
    },

    /// 已取消
    Cancelled,
}

/// 对话错误
#[derive(Debug, Clone)]
pub enum ConversationError {
    /// 对话不存在
    NotFound,

    /// 状态转换错误
    StateTransition(String),

    /// 参数错误
    ParameterError(String),

    /// 超时
    Timeout,
}

impl fmt::Display for ConversationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "对话不存在"),
            Self::StateTransition(msg) => write!(f, "状态转换错误: {}", msg),
            Self::ParameterError(msg) => write!(f, "参数错误: {}", msg),
            Self::Timeout => write!(f, "对话超时"),
        }
    }
}

impl std::error::Error for ConversationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::context::{ParameterType, ParameterValue};
    use super::super::state::ConversationState;

    #[test]
    fn test_start_conversation() {
        let mut manager = ConversationManager::new(300);

        let id = manager.start_conversation("test_intent").unwrap();

        assert!(!id.is_empty());
        assert_eq!(manager.active_count(), 1);

        let context = manager.get_context(&id).unwrap();
        assert_eq!(context.intent, "test_intent");
        assert!(matches!(
            context.state,
            ConversationState::CollectingParameters { .. }
        ));
    }

    #[test]
    fn test_parameter_collection() {
        let mut manager = ConversationManager::new(300);

        let id = manager.start_conversation("test").unwrap();

        // 添加参数规格
        manager.add_parameter_spec(
            &id,
            ParameterSpec::new("param1", ParameterType::String, "First parameter")
        ).unwrap();

        manager.add_parameter_spec(
            &id,
            ParameterSpec::new("param2", ParameterType::String, "Second parameter")
        ).unwrap();

        // 收集第一个参数
        let response = manager.collect_parameter(
            &id,
            "param1",
            ParameterValue::String("value1".to_string())
        ).unwrap();

        assert!(matches!(response, Response::AskForParameter { .. }));

        // 收集第二个参数
        let response = manager.collect_parameter(
            &id,
            "param2",
            ParameterValue::String("value2".to_string())
        ).unwrap();

        assert_eq!(response, Response::AllParametersCollected);

        // 验证参数已保存
        let context = manager.get_context(&id).unwrap();
        assert_eq!(context.parameters.len(), 2);
        assert!(context.pending_parameters.is_empty());
    }

    #[test]
    fn test_confirmation_flow() {
        let mut manager = ConversationManager::new(300);

        let id = manager.start_conversation("test").unwrap();

        // 设置为确认状态
        {
            let context = manager.get_context_mut(&id).unwrap();
            context.state = ConversationState::Confirming;
        }

        // 用户确认
        let response = manager.confirm_execution(&id, true).unwrap();
        assert_eq!(response, Response::ReadyToExecute);

        let context = manager.get_context(&id).unwrap();
        assert_eq!(context.state, ConversationState::Executing);
    }

    #[test]
    fn test_cancel_conversation() {
        let mut manager = ConversationManager::new(300);

        let id = manager.start_conversation("test").unwrap();

        manager.cancel_conversation(&id, "User requested").unwrap();

        let context = manager.get_context(&id).unwrap();
        assert!(matches!(context.state, ConversationState::Cancelled { .. }));
        assert!(context.state.is_terminal());
    }

    #[test]
    fn test_cleanup_completed() {
        let mut manager = ConversationManager::new(300);

        // 创建几个对话
        let id1 = manager.start_conversation("test1").unwrap();
        let id2 = manager.start_conversation("test2").unwrap();
        let id3 = manager.start_conversation("test3").unwrap();

        // 完成其中两个
        {
            let ctx = manager.get_context_mut(&id1).unwrap();
            ctx.state = ConversationState::Completed {
                success: true,
                message: "done".to_string(),
            };
        }
        {
            let ctx = manager.get_context_mut(&id2).unwrap();
            ctx.state = ConversationState::Cancelled {
                reason: "test".to_string(),
            };
        }

        assert_eq!(manager.active_conversations.len(), 3);
        assert_eq!(manager.active_count(), 1);

        manager.cleanup_completed();

        assert_eq!(manager.active_conversations.len(), 1);
        assert!(manager.get_context(&id3).is_ok());
        assert!(manager.get_context(&id1).is_err());
        assert!(manager.get_context(&id2).is_err());
    }

    #[test]
    fn test_timeout_detection() {
        let mut manager = ConversationManager::new(1); // 1秒超时

        let id = manager.start_conversation("test").unwrap();

        // 修改 last_active 模拟超时
        {
            let context = manager.get_context_mut(&id).unwrap();
            context.last_active = chrono::Utc::now() - chrono::Duration::seconds(2);
        }

        let timeout_ids = manager.check_timeouts();

        assert_eq!(timeout_ids.len(), 1);
        assert_eq!(timeout_ids[0], id);

        let context = manager.get_context(&id).unwrap();
        assert_eq!(context.state, ConversationState::Timeout);
    }
}
