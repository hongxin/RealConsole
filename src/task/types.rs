//! 任务系统核心数据结构
//!
//! Phase 10: 任务分解与规划系统
//!
//! 本模块定义了任务分解、规划和执行所需的核心数据结构。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 子任务定义
///
/// 代表一个可执行的原子任务单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    /// 任务唯一标识
    pub id: String,

    /// 任务名称
    pub name: String,

    /// 任务描述
    pub description: String,

    /// 要执行的命令
    pub command: String,

    /// 估计执行时间（秒）
    pub estimated_time: u32,

    /// 依赖的任务 ID 列表
    pub depends_on: Vec<String>,

    /// 任务类型
    pub task_type: TaskType,

    /// 是否可跳过（如果失败）
    pub skippable: bool,

    /// 重试策略
    pub retry_policy: Option<RetryPolicy>,
}

impl SubTask {
    /// 创建新的子任务
    pub fn new(id: impl Into<String>, name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            command: command.into(),
            estimated_time: 10,
            depends_on: Vec::new(),
            task_type: TaskType::Shell,
            skippable: false,
            retry_policy: None,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 设置估计时间
    pub fn with_estimated_time(mut self, seconds: u32) -> Self {
        self.estimated_time = seconds;
        self
    }

    /// 添加依赖
    pub fn with_dependency(mut self, task_id: impl Into<String>) -> Self {
        self.depends_on.push(task_id.into());
        self
    }

    /// 设置为可跳过
    pub fn skippable(mut self) -> Self {
        self.skippable = true;
        self
    }

    /// 设置重试策略
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    /// Shell 命令
    Shell,
    /// 文件操作
    FileOperation,
    /// 网络请求
    Network,
    /// 验证检查
    Validation,
    /// 用户交互
    UserInput,
}

impl ToString for TaskType {
    fn to_string(&self) -> String {
        match self {
            TaskType::Shell => "Shell".to_string(),
            TaskType::FileOperation => "FileOperation".to_string(),
            TaskType::Network => "Network".to_string(),
            TaskType::Validation => "Validation".to_string(),
            TaskType::UserInput => "UserInput".to_string(),
        }
    }
}

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: u32,

    /// 重试间隔（秒）
    pub retry_interval: u32,

    /// 是否指数退避
    pub exponential_backoff: bool,
}

impl RetryPolicy {
    /// 创建简单重试策略
    pub fn simple(max_retries: u32) -> Self {
        Self {
            max_retries,
            retry_interval: 1,
            exponential_backoff: false,
        }
    }

    /// 创建指数退避策略
    pub fn exponential(max_retries: u32, initial_interval: u32) -> Self {
        Self {
            max_retries,
            retry_interval: initial_interval,
            exponential_backoff: true,
        }
    }
}

/// 执行计划
///
/// 包含任务的执行顺序和并行策略
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 计划ID
    pub id: String,

    /// 原始目标
    pub goal: String,

    /// 执行阶段（每个阶段内的任务可并行执行）
    pub stages: Vec<ExecutionStage>,

    /// 总估计时间（秒）
    pub total_estimated_time: u32,

    /// 并行阶段数量
    pub parallel_stages: usize,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl ExecutionPlan {
    /// 创建新的执行计划
    pub fn new(goal: impl Into<String>, stages: Vec<ExecutionStage>) -> Self {
        let total_time = stages.iter().map(|s| s.estimated_time).sum();
        let parallel_stages = stages
            .iter()
            .filter(|s| matches!(s.execution_mode, ExecutionMode::Parallel))
            .count();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.into(),
            stages,
            total_estimated_time: total_time,
            parallel_stages,
            created_at: Utc::now(),
        }
    }

    /// 获取总任务数
    pub fn total_tasks(&self) -> usize {
        self.stages.iter().map(|s| s.tasks.len()).sum()
    }
}

/// 执行阶段
///
/// 代表可以同时执行的一组任务
#[derive(Debug, Clone)]
pub struct ExecutionStage {
    /// 阶段编号
    pub stage_num: usize,

    /// 本阶段的任务列表（可并行执行）
    pub tasks: Vec<SubTask>,

    /// 估计时间（取最长任务时间）
    pub estimated_time: u32,

    /// 执行模式
    pub execution_mode: ExecutionMode,
}

impl ExecutionStage {
    /// 创建新的执行阶段
    pub fn new(stage_num: usize, tasks: Vec<SubTask>, execution_mode: ExecutionMode) -> Self {
        let estimated_time = tasks.iter().map(|t| t.estimated_time).max().unwrap_or(0);

        Self {
            stage_num,
            tasks,
            estimated_time,
            execution_mode,
        }
    }
}

/// 执行模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    /// 串行执行
    Sequential,
    /// 并行执行
    Parallel,
}

/// 任务执行结果
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// 任务信息
    pub task: SubTask,

    /// 执行状态
    pub status: TaskStatus,

    /// 输出内容
    pub output: String,

    /// 错误信息（如果失败）
    pub error: Option<String>,

    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 结束时间
    pub end_time: DateTime<Utc>,

    /// 执行时长（秒）
    pub duration: u32,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 跳过
    Skipped,
    /// 取消
    Cancelled,
}

impl TaskStatus {
    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Success | TaskStatus::Failed | TaskStatus::Skipped | TaskStatus::Cancelled
        )
    }

    /// 是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, TaskStatus::Success)
    }
}

/// 执行结果汇总
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 计划ID
    pub plan_id: String,

    /// 总任务数
    pub total_tasks: usize,

    /// 已完成任务数
    pub completed_tasks: usize,

    /// 失败任务数
    pub failed_tasks: usize,

    /// 跳过任务数
    pub skipped_tasks: usize,

    /// 总耗时（秒）
    pub total_time: u32,

    /// 各任务的执行结果
    pub task_results: Vec<TaskResult>,
}

impl ExecutionResult {
    /// 是否全部成功
    pub fn is_success(&self) -> bool {
        self.failed_tasks == 0
            && self.completed_tasks == self.total_tasks
    }

    /// 成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            return 1.0;
        }
        (self.completed_tasks - self.failed_tasks) as f64 / self.total_tasks as f64
    }
}

/// 任务进度
#[derive(Debug, Clone)]
pub struct TaskProgress {
    /// 当前阶段
    pub current_stage: usize,

    /// 总阶段数
    pub total_stages: usize,

    /// 当前任务名称
    pub current_task: String,

    /// 已完成任务数
    pub completed_tasks: usize,

    /// 总任务数
    pub total_tasks: usize,

    /// 已用时间（秒）
    pub elapsed_time: u32,

    /// 估计剩余时间（秒）
    pub estimated_remaining: u32,
}

impl TaskProgress {
    /// 计算完成百分比
    pub fn completion_percentage(&self) -> f64 {
        if self.total_tasks == 0 {
            return 0.0;
        }
        (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0
    }
}

/// 执行上下文
///
/// 提供任务执行所需的环境信息
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 工作目录
    pub working_dir: String,

    /// 操作系统
    pub os: String,

    /// Shell 类型
    pub shell: String,

    /// 环境变量
    pub env_vars: HashMap<String, String>,

    /// 用户信息
    pub user: String,
}

impl ExecutionContext {
    /// 获取当前执行上下文
    pub fn current() -> Self {
        let working_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        let os = std::env::consts::OS.to_string();

        let shell = if cfg!(unix) {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        } else {
            "cmd".to_string()
        };

        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            working_dir,
            os,
            shell,
            env_vars: std::env::vars().collect(),
            user,
        }
    }

    /// 获取环境变量
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }
}

/// 依赖关系图
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// 节点（任务ID -> 任务）
    pub nodes: HashMap<String, SubTask>,

    /// 出边（任务ID -> 依赖它的任务ID列表）
    pub edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// 创建空的依赖图
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, task: SubTask) {
        self.nodes.insert(task.id.clone(), task);
    }

    /// 添加边（from 依赖 to）
    pub fn add_edge(&mut self, from: String, to: String) {
        self.edges.entry(from).or_insert_with(Vec::new).push(to);
    }

    /// 获取入度
    pub fn in_degree(&self, task_id: &str) -> usize {
        self.edges
            .values()
            .filter(|deps| deps.contains(&task_id.to_string()))
            .count()
    }

    /// 获取所有依赖
    pub fn get_dependencies(&self, task_id: &str) -> Vec<String> {
        if let Some(task) = self.nodes.get(task_id) {
            task.depends_on.clone()
        } else {
            Vec::new()
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtask_builder() {
        let task = SubTask::new("t1", "Test Task", "echo test")
            .with_description("A test task")
            .with_estimated_time(30)
            .with_dependency("t0")
            .skippable();

        assert_eq!(task.id, "t1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.command, "echo test");
        assert_eq!(task.description, "A test task");
        assert_eq!(task.estimated_time, 30);
        assert_eq!(task.depends_on, vec!["t0"]);
        assert!(task.skippable);
    }

    #[test]
    fn test_retry_policy() {
        let simple = RetryPolicy::simple(3);
        assert_eq!(simple.max_retries, 3);
        assert_eq!(simple.retry_interval, 1);
        assert!(!simple.exponential_backoff);

        let exponential = RetryPolicy::exponential(5, 2);
        assert_eq!(exponential.max_retries, 5);
        assert_eq!(exponential.retry_interval, 2);
        assert!(exponential.exponential_backoff);
    }

    #[test]
    fn test_execution_plan() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2"),
        ];
        let stage = ExecutionStage::new(0, tasks, ExecutionMode::Sequential);
        let plan = ExecutionPlan::new("test goal", vec![stage]);

        assert_eq!(plan.goal, "test goal");
        assert_eq!(plan.total_tasks(), 2);
        assert_eq!(plan.stages.len(), 1);
    }

    #[test]
    fn test_task_status() {
        assert!(TaskStatus::Success.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
        assert!(TaskStatus::Success.is_success());
        assert!(!TaskStatus::Failed.is_success());
    }

    #[test]
    fn test_execution_context() {
        let ctx = ExecutionContext::current();
        assert!(!ctx.working_dir.is_empty());
        assert!(!ctx.os.is_empty());
        assert!(!ctx.shell.is_empty());
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        let task1 = SubTask::new("t1", "Task 1", "cmd1");
        let task2 = SubTask::new("t2", "Task 2", "cmd2").with_dependency("t1");

        graph.add_node(task1);
        graph.add_node(task2);
        graph.add_edge("t1".to_string(), "t2".to_string());

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.in_degree("t1"), 0);
        assert_eq!(graph.in_degree("t2"), 1);
        assert_eq!(graph.get_dependencies("t2"), vec!["t1"]);
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            plan_id: "plan1".to_string(),
            total_tasks: 10,
            completed_tasks: 8,
            failed_tasks: 2,
            skipped_tasks: 0,
            total_time: 120,
            task_results: Vec::new(),
        };

        assert!(!result.is_success());
        assert_eq!(result.success_rate(), 0.6);
    }

    #[test]
    fn test_task_progress() {
        let progress = TaskProgress {
            current_stage: 2,
            total_stages: 5,
            current_task: "Building...".to_string(),
            completed_tasks: 3,
            total_tasks: 10,
            elapsed_time: 30,
            estimated_remaining: 70,
        };

        assert_eq!(progress.completion_percentage(), 30.0);
    }
}
