//! 任务执行器 (TaskExecutor)
//!
//! Phase 10: 任务分解与规划系统
//!
//! 负责按照执行计划执行任务，支持串行/并行执行、进度反馈、错误处理

use super::error::{TaskError, TaskResult as TaskOpResult};
use super::types::{
    ExecutionMode, ExecutionPlan, ExecutionResult, ExecutionStage, RetryPolicy, SubTask,
    TaskProgress, TaskResult, TaskStatus,
};
use crate::shell_executor::ShellExecutorWithFixer;
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;

/// 进度回调函数类型
pub type ProgressCallback = Arc<dyn Fn(TaskProgress) + Send + Sync>;

/// 任务执行器
///
/// 按照执行计划执行任务，支持串行和并行执行模式
pub struct TaskExecutor {
    /// Shell 执行器
    shell_executor: Arc<ShellExecutorWithFixer>,

    /// 进度回调
    progress_callback: Option<ProgressCallback>,

    /// 当前执行状态
    state: Arc<RwLock<ExecutorState>>,

    /// 超时设置（秒）
    timeout: Option<u64>,
}

/// 执行器内部状态
#[derive(Debug, Clone)]
struct ExecutorState {
    /// 当前阶段
    current_stage: usize,

    /// 总阶段数
    total_stages: usize,

    /// 当前任务
    current_task: String,

    /// 已完成任务数
    completed_tasks: usize,

    /// 总任务数
    total_tasks: usize,

    /// 开始时间
    start_time: Option<Instant>,

    /// 是否被取消
    cancelled: bool,
}

impl ExecutorState {
    fn new() -> Self {
        Self {
            current_stage: 0,
            total_stages: 0,
            current_task: String::new(),
            completed_tasks: 0,
            total_tasks: 0,
            start_time: None,
            cancelled: false,
        }
    }
}

impl TaskExecutor {
    /// 创建新的任务执行器
    pub fn new(shell_executor: Arc<ShellExecutorWithFixer>) -> Self {
        Self {
            shell_executor,
            progress_callback: None,
            state: Arc::new(RwLock::new(ExecutorState::new())),
            timeout: None,
        }
    }

    /// 设置进度回调
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// 设置任务超时（秒）
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 执行计划
    pub async fn execute(&self, plan: ExecutionPlan) -> TaskOpResult<ExecutionResult> {
        // 初始化状态
        {
            let mut state = self.state.write().await;
            state.start_time = Some(Instant::now());
            state.total_stages = plan.stages.len();
            state.total_tasks = plan.total_tasks();
            state.completed_tasks = 0;
            state.current_stage = 0;
            state.cancelled = false;
        }

        let start_time = Instant::now();
        let mut all_results = Vec::new();

        // 逐阶段执行
        for (stage_idx, stage) in plan.stages.iter().enumerate() {
            // 检查是否被取消
            if self.is_cancelled().await {
                return Err(TaskError::ExecutionCancelled);
            }

            // 更新当前阶段
            {
                let mut state = self.state.write().await;
                state.current_stage = stage_idx;
            }

            // 报告进度
            self.report_progress().await;

            // 根据执行模式执行任务
            let stage_results = match stage.execution_mode {
                ExecutionMode::Sequential => self.execute_sequential(stage).await?,
                ExecutionMode::Parallel => self.execute_parallel(stage).await?,
            };

            all_results.extend(stage_results);
        }

        let elapsed = start_time.elapsed().as_secs() as u32;

        // 统计结果
        let completed = all_results
            .iter()
            .filter(|r| r.status == TaskStatus::Success)
            .count();
        let failed = all_results
            .iter()
            .filter(|r| r.status == TaskStatus::Failed)
            .count();
        let skipped = all_results
            .iter()
            .filter(|r| r.status == TaskStatus::Skipped)
            .count();
        let total_tasks = plan.total_tasks();

        Ok(ExecutionResult {
            plan_id: plan.id,
            total_tasks,
            completed_tasks: completed,
            failed_tasks: failed,
            skipped_tasks: skipped,
            total_time: elapsed,
            task_results: all_results,
        })
    }

    /// 串行执行阶段
    ///
    /// v1.20.0: 支持环境变量共享 - 将同一 Stage 内的任务合并为一个 shell 命令执行
    async fn execute_sequential(&self, stage: &ExecutionStage) -> TaskOpResult<Vec<TaskResult>> {
        // 如果只有一个任务，使用旧逻辑（优化）
        if stage.tasks.len() == 1 {
            let result = self.execute_task(&stage.tasks[0]).await;

            // 更新状态
            {
                let mut state = self.state.write().await;
                state.completed_tasks += 1;
            }
            self.report_progress().await;

            return Ok(vec![result]);
        }

        // 多任务场景：合并执行以支持环境变量共享
        self.execute_merged_tasks(&stage.tasks).await
    }

    /// 合并执行多个任务（环境变量共享）
    ///
    /// 将多个任务的命令合并为一个 shell 脚本，在同一进程中执行
    async fn execute_merged_tasks(&self, tasks: &[SubTask]) -> TaskOpResult<Vec<TaskResult>> {

        // 1. 构建合并后的命令
        let merged_command = self.build_merged_command(tasks);

        // 2. 执行合并后的命令
        let exec_start = Utc::now();
        let (global_success, merged_output, global_error) =
            self.execute_with_retry_merged(&merged_command, tasks).await;
        let exec_end = Utc::now();

        // 3. 拆分输出并创建每个任务的结果
        let task_outputs = self.split_merged_output(&merged_output, tasks.len());
        let mut results = Vec::new();

        for (idx, task) in tasks.iter().enumerate() {
            let output = task_outputs.get(idx).cloned().unwrap_or_default();

            // 判断此任务是否成功
            let (status, error) = if !global_success {
                // 如果合并命令失败，判断是哪个任务失败
                // 策略：如果后续任务没有输出，说明当前任务失败导致链断裂
                let has_subsequent_output = (idx + 1..tasks.len())
                    .any(|i| task_outputs.get(i).is_some_and(|s| !s.trim().is_empty()));

                if !has_subsequent_output {
                    // 当前或之后的任务失败
                    (TaskStatus::Failed, global_error.clone())
                } else {
                    // 之前的任务已成功，当前任务也成功
                    (TaskStatus::Success, None)
                }
            } else {
                (TaskStatus::Success, None)
            };

            let result = TaskResult {
                task: task.clone(),
                status,
                output,
                error,
                start_time: exec_start,
                end_time: exec_end,
                duration: (exec_end - exec_start).num_seconds() as u32,
            };

            results.push(result);

            // 更新状态和进度
            {
                let mut state = self.state.write().await;
                state.completed_tasks += 1;
                state.current_task = task.name.clone();
            }
            self.report_progress().await;
        }

        Ok(results)
    }

    /// 构建合并后的命令
    ///
    /// 使用 && 连接所有命令，并在每个任务后插入分隔符
    fn build_merged_command(&self, tasks: &[SubTask]) -> String {
        let mut merged = String::new();

        for (idx, task) in tasks.iter().enumerate() {
            // 添加任务命令
            merged.push_str(&task.command);

            // 添加分隔符（用于输出拆分）
            // 使用唯一标记避免与用户输出冲突
            merged.push_str(&format!(" ; echo '__REALCONSOLE_TASK_{}_END__'", idx));

            // 添加连接符（除了最后一个任务）
            if idx < tasks.len() - 1 {
                merged.push_str(" && ");
            }
        }

        merged
    }

    /// 拆分合并后的输出
    ///
    /// 根据分隔符将输出分配给每个任务
    fn split_merged_output(&self, output: &str, task_count: usize) -> Vec<String> {
        let mut outputs = Vec::new();
        let mut current = String::new();

        for line in output.lines() {
            // 检查是否是分隔符
            if line.starts_with("__REALCONSOLE_TASK_") && line.ends_with("_END__") {
                outputs.push(current.trim_end().to_string());
                current.clear();
            } else {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(line);
            }
        }

        // 添加最后一个任务的输出
        if !current.is_empty() {
            outputs.push(current.trim_end().to_string());
        }

        // 确保返回的数组长度匹配任务数量
        while outputs.len() < task_count {
            outputs.push(String::new());
        }

        outputs
    }

    /// 执行合并后的命令（带重试）
    async fn execute_with_retry_merged(
        &self,
        command: &str,
        tasks: &[SubTask],
    ) -> (bool, String, Option<String>) {
        // 使用第一个任务的重试策略（或默认策略）
        let default_policy = RetryPolicy::simple(3);
        let retry_policy = tasks
            .first()
            .and_then(|t| t.retry_policy.as_ref())
            .unwrap_or(&default_policy);

        for attempt in 0..=retry_policy.max_retries {
            if attempt > 0 {
                let delay = if retry_policy.exponential_backoff {
                    retry_policy.retry_interval * (2_u32.pow(attempt - 1))
                } else {
                    retry_policy.retry_interval
                };
                sleep(Duration::from_secs(delay as u64)).await;
            }

            // 执行合并后的命令
            let result = self.execute_command(command).await;

            match result {
                Ok(output) => {
                    return (true, output, None);
                }
                Err(error) => {
                    if attempt == retry_policy.max_retries {
                        return (false, String::new(), Some(error.to_string()));
                    }
                }
            }
        }

        (false, String::new(), Some("重试次数用尽".to_string()))
    }

    /// 并行执行阶段
    async fn execute_parallel(&self, stage: &ExecutionStage) -> TaskOpResult<Vec<TaskResult>> {
        let mut handles = Vec::new();

        // 为每个任务创建并发任务
        for task in stage.tasks.clone() {
            let executor = self.clone_for_task();

            let handle = tokio::spawn(async move { executor.execute_task(&task).await });

            handles.push(handle);
        }

        // 等待所有任务完成
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    results.push(result);

                    // 更新状态
                    {
                        let mut state = self.state.write().await;
                        state.completed_tasks += 1;
                    }

                    // 报告进度
                    self.report_progress().await;
                }
                Err(e) => {
                    return Err(TaskError::Other(format!("任务执行失败: {}", e)));
                }
            }
        }

        Ok(results)
    }

    /// 执行单个任务
    async fn execute_task(&self, task: &SubTask) -> TaskResult {
        // 更新当前任务
        {
            let mut state = self.state.write().await;
            state.current_task = task.name.clone();
        }

        self.report_progress().await;

        let start_time = Utc::now();

        // 执行任务（带重试）
        let (status, output, error) = self.execute_with_retry(task).await;

        let end_time = Utc::now();
        let duration = (end_time - start_time).num_seconds() as u32;

        TaskResult {
            task: task.clone(),
            status,
            output,
            error,
            start_time,
            end_time,
            duration,
        }
    }

    /// 带重试的任务执行
    async fn execute_with_retry(&self, task: &SubTask) -> (TaskStatus, String, Option<String>) {
        // 默认重试策略
        let default_policy = RetryPolicy::simple(3);
        let retry_policy = task.retry_policy.as_ref().unwrap_or(&default_policy);

        for attempt in 0..=retry_policy.max_retries {
            if attempt > 0 {
                // 计算延迟（支持指数退避）
                let delay = if retry_policy.exponential_backoff {
                    retry_policy.retry_interval * (2_u32.pow(attempt - 1))
                } else {
                    retry_policy.retry_interval
                };
                sleep(Duration::from_secs(delay as u64)).await;
            }

            // 执行命令
            let result = self.execute_command(&task.command).await;

            match result {
                Ok(output) => {
                    return (TaskStatus::Success, output, None);
                }
                Err(error) => {
                    // 如果是最后一次尝试
                    if attempt == retry_policy.max_retries {
                        if task.skippable {
                            return (TaskStatus::Skipped, String::new(), Some(error.to_string()));
                        } else {
                            return (TaskStatus::Failed, String::new(), Some(error.to_string()));
                        }
                    }
                    // 否则继续重试
                }
            }
        }

        (
            TaskStatus::Failed,
            String::new(),
            Some("重试次数用尽".to_string()),
        )
    }

    /// 预处理命令（修复常见问题）
    ///
    /// 处理独立的 cd 命令问题：检测是否为单独的 cd 命令并警告
    fn preprocess_command(&self, command: &str) -> String {
        let trimmed = command.trim();

        // 检测独立的 cd 命令（这是错误的用法）
        if trimmed.starts_with("cd ") && !trimmed.contains("&&") && !trimmed.contains(";") {
            // 这是一个独立的 cd 命令，会失效
            // 记录警告但仍然执行（让用户看到错误）
            eprintln!("⚠ 警告: 检测到独立的 cd 命令 '{}'", trimmed);
            eprintln!("   cd 命令不会影响后续任务的工作目录");
            eprintln!("   建议: 使用 'cd dir && command' 的格式");
        }

        command.to_string()
    }

    /// 执行命令
    async fn execute_command(&self, command: &str) -> TaskOpResult<String> {
        // 预处理命令
        let processed_command = self.preprocess_command(command);

        // 应用超时
        if let Some(timeout) = self.timeout {
            match tokio::time::timeout(
                Duration::from_secs(timeout),
                self.shell_executor
                    .execute_with_analysis(&processed_command),
            )
            .await
            {
                Ok(exec_result) => {
                    if exec_result.success {
                        Ok(exec_result.output.clone())
                    } else {
                        let error_msg = exec_result
                            .error_analysis
                            .as_ref()
                            .map(|a| a.raw_error.clone())
                            .unwrap_or_else(|| exec_result.output.clone());
                        Err(TaskError::ShellExecutionError(error_msg))
                    }
                }
                Err(_) => Err(TaskError::ShellExecutionError(format!(
                    "命令超时 ({} 秒)",
                    timeout
                ))),
            }
        } else {
            let exec_result = self
                .shell_executor
                .execute_with_analysis(&processed_command)
                .await;
            if exec_result.success {
                Ok(exec_result.output.clone())
            } else {
                let error_msg = exec_result
                    .error_analysis
                    .as_ref()
                    .map(|a| a.raw_error.clone())
                    .unwrap_or_else(|| exec_result.output.clone());
                Err(TaskError::ShellExecutionError(error_msg))
            }
        }
    }

    /// 报告进度
    async fn report_progress(&self) {
        if let Some(callback) = &self.progress_callback {
            let state = self.state.read().await;

            let elapsed_time = state
                .start_time
                .map(|t| t.elapsed().as_secs() as u32)
                .unwrap_or(0);

            // 估算剩余时间
            let estimated_remaining = if state.completed_tasks > 0 {
                let avg_time_per_task = elapsed_time / state.completed_tasks as u32;
                let remaining_tasks = state.total_tasks.saturating_sub(state.completed_tasks);
                avg_time_per_task * remaining_tasks as u32
            } else {
                0
            };

            let progress = TaskProgress {
                current_stage: state.current_stage,
                total_stages: state.total_stages,
                current_task: state.current_task.clone(),
                completed_tasks: state.completed_tasks,
                total_tasks: state.total_tasks,
                elapsed_time,
                estimated_remaining,
            };

            callback(progress);
        }
    }

    /// 检查是否被取消
    async fn is_cancelled(&self) -> bool {
        self.state.read().await.cancelled
    }

    /// 取消执行
    pub async fn cancel(&self) {
        let mut state = self.state.write().await;
        state.cancelled = true;
    }

    /// 克隆用于并发任务
    fn clone_for_task(&self) -> Self {
        Self {
            shell_executor: Arc::clone(&self.shell_executor),
            progress_callback: self.progress_callback.clone(),
            state: Arc::clone(&self.state),
            timeout: self.timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_executor() -> TaskExecutor {
        let shell_executor = Arc::new(ShellExecutorWithFixer::new());
        TaskExecutor::new(shell_executor)
    }

    fn create_test_plan(tasks: Vec<SubTask>) -> ExecutionPlan {
        let stages = vec![ExecutionStage::new(0, tasks, ExecutionMode::Sequential)];
        ExecutionPlan::new("test plan", stages)
    }

    #[tokio::test]
    async fn test_execute_simple_plan() {
        let executor = create_test_executor();
        let tasks = vec![SubTask::new("t1", "Echo test", "echo 'hello'")];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        assert_eq!(result.completed_tasks, 1);
        assert_eq!(result.failed_tasks, 0);
    }

    #[tokio::test]
    async fn test_execute_with_failure() {
        let executor = create_test_executor();
        let tasks = vec![SubTask::new("t1", "Fail", "false")];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        assert_eq!(result.completed_tasks, 0);
        assert_eq!(result.failed_tasks, 1);
    }

    #[tokio::test]
    async fn test_execute_skippable_task() {
        let executor = create_test_executor();
        let tasks = vec![SubTask::new("t1", "Fail but skippable", "false").skippable()];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        assert_eq!(result.completed_tasks, 0);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.skipped_tasks, 1);
    }

    #[tokio::test]
    async fn test_execute_parallel() {
        let executor = create_test_executor();
        let tasks = vec![
            SubTask::new("t1", "Task 1", "echo 'task1'"),
            SubTask::new("t2", "Task 2", "echo 'task2'"),
        ];

        let stages = vec![ExecutionStage::new(0, tasks, ExecutionMode::Parallel)];
        let plan = ExecutionPlan::new("parallel test", stages);

        let result = executor.execute(plan).await.unwrap();

        assert_eq!(result.completed_tasks, 2);
    }

    #[tokio::test]
    async fn test_progress_callback() {
        use std::sync::Mutex;

        let executor = create_test_executor();
        let progress_log = Arc::new(Mutex::new(Vec::new()));
        let progress_log_clone = Arc::clone(&progress_log);

        let callback: ProgressCallback = Arc::new(move |progress| {
            progress_log_clone
                .lock()
                .unwrap()
                .push(progress.current_task.clone());
        });

        let executor = executor.with_progress_callback(callback);

        let tasks = vec![SubTask::new("t1", "Test", "echo 'test'")];
        let plan = create_test_plan(tasks);

        let _ = executor.execute(plan).await;

        let log = progress_log.lock().unwrap();
        assert!(!log.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_execution() {
        let executor = Arc::new(create_test_executor());

        let tasks = vec![
            SubTask::new("t1", "Task 1", "sleep 2"),
            SubTask::new("t2", "Task 2", "sleep 2"),
            SubTask::new("t3", "Task 3", "sleep 2"),
        ];
        let plan = create_test_plan(tasks);

        let executor_clone = Arc::clone(&executor);
        let handle = tokio::spawn(async move { executor_clone.execute(plan).await });

        // 等待一小段时间让执行开始
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 然后取消
        executor.cancel().await;

        let result = handle.await.unwrap();
        // 可能成功或取消，取决于时机
        let is_ok = result.is_ok() || matches!(result, Err(TaskError::ExecutionCancelled));
        assert!(is_ok, "Result should be either Ok or ExecutionCancelled");
    }

    #[tokio::test]
    async fn test_timeout_control() {
        // 创建一个设置了1秒超时的执行器
        let executor = create_test_executor().with_timeout(1);

        // 创建一个需要3秒的任务（会超时）
        let tasks = vec![SubTask::new("t1", "Long running task", "sleep 3")];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        // 应该失败（因为超时）
        assert_eq!(result.failed_tasks, 1);
        assert_eq!(result.completed_tasks, 0);

        // 检查错误信息包含"超时"
        let task_result = &result.task_results[0];
        assert_eq!(task_result.status, TaskStatus::Failed);
        assert!(task_result.error.is_some());
        let error_msg = task_result.error.as_ref().unwrap();
        assert!(
            error_msg.contains("超时") || error_msg.contains("timeout"),
            "Error message should contain '超时' or 'timeout', got: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_timeout_with_skippable() {
        // 创建一个设置了1秒超时的执行器
        let executor = create_test_executor().with_timeout(1);

        // 创建一个可跳过的超时任务
        let tasks = vec![SubTask::new("t1", "Skippable timeout task", "sleep 3").skippable()];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        // 应该被跳过
        assert_eq!(result.skipped_tasks, 1);
        assert_eq!(result.failed_tasks, 0);
        assert_eq!(result.completed_tasks, 0);
    }

    #[tokio::test]
    async fn test_no_timeout() {
        // 创建一个没有设置超时的执行器
        let executor = create_test_executor();

        // 创建一个快速任务
        let tasks = vec![SubTask::new("t1", "Quick task", "echo 'done'")];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        // 应该成功
        assert_eq!(result.completed_tasks, 1);
        assert_eq!(result.failed_tasks, 0);
    }

    #[tokio::test]
    async fn test_cd_command_warning() {
        // 测试独立 cd 命令会触发警告（但仍然执行）
        let executor = create_test_executor();

        // 创建包含独立 cd 命令的任务
        let tasks = vec![
            SubTask::new("t1", "Create dir", "mkdir -p /tmp/test_realconsole_cd"),
            SubTask::new(
                "t2",
                "Standalone cd (will warn)",
                "cd /tmp/test_realconsole_cd",
            ),
            SubTask::new("t3", "Try to create file", "touch test.txt"),
        ];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        // cd 命令本身会成功（只是不影响后续命令）
        // touch test.txt 会在当前目录执行，而不是 /tmp/test_realconsole_cd
        assert_eq!(result.total_tasks, 3);

        // 清理
        let _ = std::fs::remove_dir_all("/tmp/test_realconsole_cd");
    }

    #[tokio::test]
    async fn test_cd_with_command_works() {
        // 测试 cd && command 的正确用法
        let executor = create_test_executor();

        let tasks = vec![
            SubTask::new("t1", "Create and cd", "mkdir -p /tmp/test_realconsole_cd2 && cd /tmp/test_realconsole_cd2 && touch success.txt"),
        ];
        let plan = create_test_plan(tasks);

        let result = executor.execute(plan).await.unwrap();

        assert_eq!(result.completed_tasks, 1);

        // 验证文件在正确的目录
        assert!(std::path::Path::new("/tmp/test_realconsole_cd2/success.txt").exists());

        // 清理
        let _ = std::fs::remove_dir_all("/tmp/test_realconsole_cd2");
    }
}
