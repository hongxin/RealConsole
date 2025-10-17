# Phase 10: 任务分解与规划系统 - 架构设计

**Date**: 2025-01-17
**Version**: 0.10.0 (规划中)
**Status**: 🎯 **设计阶段**

## 🎯 核心目标

构建一个智能化的任务分解、规划和执行系统，使 RealConsole 能够：

1. **理解复杂任务**: 接收用户的高层次目标描述
2. **智能分解**: 将复杂任务分解为可执行的子任务序列
3. **依赖分析**: 识别任务间的依赖关系和并行机会
4. **计划生成**: 生成最优执行计划（串行/并行）
5. **自动执行**: 自动化执行任务序列，处理错误和恢复
6. **进度反馈**: 实时显示执行进度和状态

## 🏗️ 系统架构

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                         User Input                           │
│                 "部署一个 React 应用到生产环境"                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Agent (handle_text)                       │
│                  检测任务分解需求 → /plan                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                 TaskDecomposer (一分为三·分解态)              │
│  ┌─────────────┐   ┌──────────────┐   ┌─────────────┐      │
│  │ 意图理解     │→  │ LLM 分解      │→  │ 结构化输出  │      │
│  │ Intent Parse │   │ Task Breakdown│   │ Structured  │      │
│  └─────────────┘   └──────────────┘   └─────────────┘      │
│                                                               │
│  Output: Vec<SubTask>                                        │
│    - name: "安装依赖"                                         │
│    - command: "npm install"                                  │
│    - description: "安装项目依赖包"                            │
│    - estimated_time: 30s                                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                 TaskPlanner (一分为三·规划态)                │
│  ┌─────────────┐   ┌──────────────┐   ┌─────────────┐      │
│  │ 依赖分析     │→  │ 拓扑排序      │→  │ 并行识别    │      │
│  │ Dependency   │   │ Topological  │   │ Parallel    │      │
│  └─────────────┘   └──────────────┘   └─────────────┘      │
│                                                               │
│  Output: ExecutionPlan                                       │
│    - stages: [[task1, task2], [task3], [task4, task5]]      │
│    - total_estimated_time: 120s                              │
│    - parallel_stages: 2                                      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                TaskExecutor (一分为三·执行态)                │
│  ┌─────────────┐   ┌──────────────┐   ┌─────────────┐      │
│  │ 任务执行     │→  │ 错误处理      │→  │ 进度反馈    │      │
│  │ Execute      │   │ Error Handle │   │ Progress    │      │
│  └─────────────┘   └──────────────┘   └─────────────┘      │
│                                                               │
│  Features:                                                   │
│    - 串行/并行执行                                            │
│    - 实时进度显示                                             │
│    - 错误回退和恢复                                           │
│    - 中断和继续                                               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     Execution Result                         │
│  ✓ 所有任务完成 (5/5) - 耗时 95s                             │
│  │ ✓ 安装依赖 (30s)                                          │
│  │ ✓ 运行测试 (20s)                                          │
│  │ ✓ 构建生产包 (40s)                                        │
│  │ ✗ 部署到服务器 (失败)                                      │
│  │   └─ 建议: 检查服务器连接                                  │
│  │ ⊙ 验证部署 (跳过)                                         │
└─────────────────────────────────────────────────────────────┘
```

## 📦 核心组件设计

### 1. TaskDecomposer (任务分解器)

**职责**: 将用户的高层次目标分解为可执行的子任务序列

#### 数据结构

```rust
/// 子任务定义
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

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 任务分解器
pub struct TaskDecomposer {
    /// LLM 客户端
    llm: Arc<dyn LlmClient>,

    /// 任务模板库（常见任务模式）
    templates: TaskTemplateLibrary,

    /// 历史分解记录（用于学习）
    history: Arc<RwLock<Vec<DecompositionRecord>>>,
}

impl TaskDecomposer {
    /// 分解任务
    ///
    /// # Arguments
    /// * `goal` - 用户目标描述
    /// * `context` - 当前上下文信息（工作目录、环境变量等）
    ///
    /// # Returns
    /// * `Vec<SubTask>` - 子任务列表
    pub async fn decompose(
        &self,
        goal: &str,
        context: &ExecutionContext,
    ) -> Result<Vec<SubTask>, TaskError> {
        // 1. 意图理解：分析用户目标
        let intent = self.parse_intent(goal)?;

        // 2. 模板匹配：尝试匹配已知任务模板
        if let Some(template) = self.templates.find_match(&intent) {
            return Ok(template.instantiate(context));
        }

        // 3. LLM 分解：使用 LLM 智能分解
        let subtasks = self.decompose_with_llm(goal, context).await?;

        // 4. 验证和优化
        let validated_tasks = self.validate_tasks(subtasks)?;

        // 5. 记录历史（用于学习）
        self.record_decomposition(goal, &validated_tasks).await;

        Ok(validated_tasks)
    }

    /// 使用 LLM 分解任务
    async fn decompose_with_llm(
        &self,
        goal: &str,
        context: &ExecutionContext,
    ) -> Result<Vec<SubTask>, TaskError> {
        let prompt = format!(
            r#"你是一个任务分解专家。请将以下目标分解为可执行的子任务序列。

目标: {}

当前上下文:
- 工作目录: {}
- 系统: {}
- Shell: {}

请按以下 JSON 格式输出任务列表:
{{
  "tasks": [
    {{
      "id": "task1",
      "name": "任务名称",
      "description": "任务描述",
      "command": "要执行的命令",
      "estimated_time": 30,
      "depends_on": [],
      "task_type": "Shell",
      "skippable": false
    }}
  ]
}}

要求:
1. 任务应该具体可执行
2. 命令应该是有效的 shell 命令
3. 正确标识任务间的依赖关系
4. 提供合理的时间估计
5. 按执行顺序排列任务
"#,
            goal,
            context.working_dir,
            context.os,
            context.shell
        );

        // 调用 LLM
        let response = self.llm.chat(&prompt).await
            .map_err(|e| TaskError::LlmError(e.to_string()))?;

        // 解析 JSON 响应
        let parsed: TaskListResponse = serde_json::from_str(&response)
            .map_err(|e| TaskError::ParseError(e.to_string()))?;

        Ok(parsed.tasks)
    }
}
```

### 2. TaskPlanner (任务规划器)

**职责**: 分析任务依赖，生成最优执行计划

#### 数据结构

```rust
/// 执行计划
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 执行阶段（每个阶段内的任务可并行执行）
    pub stages: Vec<ExecutionStage>,

    /// 总估计时间（秒）
    pub total_estimated_time: u32,

    /// 并行阶段数量
    pub parallel_stages: usize,

    /// 依赖关系图
    pub dependency_graph: DependencyGraph,
}

/// 执行阶段
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

/// 执行模式
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    /// 串行执行
    Sequential,
    /// 并行执行
    Parallel,
}

/// 依赖图
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// 节点（任务）
    nodes: HashMap<String, SubTask>,

    /// 边（依赖关系）
    edges: HashMap<String, Vec<String>>,
}

/// 任务规划器
pub struct TaskPlanner {
    /// 最大并行度
    max_parallelism: usize,

    /// 是否允许并行执行
    allow_parallel: bool,
}

impl TaskPlanner {
    /// 生成执行计划
    ///
    /// # Arguments
    /// * `tasks` - 子任务列表
    ///
    /// # Returns
    /// * `ExecutionPlan` - 执行计划
    pub fn plan(&self, tasks: Vec<SubTask>) -> Result<ExecutionPlan, TaskError> {
        // 1. 构建依赖图
        let dep_graph = self.build_dependency_graph(&tasks)?;

        // 2. 拓扑排序（检测循环依赖）
        let sorted_tasks = self.topological_sort(&dep_graph)?;

        // 3. 识别并行机会
        let stages = if self.allow_parallel {
            self.identify_parallel_stages(&sorted_tasks, &dep_graph)?
        } else {
            self.sequential_stages(&sorted_tasks)
        };

        // 4. 计算总时间
        let total_time = stages.iter().map(|s| s.estimated_time).sum();

        Ok(ExecutionPlan {
            stages,
            total_estimated_time: total_time,
            parallel_stages: stages.iter().filter(|s| matches!(s.execution_mode, ExecutionMode::Parallel)).count(),
            dependency_graph: dep_graph,
        })
    }

    /// 拓扑排序（Kahn 算法）
    fn topological_sort(&self, graph: &DependencyGraph) -> Result<Vec<SubTask>, TaskError> {
        // 计算入度
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for node_id in graph.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
        }
        for deps in graph.edges.values() {
            for dep in deps {
                *in_degree.get_mut(dep).unwrap() += 1;
            }
        }

        // 找到所有入度为 0 的节点
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            let task = graph.nodes.get(&node_id).unwrap().clone();
            sorted.push(task);

            // 减少后继节点的入度
            if let Some(dependencies) = graph.edges.get(&node_id) {
                for dep_id in dependencies {
                    let degree = in_degree.get_mut(dep_id).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dep_id.clone());
                    }
                }
            }
        }

        // 检查是否有循环依赖
        if sorted.len() != graph.nodes.len() {
            return Err(TaskError::CyclicDependency);
        }

        Ok(sorted)
    }

    /// 识别并行执行阶段
    fn identify_parallel_stages(
        &self,
        sorted_tasks: &[SubTask],
        graph: &DependencyGraph,
    ) -> Result<Vec<ExecutionStage>, TaskError> {
        let mut stages = Vec::new();
        let mut stage_num = 0;
        let mut remaining: HashSet<String> = sorted_tasks.iter().map(|t| t.id.clone()).collect();
        let mut completed: HashSet<String> = HashSet::new();

        while !remaining.is_empty() {
            // 找出所有依赖已满足的任务（可以在本阶段执行）
            let ready_tasks: Vec<SubTask> = sorted_tasks
                .iter()
                .filter(|task| {
                    remaining.contains(&task.id)
                        && task.depends_on.iter().all(|dep| completed.contains(dep))
                })
                .cloned()
                .collect();

            if ready_tasks.is_empty() {
                return Err(TaskError::UnresolvableDependencies);
            }

            // 限制并行度
            let tasks_in_stage: Vec<SubTask> = ready_tasks
                .into_iter()
                .take(self.max_parallelism)
                .collect();

            let max_time = tasks_in_stage.iter().map(|t| t.estimated_time).max().unwrap_or(0);
            let execution_mode = if tasks_in_stage.len() > 1 {
                ExecutionMode::Parallel
            } else {
                ExecutionMode::Sequential
            };

            stages.push(ExecutionStage {
                stage_num,
                tasks: tasks_in_stage.clone(),
                estimated_time: max_time,
                execution_mode,
            });

            // 标记为已完成
            for task in &tasks_in_stage {
                remaining.remove(&task.id);
                completed.insert(task.id.clone());
            }

            stage_num += 1;
        }

        Ok(stages)
    }
}
```

### 3. TaskExecutor (任务执行器)

**职责**: 执行任务计划，处理错误和进度反馈

#### 数据结构

```rust
/// 任务执行器
pub struct TaskExecutor {
    /// Shell 执行器
    shell_executor: Arc<ShellExecutorWithFixer>,

    /// 执行历史
    history: Arc<RwLock<Vec<TaskExecutionRecord>>>,

    /// 进度回调
    progress_callback: Option<Arc<dyn Fn(TaskProgress) + Send + Sync>>,
}

/// 任务执行记录
#[derive(Debug, Clone)]
pub struct TaskExecutionRecord {
    pub task_id: String,
    pub status: TaskStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub output: String,
    pub error: Option<String>,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 执行进度
#[derive(Debug, Clone)]
pub struct TaskProgress {
    /// 当前阶段
    pub current_stage: usize,
    /// 总阶段数
    pub total_stages: usize,
    /// 当前任务
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

impl TaskExecutor {
    /// 执行任务计划
    ///
    /// # Arguments
    /// * `plan` - 执行计划
    ///
    /// # Returns
    /// * `ExecutionResult` - 执行结果
    pub async fn execute(&self, plan: ExecutionPlan) -> Result<ExecutionResult, TaskError> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        let total_tasks = plan.stages.iter().map(|s| s.tasks.len()).sum();
        let mut completed_tasks = 0;

        // 逐阶段执行
        for (stage_idx, stage) in plan.stages.iter().enumerate() {
            // 更新进度
            if let Some(ref callback) = self.progress_callback {
                callback(TaskProgress {
                    current_stage: stage_idx + 1,
                    total_stages: plan.stages.len(),
                    current_task: stage.tasks[0].name.clone(),
                    completed_tasks,
                    total_tasks,
                    elapsed_time: start_time.elapsed().as_secs() as u32,
                    estimated_remaining: plan.total_estimated_time.saturating_sub(start_time.elapsed().as_secs() as u32),
                });
            }

            // 执行本阶段任务
            let stage_results = match stage.execution_mode {
                ExecutionMode::Sequential => {
                    self.execute_sequential(&stage.tasks).await?
                }
                ExecutionMode::Parallel => {
                    self.execute_parallel(&stage.tasks).await?
                }
            };

            completed_tasks += stage.tasks.len();

            // 检查是否有关键任务失败
            for result in &stage_results {
                if result.status == TaskStatus::Failed && !result.task.skippable {
                    return Err(TaskError::CriticalTaskFailed(result.task.name.clone()));
                }
            }

            results.extend(stage_results);
        }

        Ok(ExecutionResult {
            total_tasks,
            completed_tasks,
            failed_tasks: results.iter().filter(|r| r.status == TaskStatus::Failed).count(),
            skipped_tasks: results.iter().filter(|r| r.status == TaskStatus::Skipped).count(),
            total_time: start_time.elapsed().as_secs() as u32,
            task_results: results,
        })
    }

    /// 串行执行任务
    async fn execute_sequential(
        &self,
        tasks: &[SubTask],
    ) -> Result<Vec<TaskResult>, TaskError> {
        let mut results = Vec::new();

        for task in tasks {
            let result = self.execute_single_task(task).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 并行执行任务
    async fn execute_parallel(
        &self,
        tasks: &[SubTask],
    ) -> Result<Vec<TaskResult>, TaskError> {
        let futures: Vec<_> = tasks
            .iter()
            .map(|task| self.execute_single_task(task))
            .collect();

        let results = futures::future::join_all(futures).await;

        results.into_iter().collect()
    }

    /// 执行单个任务
    async fn execute_single_task(&self, task: &SubTask) -> Result<TaskResult, TaskError> {
        let start_time = Utc::now();

        // 根据任务类型执行
        let execution_result = match task.task_type {
            TaskType::Shell => {
                self.shell_executor.execute_with_analysis(&task.command).await
            }
            TaskType::FileOperation => {
                // TODO: 实现文件操作
                todo!()
            }
            TaskType::Network => {
                // TODO: 实现网络请求
                todo!()
            }
            _ => {
                return Err(TaskError::UnsupportedTaskType);
            }
        };

        let end_time = Utc::now();
        let status = if execution_result.success {
            TaskStatus::Success
        } else {
            TaskStatus::Failed
        };

        Ok(TaskResult {
            task: task.clone(),
            status,
            output: execution_result.output,
            error: execution_result.error_analysis.map(|a| format!("{:?}", a)),
            start_time,
            end_time,
            duration: (end_time - start_time).num_seconds() as u32,
        })
    }
}
```

## 🔄 执行流程

### 完整流程示例

```rust
// 用户输入
let goal = "部署一个 React 应用到生产环境";

// 1. 任务分解
let decomposer = TaskDecomposer::new(llm_client);
let context = ExecutionContext::current();
let subtasks = decomposer.decompose(goal, &context).await?;

// subtasks:
// - task1: 安装依赖 (npm install)
// - task2: 运行测试 (npm test)
// - task3: 构建生产包 (npm run build)
// - task4: 部署到服务器 (scp build/* server:/var/www/)
// - task5: 重启服务 (ssh server "pm2 restart app")

// 2. 任务规划
let planner = TaskPlanner::new();
let plan = planner.plan(subtasks)?;

// plan.stages:
// Stage 0: [task1]  (Sequential, 30s)
// Stage 1: [task2, task3]  (Parallel, max(20s, 40s) = 40s)
// Stage 2: [task4]  (Sequential, 15s)
// Stage 3: [task5]  (Sequential, 5s)
// Total: 90s (vs 110s if all sequential)

// 3. 任务执行
let executor = TaskExecutor::new(shell_executor);
executor.set_progress_callback(|progress| {
    println!("进度: {}/{} 任务完成",
        progress.completed_tasks, progress.total_tasks);
});

let result = executor.execute(plan).await?;

// 4. 结果展示
println!("✓ 部署完成!");
println!("  总任务: {}", result.total_tasks);
println!("  成功: {}", result.completed_tasks - result.failed_tasks);
println!("  失败: {}", result.failed_tasks);
println!("  耗时: {}s", result.total_time);
```

## 🎨 用户交互设计

### /plan 命令

```bash
> /plan 部署一个 React 应用到生产环境

🤔 正在分解任务...

📋 任务分解结果:
  1. 安装依赖
     └─ npm install
     └─ 估计: 30s

  2. 运行测试
     └─ npm test
     └─ 估计: 20s
     └─ 依赖: [1]

  3. 构建生产包
     └─ npm run build
     └─ 估计: 40s
     └─ 依赖: [1]

  4. 部署到服务器
     └─ scp build/* server:/var/www/
     └─ 估计: 15s
     └─ 依赖: [3]

  5. 重启服务
     └─ ssh server "pm2 restart app"
     └─ 估计: 5s
     └─ 依赖: [4]

📊 执行计划:
  阶段 1: [任务1]  (串行, 30s)
  阶段 2: [任务2, 任务3]  (并行, 40s)
  阶段 3: [任务4]  (串行, 15s)
  阶段 4: [任务5]  (串行, 5s)

  总计: 5 个任务, 4 个阶段, 90s (节省 20s)

是否执行? [y/N]:
```

### /execute 命令

```bash
> /execute

🚀 开始执行任务计划...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  执行进度: ████████░░░░░░░░░░░░ 2/5 (40%)
  当前阶段: 2/4
  已用时间: 35s / 估计剩余: 55s
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

阶段 1: ✓ 完成 (30s)
  ✓ 任务1: 安装依赖 (30s)

阶段 2: ⚡ 执行中 (5s)
  ✓ 任务2: 运行测试 (20s)
  ⏳ 任务3: 构建生产包 (5s / 40s)

阶段 3: ⊙ 等待
  ⊙ 任务4: 部署到服务器

阶段 4: ⊙ 等待
  ⊙ 任务5: 重启服务
```

## 🔧 Agent 集成

```rust
// src/agent.rs

impl Agent {
    /// 处理 /plan 命令
    fn handle_plan_command(&self, goal: &str) -> String {
        // 1. 任务分解
        let decomposer = TaskDecomposer::new(self.llm_manager());
        let context = ExecutionContext::current();

        let subtasks = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                decomposer.decompose(goal, &context).await
            })
        }).expect("Failed to decompose task");

        // 2. 任务规划
        let planner = TaskPlanner::new();
        let plan = planner.plan(subtasks).expect("Failed to create plan");

        // 3. 保存计划到 Agent 状态
        self.save_current_plan(plan.clone());

        // 4. 显示计划
        self.display_plan(&plan)
    }

    /// 处理 /execute 命令
    fn handle_execute_command(&self) -> String {
        // 1. 获取当前计划
        let plan = match self.get_current_plan() {
            Some(p) => p,
            None => return "没有可执行的计划。请先使用 /plan 命令创建计划。".to_string(),
        };

        // 2. 确认执行
        if !self.confirm_execution() {
            return "执行已取消".to_string();
        }

        // 3. 执行计划
        let executor = TaskExecutor::new(self.shell_executor_with_fixer.clone());
        executor.set_progress_callback(Arc::new(|progress| {
            // 实时更新进度显示
            self.update_progress_display(progress);
        }));

        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                executor.execute(plan).await
            })
        }).expect("Failed to execute plan");

        // 4. 显示结果
        self.display_execution_result(&result)
    }
}
```

## 🧪 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_task_decomposition() {
        let decomposer = TaskDecomposer::new(mock_llm());
        let tasks = decomposer.decompose("run tests and build", &context()).await.unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "run tests");
        assert_eq!(tasks[1].depends_on, vec![tasks[0].id.clone()]);
    }

    #[test]
    fn test_dependency_graph() {
        let tasks = vec![
            SubTask { id: "t1", depends_on: vec![] },
            SubTask { id: "t2", depends_on: vec!["t1"] },
            SubTask { id: "t3", depends_on: vec!["t1"] },
        ];

        let planner = TaskPlanner::new();
        let plan = planner.plan(tasks).unwrap();

        assert_eq!(plan.stages.len(), 2);
        assert_eq!(plan.stages[0].tasks.len(), 1);  // t1
        assert_eq!(plan.stages[1].tasks.len(), 2);  // t2, t3 (parallel)
    }

    #[test]
    fn test_cyclic_dependency_detection() {
        let tasks = vec![
            SubTask { id: "t1", depends_on: vec!["t2"] },
            SubTask { id: "t2", depends_on: vec!["t1"] },
        ];

        let planner = TaskPlanner::new();
        let result = planner.plan(tasks);

        assert!(matches!(result, Err(TaskError::CyclicDependency)));
    }
}
```

## 📊 性能优化

1. **并行执行**: 识别可并行任务，减少总执行时间
2. **缓存分解结果**: 常见任务模式缓存避免重复 LLM 调用
3. **增量计划**: 支持动态调整计划，无需重新规划
4. **流式输出**: 实时显示任务输出，提升用户体验

## 🔒 安全考虑

1. **命令验证**: 所有命令经过 shell_executor 安全检查
2. **权限控制**: 敏感操作需要用户确认
3. **沙盒执行**: 可选的容器化执行环境
4. **审计日志**: 记录所有任务执行历史

## 📝 文件结构

```
src/
├── task/
│   ├── mod.rs              # 模块导出
│   ├── decomposer.rs       # TaskDecomposer 实现
│   ├── planner.rs          # TaskPlanner 实现
│   ├── executor.rs         # TaskExecutor 实现
│   ├── types.rs            # 核心数据结构
│   ├── templates.rs        # 任务模板库
│   └── error.rs            # 错误类型定义
```

## 🎯 实施计划

### Week 1: 核心数据结构和 TaskDecomposer
- [ ] 定义核心数据结构 (SubTask, TaskType, etc.)
- [ ] 实现 TaskDecomposer 基础框架
- [ ] LLM 集成和 Prompt Engineering
- [ ] 单元测试

### Week 2: TaskPlanner 和依赖分析
- [ ] 实现依赖图构建
- [ ] 拓扑排序算法
- [ ] 并行任务识别
- [ ] 循环依赖检测
- [ ] 单元测试

### Week 3: TaskExecutor 和进度反馈
- [ ] 实现任务执行引擎
- [ ] 串行/并行执行
- [ ] 进度回调和显示
- [ ] 错误处理和恢复
- [ ] 单元测试

### Week 4: Agent 集成和端到端测试
- [ ] /plan 命令集成
- [ ] /execute 命令集成
- [ ] UI/UX 优化
- [ ] 端到端测试
- [ ] 性能优化
- [ ] 文档完善

---

**Status**: 🎯 Ready for Implementation
**Next**: 开始实现核心数据结构和 TaskDecomposer
