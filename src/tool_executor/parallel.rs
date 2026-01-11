//! v1.90.0: 并行工具执行增强
//!
//! 提供智能并行执行能力：
//! - **依赖分析**: 检测工具调用之间的数据依赖
//! - **ExecutionDAG**: 执行计划的有向无环图表示
//! - **分阶段执行**: 独立工具并行，依赖工具顺序
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::tool_executor::{DependencyAnalyzer, ToolCallRequest};
//!
//! let analyzer = DependencyAnalyzer::new();
//! let calls = vec![...];
//! let dag = analyzer.analyze(&calls);
//!
//! // 可视化执行计划
//! println!("{}", dag.visualize());
//!
//! // 按阶段执行
//! for stage in dag.stages() {
//!     // 并行执行 stage.tool_ids 中的工具
//! }
//! ```

use super::ToolCallRequest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// 依赖关系
// ============================================================================

/// 工具依赖关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependency {
    /// 源工具ID（被依赖的工具）
    pub from_id: String,
    /// 目标工具ID（依赖其他工具的工具）
    pub to_id: String,
    /// 依赖类型
    pub dependency_type: DependencyType,
}

/// 依赖类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// 数据依赖：工具B的参数引用工具A的结果
    DataFlow,
    /// 顺序依赖：工具B需要在工具A之后执行（无数据传递）
    Ordering,
    /// 资源依赖：两个工具访问同一资源（如同一文件）
    Resource,
}

// ============================================================================
// 执行阶段
// ============================================================================

/// 执行阶段
///
/// 同一阶段内的工具可以并行执行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStage {
    /// 阶段编号（从0开始）
    pub stage_id: usize,
    /// 本阶段的工具ID列表
    pub tool_ids: Vec<String>,
    /// 本阶段是否可以并行执行
    pub parallel: bool,
}

impl ExecutionStage {
    /// 创建新阶段
    pub fn new(stage_id: usize) -> Self {
        Self {
            stage_id,
            tool_ids: Vec::new(),
            parallel: true,
        }
    }

    /// 添加工具到阶段
    pub fn add_tool(&mut self, tool_id: String) {
        self.tool_ids.push(tool_id);
    }

    /// 阶段内工具数量
    pub fn len(&self) -> usize {
        self.tool_ids.len()
    }

    /// 阶段是否为空
    pub fn is_empty(&self) -> bool {
        self.tool_ids.is_empty()
    }
}

// ============================================================================
// 执行DAG
// ============================================================================

/// 执行计划的有向无环图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDAG {
    /// 所有工具ID
    tool_ids: Vec<String>,
    /// 工具名称映射
    tool_names: HashMap<String, String>,
    /// 依赖关系列表
    dependencies: Vec<ToolDependency>,
    /// 执行阶段
    stages: Vec<ExecutionStage>,
    /// 统计信息
    stats: DAGStats,
}

/// DAG统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DAGStats {
    /// 总工具数
    pub total_tools: usize,
    /// 阶段数
    pub total_stages: usize,
    /// 依赖数
    pub total_dependencies: usize,
    /// 最大并行度（某阶段内最多工具数）
    pub max_parallelism: usize,
    /// 关键路径长度（最长依赖链）
    pub critical_path_length: usize,
}

impl ExecutionDAG {
    /// 创建新的DAG
    pub fn new() -> Self {
        Self {
            tool_ids: Vec::new(),
            tool_names: HashMap::new(),
            dependencies: Vec::new(),
            stages: Vec::new(),
            stats: DAGStats::default(),
        }
    }

    /// 添加工具
    pub fn add_tool(&mut self, id: String, name: String) {
        if !self.tool_ids.contains(&id) {
            self.tool_ids.push(id.clone());
            self.tool_names.insert(id, name);
        }
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, from_id: String, to_id: String, dep_type: DependencyType) {
        self.dependencies.push(ToolDependency {
            from_id,
            to_id,
            dependency_type: dep_type,
        });
    }

    /// 计算执行阶段
    pub fn compute_stages(&mut self) {
        if self.tool_ids.is_empty() {
            return;
        }

        // 构建入度映射
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for id in &self.tool_ids {
            in_degree.insert(id.clone(), 0);
            dependents.insert(id.clone(), Vec::new());
        }

        for dep in &self.dependencies {
            *in_degree.entry(dep.to_id.clone()).or_insert(0) += 1;
            dependents
                .entry(dep.from_id.clone())
                .or_default()
                .push(dep.to_id.clone());
        }

        // Kahn算法进行拓扑排序并分阶段
        let mut stages = Vec::new();
        let mut remaining: HashSet<String> = self.tool_ids.iter().cloned().collect();

        while !remaining.is_empty() {
            // 找出当前入度为0的节点
            let ready: Vec<String> = remaining
                .iter()
                .filter(|id| in_degree.get(*id).copied().unwrap_or(0) == 0)
                .cloned()
                .collect();

            if ready.is_empty() {
                // 存在循环依赖，将剩余工具放入最后一个阶段
                let mut stage = ExecutionStage::new(stages.len());
                stage.parallel = false; // 有循环，不能并行
                for id in remaining.drain() {
                    stage.add_tool(id);
                }
                stages.push(stage);
                break;
            }

            // 创建新阶段
            let mut stage = ExecutionStage::new(stages.len());
            for id in &ready {
                stage.add_tool(id.clone());
                remaining.remove(id);

                // 更新依赖此工具的工具的入度
                if let Some(deps) = dependents.get(id) {
                    for dep_id in deps {
                        if let Some(deg) = in_degree.get_mut(dep_id) {
                            *deg = deg.saturating_sub(1);
                        }
                    }
                }
            }
            stages.push(stage);
        }

        // 更新统计信息
        self.stats.total_tools = self.tool_ids.len();
        self.stats.total_stages = stages.len();
        self.stats.total_dependencies = self.dependencies.len();
        self.stats.max_parallelism = stages.iter().map(|s| s.len()).max().unwrap_or(0);
        self.stats.critical_path_length = stages.len();

        self.stages = stages;
    }

    /// 获取执行阶段
    pub fn stages(&self) -> &[ExecutionStage] {
        &self.stages
    }

    /// 获取统计信息
    pub fn stats(&self) -> &DAGStats {
        &self.stats
    }

    /// 获取依赖关系
    pub fn dependencies(&self) -> &[ToolDependency] {
        &self.dependencies
    }

    /// 可视化DAG（ASCII艺术）
    pub fn visualize(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Execution DAG ({} tools, {} stages, {} deps)",
            self.stats.total_tools, self.stats.total_stages, self.stats.total_dependencies
        ));
        lines.push("=".repeat(50));

        for stage in &self.stages {
            let tools: Vec<String> = stage
                .tool_ids
                .iter()
                .map(|id| {
                    self.tool_names
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| id.clone())
                })
                .collect();

            let mode = if stage.parallel && stage.len() > 1 {
                "parallel"
            } else {
                "sequential"
            };

            lines.push(format!(
                "Stage {}: [{}] ({})",
                stage.stage_id,
                tools.join(", "),
                mode
            ));
        }

        if !self.dependencies.is_empty() {
            lines.push("-".repeat(50));
            lines.push("Dependencies:".to_string());
            for dep in &self.dependencies {
                let from_name = self
                    .tool_names
                    .get(&dep.from_id)
                    .cloned()
                    .unwrap_or_else(|| dep.from_id.clone());
                let to_name = self
                    .tool_names
                    .get(&dep.to_id)
                    .cloned()
                    .unwrap_or_else(|| dep.to_id.clone());
                let arrow = match dep.dependency_type {
                    DependencyType::DataFlow => "-->",
                    DependencyType::Ordering => "==>",
                    DependencyType::Resource => "~~>",
                };
                lines.push(format!("  {} {} {}", from_name, arrow, to_name));
            }
        }

        lines.push("=".repeat(50));
        lines.push(format!(
            "Max parallelism: {}, Critical path: {} stages",
            self.stats.max_parallelism, self.stats.critical_path_length
        ));

        lines.join("\n")
    }

    /// 生成Mermaid图表格式
    pub fn to_mermaid(&self) -> String {
        let mut lines = vec!["graph TD".to_string()];

        // 添加节点
        for (id, name) in &self.tool_names {
            lines.push(format!("    {}[{}]", id.replace('-', "_"), name));
        }

        // 添加边
        for dep in &self.dependencies {
            let from = dep.from_id.replace('-', "_");
            let to = dep.to_id.replace('-', "_");
            let arrow = match dep.dependency_type {
                DependencyType::DataFlow => "-->",
                DependencyType::Ordering => "-.->",
                DependencyType::Resource => "~~~",
            };
            lines.push(format!("    {} {} {}", from, arrow, to));
        }

        lines.join("\n")
    }
}

impl Default for ExecutionDAG {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 依赖分析器
// ============================================================================

/// 依赖分析器
///
/// 分析工具调用之间的依赖关系
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer {
    /// 已知的资源模式（用于检测资源依赖）
    resource_patterns: Vec<ResourcePattern>,
}

/// 资源模式
#[derive(Debug, Clone)]
struct ResourcePattern {
    /// 工具名称
    tool_name: String,
    /// 资源参数名
    resource_param: String,
}

impl DependencyAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            resource_patterns: vec![
                ResourcePattern {
                    tool_name: "file_read".to_string(),
                    resource_param: "path".to_string(),
                },
                ResourcePattern {
                    tool_name: "file_write".to_string(),
                    resource_param: "path".to_string(),
                },
                ResourcePattern {
                    tool_name: "http_get".to_string(),
                    resource_param: "url".to_string(),
                },
            ],
        }
    }

    /// 分析工具调用，生成执行DAG
    pub fn analyze(&self, calls: &[ToolCallRequest]) -> ExecutionDAG {
        let mut dag = ExecutionDAG::new();

        // 添加所有工具
        for call in calls {
            dag.add_tool(call.id.clone(), call.name.clone());
        }

        // 检测数据流依赖（参数中引用其他工具ID）
        self.detect_data_dependencies(calls, &mut dag);

        // 检测资源依赖（访问同一资源）
        self.detect_resource_dependencies(calls, &mut dag);

        // 计算执行阶段
        dag.compute_stages();

        dag
    }

    /// 检测数据流依赖
    fn detect_data_dependencies(&self, calls: &[ToolCallRequest], dag: &mut ExecutionDAG) {
        let tool_ids: HashSet<String> = calls.iter().map(|c| c.id.clone()).collect();

        for call in calls {
            // 检查参数中是否引用了其他工具的ID
            let args_str = call.arguments.to_string();
            for other_id in &tool_ids {
                if other_id != &call.id && args_str.contains(other_id) {
                    dag.add_dependency(other_id.clone(), call.id.clone(), DependencyType::DataFlow);
                }
            }

            // 检查参数中是否引用了"result"或"output"等关键词
            // 这可能表示隐式依赖
            if let Some(obj) = call.arguments.as_object() {
                for (_, value) in obj {
                    if let Some(s) = value.as_str() {
                        // 检查是否引用其他工具的结果
                        for other in calls {
                            if other.id != call.id {
                                // 检查是否引用工具名
                                if s.contains(&other.name) || s.contains(&other.id) {
                                    dag.add_dependency(
                                        other.id.clone(),
                                        call.id.clone(),
                                        DependencyType::DataFlow,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 检测资源依赖
    fn detect_resource_dependencies(&self, calls: &[ToolCallRequest], dag: &mut ExecutionDAG) {
        // 收集每个工具访问的资源
        let mut resource_access: HashMap<String, Vec<(String, bool)>> = HashMap::new(); // resource -> [(tool_id, is_write)]

        for call in calls {
            for pattern in &self.resource_patterns {
                if call.name == pattern.tool_name {
                    if let Some(obj) = call.arguments.as_object() {
                        if let Some(resource) = obj.get(&pattern.resource_param) {
                            if let Some(resource_str) = resource.as_str() {
                                let is_write = call.name.contains("write");
                                resource_access
                                    .entry(resource_str.to_string())
                                    .or_default()
                                    .push((call.id.clone(), is_write));
                            }
                        }
                    }
                }
            }
        }

        // 对于同一资源的访问，添加依赖
        for (_, accesses) in resource_access {
            if accesses.len() > 1 {
                // 按顺序添加依赖（简化：前面的工具必须在后面的之前执行）
                for i in 0..accesses.len() - 1 {
                    dag.add_dependency(
                        accesses[i].0.clone(),
                        accesses[i + 1].0.clone(),
                        DependencyType::Resource,
                    );
                }
            }
        }
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_execution_stage() {
        let mut stage = ExecutionStage::new(0);
        assert!(stage.is_empty());

        stage.add_tool("tool1".to_string());
        stage.add_tool("tool2".to_string());
        assert_eq!(stage.len(), 2);
        assert!(!stage.is_empty());
    }

    #[test]
    fn test_execution_dag_basic() {
        let mut dag = ExecutionDAG::new();

        dag.add_tool("t1".to_string(), "tool_a".to_string());
        dag.add_tool("t2".to_string(), "tool_b".to_string());
        dag.add_tool("t3".to_string(), "tool_c".to_string());

        dag.compute_stages();

        // 无依赖时，所有工具在同一阶段
        assert_eq!(dag.stages().len(), 1);
        assert_eq!(dag.stages()[0].len(), 3);
    }

    #[test]
    fn test_execution_dag_with_dependency() {
        let mut dag = ExecutionDAG::new();

        dag.add_tool("t1".to_string(), "tool_a".to_string());
        dag.add_tool("t2".to_string(), "tool_b".to_string());
        dag.add_tool("t3".to_string(), "tool_c".to_string());

        // t2 依赖 t1
        dag.add_dependency("t1".to_string(), "t2".to_string(), DependencyType::DataFlow);

        dag.compute_stages();

        // 应该有2个阶段：[t1, t3] 和 [t2]
        assert_eq!(dag.stages().len(), 2);
    }

    #[test]
    fn test_execution_dag_chain() {
        let mut dag = ExecutionDAG::new();

        dag.add_tool("t1".to_string(), "tool_a".to_string());
        dag.add_tool("t2".to_string(), "tool_b".to_string());
        dag.add_tool("t3".to_string(), "tool_c".to_string());

        // t1 -> t2 -> t3
        dag.add_dependency("t1".to_string(), "t2".to_string(), DependencyType::DataFlow);
        dag.add_dependency("t2".to_string(), "t3".to_string(), DependencyType::DataFlow);

        dag.compute_stages();

        // 应该有3个阶段，每个阶段1个工具
        assert_eq!(dag.stages().len(), 3);
        assert_eq!(dag.stats().critical_path_length, 3);
    }

    #[test]
    fn test_dependency_analyzer_independent() {
        let analyzer = DependencyAnalyzer::new();

        let calls = vec![
            ToolCallRequest {
                id: "call_1".to_string(),
                name: "tool_a".to_string(),
                arguments: json!({"x": 1}),
            },
            ToolCallRequest {
                id: "call_2".to_string(),
                name: "tool_b".to_string(),
                arguments: json!({"y": 2}),
            },
        ];

        let dag = analyzer.analyze(&calls);

        // 独立工具应该在同一阶段
        assert_eq!(dag.stages().len(), 1);
        assert_eq!(dag.stages()[0].len(), 2);
        assert!(dag.dependencies().is_empty());
    }

    #[test]
    fn test_dependency_analyzer_data_flow() {
        let analyzer = DependencyAnalyzer::new();

        let calls = vec![
            ToolCallRequest {
                id: "call_1".to_string(),
                name: "fetch_data".to_string(),
                arguments: json!({"source": "api"}),
            },
            ToolCallRequest {
                id: "call_2".to_string(),
                name: "process_data".to_string(),
                arguments: json!({"input": "call_1"}), // 引用 call_1
            },
        ];

        let dag = analyzer.analyze(&calls);

        // 应该检测到依赖（可能检测到多个，如 ID引用 + 名称引用）
        assert!(!dag.dependencies().is_empty());
        // 关键是：应该有2个阶段（call_1先，call_2后）
        assert_eq!(dag.stages().len(), 2);
    }

    #[test]
    fn test_dependency_analyzer_resource() {
        let analyzer = DependencyAnalyzer::new();

        let calls = vec![
            ToolCallRequest {
                id: "call_1".to_string(),
                name: "file_write".to_string(),
                arguments: json!({"path": "/tmp/test.txt", "content": "hello"}),
            },
            ToolCallRequest {
                id: "call_2".to_string(),
                name: "file_read".to_string(),
                arguments: json!({"path": "/tmp/test.txt"}),
            },
        ];

        let dag = analyzer.analyze(&calls);

        // 应该检测到资源依赖
        assert!(!dag.dependencies().is_empty());
        assert_eq!(dag.stages().len(), 2);
    }

    #[test]
    fn test_dag_visualize() {
        let mut dag = ExecutionDAG::new();
        dag.add_tool("t1".to_string(), "fetch".to_string());
        dag.add_tool("t2".to_string(), "process".to_string());
        dag.add_dependency("t1".to_string(), "t2".to_string(), DependencyType::DataFlow);
        dag.compute_stages();

        let viz = dag.visualize();
        assert!(viz.contains("Execution DAG"));
        assert!(viz.contains("fetch"));
        assert!(viz.contains("process"));
    }

    #[test]
    fn test_dag_mermaid() {
        let mut dag = ExecutionDAG::new();
        dag.add_tool("t1".to_string(), "fetch".to_string());
        dag.add_tool("t2".to_string(), "process".to_string());
        dag.add_dependency("t1".to_string(), "t2".to_string(), DependencyType::DataFlow);

        let mermaid = dag.to_mermaid();
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("fetch"));
        assert!(mermaid.contains("process"));
        assert!(mermaid.contains("-->"));
    }

    #[test]
    fn test_dag_stats() {
        let mut dag = ExecutionDAG::new();
        dag.add_tool("t1".to_string(), "a".to_string());
        dag.add_tool("t2".to_string(), "b".to_string());
        dag.add_tool("t3".to_string(), "c".to_string());
        dag.add_dependency("t1".to_string(), "t2".to_string(), DependencyType::DataFlow);
        dag.compute_stages();

        let stats = dag.stats();
        assert_eq!(stats.total_tools, 3);
        assert_eq!(stats.total_dependencies, 1);
        assert_eq!(stats.max_parallelism, 2); // t1 and t3 can run in parallel
    }
}
