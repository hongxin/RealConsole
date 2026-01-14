//! v2.0.0-alpha.1: Cell Execution Engine
//!
//! Provides the execution infrastructure for notebook cells:
//! - Natural language cells → LLM processing
//! - Command cells → System command handling
//! - Code cells → Shell execution
//!
//! # Example
//!
//! ```ignore
//! use realconsole::notebook::{CellExecutor, ExecutionConfig, Cell};
//!
//! let executor = CellExecutor::new(config);
//!
//! let mut cell = Cell::code("!ls -la");
//! let result = executor.execute(&mut cell).await?;
//!
//! println!("Outputs: {:?}", result.outputs);
//! ```

use super::types::{Cell, CellOutput, CellState, CellType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

/// Execution error
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Cell not executable: {0}")]
    NotExecutable(CellType),

    #[error("Execution timeout after {0}ms")]
    Timeout(u64),

    #[error("Execution cancelled")]
    Cancelled,

    #[error("Cell already running: {0}")]
    AlreadyRunning(Uuid),

    #[error("LLM not configured")]
    LlmNotConfigured,

    #[error("Shell execution failed: {0}")]
    ShellError(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Execution result type
pub type ExecResult<T> = Result<T, ExecutionError>;

// ============================================================================
// Execution Config
// ============================================================================

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Execution timeout (milliseconds)
    pub timeout_ms: u64,

    /// Maximum output size (bytes)
    pub max_output_bytes: usize,

    /// Enable streaming output
    pub stream_output: bool,

    /// Maximum concurrent executions
    pub max_concurrent: usize,

    /// Shell command prefix (for code cells)
    pub shell_prefix: String,

    /// Default shell
    pub default_shell: String,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,  // 30 seconds
            max_output_bytes: 1024 * 1024,  // 1MB
            stream_output: true,
            max_concurrent: 4,
            shell_prefix: "!".to_string(),
            default_shell: "bash".to_string(),
        }
    }
}

impl ExecutionConfig {
    /// Create config with timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Create config with max output
    pub fn with_max_output(mut self, max_bytes: usize) -> Self {
        self.max_output_bytes = max_bytes;
        self
    }

    /// Create config with streaming
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.stream_output = enabled;
        self
    }
}

// ============================================================================
// Execution Result
// ============================================================================

/// Result of cell execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Cell ID
    pub cell_id: Uuid,

    /// Execution success
    pub success: bool,

    /// Outputs generated
    pub outputs: Vec<CellOutput>,

    /// Start time
    pub started_at: DateTime<Utc>,

    /// End time
    pub ended_at: DateTime<Utc>,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Error message (if failed)
    pub error: Option<String>,
}

impl ExecutionResult {
    /// Create success result
    pub fn success(cell_id: Uuid, outputs: Vec<CellOutput>, duration_ms: u64) -> Self {
        let now = Utc::now();
        Self {
            cell_id,
            success: true,
            outputs,
            started_at: now,
            ended_at: now,
            duration_ms,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(cell_id: Uuid, error: impl Into<String>, duration_ms: u64) -> Self {
        let now = Utc::now();
        let error_msg = error.into();
        Self {
            cell_id,
            success: false,
            outputs: vec![CellOutput::error(&error_msg)],
            started_at: now,
            ended_at: now,
            duration_ms,
            error: Some(error_msg),
        }
    }

    /// Check if result has errors
    pub fn has_error(&self) -> bool {
        !self.success || self.error.is_some()
    }
}

// ============================================================================
// Execution Context
// ============================================================================

/// Context for cell execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Cell ID being executed
    pub cell_id: Uuid,

    /// Notebook ID
    pub notebook_id: Uuid,

    /// Execution count
    pub execution_count: u32,

    /// Working directory
    pub working_dir: Option<String>,

    /// Environment variables
    pub env_vars: std::collections::HashMap<String, String>,

    /// Cancellation flag
    cancelled: Arc<AtomicBool>,
}

impl ExecutionContext {
    /// Create new context
    pub fn new(cell_id: Uuid, notebook_id: Uuid, execution_count: u32) -> Self {
        Self {
            cell_id,
            notebook_id,
            execution_count,
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Cancel execution
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Get cancellation token
    pub fn cancellation_token(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }
}

// ============================================================================
// Cell Executor
// ============================================================================

/// Cell execution engine
pub struct CellExecutor {
    /// Configuration
    config: ExecutionConfig,

    /// Active executions
    active: RwLock<std::collections::HashSet<Uuid>>,

    /// Execution counter
    execution_count: AtomicU64,

    /// Statistics
    stats: ExecutorStats,
}

/// Executor statistics
#[derive(Debug, Default)]
struct ExecutorStats {
    total_executions: AtomicU64,
    successful_executions: AtomicU64,
    failed_executions: AtomicU64,
    total_duration_ms: AtomicU64,
}

impl CellExecutor {
    /// Create new executor
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            config,
            active: RwLock::new(std::collections::HashSet::new()),
            execution_count: AtomicU64::new(0),
            stats: ExecutorStats::default(),
        }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(ExecutionConfig::default())
    }

    /// Execute a cell
    pub async fn execute(&self, cell: &mut Cell) -> ExecResult<ExecutionResult> {
        // Check if cell is executable
        if !cell.cell_type.is_executable() {
            return Err(ExecutionError::NotExecutable(cell.cell_type));
        }

        // Check if already running
        {
            let active = self.active.read().await;
            if active.contains(&cell.id) {
                return Err(ExecutionError::AlreadyRunning(cell.id));
            }
        }

        // Mark as active
        {
            let mut active = self.active.write().await;
            active.insert(cell.id);
        }

        // Prepare execution
        let start = Instant::now();
        cell.mark_running();
        let exec_count = self.execution_count.fetch_add(1, Ordering::SeqCst) as u32 + 1;
        cell.execution_count = Some(exec_count);

        // Execute based on cell type
        let result = match cell.cell_type {
            CellType::Natural => self.execute_natural(cell).await,
            CellType::Command => self.execute_command(cell).await,
            CellType::Code => self.execute_code(cell).await,
            CellType::Markdown => {
                // Markdown cells don't execute, just return success
                Ok(vec![])
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        // Remove from active
        {
            let mut active = self.active.write().await;
            active.remove(&cell.id);
        }

        // Update statistics
        self.stats.total_executions.fetch_add(1, Ordering::Relaxed);
        self.stats.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);

        // Create result
        let execution_result = match result {
            Ok(outputs) => {
                self.stats.successful_executions.fetch_add(1, Ordering::Relaxed);
                cell.outputs = outputs.clone();
                cell.mark_success(duration_ms);
                ExecutionResult::success(cell.id, outputs, duration_ms)
            }
            Err(e) => {
                self.stats.failed_executions.fetch_add(1, Ordering::Relaxed);
                let error_msg = e.to_string();
                cell.mark_failed(&error_msg);
                ExecutionResult::failure(cell.id, error_msg, duration_ms)
            }
        };

        Ok(execution_result)
    }

    /// Execute natural language cell
    async fn execute_natural(&self, cell: &Cell) -> ExecResult<Vec<CellOutput>> {
        // For now, just return a placeholder
        // In full implementation, this would call the LLM client
        let response = format!(
            "[Natural Language Processing]\nInput: {}\n\nThis would be processed by LLM.",
            cell.source
        );

        Ok(vec![CellOutput::text(response)])
    }

    /// Execute command cell
    async fn execute_command(&self, cell: &Cell) -> ExecResult<Vec<CellOutput>> {
        let source = cell.source.trim();

        // Parse command
        if !source.starts_with('/') {
            return Err(ExecutionError::CommandNotFound(source.to_string()));
        }

        let parts: Vec<&str> = source[1..].splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).unwrap_or(&"");

        // Handle built-in commands (simplified)
        let output = match command {
            "help" => CellOutput::text(
                "Available commands:\n\
                 /help - Show this help\n\
                 /version - Show version\n\
                 /clear - Clear outputs\n\
                 /memory - Memory operations\n\
                 /trace - Trace operations"
            ),
            "version" => CellOutput::text(format!("RealConsole Notebook v{}", super::NOTEBOOK_VERSION)),
            "echo" => CellOutput::text(args.to_string()),
            _ => {
                return Err(ExecutionError::CommandNotFound(command.to_string()));
            }
        };

        Ok(vec![output])
    }

    /// Execute code cell
    async fn execute_code(&self, cell: &Cell) -> ExecResult<Vec<CellOutput>> {
        let source = cell.source.trim();

        // Extract shell command
        let command = if source.starts_with(&self.config.shell_prefix) {
            source[self.config.shell_prefix.len()..].trim().to_string()
        } else if source.starts_with("```") {
            // Extract from code block
            let lines: Vec<&str> = source.lines().collect();
            if lines.len() >= 2 {
                lines[1..lines.len()-1].join("\n")
            } else {
                return Ok(vec![CellOutput::text("Empty code block")]);
            }
        } else {
            source.to_string()
        };

        if command.is_empty() {
            return Ok(vec![CellOutput::text("No command to execute")]);
        }

        // Execute shell command with timeout
        let timeout = Duration::from_millis(self.config.timeout_ms);
        let result = tokio::time::timeout(timeout, self.run_shell_command(&command)).await;

        match result {
            Ok(Ok(output)) => Ok(vec![output]),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ExecutionError::Timeout(self.config.timeout_ms)),
        }
    }

    /// Run a shell command
    async fn run_shell_command(&self, command: &str) -> ExecResult<CellOutput> {
        let output = tokio::process::Command::new(&self.config.default_shell)
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| ExecutionError::ShellError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            if stderr.is_empty() {
                Ok(CellOutput::code("text", stdout.to_string()))
            } else {
                Ok(CellOutput::code("text", format!("{}\n{}", stdout, stderr)))
            }
        } else {
            let error_msg = if stderr.is_empty() {
                format!("Command failed with exit code: {:?}", output.status.code())
            } else {
                stderr.to_string()
            };
            Err(ExecutionError::ShellError(error_msg))
        }
    }

    /// Cancel execution
    pub async fn cancel(&self, cell_id: Uuid) -> bool {
        let active = self.active.read().await;
        active.contains(&cell_id)
        // Note: Full implementation would send cancellation signal
    }

    /// Get active execution count
    pub async fn active_count(&self) -> usize {
        self.active.read().await.len()
    }

    /// Get statistics
    pub fn stats(&self) -> ExecutorStatsSnapshot {
        ExecutorStatsSnapshot {
            total_executions: self.stats.total_executions.load(Ordering::Relaxed),
            successful_executions: self.stats.successful_executions.load(Ordering::Relaxed),
            failed_executions: self.stats.failed_executions.load(Ordering::Relaxed),
            total_duration_ms: self.stats.total_duration_ms.load(Ordering::Relaxed),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }
}

/// Statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorStatsSnapshot {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_duration_ms: u64,
}

impl ExecutorStatsSnapshot {
    /// Calculate average duration
    pub fn avg_duration_ms(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.total_duration_ms as f64 / self.total_executions as f64
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.timeout_ms, 30_000);
        assert_eq!(config.max_output_bytes, 1024 * 1024);
        assert!(config.stream_output);
    }

    #[test]
    fn test_execution_config_builder() {
        let config = ExecutionConfig::default()
            .with_timeout(60_000)
            .with_max_output(2 * 1024 * 1024)
            .with_streaming(false);

        assert_eq!(config.timeout_ms, 60_000);
        assert_eq!(config.max_output_bytes, 2 * 1024 * 1024);
        assert!(!config.stream_output);
    }

    #[test]
    fn test_execution_result_success() {
        let cell_id = Uuid::new_v4();
        let result = ExecutionResult::success(
            cell_id,
            vec![CellOutput::text("Hello")],
            100,
        );

        assert!(result.success);
        assert!(!result.has_error());
        assert_eq!(result.outputs.len(), 1);
    }

    #[test]
    fn test_execution_result_failure() {
        let cell_id = Uuid::new_v4();
        let result = ExecutionResult::failure(cell_id, "Something went wrong", 50);

        assert!(!result.success);
        assert!(result.has_error());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_execution_context() {
        let context = ExecutionContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
        )
        .with_working_dir("/tmp")
        .with_env("FOO", "bar");

        assert_eq!(context.working_dir, Some("/tmp".to_string()));
        assert_eq!(context.env_vars.get("FOO"), Some(&"bar".to_string()));
        assert!(!context.is_cancelled());
    }

    #[test]
    fn test_execution_context_cancellation() {
        let context = ExecutionContext::new(Uuid::new_v4(), Uuid::new_v4(), 1);

        assert!(!context.is_cancelled());
        context.cancel();
        assert!(context.is_cancelled());
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = CellExecutor::with_defaults();
        assert_eq!(executor.active_count().await, 0);
    }

    #[tokio::test]
    async fn test_executor_command_help() {
        let executor = CellExecutor::with_defaults();
        let mut cell = Cell::command("/help");

        let result = executor.execute(&mut cell).await.unwrap();
        assert!(result.success);
        assert!(!result.outputs.is_empty());
    }

    #[tokio::test]
    async fn test_executor_command_version() {
        let executor = CellExecutor::with_defaults();
        let mut cell = Cell::command("/version");

        let result = executor.execute(&mut cell).await.unwrap();
        assert!(result.success);

        if let CellOutput::Text { content } = &result.outputs[0] {
            assert!(content.contains("RealConsole"));
        }
    }

    #[tokio::test]
    async fn test_executor_command_echo() {
        let executor = CellExecutor::with_defaults();
        let mut cell = Cell::command("/echo Hello World");

        let result = executor.execute(&mut cell).await.unwrap();
        assert!(result.success);

        if let CellOutput::Text { content } = &result.outputs[0] {
            assert_eq!(content, "Hello World");
        }
    }

    #[tokio::test]
    async fn test_executor_command_not_found() {
        let executor = CellExecutor::with_defaults();
        let mut cell = Cell::command("/nonexistent");

        let result = executor.execute(&mut cell).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_executor_markdown_noop() {
        let executor = CellExecutor::with_defaults();
        let mut cell = Cell::markdown("# Title");

        // Markdown is not executable
        let result = executor.execute(&mut cell).await;
        assert!(matches!(result, Err(ExecutionError::NotExecutable(_))));
    }

    #[tokio::test]
    async fn test_executor_natural_placeholder() {
        let executor = CellExecutor::with_defaults();
        let mut cell = Cell::natural("Tell me a joke");

        let result = executor.execute(&mut cell).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_executor_stats() {
        let executor = CellExecutor::with_defaults();

        let mut cell1 = Cell::command("/help");
        let mut cell2 = Cell::command("/version");

        executor.execute(&mut cell1).await.unwrap();
        executor.execute(&mut cell2).await.unwrap();

        let stats = executor.stats();
        assert_eq!(stats.total_executions, 2);
        assert_eq!(stats.successful_executions, 2);
    }

    #[test]
    fn test_stats_snapshot_calculations() {
        let stats = ExecutorStatsSnapshot {
            total_executions: 10,
            successful_executions: 8,
            failed_executions: 2,
            total_duration_ms: 1000,
        };

        assert!((stats.avg_duration_ms() - 100.0).abs() < 0.001);
        assert!((stats.success_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_stats_snapshot_empty() {
        let stats = ExecutorStatsSnapshot {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_duration_ms: 0,
        };

        assert_eq!(stats.avg_duration_ms(), 0.0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_execution_error_display() {
        let err = ExecutionError::Timeout(5000);
        assert!(err.to_string().contains("5000"));

        let err = ExecutionError::Cancelled;
        assert!(err.to_string().contains("cancelled"));
    }
}
