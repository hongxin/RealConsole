//! 计划执行器

use super::types::{ExecutionPlan, ExecutionResult, StepProgress, StepStatus};
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

/// 进度回调函数类型
pub type ProgressCallback = Arc<dyn Fn(StepProgress) + Send + Sync>;

/// 计划执行器
///
/// 顺序执行拆解后的步骤计划，并提供进度回调
pub struct PlanExecutor {
    /// 进度回调（可选）
    progress_callback: Option<ProgressCallback>,
}

impl PlanExecutor {
    /// 创建新的执行器
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    /// 设置进度回调
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// 执行计划
    ///
    /// # 参数
    ///
    /// - `plan`: 待执行的计划
    ///
    /// # 返回
    ///
    /// 执行结果，包含更新后的计划和每步输出
    pub async fn execute(&self, mut plan: ExecutionPlan) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        let mut outputs = Vec::new();
        let total_steps = plan.steps.len();  // 在循环前保存

        for (index, step) in plan.steps.iter_mut().enumerate() {
            // 1. 通知开始执行
            step.mark_running();
            self.notify_progress(StepProgress {
                step_index: index,
                total_steps,
                step_id: step.id.clone(),
                description: step.description.clone(),
                status: StepStatus::Running,
                elapsed_time: None,
            });

            // 2. 执行步骤
            let step_start = Instant::now();
            match self.execute_step(&step.tool, &step.description).await {
                Ok(output) => {
                    let elapsed = step_start.elapsed().as_secs_f64();
                    step.mark_success(elapsed);
                    outputs.push(output);

                    // 通知成功
                    self.notify_progress(StepProgress {
                        step_index: index,
                        total_steps,
                        step_id: step.id.clone(),
                        description: step.description.clone(),
                        status: StepStatus::Success,
                        elapsed_time: Some(elapsed),
                    });
                }
                Err(e) => {
                    let elapsed = step_start.elapsed().as_secs_f64();
                    step.mark_failed(e.to_string(), elapsed);

                    // 通知失败
                    self.notify_progress(StepProgress {
                        step_index: index,
                        total_steps,
                        step_id: step.id.clone(),
                        description: step.description.clone(),
                        status: StepStatus::Failed,
                        elapsed_time: Some(elapsed),
                    });

                    // 失败后停止执行
                    return Err(e);
                }
            }
        }

        Ok(ExecutionResult::new(
            plan,
            outputs,
            start_time.elapsed().as_secs_f64(),
        ))
    }

    /// 执行单个步骤（占位符实现）
    ///
    /// TODO: 集成 ToolExecutor 进行真实执行
    async fn execute_step(&self, tool: &str, description: &str) -> Result<String> {
        // 占位符：模拟执行
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(format!("执行 {} ({}): 成功", tool, description))
    }

    /// 通知进度更新
    fn notify_progress(&self, progress: StepProgress) {
        if let Some(callback) = &self.progress_callback {
            callback(progress);
        }
    }
}

impl Default for PlanExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::decomposition::types::ExecutionStep;

    #[tokio::test]
    async fn test_execute_plan() {
        let steps = vec![
            ExecutionStep::new("步骤 1".to_string(), "tool1".to_string(), 1.0),
            ExecutionStep::new("步骤 2".to_string(), "tool2".to_string(), 1.0),
        ];

        let plan = ExecutionPlan::new("测试计划".to_string(), steps);
        let executor = PlanExecutor::new();

        let result = executor.execute(plan).await.unwrap();

        assert!(result.is_success());
        assert_eq!(result.outputs.len(), 2);
    }

    #[tokio::test]
    async fn test_progress_callback() {
        use std::sync::Mutex;

        let steps = vec![
            ExecutionStep::new("步骤 1".to_string(), "tool1".to_string(), 1.0),
        ];

        let plan = ExecutionPlan::new("测试计划".to_string(), steps);

        // 捕获进度回调
        let progress_updates = Arc::new(Mutex::new(Vec::new()));
        let updates_clone = progress_updates.clone();

        let callback: ProgressCallback = Arc::new(move |progress| {
            updates_clone.lock().unwrap().push(progress.status);
        });

        let executor = PlanExecutor::new().with_progress_callback(callback);
        let _result = executor.execute(plan).await.unwrap();

        let updates = progress_updates.lock().unwrap();
        assert!(updates.contains(&StepStatus::Running));
        assert!(updates.contains(&StepStatus::Success));
    }
}
