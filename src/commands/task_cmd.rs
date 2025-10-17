//! 任务分解与规划命令
//!
//! Phase 10: Task Decomposition & Planning System
//!
//! 提供任务分解、规划和执行的命令接口

use crate::command::{Command, CommandRegistry};
use crate::task::{
    ExecutionContext, ExecutionPlan, ExecutionResult, TaskDecomposer, TaskExecutor, TaskPlanner,
};
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 任务管理器状态
///
/// 保存当前的任务计划和执行结果
pub struct TaskManager {
    /// 最近的执行计划
    current_plan: Option<ExecutionPlan>,

    /// 历史计划（最多保存10个）
    history: Vec<ExecutionPlan>,

    /// 最近的执行结果
    last_result: Option<ExecutionResult>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            current_plan: None,
            history: Vec::new(),
            last_result: None,
        }
    }

    /// 保存计划
    pub fn save_plan(&mut self, plan: ExecutionPlan) {
        // 如果有当前计划，移到历史
        if let Some(current) = self.current_plan.take() {
            self.history.push(current);
            // 限制历史记录数量
            if self.history.len() > 10 {
                self.history.remove(0);
            }
        }
        self.current_plan = Some(plan);
    }

    /// 获取当前计划
    pub fn get_current_plan(&self) -> Option<&ExecutionPlan> {
        self.current_plan.as_ref()
    }

    /// 保存执行结果
    pub fn save_result(&mut self, result: ExecutionResult) {
        self.last_result = Some(result);
    }

    /// 获取最近的执行结果
    pub fn get_last_result(&self) -> Option<&ExecutionResult> {
        self.last_result.as_ref()
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 注册任务相关命令
pub fn register_task_commands(
    registry: &mut CommandRegistry,
    llm_manager: Arc<tokio::sync::RwLock<crate::llm_manager::LlmManager>>,
    shell_executor: Arc<crate::shell_executor::ShellExecutorWithFixer>,
) {
    // 创建共享的任务管理器
    let task_manager = Arc::new(RwLock::new(TaskManager::new()));

    // /plan 命令 - 分解和规划任务
    {
        let llm_manager = Arc::clone(&llm_manager);
        let manager = Arc::clone(&task_manager);

        registry.register(Command::from_fn(
            "plan",
            "分解和规划任务",
            move |goal: &str| {
                if goal.trim().is_empty() {
                    return format!(
                        "{}\n使用方式: /plan <目标描述>",
                        "❌ 请提供任务目标".red()
                    );
                }

                let llm_manager = Arc::clone(&llm_manager);
                let manager = Arc::clone(&manager);
                let goal = goal.to_string();

                // 在同步上下文中执行异步代码
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        execute_plan_command(&llm_manager, &manager, &goal).await
                    })
                })
            },
        ));
    }

    // /execute 命令 - 执行任务计划
    {
        let shell_executor = Arc::clone(&shell_executor);
        let manager = Arc::clone(&task_manager);

        registry.register(Command::from_fn(
            "execute",
            "执行任务计划",
            move |_arg: &str| {
                let shell_executor = Arc::clone(&shell_executor);
                let manager = Arc::clone(&manager);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        execute_tasks_command(&shell_executor, &manager).await
                    })
                })
            },
        ));
    }

    // /tasks 命令 - 查看当前任务计划
    {
        let manager = Arc::clone(&task_manager);

        registry.register(Command::from_fn(
            "tasks",
            "查看当前任务计划",
            move |_arg: &str| {
                let manager = Arc::clone(&manager);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        view_tasks_command(&manager).await
                    })
                })
            },
        ));
    }

    // /task_status 命令 - 查看任务执行状态
    {
        let manager = Arc::clone(&task_manager);

        registry.register(Command::from_fn(
            "task_status",
            "查看任务执行状态",
            move |_arg: &str| {
                let manager = Arc::clone(&manager);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        view_task_status_command(&manager).await
                    })
                })
            },
        ));
    }
}

/// 执行 /plan 命令
async fn execute_plan_command(
    llm_manager: &Arc<tokio::sync::RwLock<crate::llm_manager::LlmManager>>,
    manager: &Arc<RwLock<TaskManager>>,
    goal: &str,
) -> String {
    let mut output = String::new();

    // 1. 获取执行上下文
    let context = ExecutionContext::current();

    // 2. 获取 LLM 客户端
    let llm = {
        let mgr = llm_manager.read().await;
        match mgr.primary().or(mgr.fallback()) {
            Some(llm) => llm.clone(),
            None => {
                return format!("❌ 未配置 LLM 客户端\n{}", "提示: 需要 LLM 来智能分解任务".dimmed());
            }
        }
    };

    // 3. 分解任务
    let decomposer = TaskDecomposer::new(llm);
    let subtasks = match decomposer.decompose(goal, &context).await {
        Ok(tasks) => tasks,
        Err(e) => {
            return format!("❌ 任务分解失败: {}", e);
        }
    };

    // 4. 生成执行计划
    let planner = TaskPlanner::new();
    let plan = match planner.plan(goal, subtasks) {
        Ok(p) => p,
        Err(e) => {
            return format!("❌ 计划生成失败: {}", e);
        }
    };

    // 5. 分析计划
    let analysis = planner.analyze_plan(&plan);

    // 6. 紧凑的输出格式
    output.push_str(&format!("\n{}\n", goal.bold()));

    // 摘要行（单行显示核心信息）
    let summary = format!(
        "{} {} 阶段 · {} 任务 · {}{}秒",
        "▸".dimmed(),
        analysis.total_stages,
        analysis.total_tasks,
        if analysis.parallel_stages > 0 { "⚡ " } else { "" },
        analysis.parallel_time
    );

    if analysis.time_saved > 0 {
        output.push_str(&format!(
            "{} {}\n",
            summary.dimmed(),
            format!("(节省 {}秒)", analysis.time_saved).green()
        ));
    } else {
        output.push_str(&format!("{}\n", summary.dimmed()));
    }

    // 7. 树状结构显示任务
    for (idx, stage) in plan.stages.iter().enumerate() {
        let is_last_stage = idx == plan.stages.len() - 1;
        let branch = if is_last_stage { "└─" } else { "├─" };
        let pipe = if is_last_stage { "  " } else { "│ " };

        let mode_icon = match stage.execution_mode {
            crate::task::ExecutionMode::Sequential => "→",
            crate::task::ExecutionMode::Parallel => "⇉",
        };

        output.push_str(&format!(
            "{} {} {} {}\n",
            branch.dimmed(),
            mode_icon.cyan(),
            format!("Stage {}", idx + 1).dimmed(),
            format!("({}s)", stage.estimated_time).dimmed()
        ));

        for (task_idx, task) in stage.tasks.iter().enumerate() {
            let is_last_task = task_idx == stage.tasks.len() - 1;
            let task_branch = if is_last_task { "└─" } else { "├─" };

            output.push_str(&format!(
                "{} {}  {} {}\n",
                pipe.dimmed(),
                task_branch.dimmed(),
                task.name,
                format!("$ {}", task.command).dimmed()
            ));
        }
    }

    // 8. 保存计划
    {
        let mut mgr = manager.write().await;
        mgr.save_plan(plan);
    }

    output.push_str(&format!("\n{}\n", format!("使用 {} 执行", "/execute".cyan()).dimmed()));

    output
}

/// 执行 /execute 命令
async fn execute_tasks_command(
    shell_executor: &Arc<crate::shell_executor::ShellExecutorWithFixer>,
    manager: &Arc<RwLock<TaskManager>>,
) -> String {
    // 1. 获取当前计划
    let plan = {
        let mgr = manager.read().await;
        match mgr.get_current_plan() {
            Some(p) => p.clone(),
            None => {
                return format!("❌ 无待执行计划\n{}", "提示: /plan <目标>".dimmed());
            }
        }
    };

    let mut output = String::new();

    // 2. 创建执行器
    let executor = TaskExecutor::new(Arc::clone(shell_executor))
        .with_timeout(300);

    // 3. 执行计划
    let result = match executor.execute(plan.clone()).await {
        Ok(r) => r,
        Err(e) => {
            return format!("❌ 执行失败: {}", e);
        }
    };

    // 4. 保存结果
    {
        let mut mgr = manager.write().await;
        mgr.save_result(result.clone());
    }

    // 5. 紧凑的结果显示
    let status = if result.is_success() {
        "✓".green()
    } else {
        "✗".red()
    };

    output.push_str(&format!(
        "\n{} {} · {} · {}秒\n",
        status,
        format!("{}/{}", result.completed_tasks, result.total_tasks).bold(),
        format!("{:.0}%", result.success_rate() * 100.0).dimmed(),
        result.total_time
    ));

    // 6. 仅在有失败时显示详情
    if result.failed_tasks > 0 {
        for task_result in &result.task_results {
            if matches!(task_result.status, crate::task::TaskStatus::Failed) {
                output.push_str(&format!(
                    "  {} {} {}\n",
                    "✗".red(),
                    task_result.task.name,
                    format!("$ {}", task_result.task.command).dimmed()
                ));
                if let Some(error) = &task_result.error {
                    output.push_str(&format!("    {}\n", error.red()));
                }
            }
        }
    }

    output
}

/// 执行 /tasks 命令
async fn view_tasks_command(manager: &Arc<RwLock<TaskManager>>) -> String {
    let mgr = manager.read().await;

    match mgr.get_current_plan() {
        Some(plan) => {
            let mut output = String::new();

            // 紧凑的标题行
            output.push_str(&format!(
                "\n{} {} · {} 阶段 · {}秒\n",
                plan.goal.bold(),
                format!("{} 任务", plan.total_tasks()).dimmed(),
                plan.stages.len(),
                plan.total_estimated_time
            ));

            // 树状任务列表
            for (idx, stage) in plan.stages.iter().enumerate() {
                let is_last_stage = idx == plan.stages.len() - 1;
                let branch = if is_last_stage { "└─" } else { "├─" };
                let pipe = if is_last_stage { "  " } else { "│ " };

                let mode = match stage.execution_mode {
                    crate::task::ExecutionMode::Sequential => "→",
                    crate::task::ExecutionMode::Parallel => "⇉",
                };

                output.push_str(&format!(
                    "{} {} {}\n",
                    branch.dimmed(),
                    mode.cyan(),
                    format!("Stage {}", idx + 1).dimmed()
                ));

                for (task_idx, task) in stage.tasks.iter().enumerate() {
                    let is_last = task_idx == stage.tasks.len() - 1;
                    let task_branch = if is_last { "└─" } else { "├─" };

                    output.push_str(&format!(
                        "{} {} {}\n",
                        pipe.dimmed(),
                        task_branch.dimmed(),
                        task.name
                    ));
                }
            }

            output.push_str(&format!("\n{}\n", format!("使用 {} 执行", "/execute".cyan()).dimmed()));

            output
        }
        None => {
            format!("无当前计划\n{}", "提示: /plan <目标>".dimmed())
        }
    }
}

/// 执行 /task_status 命令
async fn view_task_status_command(manager: &Arc<RwLock<TaskManager>>) -> String {
    let mgr = manager.read().await;

    match mgr.get_last_result() {
        Some(result) => {
            let mut output = String::new();

            // 紧凑的摘要行
            let status = if result.is_success() { "✓" } else { "✗" };
            output.push_str(&format!(
                "\n{} {} · {}秒 · {:.0}%\n",
                status,
                format!("{}/{}", result.completed_tasks, result.total_tasks).bold(),
                result.total_time,
                result.success_rate() * 100.0
            ));

            // 紧凑的任务列表
            for task_result in &result.task_results {
                let icon = match task_result.status {
                    crate::task::TaskStatus::Success => "✓".green(),
                    crate::task::TaskStatus::Failed => "✗".red(),
                    crate::task::TaskStatus::Skipped => "⊘".yellow(),
                    _ => "•".dimmed(),
                };

                output.push_str(&format!(
                    "{} {} {}\n",
                    icon,
                    task_result.task.name,
                    format!("({}s)", task_result.duration).dimmed()
                ));

                // 仅显示失败任务的错误信息
                if matches!(task_result.status, crate::task::TaskStatus::Failed) {
                    if let Some(error) = &task_result.error {
                        output.push_str(&format!("  {}\n", error.red()));
                    }
                }
            }

            output
        }
        None => {
            format!("无执行记录\n{}", "提示: /execute".dimmed())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_manager_new() {
        let manager = TaskManager::new();
        assert!(manager.current_plan.is_none());
        assert!(manager.history.is_empty());
        assert!(manager.last_result.is_none());
    }

    #[test]
    fn test_task_manager_save_plan() {
        let mut manager = TaskManager::new();

        let plan = ExecutionPlan::new(
            "test goal",
            vec![]
        );

        manager.save_plan(plan);
        assert!(manager.current_plan.is_some());
    }
}
