//! 多轮对话与智能参数补全
//!
//! 核心功能：
//! - 会话上下文管理
//! - 参数缺失检测
//! - 智能提问生成
//! - 跨轮任务追踪

pub mod analyzer;
pub mod context;
pub mod current;
pub mod manager;
pub mod parameter;
pub mod state;

// 导出核心类型
pub use context::ParameterSpec;
pub use current::{clear_current_conversation, get_current_conversation, has_active_conversation, set_current_conversation};
pub use manager::{ConversationManager, Response};
pub use parameter::{ParameterType, ParameterValue};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 对话轮次（预留给未来的轮次追踪功能）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// 轮次 ID
    pub id: Uuid,

    /// 用户输入
    pub user_input: String,

    /// 助手回复
    pub assistant_response: String,

    /// 调用的工具
    pub tools_called: Vec<ToolCall>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 是否成功
    pub success: bool,
}

impl Turn {
    /// 创建新的对话轮次
    #[allow(dead_code)]
    pub fn new(user_input: String, assistant_response: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_input,
            assistant_response,
            tools_called: Vec::new(),
            timestamp: Utc::now(),
            success: true,
        }
    }

    /// 添加工具调用记录
    #[allow(dead_code)]
    pub fn add_tool_call(&mut self, tool_call: ToolCall) {
        self.tools_called.push(tool_call);
    }
}

/// 工具调用记录（预留给未来功能）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 工具名称
    pub tool_name: String,

    /// 参数
    pub parameters: HashMap<String, serde_json::Value>,

    /// 结果
    pub result: Option<String>,

    /// 是否成功
    pub success: bool,
}

/// 任务定义（预留给复杂任务跟踪功能）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: Uuid,

    /// 任务类型
    pub task_type: TaskType,

    /// 任务描述
    pub description: String,

    /// 必需参数
    pub required_params: Vec<String>,

    /// 可选参数
    pub optional_params: Vec<String>,

    /// 已收集参数
    pub collected_params: HashMap<String, ParameterValue>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Task {
    /// 创建新任务
    #[allow(dead_code)]
    pub fn new(task_type: TaskType, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type,
            description,
            required_params: Vec::new(),
            optional_params: Vec::new(),
            collected_params: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// 添加必需参数
    #[allow(dead_code)]
    pub fn add_required_param(&mut self, param: String) {
        self.required_params.push(param);
    }

    /// 添加可选参数
    #[allow(dead_code)]
    pub fn add_optional_param(&mut self, param: String) {
        self.optional_params.push(param);
    }

    /// 收集参数
    #[allow(dead_code)]
    pub fn collect_param(&mut self, name: String, value: ParameterValue) {
        self.collected_params.insert(name, value);
    }

    /// 检查是否所有必需参数都已收集
    #[allow(dead_code)]
    pub fn has_all_required_params(&self) -> bool {
        self.required_params.iter().all(|param| {
            self.collected_params.contains_key(param)
        })
    }

    /// 获取缺失的必需参数
    #[allow(dead_code)]
    pub fn missing_required_params(&self) -> Vec<String> {
        self.required_params
            .iter()
            .filter(|param| !self.collected_params.contains_key(*param))
            .cloned()
            .collect()
    }

    /// 获取缺失的可选参数
    #[allow(dead_code)]
    pub fn missing_optional_params(&self) -> Vec<String> {
        self.optional_params
            .iter()
            .filter(|param| !self.collected_params.contains_key(*param))
            .cloned()
            .collect()
    }
}

/// 任务类型
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// 文件操作
    FileOperation,

    /// 日志分析
    LogAnalysis,

    /// 系统监控
    SystemMonitor,

    /// Git 操作
    GitOperation,

    /// Shell 命令
    ShellCommand,

    /// LLM 对话
    LlmChat,

    /// 其他
    Other(String),
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::FileOperation => write!(f, "文件操作"),
            TaskType::LogAnalysis => write!(f, "日志分析"),
            TaskType::SystemMonitor => write!(f, "系统监控"),
            TaskType::GitOperation => write!(f, "Git 操作"),
            TaskType::ShellCommand => write!(f, "Shell 命令"),
            TaskType::LlmChat => write!(f, "LLM 对话"),
            TaskType::Other(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_creation() {
        let turn = Turn::new(
            "显示最大的文件".to_string(),
            "在哪个目录？".to_string(),
        );

        assert_eq!(turn.user_input, "显示最大的文件");
        assert_eq!(turn.assistant_response, "在哪个目录？");
        assert!(turn.success);
        assert_eq!(turn.tools_called.len(), 0);
    }

    #[test]
    fn test_task_parameter_tracking() {
        let mut task = Task::new(
            TaskType::FileOperation,
            "查找最大文件".to_string(),
        );

        task.add_required_param("directory".to_string());
        task.add_required_param("count".to_string());
        task.add_optional_param("extension".to_string());

        // 初始状态：所有必需参数都缺失
        assert!(!task.has_all_required_params());
        assert_eq!(task.missing_required_params().len(), 2);

        // 收集第一个参数
        task.collect_param(
            "directory".to_string(),
            ParameterValue::String("/home".to_string()),
        );

        assert!(!task.has_all_required_params());
        assert_eq!(task.missing_required_params().len(), 1);

        // 收集第二个参数
        task.collect_param(
            "count".to_string(),
            ParameterValue::Integer(5),
        );

        assert!(task.has_all_required_params());
        assert_eq!(task.missing_required_params().len(), 0);
        assert_eq!(task.missing_optional_params().len(), 1);
    }
}
