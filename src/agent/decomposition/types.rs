//! 意图拆解类型定义

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// 计划 ID
    pub id: String,

    /// 对用户意图的理解
    pub understanding: String,

    /// 执行步骤列表
    pub steps: Vec<ExecutionStep>,

    /// 总预计执行时间（秒）
    pub total_estimated_time: f64,

    /// 创建时间戳
    pub created_at: String,
}

impl ExecutionPlan {
    /// 创建新的执行计划
    pub fn new(understanding: String, steps: Vec<ExecutionStep>) -> Self {
        let total_estimated_time = steps.iter().map(|s| s.estimated_time).sum();

        Self {
            id: format!("plan-{}", Uuid::new_v4()),
            understanding,
            steps,
            total_estimated_time,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 获取步骤数量
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// 检查是否所有步骤都成功
    pub fn is_all_success(&self) -> bool {
        self.steps.iter().all(|s| s.status == StepStatus::Success)
    }

    /// 获取失败的步骤数量
    pub fn failed_count(&self) -> usize {
        self.steps.iter().filter(|s| s.status == StepStatus::Failed).count()
    }
}

/// 执行步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// 步骤 ID
    pub id: String,

    /// 步骤描述
    pub description: String,

    /// 使用的工具
    pub tool: String,

    /// 工具参数（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,

    /// 预计执行时间（秒）
    pub estimated_time: f64,

    /// 执行状态
    #[serde(default)]
    pub status: StepStatus,

    /// 实际执行时间（秒，执行后填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_time: Option<f64>,

    /// 错误信息（如果失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ExecutionStep {
    /// 创建新的执行步骤
    pub fn new(description: String, tool: String, estimated_time: f64) -> Self {
        Self {
            id: format!("step-{}", Uuid::new_v4()),
            description,
            tool,
            params: None,
            estimated_time,
            status: StepStatus::Pending,
            actual_time: None,
            error: None,
        }
    }

    /// 设置参数
    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }

    /// 标记为运行中
    pub fn mark_running(&mut self) {
        self.status = StepStatus::Running;
    }

    /// 标记为成功
    pub fn mark_success(&mut self, actual_time: f64) {
        self.status = StepStatus::Success;
        self.actual_time = Some(actual_time);
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error: String, actual_time: f64) {
        self.status = StepStatus::Failed;
        self.error = Some(error);
        self.actual_time = Some(actual_time);
    }
}

/// 步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// 等待执行
    #[default]
    Pending,
    /// 执行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 跳过
    Skipped,
}

/// 步骤执行进度
#[derive(Debug, Clone, Serialize)]
pub struct StepProgress {
    /// 步骤索引（从 0 开始）
    pub step_index: usize,

    /// 总步骤数
    pub total_steps: usize,

    /// 步骤 ID
    pub step_id: String,

    /// 步骤描述
    pub description: String,

    /// 执行状态
    pub status: StepStatus,

    /// 已用时间（秒）
    pub elapsed_time: Option<f64>,
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 执行的计划
    pub plan: ExecutionPlan,

    /// 每个步骤的输出（字符串形式）
    pub outputs: Vec<String>,

    /// 总执行时间（秒）
    pub total_time: f64,
}

impl ExecutionResult {
    /// 创建新的执行结果
    pub fn new(plan: ExecutionPlan, outputs: Vec<String>, total_time: f64) -> Self {
        Self {
            plan,
            outputs,
            total_time,
        }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.plan.is_all_success()
    }

    /// 获取失败信息
    pub fn get_errors(&self) -> Vec<(usize, &str)> {
        self.plan
            .steps
            .iter()
            .enumerate()
            .filter_map(|(i, step)| {
                if let Some(error) = &step.error {
                    Some((i, error.as_str()))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_plan_creation() {
        let steps = vec![
            ExecutionStep::new("步骤 1".to_string(), "tool1".to_string(), 1.0),
            ExecutionStep::new("步骤 2".to_string(), "tool2".to_string(), 2.0),
        ];

        let plan = ExecutionPlan::new("测试计划".to_string(), steps);

        assert_eq!(plan.step_count(), 2);
        assert_eq!(plan.total_estimated_time, 3.0);
        assert!(!plan.is_all_success()); // 初始状态为 Pending
    }

    #[test]
    fn test_step_status_transitions() {
        let mut step = ExecutionStep::new("测试步骤".to_string(), "test_tool".to_string(), 1.0);

        assert_eq!(step.status, StepStatus::Pending);

        step.mark_running();
        assert_eq!(step.status, StepStatus::Running);

        step.mark_success(1.5);
        assert_eq!(step.status, StepStatus::Success);
        assert_eq!(step.actual_time, Some(1.5));
    }

    #[test]
    fn test_execution_result() {
        let mut steps = vec![
            ExecutionStep::new("步骤 1".to_string(), "tool1".to_string(), 1.0),
            ExecutionStep::new("步骤 2".to_string(), "tool2".to_string(), 2.0),
        ];

        steps[0].mark_success(1.0);
        steps[1].mark_failed("测试错误".to_string(), 2.0);

        let plan = ExecutionPlan::new("测试计划".to_string(), steps);
        let result = ExecutionResult::new(plan, vec!["输出1".to_string(), "输出2".to_string()], 3.0);

        assert!(!result.is_success());
        assert_eq!(result.get_errors().len(), 1);
    }
}
