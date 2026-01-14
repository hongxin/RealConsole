//! v2.0.0-alpha.3: Cell Dependency and Parallel Execution
//!
//! Provides dependency tracking and parallel execution for notebook cells:
//! - Dependency graph (DAG) for tracking cell relationships
//! - Smart execution scheduling based on dependencies
//! - Parallel execution of independent cells
//!
//! # Dependency Types
//!
//! - **Explicit**: User-defined dependency via cell metadata
//! - **Variable**: Dependency through shared variables/outputs
//! - **Sequential**: Order-based dependency (cell N depends on N-1)
//!
//! # Example
//!
//! ```ignore
//! use realconsole::notebook::{DependencyGraph, ExecutionScheduler, Cell};
//!
//! let mut graph = DependencyGraph::new();
//! graph.add_cell(cell1.id);
//! graph.add_cell(cell2.id);
//! graph.add_dependency(cell2.id, cell1.id, DependencyType::Explicit);
//!
//! let scheduler = ExecutionScheduler::new(graph);
//! let batches = scheduler.schedule();
//! // batches[0] = [cell1], batches[1] = [cell2]
//! ```

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Dependency Types
// ============================================================================

/// Type of dependency between cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    /// Explicit dependency defined by user
    Explicit,
    /// Dependency through variable reference
    Variable,
    /// Sequential dependency (execution order)
    Sequential,
    /// Output dependency (uses output of another cell)
    Output,
}

impl DependencyType {
    /// Check if this is a strong dependency (must wait)
    pub fn is_strong(&self) -> bool {
        matches!(self, DependencyType::Explicit | DependencyType::Variable | DependencyType::Output)
    }

    /// Check if this is a weak dependency (prefer order but can skip)
    pub fn is_weak(&self) -> bool {
        matches!(self, DependencyType::Sequential)
    }
}

/// A dependency edge in the graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Source cell (depends on target)
    pub from: Uuid,
    /// Target cell (depended upon)
    pub to: Uuid,
    /// Type of dependency
    pub dep_type: DependencyType,
    /// Optional variable name for variable dependencies
    pub variable: Option<String>,
}

impl DependencyEdge {
    /// Create new dependency edge
    pub fn new(from: Uuid, to: Uuid, dep_type: DependencyType) -> Self {
        Self {
            from,
            to,
            dep_type,
            variable: None,
        }
    }

    /// Create variable dependency
    pub fn variable(from: Uuid, to: Uuid, var_name: impl Into<String>) -> Self {
        Self {
            from,
            to,
            dep_type: DependencyType::Variable,
            variable: Some(var_name.into()),
        }
    }
}

// ============================================================================
// Dependency Graph
// ============================================================================

/// Error types for dependency operations
#[derive(Debug, thiserror::Error)]
pub enum DependencyError {
    #[error("Cycle detected involving cell: {0}")]
    CycleDetected(Uuid),

    #[error("Cell not found: {0}")]
    CellNotFound(Uuid),

    #[error("Self-dependency not allowed: {0}")]
    SelfDependency(Uuid),

    #[error("Duplicate dependency")]
    DuplicateDependency,
}

/// Result type for dependency operations
pub type DependencyResult<T> = Result<T, DependencyError>;

/// Directed Acyclic Graph for cell dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// All cells in the graph
    cells: HashSet<Uuid>,
    /// Forward edges (cell -> cells it depends on)
    dependencies: HashMap<Uuid, HashSet<DependencyEdge>>,
    /// Reverse edges (cell -> cells that depend on it)
    dependents: HashMap<Uuid, HashSet<Uuid>>,
}

impl DependencyGraph {
    /// Create empty dependency graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a cell to the graph
    pub fn add_cell(&mut self, cell_id: Uuid) {
        self.cells.insert(cell_id);
        self.dependencies.entry(cell_id).or_default();
        self.dependents.entry(cell_id).or_default();
    }

    /// Remove a cell from the graph
    pub fn remove_cell(&mut self, cell_id: Uuid) {
        self.cells.remove(&cell_id);

        // Remove forward edges
        self.dependencies.remove(&cell_id);

        // Remove reverse edges pointing to this cell
        for deps in self.dependents.values_mut() {
            deps.remove(&cell_id);
        }
        self.dependents.remove(&cell_id);

        // Remove edges from other cells to this cell
        for edges in self.dependencies.values_mut() {
            edges.retain(|e| e.to != cell_id);
        }
    }

    /// Add a dependency (from depends on to)
    pub fn add_dependency(
        &mut self,
        from: Uuid,
        to: Uuid,
        dep_type: DependencyType,
    ) -> DependencyResult<()> {
        // Check for self-dependency
        if from == to {
            return Err(DependencyError::SelfDependency(from));
        }

        // Ensure both cells exist
        if !self.cells.contains(&from) {
            return Err(DependencyError::CellNotFound(from));
        }
        if !self.cells.contains(&to) {
            return Err(DependencyError::CellNotFound(to));
        }

        // Add edge
        let edge = DependencyEdge::new(from, to, dep_type);
        self.dependencies.entry(from).or_default().insert(edge);
        self.dependents.entry(to).or_default().insert(from);

        // Check for cycles
        if self.has_cycle() {
            // Rollback
            self.dependencies.entry(from).or_default().retain(|e| e.to != to || e.dep_type != dep_type);
            self.dependents.entry(to).or_default().remove(&from);
            return Err(DependencyError::CycleDetected(from));
        }

        Ok(())
    }

    /// Add variable dependency
    pub fn add_variable_dependency(
        &mut self,
        from: Uuid,
        to: Uuid,
        variable: impl Into<String>,
    ) -> DependencyResult<()> {
        if from == to {
            return Err(DependencyError::SelfDependency(from));
        }

        if !self.cells.contains(&from) || !self.cells.contains(&to) {
            return Err(DependencyError::CellNotFound(if !self.cells.contains(&from) { from } else { to }));
        }

        let edge = DependencyEdge::variable(from, to, variable);
        self.dependencies.entry(from).or_default().insert(edge);
        self.dependents.entry(to).or_default().insert(from);

        if self.has_cycle() {
            self.remove_dependency(from, to);
            return Err(DependencyError::CycleDetected(from));
        }

        Ok(())
    }

    /// Remove a dependency
    pub fn remove_dependency(&mut self, from: Uuid, to: Uuid) {
        if let Some(edges) = self.dependencies.get_mut(&from) {
            edges.retain(|e| e.to != to);
        }
        if let Some(deps) = self.dependents.get_mut(&to) {
            deps.remove(&from);
        }
    }

    /// Clear all dependencies for a cell (keep the cell)
    pub fn clear_dependencies(&mut self, cell_id: Uuid) {
        if let Some(edges) = self.dependencies.get(&cell_id) {
            let targets: Vec<Uuid> = edges.iter().map(|e| e.to).collect();
            for target in targets {
                if let Some(deps) = self.dependents.get_mut(&target) {
                    deps.remove(&cell_id);
                }
            }
        }
        self.dependencies.insert(cell_id, HashSet::new());
    }

    /// Get direct dependencies of a cell
    pub fn get_dependencies(&self, cell_id: Uuid) -> Vec<&DependencyEdge> {
        self.dependencies
            .get(&cell_id)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Get cells that depend on this cell
    pub fn get_dependents(&self, cell_id: Uuid) -> Vec<Uuid> {
        self.dependents
            .get(&cell_id)
            .map(|deps| deps.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get all cells with no dependencies (roots)
    pub fn get_roots(&self) -> Vec<Uuid> {
        self.cells
            .iter()
            .filter(|&id| {
                self.dependencies
                    .get(id)
                    .map(|deps| deps.is_empty())
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }

    /// Get all cells with no dependents (leaves)
    pub fn get_leaves(&self) -> Vec<Uuid> {
        self.cells
            .iter()
            .filter(|&id| {
                self.dependents
                    .get(id)
                    .map(|deps| deps.is_empty())
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }

    /// Check if the graph has a cycle (using DFS)
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &cell in &self.cells {
            if self.has_cycle_dfs(cell, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    fn has_cycle_dfs(
        &self,
        cell: Uuid,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> bool {
        if rec_stack.contains(&cell) {
            return true;
        }
        if visited.contains(&cell) {
            return false;
        }

        visited.insert(cell);
        rec_stack.insert(cell);

        if let Some(edges) = self.dependencies.get(&cell) {
            for edge in edges {
                if self.has_cycle_dfs(edge.to, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(&cell);
        false
    }

    /// Topological sort using Kahn's algorithm
    pub fn topological_sort(&self) -> Option<Vec<Uuid>> {
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        for &cell in &self.cells {
            in_degree.insert(cell, 0);
        }

        // Calculate in-degrees
        for edges in self.dependencies.values() {
            for edge in edges {
                // edge.from depends on edge.to, so edge.from has incoming edge from edge.to
                *in_degree.entry(edge.from).or_insert(0) += 0; // from already counted
            }
        }

        // Count incoming edges
        for &cell in &self.cells {
            let deps = self.dependencies.get(&cell).map(|e| e.len()).unwrap_or(0);
            *in_degree.entry(cell).or_insert(0) = deps;
        }

        // Start with cells that have no dependencies
        let mut queue: VecDeque<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(cell) = queue.pop_front() {
            result.push(cell);

            // For each cell that depends on this one
            if let Some(dependents) = self.dependents.get(&cell) {
                for &dependent in dependents {
                    if let Some(deg) = in_degree.get_mut(&dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        if result.len() == self.cells.len() {
            Some(result)
        } else {
            None // Cycle detected
        }
    }

    /// Get number of cells
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Get total number of dependency edges
    pub fn edge_count(&self) -> usize {
        self.dependencies.values().map(|e| e.len()).sum()
    }

    /// Check if cell exists
    pub fn contains(&self, cell_id: Uuid) -> bool {
        self.cells.contains(&cell_id)
    }

    /// Check if there's a dependency path from one cell to another
    pub fn has_path(&self, from: Uuid, to: Uuid) -> bool {
        if from == to {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(edges) = self.dependencies.get(&current) {
                for edge in edges {
                    if edge.to == to {
                        return true;
                    }
                    queue.push_back(edge.to);
                }
            }
        }

        false
    }
}

// ============================================================================
// Execution Scheduler
// ============================================================================

/// Execution batch - cells that can run in parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionBatch {
    /// Batch index (0-based)
    pub index: usize,
    /// Cells in this batch
    pub cells: Vec<Uuid>,
}

impl ExecutionBatch {
    /// Create new batch
    pub fn new(index: usize, cells: Vec<Uuid>) -> Self {
        Self { index, cells }
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Get batch size
    pub fn len(&self) -> usize {
        self.cells.len()
    }
}

/// Execution schedule - batches of cells to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSchedule {
    /// Batches in execution order
    pub batches: Vec<ExecutionBatch>,
    /// Total cells scheduled
    pub total_cells: usize,
    /// Maximum parallelism achieved
    pub max_parallelism: usize,
}

impl ExecutionSchedule {
    /// Create empty schedule
    pub fn empty() -> Self {
        Self {
            batches: Vec::new(),
            total_cells: 0,
            max_parallelism: 0,
        }
    }

    /// Get number of batches
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Iterate over batches
    pub fn iter(&self) -> impl Iterator<Item = &ExecutionBatch> {
        self.batches.iter()
    }
}

/// Scheduler for parallel cell execution
#[derive(Debug)]
pub struct ExecutionScheduler {
    /// Dependency graph
    graph: DependencyGraph,
    /// Maximum cells per batch
    max_batch_size: usize,
    /// Only consider strong dependencies
    strong_only: bool,
}

impl ExecutionScheduler {
    /// Create new scheduler
    pub fn new(graph: DependencyGraph) -> Self {
        Self {
            graph,
            max_batch_size: usize::MAX,
            strong_only: false,
        }
    }

    /// Set maximum batch size
    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    /// Only consider strong dependencies
    pub fn with_strong_only(mut self, strong_only: bool) -> Self {
        self.strong_only = strong_only;
        self
    }

    /// Schedule cells for execution
    pub fn schedule(&self) -> ExecutionSchedule {
        if self.graph.cell_count() == 0 {
            return ExecutionSchedule::empty();
        }

        // Calculate levels using BFS from roots
        let mut levels: HashMap<Uuid, usize> = HashMap::new();
        let mut max_level = 0;

        // Initialize roots at level 0
        let roots = self.graph.get_roots();
        for root in &roots {
            levels.insert(*root, 0);
        }

        // BFS to assign levels
        let mut queue: VecDeque<Uuid> = roots.into_iter().collect();
        let mut visited = HashSet::new();

        while let Some(cell) = queue.pop_front() {
            if visited.contains(&cell) {
                continue;
            }
            visited.insert(cell);

            let current_level = *levels.get(&cell).unwrap_or(&0);

            // Process dependents
            for dependent in self.graph.get_dependents(cell) {
                let new_level = current_level + 1;
                let existing = levels.entry(dependent).or_insert(0);
                if new_level > *existing {
                    *existing = new_level;
                    max_level = max_level.max(new_level);
                }
                queue.push_back(dependent);
            }
        }

        // Handle any unvisited cells (isolated nodes)
        for &cell in &self.graph.cells {
            if !visited.contains(&cell) {
                levels.insert(cell, 0);
            }
        }

        // Group cells by level
        let mut level_cells: HashMap<usize, Vec<Uuid>> = HashMap::new();
        for (&cell, &level) in &levels {
            level_cells.entry(level).or_default().push(cell);
        }

        // Create batches
        let mut batches = Vec::new();
        let mut total_cells = 0;
        let mut max_parallelism = 0;

        for level in 0..=max_level {
            if let Some(cells) = level_cells.get(&level) {
                // Split into sub-batches if needed
                for chunk in cells.chunks(self.max_batch_size) {
                    let batch = ExecutionBatch::new(batches.len(), chunk.to_vec());
                    max_parallelism = max_parallelism.max(batch.len());
                    total_cells += batch.len();
                    batches.push(batch);
                }
            }
        }

        ExecutionSchedule {
            batches,
            total_cells,
            max_parallelism,
        }
    }

    /// Get cells ready to execute given completed cells
    pub fn get_ready_cells(&self, completed: &HashSet<Uuid>) -> Vec<Uuid> {
        self.graph
            .cells
            .iter()
            .filter(|&&cell| {
                // Not yet completed
                if completed.contains(&cell) {
                    return false;
                }

                // All dependencies completed
                self.graph
                    .get_dependencies(cell)
                    .iter()
                    .all(|edge| {
                        if self.strong_only && !edge.dep_type.is_strong() {
                            return true;
                        }
                        completed.contains(&edge.to)
                    })
            })
            .copied()
            .collect()
    }
}

// ============================================================================
// Dependency Analyzer
// ============================================================================

/// Analyzes code to detect dependencies
#[derive(Debug, Default)]
pub struct DependencyAnalyzer {
    /// Variable definitions (cell_id -> variables defined)
    definitions: HashMap<Uuid, HashSet<String>>,
    /// Variable usages (cell_id -> variables used)
    usages: HashMap<Uuid, HashSet<String>>,
}

impl DependencyAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a cell's source code
    pub fn analyze_cell(&mut self, cell_id: Uuid, source: &str) {
        let (defs, uses) = self.extract_variables(source);
        self.definitions.insert(cell_id, defs);
        self.usages.insert(cell_id, uses);
    }

    /// Extract variable definitions and usages from source
    fn extract_variables(&self, source: &str) -> (HashSet<String>, HashSet<String>) {
        let mut definitions = HashSet::new();
        let mut usages = HashSet::new();

        // Simple pattern matching for common variable patterns
        // This is a basic implementation - can be enhanced with proper parsing

        for line in source.lines() {
            let line = line.trim();

            // Shell variable assignment: VAR=value or export VAR=value
            if let Some(eq_pos) = line.find('=') {
                let before = line[..eq_pos].trim();
                // Handle export prefix
                let var_part = if before.starts_with("export ") {
                    before.strip_prefix("export ").unwrap_or(before).trim()
                } else {
                    before
                };
                // Check if it's an assignment (not comparison)
                if !var_part.contains(' ')
                    && !var_part.contains('<')
                    && !var_part.contains('>')
                    && is_valid_variable_name(var_part)
                {
                    definitions.insert(var_part.to_string());
                }
            }

            // Shell variable usage: $VAR or ${VAR}
            let mut chars = line.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '$' {
                    let mut var_name = String::new();
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        while let Some(&c) = chars.peek() {
                            if c == '}' {
                                chars.next();
                                break;
                            }
                            var_name.push(c);
                            chars.next();
                        }
                    } else {
                        while let Some(&c) = chars.peek() {
                            if c.is_alphanumeric() || c == '_' {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if !var_name.is_empty() && is_valid_variable_name(&var_name) {
                        usages.insert(var_name);
                    }
                }
            }
        }

        // Remove self-defined variables from usages
        for def in &definitions {
            usages.remove(def);
        }

        (definitions, usages)
    }

    /// Build dependency graph from analyzed cells
    pub fn build_graph(&self, cell_order: &[Uuid]) -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        // Add all cells
        for &cell_id in cell_order {
            graph.add_cell(cell_id);
        }

        // Build variable -> defining cell map
        let mut var_to_cell: HashMap<&String, Uuid> = HashMap::new();
        for &cell_id in cell_order {
            if let Some(defs) = self.definitions.get(&cell_id) {
                for var in defs {
                    var_to_cell.insert(var, cell_id);
                }
            }
        }

        // Add dependencies based on variable usage
        for &cell_id in cell_order {
            if let Some(uses) = self.usages.get(&cell_id) {
                for var in uses {
                    if let Some(&def_cell) = var_to_cell.get(var) {
                        if def_cell != cell_id {
                            let _ = graph.add_variable_dependency(cell_id, def_cell, var.clone());
                        }
                    }
                }
            }
        }

        graph
    }

    /// Get variables defined by a cell
    pub fn get_definitions(&self, cell_id: Uuid) -> Option<&HashSet<String>> {
        self.definitions.get(&cell_id)
    }

    /// Get variables used by a cell
    pub fn get_usages(&self, cell_id: Uuid) -> Option<&HashSet<String>> {
        self.usages.get(&cell_id)
    }
}

/// Check if a string is a valid variable name
fn is_valid_variable_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn uuid(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn test_dependency_type() {
        assert!(DependencyType::Explicit.is_strong());
        assert!(DependencyType::Variable.is_strong());
        assert!(DependencyType::Output.is_strong());
        assert!(!DependencyType::Sequential.is_strong());
        assert!(DependencyType::Sequential.is_weak());
    }

    #[test]
    fn test_dependency_edge() {
        let edge = DependencyEdge::new(uuid(1), uuid(2), DependencyType::Explicit);
        assert_eq!(edge.from, uuid(1));
        assert_eq!(edge.to, uuid(2));
        assert!(edge.variable.is_none());

        let var_edge = DependencyEdge::variable(uuid(3), uuid(4), "MY_VAR");
        assert_eq!(var_edge.dep_type, DependencyType::Variable);
        assert_eq!(var_edge.variable, Some("MY_VAR".to_string()));
    }

    #[test]
    fn test_graph_add_cell() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));

        assert_eq!(graph.cell_count(), 2);
        assert!(graph.contains(uuid(1)));
        assert!(graph.contains(uuid(2)));
    }

    #[test]
    fn test_graph_remove_cell() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();

        graph.remove_cell(uuid(1));

        assert_eq!(graph.cell_count(), 1);
        assert!(!graph.contains(uuid(1)));
        assert!(graph.get_dependencies(uuid(2)).is_empty());
    }

    #[test]
    fn test_graph_add_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));

        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();

        let deps = graph.get_dependencies(uuid(2));
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].to, uuid(1));

        let dependents = graph.get_dependents(uuid(1));
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], uuid(2));
    }

    #[test]
    fn test_graph_self_dependency_error() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));

        let result = graph.add_dependency(uuid(1), uuid(1), DependencyType::Explicit);
        assert!(matches!(result, Err(DependencyError::SelfDependency(_))));
    }

    #[test]
    fn test_graph_cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_cell(uuid(3));

        // 1 <- 2 <- 3
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(3), uuid(2), DependencyType::Explicit).unwrap();

        // Try to add 1 <- 3 (would create cycle)
        let result = graph.add_dependency(uuid(1), uuid(3), DependencyType::Explicit);
        assert!(matches!(result, Err(DependencyError::CycleDetected(_))));

        // Graph should not have the cycle
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_graph_roots_and_leaves() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_cell(uuid(3));

        // 1 <- 2 <- 3
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(3), uuid(2), DependencyType::Explicit).unwrap();

        let roots = graph.get_roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], uuid(1));

        let leaves = graph.get_leaves();
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0], uuid(3));
    }

    #[test]
    fn test_graph_topological_sort() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_cell(uuid(3));

        // 1 <- 2 <- 3
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(3), uuid(2), DependencyType::Explicit).unwrap();

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);

        // 1 should come before 2, 2 before 3
        let pos1 = sorted.iter().position(|&x| x == uuid(1)).unwrap();
        let pos2 = sorted.iter().position(|&x| x == uuid(2)).unwrap();
        let pos3 = sorted.iter().position(|&x| x == uuid(3)).unwrap();
        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
    }

    #[test]
    fn test_graph_has_path() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_cell(uuid(3));
        graph.add_cell(uuid(4));

        // 1 <- 2 <- 3
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(3), uuid(2), DependencyType::Explicit).unwrap();

        assert!(graph.has_path(uuid(3), uuid(1)));
        assert!(graph.has_path(uuid(2), uuid(1)));
        assert!(!graph.has_path(uuid(1), uuid(3)));
        assert!(!graph.has_path(uuid(4), uuid(1)));
    }

    #[test]
    fn test_scheduler_empty() {
        let graph = DependencyGraph::new();
        let scheduler = ExecutionScheduler::new(graph);
        let schedule = scheduler.schedule();

        assert_eq!(schedule.batch_count(), 0);
        assert_eq!(schedule.total_cells, 0);
    }

    #[test]
    fn test_scheduler_independent_cells() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_cell(uuid(3));

        let scheduler = ExecutionScheduler::new(graph);
        let schedule = scheduler.schedule();

        // All cells can run in parallel (single batch)
        assert_eq!(schedule.batch_count(), 1);
        assert_eq!(schedule.total_cells, 3);
        assert_eq!(schedule.max_parallelism, 3);
    }

    #[test]
    fn test_scheduler_linear_chain() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_cell(uuid(3));

        // 1 <- 2 <- 3
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(3), uuid(2), DependencyType::Explicit).unwrap();

        let scheduler = ExecutionScheduler::new(graph);
        let schedule = scheduler.schedule();

        // Should have 3 batches (sequential)
        assert_eq!(schedule.batch_count(), 3);
        assert_eq!(schedule.max_parallelism, 1);

        // Verify order
        assert!(schedule.batches[0].cells.contains(&uuid(1)));
        assert!(schedule.batches[1].cells.contains(&uuid(2)));
        assert!(schedule.batches[2].cells.contains(&uuid(3)));
    }

    #[test]
    fn test_scheduler_diamond() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1)); // Root
        graph.add_cell(uuid(2)); // Left branch
        graph.add_cell(uuid(3)); // Right branch
        graph.add_cell(uuid(4)); // Join

        // Diamond: 1 <- {2, 3} <- 4
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(3), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(4), uuid(2), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(4), uuid(3), DependencyType::Explicit).unwrap();

        let scheduler = ExecutionScheduler::new(graph);
        let schedule = scheduler.schedule();

        // Should have 3 batches: [1], [2, 3], [4]
        assert_eq!(schedule.batch_count(), 3);
        assert_eq!(schedule.max_parallelism, 2);
    }

    #[test]
    fn test_scheduler_with_batch_limit() {
        let mut graph = DependencyGraph::new();
        for i in 1..=6 {
            graph.add_cell(uuid(i as u128));
        }

        let scheduler = ExecutionScheduler::new(graph).with_max_batch_size(2);
        let schedule = scheduler.schedule();

        // 6 independent cells with batch size 2 = 3 batches
        assert_eq!(schedule.batch_count(), 3);
        for batch in &schedule.batches {
            assert!(batch.len() <= 2);
        }
    }

    #[test]
    fn test_scheduler_get_ready_cells() {
        let mut graph = DependencyGraph::new();
        graph.add_cell(uuid(1));
        graph.add_cell(uuid(2));
        graph.add_cell(uuid(3));

        // 1 <- 2 <- 3
        graph.add_dependency(uuid(2), uuid(1), DependencyType::Explicit).unwrap();
        graph.add_dependency(uuid(3), uuid(2), DependencyType::Explicit).unwrap();

        let scheduler = ExecutionScheduler::new(graph);

        // Initially, only cell 1 is ready
        let ready = scheduler.get_ready_cells(&HashSet::new());
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&uuid(1)));

        // After cell 1, cell 2 is ready
        let mut completed = HashSet::new();
        completed.insert(uuid(1));
        let ready = scheduler.get_ready_cells(&completed);
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&uuid(2)));

        // After cells 1 and 2, cell 3 is ready
        completed.insert(uuid(2));
        let ready = scheduler.get_ready_cells(&completed);
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&uuid(3)));
    }

    #[test]
    fn test_analyzer_variable_detection() {
        let mut analyzer = DependencyAnalyzer::new();

        let source1 = "export MY_VAR=hello\necho $OTHER";
        analyzer.analyze_cell(uuid(1), source1);

        let defs = analyzer.get_definitions(uuid(1)).unwrap();
        let uses = analyzer.get_usages(uuid(1)).unwrap();

        assert!(defs.contains("MY_VAR"));
        assert!(uses.contains("OTHER"));
        assert!(!uses.contains("MY_VAR")); // Self-defined not in usages
    }

    #[test]
    fn test_analyzer_build_graph() {
        let mut analyzer = DependencyAnalyzer::new();

        // Cell 1 defines FOO
        analyzer.analyze_cell(uuid(1), "FOO=bar");
        // Cell 2 uses FOO, defines BAR
        analyzer.analyze_cell(uuid(2), "echo $FOO\nBAR=baz");
        // Cell 3 uses BAR
        analyzer.analyze_cell(uuid(3), "echo $BAR");

        let graph = analyzer.build_graph(&[uuid(1), uuid(2), uuid(3)]);

        assert_eq!(graph.cell_count(), 3);

        // Cell 2 depends on cell 1 (via FOO)
        let deps2 = graph.get_dependencies(uuid(2));
        assert!(deps2.iter().any(|e| e.to == uuid(1)));

        // Cell 3 depends on cell 2 (via BAR)
        let deps3 = graph.get_dependencies(uuid(3));
        assert!(deps3.iter().any(|e| e.to == uuid(2)));
    }

    #[test]
    fn test_is_valid_variable_name() {
        assert!(is_valid_variable_name("FOO"));
        assert!(is_valid_variable_name("_bar"));
        assert!(is_valid_variable_name("foo123"));
        assert!(is_valid_variable_name("_"));
        assert!(!is_valid_variable_name("123foo"));
        assert!(!is_valid_variable_name(""));
        assert!(!is_valid_variable_name("foo-bar"));
    }

    #[test]
    fn test_execution_batch() {
        let batch = ExecutionBatch::new(0, vec![uuid(1), uuid(2)]);
        assert_eq!(batch.index, 0);
        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_execution_schedule_empty() {
        let schedule = ExecutionSchedule::empty();
        assert_eq!(schedule.batch_count(), 0);
        assert_eq!(schedule.total_cells, 0);
    }
}
