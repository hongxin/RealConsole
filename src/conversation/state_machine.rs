//! 会话状态机
//!
//! 管理对话流程的状态转换

use serde::{Deserialize, Serialize};

/// 会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationState {
    /// 空闲状态（无活跃任务）
    Idle,

    /// 理解意图中
    UnderstandingIntent,

    /// 等待参数（需要用户补充信息）
    AwaitingParameters,

    /// 准备执行（所有参数已收集）
    ReadyToExecute,

    /// 执行中
    Executing,

    /// 完成
    Completed,

    /// 失败
    Failed,
}

impl std::fmt::Display for ConversationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversationState::Idle => write!(f, "空闲"),
            ConversationState::UnderstandingIntent => write!(f, "理解意图中"),
            ConversationState::AwaitingParameters => write!(f, "等待参数"),
            ConversationState::ReadyToExecute => write!(f, "准备执行"),
            ConversationState::Executing => write!(f, "执行中"),
            ConversationState::Completed => write!(f, "已完成"),
            ConversationState::Failed => write!(f, "失败"),
        }
    }
}

impl ConversationState {
    /// 是否可以接收新输入
    pub fn can_accept_input(&self) -> bool {
        matches!(
            self,
            ConversationState::Idle
                | ConversationState::AwaitingParameters
                | ConversationState::Completed
                | ConversationState::Failed
        )
    }

    /// 是否是终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ConversationState::Completed | ConversationState::Failed
        )
    }

    /// 是否正在执行
    pub fn is_executing(&self) -> bool {
        matches!(self, ConversationState::Executing)
    }
}

/// 状态转换
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// 源状态
    pub from: ConversationState,

    /// 目标状态
    pub to: ConversationState,

    /// 转换原因
    pub reason: String,
}

impl StateTransition {
    /// 创建新的状态转换
    pub fn new(from: ConversationState, to: ConversationState, reason: String) -> Self {
        Self { from, to, reason }
    }

    /// 验证转换是否合法
    pub fn is_valid(&self) -> bool {
        match (self.from, self.to) {
            // 空闲 -> 理解意图
            (ConversationState::Idle, ConversationState::UnderstandingIntent) => true,

            // 理解意图 -> 等待参数
            (ConversationState::UnderstandingIntent, ConversationState::AwaitingParameters) => {
                true
            }

            // 理解意图 -> 准备执行（无需补充参数）
            (ConversationState::UnderstandingIntent, ConversationState::ReadyToExecute) => true,

            // 理解意图 -> 执行中（直接执行）
            (ConversationState::UnderstandingIntent, ConversationState::Executing) => true,

            // 等待参数 -> 准备执行（参数收集完成）
            (ConversationState::AwaitingParameters, ConversationState::ReadyToExecute) => true,

            // 准备执行 -> 执行中
            (ConversationState::ReadyToExecute, ConversationState::Executing) => true,

            // 执行中 -> 完成
            (ConversationState::Executing, ConversationState::Completed) => true,

            // 执行中 -> 失败
            (ConversationState::Executing, ConversationState::Failed) => true,

            // 完成/失败 -> 空闲（开始新任务）
            (ConversationState::Completed, ConversationState::Idle) => true,
            (ConversationState::Failed, ConversationState::Idle) => true,

            // 任何状态 -> 失败（出错）
            (_, ConversationState::Failed) => true,

            // 其他转换不合法
            _ => false,
        }
    }
}

/// 状态机
pub struct StateMachine {
    /// 当前状态
    current_state: ConversationState,

    /// 状态历史
    history: Vec<StateTransition>,
}

impl StateMachine {
    /// 创建新的状态机
    pub fn new() -> Self {
        Self {
            current_state: ConversationState::Idle,
            history: Vec::new(),
        }
    }

    /// 获取当前状态
    pub fn current_state(&self) -> ConversationState {
        self.current_state
    }

    /// 转换状态
    pub fn transition(&mut self, to: ConversationState, reason: String) -> Result<(), String> {
        let transition = StateTransition::new(self.current_state, to, reason);

        if !transition.is_valid() {
            return Err(format!(
                "非法的状态转换：{} -> {}",
                self.current_state, to
            ));
        }

        self.history.push(transition);
        self.current_state = to;

        Ok(())
    }

    /// 重置状态机
    pub fn reset(&mut self) {
        self.current_state = ConversationState::Idle;
        self.history.clear();
    }

    /// 获取状态历史
    pub fn history(&self) -> &[StateTransition] {
        &self.history
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let sm = StateMachine::new();
        assert_eq!(sm.current_state(), ConversationState::Idle);
    }

    #[test]
    fn test_valid_transitions() {
        let mut sm = StateMachine::new();

        // 空闲 -> 理解意图
        assert!(sm
            .transition(
                ConversationState::UnderstandingIntent,
                "开始理解".to_string()
            )
            .is_ok());

        // 理解意图 -> 等待参数
        assert!(sm
            .transition(
                ConversationState::AwaitingParameters,
                "需要补充参数".to_string()
            )
            .is_ok());

        // 等待参数 -> 准备执行
        assert!(sm
            .transition(ConversationState::ReadyToExecute, "参数齐全".to_string())
            .is_ok());

        // 准备执行 -> 执行中
        assert!(sm
            .transition(ConversationState::Executing, "开始执行".to_string())
            .is_ok());

        // 执行中 -> 完成
        assert!(sm
            .transition(ConversationState::Completed, "执行成功".to_string())
            .is_ok());
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = StateMachine::new();

        // 空闲 -> 执行中（不合法）
        assert!(sm
            .transition(ConversationState::Executing, "直接执行".to_string())
            .is_err());
    }

    #[test]
    fn test_state_helpers() {
        assert!(ConversationState::Idle.can_accept_input());
        assert!(!ConversationState::Executing.can_accept_input());

        assert!(ConversationState::Completed.is_terminal());
        assert!(!ConversationState::Idle.is_terminal());

        assert!(ConversationState::Executing.is_executing());
        assert!(!ConversationState::Idle.is_executing());
    }

    #[test]
    fn test_state_machine_reset() {
        let mut sm = StateMachine::new();

        sm.transition(
            ConversationState::UnderstandingIntent,
            "测试".to_string(),
        )
        .unwrap();

        assert_ne!(sm.current_state(), ConversationState::Idle);

        sm.reset();
        assert_eq!(sm.current_state(), ConversationState::Idle);
        assert_eq!(sm.history().len(), 0);
    }
}
