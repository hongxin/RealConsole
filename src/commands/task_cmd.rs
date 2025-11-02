//! 任务分解与规划命令
//!
//! Phase 10: Task Decomposition & Planning System
//!
//! 提供任务分解、规划和执行的命令接口

use crate::command::{Command, CommandRegistry};
use crate::config::Config;
use crate::display::Display;
use crate::spinner::Spinner;
use crate::task::{
    ExecutionContext, ExecutionPlan, ExecutionResult, SubTask, TaskDecomposer, TaskExecutionResult,
    TaskExecutor, TaskPlanner, TaskStatus,
};
use chrono::Utc;
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

    // ========================================================================
    // ✨ v1.22.0 Phase 1: 任务持久化方法
    // ========================================================================

    /// 保存当前任务到文件
    ///
    /// 返回保存的文件路径
    pub fn save_current(&self, name: Option<String>) -> anyhow::Result<std::path::PathBuf> {
        let plan = self
            .current_plan
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("无待保存计划"))?;

        let saved = SavedTask::new(plan.goal.clone(), plan.clone(), self.last_result.clone());

        let saved = if let Some(name) = name {
            saved.with_name(name)
        } else {
            saved
        };

        saved.save_to_file()
    }

    /// 加载任务到当前计划
    pub fn load_task(&mut self, task: SavedTask) {
        self.save_plan(task.plan);
        if let Some(result) = task.result {
            self.save_result(result);
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ✨ v1.22.0 Phase 1: 任务持久化数据结构
// ============================================================================

/// 持久化的任务数据结构
///
/// 用于保存和加载任务计划及执行结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedTask {
    /// 任务 ID (UUID)
    pub id: String,

    /// 用户定义的任务名称（可选）
    pub name: Option<String>,

    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// 任务目标
    pub goal: String,

    /// 执行计划
    pub plan: ExecutionPlan,

    /// 执行结果（可选）
    pub result: Option<ExecutionResult>,
}

impl SavedTask {
    /// 从当前计划和结果创建保存任务
    pub fn new(goal: String, plan: ExecutionPlan, result: Option<ExecutionResult>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: None,
            created_at: chrono::Utc::now(),
            goal,
            plan,
            result,
        }
    }

    /// 设置任务名称
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// 保存到文件
    ///
    /// 文件路径: ~/.realconsole/tasks/{timestamp}_{id}.json
    pub fn save_to_file(&self) -> anyhow::Result<std::path::PathBuf> {
        // 获取任务保存目录
        let tasks_dir = Self::tasks_dir()?;

        // 确保目录存在
        std::fs::create_dir_all(&tasks_dir)?;

        // 生成文件名: {timestamp}_{id}.json
        let timestamp = self.created_at.format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.json", timestamp, &self.id[..8]);
        let filepath = tasks_dir.join(filename);

        // 序列化并写入文件
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&filepath, json)?;

        Ok(filepath)
    }

    /// 从文件加载任务
    pub fn load_from_file(filepath: &std::path::Path) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(filepath)?;
        let task: SavedTask = serde_json::from_str(&json)?;
        Ok(task)
    }

    /// 列出所有保存的任务
    ///
    /// 返回 (filepath, SavedTask) 列表，按创建时间倒序排列
    pub fn list_all() -> anyhow::Result<Vec<(std::path::PathBuf, SavedTask)>> {
        let tasks_dir = Self::tasks_dir()?;

        // 如果目录不存在，返回空列表
        if !tasks_dir.exists() {
            return Ok(Vec::new());
        }

        // 读取所有 .json 文件
        let mut tasks = Vec::new();
        for entry in std::fs::read_dir(tasks_dir)? {
            let entry = entry?;
            let path = entry.path();

            // 只处理 .json 文件
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(task) = Self::load_from_file(&path) {
                    tasks.push((path, task));
                }
            }
        }

        // 按创建时间倒序排列
        tasks.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        Ok(tasks)
    }

    /// 获取任务保存目录
    ///
    /// ~/.realconsole/tasks/
    fn tasks_dir() -> anyhow::Result<std::path::PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        Ok(home_dir.join(".realconsole").join("tasks"))
    }
}

/// 注册任务相关命令
pub fn register_task_commands(
    registry: &mut CommandRegistry,
    llm_manager: Arc<tokio::sync::RwLock<crate::llm_manager::LlmManager>>,
    shell_executor: Arc<crate::shell_executor::ShellExecutorWithFixer>,
    config: Config,
) {
    // 创建共享的任务管理器和配置
    let task_manager = Arc::new(RwLock::new(TaskManager::new()));
    let config = Arc::new(config);

    // /plan 命令 - 分解和规划任务
    {
        let llm_manager = Arc::clone(&llm_manager);
        let manager = Arc::clone(&task_manager);

        registry.register(Command::from_fn(
            "plan",
            "分解和规划任务",
            move |goal: &str| {
                if goal.trim().is_empty() {
                    return format!("{}\n使用方式: /plan <目标描述>", "[ERROR] 请提供任务目标".red());
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
        let config = Arc::clone(&config);

        registry.register(Command::from_fn(
            "execute",
            "执行任务计划",
            move |_arg: &str| {
                let shell_executor = Arc::clone(&shell_executor);
                let manager = Arc::clone(&manager);
                let config = Arc::clone(&config);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        execute_tasks_command(&shell_executor, &manager, &config).await
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
                    tokio::runtime::Handle::current()
                        .block_on(async { view_tasks_command(&manager).await })
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
                    tokio::runtime::Handle::current()
                        .block_on(async { view_task_status_command(&manager).await })
                })
            },
        ));
    }

    // ✨ v1.21.0 Phase 2: /task_output 命令 - 查看完整任务输出
    {
        let manager = Arc::clone(&task_manager);
        let config = Arc::clone(&config);

        registry.register(Command::from_fn(
            "task_output",
            "查看完整任务输出",
            move |arg: &str| {
                let manager = Arc::clone(&manager);
                let config = Arc::clone(&config);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { view_task_output_command(&manager, &config, arg).await })
                })
            },
        ));
    }

    // ✨ v1.22.0 Phase 1: /task_save 命令 - 保存当前任务
    {
        let manager = Arc::clone(&task_manager);

        registry.register(Command::from_fn(
            "task_save",
            "保存当前任务",
            move |arg: &str| {
                let manager = Arc::clone(&manager);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { task_save_command(&manager, arg).await })
                })
            },
        ));
    }

    // ✨ v1.22.0 Phase 1: /task_list 命令 - 列出所有保存的任务
    {
        registry.register(Command::from_fn(
            "task_list",
            "列出所有保存的任务",
            move |_arg: &str| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async { task_list_command().await })
                })
            },
        ));
    }

    // ✨ v1.22.0 Phase 1: /task_load 命令 - 加载保存的任务
    {
        let manager = Arc::clone(&task_manager);

        registry.register(Command::from_fn(
            "task_load",
            "加载保存的任务",
            move |arg: &str| {
                let manager = Arc::clone(&manager);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { task_load_command(&manager, arg).await })
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
                return format!(
                    "[ERROR] 未配置 LLM 客户端\n{}",
                    "提示: 需要 LLM 来智能分解任务".dimmed()
                );
            }
        }
    };

    // 3. 分解任务（显示 spinner 带模型名称）
    use crate::spinner::simplify_model_name;
    let model_name = simplify_model_name(llm.model());
    let spinner = Spinner::with_label(&model_name);

    let decomposer = TaskDecomposer::new(llm);
    let subtasks = match decomposer.decompose(goal, &context).await {
        Ok(tasks) => {
            spinner.stop();
            tasks
        }
        Err(e) => {
            spinner.stop();
            return format!("[ERROR] 任务分解失败: {}", e);
        }
    };

    // 4. 生成执行计划
    let planner = TaskPlanner::new();
    let plan = match planner.plan(goal, subtasks) {
        Ok(p) => p,
        Err(e) => {
            return format!("[ERROR] 计划生成失败: {}", e);
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
        if analysis.parallel_stages > 0 {
            "[>] "
        } else {
            ""
        },
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
            mode_icon,
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

    output.push_str(&format!(
        "\n{}\n",
        format!("使用 {} 执行", "/execute".cyan()).dimmed()
    ));

    output
}

/// 执行 /execute 命令
async fn execute_tasks_command(
    shell_executor: &Arc<crate::shell_executor::ShellExecutorWithFixer>,
    manager: &Arc<RwLock<TaskManager>>,
    config: &Arc<Config>,
) -> String {
    // 1. 获取当前计划
    let plan = {
        let mgr = manager.read().await;
        match mgr.get_current_plan() {
            Some(p) => p.clone(),
            None => {
                return format!("[ERROR] 无待执行计划\n{}", "提示: /plan <目标>".dimmed());
            }
        }
    };

    // 2. 创建执行器
    // ✨ v1.22.0 Phase 3: 应用配置
    let executor = TaskExecutor::new(Arc::clone(shell_executor))
        .with_timeout(300)
        .with_merge_config(
            config.task.execution.merge_stages,
            config.task.execution.max_merged_tasks,
        );

    // 3. 执行计划
    let result = match executor.execute(plan.clone()).await {
        Ok(r) => r,
        Err(e) => {
            return format!("[ERROR] 执行失败: {}", e);
        }
    };

    // 4. 保存结果
    {
        let mut mgr = manager.write().await;
        mgr.save_result(result.clone());
    }

    // 5. 使用统一的显示方法（根据 DisplayMode 显示不同详细程度）
    // ✨ v1.21.0: 添加 config 参数以支持 TaskDisplayConfig
    Display::task_execution_result(config.display.mode, &result, Some(&plan), config);

    // 返回空字符串（显示已经通过 Display 完成）
    String::new()
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
                    mode,
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

            output.push_str(&format!(
                "\n{}\n",
                format!("使用 {} 执行", "/execute".cyan()).dimmed()
            ));

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

/// ✨ v1.21.0 Phase 2: 执行 /task_output 命令 - 查看完整任务输出
async fn view_task_output_command(
    manager: &Arc<RwLock<TaskManager>>,
    config: &Arc<Config>,
    arg: &str,
) -> String {
    let mgr = manager.read().await;

    // 1. 获取最近的执行结果
    let result = match mgr.get_last_result() {
        Some(r) => r,
        None => {
            return format!(
                "{}\n{}",
                "[ERROR] 没有可用的任务执行结果".red(),
                "提示: 请先使用 /execute 执行任务".dimmed()
            );
        }
    };

    // 2. 解析任务 ID
    let task_id_str = arg.trim();
    if task_id_str.is_empty() {
        return format!(
            "{}\n使用方式: /task_output <id>",
            "[ERROR] 请提供任务 ID".red()
        );
    }

    let task_id: usize = match task_id_str.parse() {
        Ok(id) => id,
        Err(_) => {
            return format!(
                "{}\n使用方式: /task_output <id>",
                format!("[ERROR] 无效的任务 ID: {}", task_id_str).red()
            );
        }
    };

    // 3. 验证任务 ID 有效性
    if task_id >= result.task_results.len() {
        return format!(
            "{}\n有效范围: 0-{}",
            format!("[ERROR] 任务 ID {} 不存在", task_id).red(),
            result.task_results.len() - 1
        );
    }

    // 4. 获取任务结果
    let task_result = &result.task_results[task_id];

    // 5. 显示完整输出
    let mut output = String::new();

    // 分隔线
    output.push_str(&format!("\n{}\n", "=".repeat(80).dimmed()));

    // 任务状态和名称
    let task_icon = match task_result.status {
        TaskStatus::Success => "✓".green(),
        TaskStatus::Failed => "✗".red(),
        TaskStatus::Skipped => "⊘".yellow(),
        TaskStatus::Cancelled => "⊗".dimmed(),
        _ => "•".dimmed(),
    };

    output.push_str(&format!(
        "{} {} {}\n",
        task_icon,
        task_result.task.name.bold(),
        format!("(ID: {})", task_id).dimmed()
    ));

    // 执行时间（如果配置启用）
    if config.task.display.show_task_duration {
        output.push_str(&format!(
            "  {} {}s\n",
            "执行时间:".dimmed(),
            task_result.duration
        ));
    }

    output.push_str(&format!("{}\n", "=".repeat(80).dimmed()));

    // 输出内容（完整，无行数限制）
    if !task_result.output.trim().is_empty() {
        output.push('\n');
        output.push_str(&task_result.output);
        output.push('\n');
    } else {
        output.push_str(&format!("\n{}\n", "（无输出）".dimmed()));
    }

    // 错误信息
    if let Some(error) = &task_result.error {
        output.push('\n');
        output.push_str(&format!("{} {}\n", "错误:".red().bold(), error.red()));
    }

    // 底部分隔线
    output.push_str(&format!("\n{}\n", "=".repeat(80).dimmed()));

    output
}

// ============================================================================
// ✨ v1.22.0 Phase 1: 任务持久化命令处理函数
// ============================================================================

/// 执行 /task_save 命令 - 保存当前任务
async fn task_save_command(manager: &Arc<RwLock<TaskManager>>, arg: &str) -> String {
    let mgr = manager.read().await;

    // 检查是否有当前计划
    if mgr.get_current_plan().is_none() {
        return format!(
            "{}\n{}",
            "[ERROR] 无待保存计划".red(),
            "提示: 请先使用 /plan <目标> 创建计划".dimmed()
        );
    }

    // 获取可选的任务名称
    let name = if arg.trim().is_empty() {
        None
    } else {
        Some(arg.trim().to_string())
    };

    // 保存任务
    match mgr.save_current(name.clone()) {
        Ok(filepath) => {
            let mut output = String::new();

            output.push_str(&format!("\n{} 任务已保存\n", "✓".green()));

            // 显示保存信息
            if let Some(name) = &name {
                output.push_str(&format!("  {} {}\n", "名称:".dimmed(), name.bold()));
            }

            output.push_str(&format!(
                "  {} {}\n",
                "路径:".dimmed(),
                filepath.display().to_string().dimmed()
            ));

            output.push_str(&format!(
                "\n{}\n",
                format!("使用 {} 查看所有保存的任务", "/task_list".cyan()).dimmed()
            ));

            output
        }
        Err(e) => {
            format!("{}\n错误: {}", "[ERROR] 保存失败".red(), e)
        }
    }
}

/// 执行 /task_list 命令 - 列出所有保存的任务
async fn task_list_command() -> String {
    match SavedTask::list_all() {
        Ok(tasks) => {
            if tasks.is_empty() {
                return format!(
                    "{}\n{}",
                    "暂无保存的任务".dimmed(),
                    "提示: 使用 /task_save [名称] 保存当前任务".dimmed()
                );
            }

            let mut output = String::new();

            output.push_str(&format!("\n{} 保存的任务\n", tasks.len().to_string().bold()));

            for (idx, (_path, task)) in tasks.iter().enumerate() {
                let name_display = if let Some(name) = &task.name {
                    format!("{} ", name.bold())
                } else {
                    String::new()
                };

                let time_display = task
                    .created_at
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
                    .dimmed();

                let status = if task.result.is_some() {
                    "✓".green()
                } else {
                    "○".dimmed()
                };

                output.push_str(&format!(
                    "  {} {} {}{}\n",
                    format!("[{}]", idx).dimmed(),
                    status,
                    name_display,
                    time_display
                ));

                // 显示目标（缩短显示）
                let goal = if task.goal.len() > 60 {
                    format!("{}...", &task.goal[..57])
                } else {
                    task.goal.clone()
                };

                output.push_str(&format!("     {}\n", goal.dimmed()));
            }

            output.push_str(&format!(
                "\n{}\n",
                format!("使用 {} <id> 加载任务", "/task_load".cyan()).dimmed()
            ));

            output
        }
        Err(e) => {
            format!("{}\n错误: {}", "[ERROR] 无法列出任务".red(), e)
        }
    }
}

/// 执行 /task_load 命令 - 加载保存的任务
async fn task_load_command(manager: &Arc<RwLock<TaskManager>>, arg: &str) -> String {
    // 解析任务 ID
    let task_id_str = arg.trim();
    if task_id_str.is_empty() {
        return format!(
            "{}\n使用方式: /task_load <id>",
            "[ERROR] 请提供任务 ID".red()
        );
    }

    let task_id: usize = match task_id_str.parse() {
        Ok(id) => id,
        Err(_) => {
            return format!(
                "{}\n使用方式: /task_load <id>",
                format!("[ERROR] 无效的任务 ID: {}", task_id_str).red()
            );
        }
    };

    // 获取所有保存的任务
    let tasks = match SavedTask::list_all() {
        Ok(tasks) => tasks,
        Err(e) => {
            return format!("{}\n错误: {}", "[ERROR] 无法读取任务列表".red(), e);
        }
    };

    // 验证 ID 有效性
    if task_id >= tasks.len() {
        return format!(
            "{}\n有效范围: 0-{}\n提示: 使用 /task_list 查看所有任务",
            format!("[ERROR] 任务 ID {} 不存在", task_id).red(),
            tasks.len() - 1
        );
    }

    // 加载任务
    let (_path, task) = &tasks[task_id];

    {
        let mut mgr = manager.write().await;
        mgr.load_task(task.clone());
    }

    let mut output = String::new();

    output.push_str(&format!("\n{} 任务已加载\n", "✓".green()));

    if let Some(name) = &task.name {
        output.push_str(&format!("  {} {}\n", "名称:".dimmed(), name.bold()));
    }

    output.push_str(&format!(
        "  {} {}\n",
        "目标:".dimmed(),
        task.goal.bold()
    ));

    output.push_str(&format!(
        "  {} {} 任务\n",
        "计划:".dimmed(),
        task.plan.total_tasks()
    ));

    if task.result.is_some() {
        output.push_str(&format!("  {} {}\n", "状态:".dimmed(), "已执行".green()));
    } else {
        output.push_str(&format!("  {} {}\n", "状态:".dimmed(), "未执行".yellow()));
    }

    output.push_str(&format!(
        "\n{}\n",
        format!(
            "使用 {} 查看计划，{} 执行任务",
            "/tasks".cyan(),
            "/execute".cyan()
        )
        .dimmed()
    ));

    output
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

        let plan = ExecutionPlan::new("test goal", vec![]);

        manager.save_plan(plan);
        assert!(manager.current_plan.is_some());
    }

    // ============================================================================
    // ✨ v1.22.0 Phase 1: 任务持久化测试
    // ============================================================================

    #[test]
    fn test_saved_task_creation() {
        let plan = ExecutionPlan::new("test goal", vec![]);
        let result = ExecutionResult {
            plan_id: "test-plan".to_string(),
            total_tasks: 1,
            completed_tasks: 1,
            failed_tasks: 0,
            skipped_tasks: 0,
            total_time: 10,
            task_results: vec![],
        };

        let saved = SavedTask::new("test goal".to_string(), plan.clone(), Some(result.clone()));

        assert!(!saved.id.is_empty());
        assert_eq!(saved.goal, "test goal");
        assert_eq!(saved.plan.goal, "test goal");
        assert!(saved.result.is_some());
        assert!(saved.name.is_none());
    }

    #[test]
    fn test_saved_task_with_name() {
        let plan = ExecutionPlan::new("test goal", vec![]);
        let saved = SavedTask::new("test goal".to_string(), plan, None)
            .with_name("my_task".to_string());

        assert_eq!(saved.name, Some("my_task".to_string()));
    }

    #[test]
    fn test_saved_task_save_load() -> anyhow::Result<()> {
        use tempfile::TempDir;

        // 使用临时目录进行测试
        let temp_dir = TempDir::new()?;
        let task_file = temp_dir.path().join("test_task.json");

        // 创建一个测试任务
        let plan = ExecutionPlan::new("test save/load", vec![]);
        let saved = SavedTask::new("test save/load".to_string(), plan, None)
            .with_name("test_task".to_string());

        // 序列化保存
        let json = serde_json::to_string_pretty(&saved)?;
        std::fs::write(&task_file, json)?;

        // 从文件加载
        let loaded = SavedTask::load_from_file(&task_file)?;

        // 验证
        assert_eq!(loaded.id, saved.id);
        assert_eq!(loaded.name, saved.name);
        assert_eq!(loaded.goal, saved.goal);
        assert_eq!(loaded.plan.goal, saved.plan.goal);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_manager_save_current_no_plan() {
        let manager = TaskManager::new();

        let result = manager.save_current(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无待保存计划"));
    }

    #[tokio::test]
    async fn test_task_manager_save_current_success() -> anyhow::Result<()> {
        use tempfile::TempDir;

        // 使用临时目录
        let _temp_dir = TempDir::new()?;

        let mut manager = TaskManager::new();
        let plan = ExecutionPlan::new("test save", vec![]);
        manager.save_plan(plan);

        // 注意：此测试会实际创建 ~/.realconsole/tasks/ 目录
        // 在 CI 环境中可能需要 mock
        let result = manager.save_current(Some("test_save".to_string()));

        // 如果成功，清理文件
        if let Ok(filepath) = result {
            let _ = std::fs::remove_file(filepath);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_task_manager_load_task() {
        let mut manager = TaskManager::new();

        let plan = ExecutionPlan::new("loaded goal", vec![]);
        let result = ExecutionResult {
            plan_id: "loaded-plan".to_string(),
            total_tasks: 1,
            completed_tasks: 1,
            failed_tasks: 0,
            skipped_tasks: 0,
            total_time: 10,
            task_results: vec![],
        };

        let saved = SavedTask::new("loaded goal".to_string(), plan, Some(result));

        manager.load_task(saved.clone());

        assert!(manager.get_current_plan().is_some());
        assert_eq!(manager.get_current_plan().unwrap().goal, "loaded goal");
        assert!(manager.get_last_result().is_some());
    }

    // ============================================================================
    // ✨ v1.21.0 Phase 2: /task_output 命令测试
    // ============================================================================

    #[tokio::test]
    async fn test_view_task_output_no_result() {
        // 测试：没有执行结果时的错误处理
        let manager = Arc::new(RwLock::new(TaskManager::new()));
        let config = Arc::new(crate::config::Config::default());

        let output = view_task_output_command(&manager, &config, "0").await;

        assert!(output.contains("ERROR"));
        assert!(output.contains("没有可用的任务执行结果"));
    }

    #[tokio::test]
    async fn test_view_task_output_invalid_id_format() {
        // 测试：无效的任务 ID 格式
        let manager = Arc::new(RwLock::new(TaskManager::new()));
        let config = Arc::new(crate::config::Config::default());

        // 创建一个测试结果
        let mut mgr = manager.write().await;
        let plan = crate::task::ExecutionPlan::new("test", vec![]);
        let result = crate::task::ExecutionResult {
            plan_id: "test-plan".to_string(),
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            skipped_tasks: 0,
            total_time: 0,
            task_results: vec![],
        };
        mgr.save_plan(plan);
        mgr.save_result(result);
        drop(mgr);

        let output = view_task_output_command(&manager, &config, "abc").await;

        assert!(output.contains("ERROR"));
        assert!(output.contains("无效的任务 ID"));
    }

    #[tokio::test]
    async fn test_view_task_output_id_out_of_range() {
        // 测试：任务 ID 超出范围
        let manager = Arc::new(RwLock::new(TaskManager::new()));
        let config = Arc::new(crate::config::Config::default());

        // 创建一个包含 2 个任务的结果
        let mut mgr = manager.write().await;
        let plan = crate::task::ExecutionPlan::new("test", vec![]);
        let now = Utc::now();
        let task1 = TaskExecutionResult {
            task: SubTask::new("t1", "Task 1", "echo 'test1'"),
            status: TaskStatus::Success,
            output: "test1".to_string(),
            error: None,
            start_time: now,
            end_time: now,
            duration: 0,
        };
        let task2 = TaskExecutionResult {
            task: SubTask::new("t2", "Task 2", "echo 'test2'"),
            status: TaskStatus::Success,
            output: "test2".to_string(),
            error: None,
            start_time: now,
            end_time: now,
            duration: 0,
        };
        let result = crate::task::ExecutionResult {
            plan_id: "test-plan".to_string(),
            total_tasks: 2,
            completed_tasks: 2,
            failed_tasks: 0,
            skipped_tasks: 0,
            total_time: 0,
            task_results: vec![task1, task2],
        };
        mgr.save_plan(plan);
        mgr.save_result(result);
        drop(mgr);

        let output = view_task_output_command(&manager, &config, "5").await;

        assert!(output.contains("ERROR"));
        assert!(output.contains("任务 ID 5 不存在"));
        assert!(output.contains("有效范围: 0-1"));
    }

    #[tokio::test]
    async fn test_view_task_output_success() {
        // 测试：正常查看任务输出
        let manager = Arc::new(RwLock::new(TaskManager::new()));
        let config = Arc::new(crate::config::Config::default());

        // 创建一个包含 2 个任务的结果
        let mut mgr = manager.write().await;
        let plan = crate::task::ExecutionPlan::new("test goal", vec![]);
        let now = Utc::now();
        let task1 = TaskExecutionResult {
            task: SubTask::new("t1", "Calculate Sum", "echo 'Sum: 55'"),
            status: TaskStatus::Success,
            output: "Sum: 55".to_string(),
            error: None,
            start_time: now,
            end_time: now,
            duration: 0,
        };
        let task2 = TaskExecutionResult {
            task: SubTask::new("t2", "Multiply", "echo 'Result: 110'"),
            status: TaskStatus::Failed,
            output: "Result: 110".to_string(),
            error: Some("Command failed with exit code 1".to_string()),
            start_time: now,
            end_time: now,
            duration: 0,
        };
        let result = crate::task::ExecutionResult {
            plan_id: "test-plan".to_string(),
            total_tasks: 2,
            completed_tasks: 1,
            failed_tasks: 1,
            skipped_tasks: 0,
            total_time: 0,
            task_results: vec![task1, task2],
        };
        mgr.save_plan(plan);
        mgr.save_result(result);
        drop(mgr);

        // 测试查看第一个任务
        let output1 = view_task_output_command(&manager, &config, "0").await;
        assert!(output1.contains("Calculate Sum"));
        assert!(output1.contains("ID: 0"));
        assert!(output1.contains("Sum: 55"));
        assert!(output1.contains("0s"));

        // 测试查看第二个任务
        let output2 = view_task_output_command(&manager, &config, "1").await;
        assert!(output2.contains("Multiply"));
        assert!(output2.contains("ID: 1"));
        assert!(output2.contains("Result: 110"));
        assert!(output2.contains("0s"));
        assert!(output2.contains("错误"));
        assert!(output2.contains("Command failed with exit code 1"));
    }

    #[tokio::test]
    async fn test_view_task_output_empty_output() {
        // 测试：任务没有输出的情况
        let manager = Arc::new(RwLock::new(TaskManager::new()));
        let config = Arc::new(crate::config::Config::default());

        let mut mgr = manager.write().await;
        let plan = crate::task::ExecutionPlan::new("test", vec![]);
        let now = Utc::now();
        let task = TaskExecutionResult {
            task: SubTask::new("t1", "Empty Task", "true"),
            status: TaskStatus::Success,
            output: "".to_string(),
            error: None,
            start_time: now,
            end_time: now,
            duration: 0,
        };
        let result = crate::task::ExecutionResult {
            plan_id: "test-plan".to_string(),
            total_tasks: 1,
            completed_tasks: 1,
            failed_tasks: 0,
            skipped_tasks: 0,
            total_time: 0,
            task_results: vec![task],
        };
        mgr.save_plan(plan);
        mgr.save_result(result);
        drop(mgr);

        let output = view_task_output_command(&manager, &config, "0").await;

        assert!(output.contains("Empty Task"));
        assert!(output.contains("（无输出）"));
    }
}
