//! 对话状态机
//!
//! 定义对话的各种状态和状态转换规则

use std::fmt;

/// 对话状态
///
/// 遵循"一分为三"原则：初始 - 执行 - 完成，中间穿插验证和确认
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationState {
    /// 初始化：理解用户意图
    Initializing,

    /// 收集参数：询问缺失的参数
    CollectingParameters {
        current_param: String,
        retry_count: u8,
    },

    /// 验证中：验证所有参数
    Validating,

    /// 确认中：等待用户确认执行
    Confirming,

    /// 执行中：执行任务
    Executing,

    /// 完成：任务完成
    Completed {
        success: bool,
        message: String,
    },

    /// 已取消：用户取消操作
    Cancelled {
        reason: String,
    },

    /// 超时：对话超时未完成
    Timeout,
}

/// 状态事件
///
/// 驱动状态转换的事件
#[derive(Debug, Clone)]
pub enum StateEvent {
    /// 意图已识别
    IntentRecognized,

    /// 参数已提供
    ParameterProvided { name: String },

    /// 所有参数已收集
    AllParametersCollected,

    /// 验证通过
    ValidationPassed,

    /// 验证失败
    ValidationFailed { reason: String },

    /// 用户确认
    UserConfirmed,

    /// 用户拒绝
    UserRejected,

    /// 执行完成
    ExecutionCompleted { success: bool, message: String },

    /// 用户取消
    UserCancelled { reason: String },

    /// 超时
    Timeout,
}

impl ConversationState {
    /// 状态转换
    ///
    /// 根据事件转换到下一个状态，返回是否成功
    pub fn transition(&mut self, event: StateEvent) -> Result<(), StateTransitionError> {
        let old_state = self.clone();

        match (self.clone(), event.clone()) {
            // 初始化 -> 收集参数
            (Self::Initializing, StateEvent::IntentRecognized) => {
                *self = Self::CollectingParameters {
                    current_param: String::new(),
                    retry_count: 0,
                };
                Ok(())
            }

            // 初始化 -> 验证（没有参数需要收集）
            (Self::Initializing, StateEvent::AllParametersCollected) => {
                *self = Self::Validating;
                Ok(())
            }

            // 收集参数 -> 继续收集下一个
            (Self::CollectingParameters { retry_count, .. }, StateEvent::ParameterProvided { name }) => {
                *self = Self::CollectingParameters {
                    current_param: name,
                    retry_count,
                };
                Ok(())
            }

            // 收集参数 -> 验证（所有参数已收集）
            (Self::CollectingParameters { .. }, StateEvent::AllParametersCollected) => {
                *self = Self::Validating;
                Ok(())
            }

            // 收集参数 -> 重试（验证失败）
            (Self::CollectingParameters { current_param, retry_count }, StateEvent::ValidationFailed { reason: _ }) => {
                if retry_count >= 3 {
                    *self = Self::Cancelled {
                        reason: "Too many invalid attempts".to_string(),
                    };
                } else {
                    *self = Self::CollectingParameters {
                        current_param,
                        retry_count: retry_count + 1,
                    };
                }
                Ok(())
            }

            // 验证 -> 确认
            (Self::Validating, StateEvent::ValidationPassed) => {
                *self = Self::Confirming;
                Ok(())
            }

            // 验证 -> 收集参数（验证失败，需要重新收集）
            (Self::Validating, StateEvent::ValidationFailed { reason: _ }) => {
                *self = Self::CollectingParameters {
                    current_param: String::new(),
                    retry_count: 0,
                };
                Ok(())
            }

            // 确认 -> 执行
            (Self::Confirming, StateEvent::UserConfirmed) => {
                *self = Self::Executing;
                Ok(())
            }

            // 确认 -> 取消
            (Self::Confirming, StateEvent::UserRejected) => {
                *self = Self::Cancelled {
                    reason: "User rejected".to_string(),
                };
                Ok(())
            }

            // 执行 -> 完成
            (Self::Executing, StateEvent::ExecutionCompleted { success, message }) => {
                *self = Self::Completed { success, message };
                Ok(())
            }

            // 任何状态 -> 取消
            (_, StateEvent::UserCancelled { reason }) => {
                *self = Self::Cancelled { reason };
                Ok(())
            }

            // 任何状态 -> 超时
            (_, StateEvent::Timeout) => {
                *self = Self::Timeout;
                Ok(())
            }

            // 无效转换
            _ => Err(StateTransitionError {
                from: old_state,
                event,
                reason: "Invalid state transition".to_string(),
            }),
        }
    }

    /// 检查是否为终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Cancelled { .. } | Self::Timeout
        )
    }

    /// 获取状态名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Initializing => "初始化",
            Self::CollectingParameters { .. } => "收集参数",
            Self::Validating => "验证中",
            Self::Confirming => "确认中",
            Self::Executing => "执行中",
            Self::Completed { .. } => "已完成",
            Self::Cancelled { .. } => "已取消",
            Self::Timeout => "已超时",
        }
    }
}

impl fmt::Display for ConversationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 状态转换错误
#[derive(Debug, Clone)]
pub struct StateTransitionError {
    pub from: ConversationState,
    pub event: StateEvent,
    pub reason: String,
}

impl fmt::Display for StateTransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "无法从状态 {} 通过事件 {:?} 转换: {}",
            self.from, self.event, self.reason
        )
    }
}

impl std::error::Error for StateTransitionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_to_collecting() {
        let mut state = ConversationState::Initializing;

        assert!(state.transition(StateEvent::IntentRecognized).is_ok());

        assert!(matches!(
            state,
            ConversationState::CollectingParameters { .. }
        ));
    }

    #[test]
    fn test_collecting_to_validating() {
        let mut state = ConversationState::CollectingParameters {
            current_param: "test".to_string(),
            retry_count: 0,
        };

        assert!(state.transition(StateEvent::AllParametersCollected).is_ok());

        assert_eq!(state, ConversationState::Validating);
    }

    #[test]
    fn test_validating_to_confirming() {
        let mut state = ConversationState::Validating;

        assert!(state.transition(StateEvent::ValidationPassed).is_ok());

        assert_eq!(state, ConversationState::Confirming);
    }

    #[test]
    fn test_confirming_to_executing() {
        let mut state = ConversationState::Confirming;

        assert!(state.transition(StateEvent::UserConfirmed).is_ok());

        assert_eq!(state, ConversationState::Executing);
    }

    #[test]
    fn test_executing_to_completed() {
        let mut state = ConversationState::Executing;

        assert!(state
            .transition(StateEvent::ExecutionCompleted {
                success: true,
                message: "Done".to_string()
            })
            .is_ok());

        assert!(matches!(
            state,
            ConversationState::Completed {
                success: true,
                message: _
            }
        ));
    }

    #[test]
    fn test_user_cancel_from_any_state() {
        let states = vec![
            ConversationState::Initializing,
            ConversationState::CollectingParameters {
                current_param: "test".to_string(),
                retry_count: 0,
            },
            ConversationState::Validating,
            ConversationState::Confirming,
            ConversationState::Executing,
        ];

        for mut state in states {
            assert!(state
                .transition(StateEvent::UserCancelled {
                    reason: "User requested".to_string()
                })
                .is_ok());

            assert!(matches!(state, ConversationState::Cancelled { .. }));
        }
    }

    #[test]
    fn test_timeout_from_any_state() {
        let mut state = ConversationState::CollectingParameters {
            current_param: "test".to_string(),
            retry_count: 0,
        };

        assert!(state.transition(StateEvent::Timeout).is_ok());

        assert_eq!(state, ConversationState::Timeout);
    }

    #[test]
    fn test_invalid_transition() {
        let mut state = ConversationState::Executing;

        // 不能从执行状态直接返回到收集参数
        let result = state.transition(StateEvent::IntentRecognized);

        assert!(result.is_err());
    }

    #[test]
    fn test_retry_limit() {
        let mut state = ConversationState::CollectingParameters {
            current_param: "test".to_string(),
            retry_count: 3,
        };

        assert!(state
            .transition(StateEvent::ValidationFailed {
                reason: "Invalid input".to_string()
            })
            .is_ok());

        // 超过重试次数应该转到取消状态
        assert!(matches!(state, ConversationState::Cancelled { .. }));
    }

    #[test]
    fn test_is_terminal() {
        assert!(ConversationState::Completed {
            success: true,
            message: "".to_string()
        }
        .is_terminal());

        assert!(ConversationState::Cancelled {
            reason: "".to_string()
        }
        .is_terminal());

        assert!(ConversationState::Timeout.is_terminal());

        assert!(!ConversationState::Initializing.is_terminal());
        assert!(!ConversationState::Executing.is_terminal());
    }

    #[test]
    fn test_state_names() {
        assert_eq!(ConversationState::Initializing.name(), "初始化");
        assert_eq!(ConversationState::Validating.name(), "验证中");
        assert_eq!(ConversationState::Confirming.name(), "确认中");
        assert_eq!(ConversationState::Executing.name(), "执行中");
    }
}
