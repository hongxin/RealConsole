//! 任务规划器 (TaskPlanner)
//!
//! Phase 10: 任务分解与规划系统
//!
//! 负责分析任务依赖关系，生成最优执行计划

use super::error::{TaskError, TaskResult};
use super::types::{DependencyGraph, ExecutionMode, ExecutionPlan, ExecutionStage, SubTask};
use std::collections::{HashMap, HashSet, VecDeque};

/// 任务规划器
///
/// 分析任务依赖关系，生成串行或并行的执行计划
pub struct TaskPlanner {
    /// 最大并行度（同时执行的任务数）
    max_parallelism: usize,

    /// 是否允许并行执行
    allow_parallel: bool,
}

impl TaskPlanner {
    /// 创建新的任务规划器
    pub fn new() -> Self {
        Self {
            max_parallelism: 4,
            allow_parallel: true,
        }
    }

    /// 设置最大并行度
    pub fn with_max_parallelism(mut self, max: usize) -> Self {
        self.max_parallelism = max.max(1); // 至少为 1
        self
    }

    /// 禁用并行执行
    pub fn sequential_only(mut self) -> Self {
        self.allow_parallel = false;
        self
    }

    /// 生成执行计划
    ///
    /// # Arguments
    /// * `goal` - 目标描述
    /// * `tasks` - 子任务列表
    ///
    /// # Returns
    /// * `ExecutionPlan` - 执行计划
    pub fn plan(&self, goal: impl Into<String>, tasks: Vec<SubTask>) -> TaskResult<ExecutionPlan> {
        if tasks.is_empty() {
            return Err(TaskError::ParseError("任务列表为空".to_string()));
        }

        // 1. 构建依赖图
        let dep_graph = self.build_dependency_graph(&tasks)?;

        // 2. 拓扑排序（检测循环依赖）
        let sorted_tasks = self.topological_sort(&dep_graph)?;

        // 3. 识别并行机会并生成执行阶段
        let stages = if self.allow_parallel {
            self.identify_parallel_stages(&sorted_tasks, &dep_graph)?
        } else {
            self.sequential_stages(&sorted_tasks)
        };

        // 4. 创建执行计划
        Ok(ExecutionPlan::new(goal, stages))
    }

    /// 构建依赖关系图
    ///
    /// 将任务列表转换为依赖图数据结构
    fn build_dependency_graph(&self, tasks: &[SubTask]) -> TaskResult<DependencyGraph> {
        let mut graph = DependencyGraph::new();

        // 添加所有节点
        for task in tasks {
            graph.add_node(task.clone());
        }

        // 添加所有边（依赖关系）
        for task in tasks {
            for dep_id in &task.depends_on {
                // 检查依赖的任务是否存在
                if !graph.nodes.contains_key(dep_id) {
                    return Err(TaskError::ParseError(format!(
                        "任务 {} 依赖的任务 {} 不存在",
                        task.id, dep_id
                    )));
                }
                // 添加边：dep_id -> task.id
                graph.add_edge(dep_id.clone(), task.id.clone());
            }
        }

        Ok(graph)
    }

    /// 拓扑排序（Kahn 算法）
    ///
    /// 对任务进行拓扑排序，同时检测循环依赖
    ///
    /// # Arguments
    /// * `graph` - 依赖关系图
    ///
    /// # Returns
    /// * `Vec<SubTask>` - 按拓扑序排列的任务列表
    ///
    /// # Errors
    /// * `TaskError::CyclicDependency` - 检测到循环依赖
    fn topological_sort(&self, graph: &DependencyGraph) -> TaskResult<Vec<SubTask>> {
        // 计算每个节点的入度
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for node_id in graph.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
        }

        // 统计入度 - 每个任务的入度等于它依赖的任务数
        for task in graph.nodes.values() {
            *in_degree.get_mut(&task.id).unwrap() = task.depends_on.len();
        }

        // 找到所有入度为 0 的节点（没有依赖的任务）
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();

        // Kahn 算法主循环
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
    ///
    /// 将任务分组到可并行执行的阶段
    ///
    /// # 算法
    /// 1. 找出所有依赖已满足的任务（可立即执行）
    /// 2. 将这些任务分组到一个执行阶段
    /// 3. 标记这些任务为已完成
    /// 4. 重复直到所有任务都被分组
    fn identify_parallel_stages(
        &self,
        sorted_tasks: &[SubTask],
        _graph: &DependencyGraph,
    ) -> TaskResult<Vec<ExecutionStage>> {
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

            // 确定执行模式
            let execution_mode = if tasks_in_stage.len() > 1 {
                ExecutionMode::Parallel
            } else {
                ExecutionMode::Sequential
            };

            // 创建执行阶段
            stages.push(ExecutionStage::new(
                stage_num,
                tasks_in_stage.clone(),
                execution_mode,
            ));

            // 标记为已完成
            for task in &tasks_in_stage {
                remaining.remove(&task.id);
                completed.insert(task.id.clone());
            }

            stage_num += 1;
        }

        Ok(stages)
    }

    /// 生成纯串行执行阶段
    ///
    /// 每个阶段只包含一个任务
    fn sequential_stages(&self, sorted_tasks: &[SubTask]) -> Vec<ExecutionStage> {
        sorted_tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                ExecutionStage::new(i, vec![task.clone()], ExecutionMode::Sequential)
            })
            .collect()
    }

    /// 分析执行计划统计信息
    ///
    /// 返回计划的统计摘要
    pub fn analyze_plan(&self, plan: &ExecutionPlan) -> PlanAnalysis {
        let total_tasks = plan.total_tasks();
        let total_stages = plan.stages.len();
        let parallel_tasks = plan
            .stages
            .iter()
            .filter(|s| matches!(s.execution_mode, ExecutionMode::Parallel))
            .map(|s| s.tasks.len())
            .sum();

        let sequential_time: u32 = plan
            .stages
            .iter()
            .flat_map(|s| &s.tasks)
            .map(|t| t.estimated_time)
            .sum();

        let parallel_time = plan.total_estimated_time;
        let time_saved = sequential_time.saturating_sub(parallel_time);
        let efficiency = if sequential_time > 0 {
            (time_saved as f64 / sequential_time as f64) * 100.0
        } else {
            0.0
        };

        PlanAnalysis {
            total_tasks,
            total_stages,
            parallel_stages: plan.parallel_stages,
            parallel_tasks,
            sequential_time,
            parallel_time,
            time_saved,
            efficiency_gain: efficiency,
        }
    }
}

impl Default for TaskPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行计划分析结果
#[derive(Debug, Clone)]
pub struct PlanAnalysis {
    /// 总任务数
    pub total_tasks: usize,

    /// 总阶段数
    pub total_stages: usize,

    /// 并行阶段数
    pub parallel_stages: usize,

    /// 可并行执行的任务数
    pub parallel_tasks: usize,

    /// 串行执行总时间（秒）
    pub sequential_time: u32,

    /// 并行执行总时间（秒）
    pub parallel_time: u32,

    /// 节省的时间（秒）
    pub time_saved: u32,

    /// 效率提升（百分比）
    pub efficiency_gain: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_plan() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2"),
        ];

        let planner = TaskPlanner::new();
        let plan = planner.plan("test goal", tasks).unwrap();

        assert_eq!(plan.total_tasks(), 2);
        assert_eq!(plan.stages.len(), 1); // 两个任务无依赖，可并行
        assert_eq!(plan.stages[0].tasks.len(), 2);
    }

    #[test]
    fn test_sequential_dependencies() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2").with_dependency("t1"),
            SubTask::new("t3", "Task 3", "cmd3").with_dependency("t2"),
        ];

        let planner = TaskPlanner::new();
        let plan = planner.plan("test", tasks).unwrap();

        assert_eq!(plan.stages.len(), 3); // 串行依赖，3个阶段
        assert_eq!(plan.stages[0].tasks[0].id, "t1");
        assert_eq!(plan.stages[1].tasks[0].id, "t2");
        assert_eq!(plan.stages[2].tasks[0].id, "t3");
    }

    #[test]
    fn test_parallel_branches() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2").with_dependency("t1"),
            SubTask::new("t3", "Task 3", "cmd3").with_dependency("t1"),
        ];

        let planner = TaskPlanner::new();
        let plan = planner.plan("test", tasks).unwrap();

        assert_eq!(plan.stages.len(), 2);
        assert_eq!(plan.stages[0].tasks.len(), 1); // t1
        assert_eq!(plan.stages[1].tasks.len(), 2); // t2, t3 并行
        assert!(matches!(
            plan.stages[1].execution_mode,
            ExecutionMode::Parallel
        ));
    }

    #[test]
    fn test_cyclic_dependency() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1").with_dependency("t2"),
            SubTask::new("t2", "Task 2", "cmd2").with_dependency("t1"),
        ];

        let planner = TaskPlanner::new();
        let result = planner.plan("test", tasks);

        assert!(matches!(result, Err(TaskError::CyclicDependency)));
    }

    #[test]
    fn test_max_parallelism() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2"),
            SubTask::new("t3", "Task 3", "cmd3"),
            SubTask::new("t4", "Task 4", "cmd4"),
        ];

        let planner = TaskPlanner::new().with_max_parallelism(2);
        let plan = planner.plan("test", tasks).unwrap();

        // 4个无依赖任务，限制并行度为2，应该分为2个阶段
        assert_eq!(plan.stages.len(), 2);
        assert_eq!(plan.stages[0].tasks.len(), 2);
        assert_eq!(plan.stages[1].tasks.len(), 2);
    }

    #[test]
    fn test_sequential_only() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2"),
        ];

        let planner = TaskPlanner::new().sequential_only();
        let plan = planner.plan("test", tasks).unwrap();

        assert_eq!(plan.stages.len(), 2); // 强制串行
        assert!(plan
            .stages
            .iter()
            .all(|s| matches!(s.execution_mode, ExecutionMode::Sequential)));
    }

    #[test]
    fn test_invalid_dependency() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2").with_dependency("t999"), // 不存在的依赖
        ];

        let planner = TaskPlanner::new();
        let result = planner.plan("test", tasks);

        assert!(matches!(result, Err(TaskError::ParseError(_))));
    }

    #[test]
    fn test_complex_dag() {
        //     t1
        //    /  \
        //   t2  t3
        //    \  /
        //     t4
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2").with_dependency("t1"),
            SubTask::new("t3", "Task 3", "cmd3").with_dependency("t1"),
            SubTask::new("t4", "Task 4", "cmd4")
                .with_dependency("t2")
                .with_dependency("t3"),
        ];

        let planner = TaskPlanner::new();
        let plan = planner.plan("test", tasks).unwrap();

        assert_eq!(plan.stages.len(), 3);
        assert_eq!(plan.stages[0].tasks.len(), 1); // t1
        assert_eq!(plan.stages[1].tasks.len(), 2); // t2, t3 并行
        assert_eq!(plan.stages[2].tasks.len(), 1); // t4
    }

    #[test]
    fn test_empty_tasks() {
        let planner = TaskPlanner::new();
        let result = planner.plan("test", vec![]);

        assert!(matches!(result, Err(TaskError::ParseError(_))));
    }

    #[test]
    fn test_plan_analysis() {
        let tasks = vec![
            SubTask::new("t1", "Task 1", "cmd1").with_estimated_time(10),
            SubTask::new("t2", "Task 2", "cmd2")
                .with_estimated_time(20)
                .with_dependency("t1"),
            SubTask::new("t3", "Task 3", "cmd3")
                .with_estimated_time(15)
                .with_dependency("t1"),
        ];

        let planner = TaskPlanner::new();
        let plan = planner.plan("test", tasks).unwrap();
        let analysis = planner.analyze_plan(&plan);

        assert_eq!(analysis.total_tasks, 3);
        assert_eq!(analysis.total_stages, 2);
        assert_eq!(analysis.sequential_time, 45); // 10 + 20 + 15
        assert_eq!(analysis.parallel_time, 30); // 10 + max(20, 15)
        assert_eq!(analysis.time_saved, 15);
    }

    #[test]
    fn test_topological_sort() {
        let tasks = vec![
            SubTask::new("t3", "Task 3", "cmd3").with_dependency("t1"),
            SubTask::new("t1", "Task 1", "cmd1"),
            SubTask::new("t2", "Task 2", "cmd2").with_dependency("t1"),
        ];

        let planner = TaskPlanner::new();
        let graph = planner.build_dependency_graph(&tasks).unwrap();
        let sorted = planner.topological_sort(&graph).unwrap();

        // t1 应该在 t2 和 t3 之前
        let t1_idx = sorted.iter().position(|t| t.id == "t1").unwrap();
        let t2_idx = sorted.iter().position(|t| t.id == "t2").unwrap();
        let t3_idx = sorted.iter().position(|t| t.id == "t3").unwrap();

        assert!(t1_idx < t2_idx);
        assert!(t1_idx < t3_idx);
    }
}
