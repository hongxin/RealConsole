//! v2.0.0-alpha.1: Notebook Core Types
//!
//! Defines the fundamental data structures for the Notebook system:
//! - **Cell**: Basic unit of content and execution
//! - **CellType**: Classification of cell content (Natural, Command, Code, Markdown)
//! - **CellState**: Execution lifecycle states
//! - **CellOutput**: Rich output types (Text, Code, Chart, Image, Table)
//! - **Notebook**: Container for cells with metadata

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Cell Type
// ============================================================================

/// Cell content type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    /// Natural language input - processed by LLM
    #[default]
    Natural,
    /// System command (/help, /trace, /memory)
    Command,
    /// Code block (Shell, Python, etc.)
    Code,
    /// Markdown documentation
    Markdown,
}

impl CellType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            CellType::Natural => "Natural",
            CellType::Command => "Command",
            CellType::Code => "Code",
            CellType::Markdown => "Markdown",
        }
    }

    /// Get icon for display
    pub fn icon(&self) -> &'static str {
        match self {
            CellType::Natural => "💬",
            CellType::Command => "⚡",
            CellType::Code => "📝",
            CellType::Markdown => "📄",
        }
    }

    /// Check if this cell type is executable
    pub fn is_executable(&self) -> bool {
        matches!(self, CellType::Natural | CellType::Command | CellType::Code)
    }

    /// Detect cell type from content
    pub fn detect(content: &str) -> Self {
        let trimmed = content.trim();
        if trimmed.starts_with('/') {
            CellType::Command
        } else if trimmed.starts_with("```") || trimmed.starts_with("!") {
            CellType::Code
        } else if trimmed.starts_with('#') || trimmed.starts_with("---") {
            CellType::Markdown
        } else {
            CellType::Natural
        }
    }
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// Cell State
// ============================================================================

/// Cell execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CellState {
    /// Not executed yet
    #[default]
    Idle,
    /// Queued for execution
    Pending,
    /// Currently executing
    Running,
    /// Execution completed successfully
    Success,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
}

impl CellState {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            CellState::Idle => "Idle",
            CellState::Pending => "Pending",
            CellState::Running => "Running",
            CellState::Success => "Success",
            CellState::Failed => "Failed",
            CellState::Cancelled => "Cancelled",
        }
    }

    /// Get icon for display
    pub fn icon(&self) -> &'static str {
        match self {
            CellState::Idle => "⚪",
            CellState::Pending => "🟡",
            CellState::Running => "🔵",
            CellState::Success => "🟢",
            CellState::Failed => "🔴",
            CellState::Cancelled => "⚫",
        }
    }

    /// Check if cell has been executed
    pub fn is_executed(&self) -> bool {
        matches!(self, CellState::Success | CellState::Failed | CellState::Cancelled)
    }

    /// Check if cell is currently running
    pub fn is_running(&self) -> bool {
        matches!(self, CellState::Running | CellState::Pending)
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        matches!(self, CellState::Success)
    }
}

impl std::fmt::Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// Cell Output
// ============================================================================

/// Rich output types for cell execution results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CellOutput {
    /// Plain text output
    Text {
        content: String,
    },

    /// Syntax-highlighted code
    Code {
        language: String,
        content: String,
    },

    /// Chart/visualization data
    Chart {
        chart_type: String,
        title: Option<String>,
        data: serde_json::Value,
    },

    /// Image data (base64 or URL)
    Image {
        mime_type: String,
        data: String,
        alt: Option<String>,
    },

    /// Tabular data
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },

    /// Error message
    Error {
        message: String,
        traceback: Option<String>,
    },

    /// Stream output (for real-time updates)
    Stream {
        name: String,  // "stdout" or "stderr"
        content: String,
    },
}

impl CellOutput {
    /// Create text output
    pub fn text(content: impl Into<String>) -> Self {
        CellOutput::Text {
            content: content.into(),
        }
    }

    /// Create code output
    pub fn code(language: impl Into<String>, content: impl Into<String>) -> Self {
        CellOutput::Code {
            language: language.into(),
            content: content.into(),
        }
    }

    /// Create error output
    pub fn error(message: impl Into<String>) -> Self {
        CellOutput::Error {
            message: message.into(),
            traceback: None,
        }
    }

    /// Create error output with traceback
    pub fn error_with_traceback(message: impl Into<String>, traceback: impl Into<String>) -> Self {
        CellOutput::Error {
            message: message.into(),
            traceback: Some(traceback.into()),
        }
    }

    /// Create table output
    pub fn table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        CellOutput::Table { headers, rows }
    }

    /// Create image output (base64)
    pub fn image_base64(mime_type: impl Into<String>, data: impl Into<String>) -> Self {
        CellOutput::Image {
            mime_type: mime_type.into(),
            data: data.into(),
            alt: None,
        }
    }

    /// Create stream output
    pub fn stdout(content: impl Into<String>) -> Self {
        CellOutput::Stream {
            name: "stdout".to_string(),
            content: content.into(),
        }
    }

    /// Create stream output
    pub fn stderr(content: impl Into<String>) -> Self {
        CellOutput::Stream {
            name: "stderr".to_string(),
            content: content.into(),
        }
    }

    /// Check if this is an error output
    pub fn is_error(&self) -> bool {
        matches!(self, CellOutput::Error { .. })
    }

    /// Get output type name
    pub fn type_name(&self) -> &'static str {
        match self {
            CellOutput::Text { .. } => "text",
            CellOutput::Code { .. } => "code",
            CellOutput::Chart { .. } => "chart",
            CellOutput::Image { .. } => "image",
            CellOutput::Table { .. } => "table",
            CellOutput::Error { .. } => "error",
            CellOutput::Stream { .. } => "stream",
        }
    }
}

// ============================================================================
// Cell Metadata
// ============================================================================

/// Cell metadata for additional information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CellMetadata {
    /// Language hint for code cells
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Cell tags for organization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Whether cell is collapsed in UI
    #[serde(default)]
    pub collapsed: bool,

    /// Whether cell is editable
    #[serde(default = "default_true")]
    pub editable: bool,

    /// Custom key-value pairs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl CellMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set collapsed state
    pub fn with_collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Set custom value
    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

// ============================================================================
// Cell
// ============================================================================

/// A single cell in the notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Unique cell identifier
    pub id: Uuid,

    /// Cell content type
    pub cell_type: CellType,

    /// Source content (user input)
    pub source: String,

    /// Execution state
    pub state: CellState,

    /// Execution outputs
    #[serde(default)]
    pub outputs: Vec<CellOutput>,

    /// Execution count (for display, like Jupyter's [1], [2], etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_count: Option<u32>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last execution timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_at: Option<DateTime<Utc>>,

    /// Execution duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Cell metadata
    #[serde(default)]
    pub metadata: CellMetadata,
}

impl Cell {
    /// Create new cell
    pub fn new(cell_type: CellType, source: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            cell_type,
            source: source.into(),
            state: CellState::Idle,
            outputs: Vec::new(),
            execution_count: None,
            created_at: Utc::now(),
            executed_at: None,
            duration_ms: None,
            metadata: CellMetadata::default(),
        }
    }

    /// Create natural language cell
    pub fn natural(source: impl Into<String>) -> Self {
        Self::new(CellType::Natural, source)
    }

    /// Create command cell
    pub fn command(source: impl Into<String>) -> Self {
        Self::new(CellType::Command, source)
    }

    /// Create code cell
    pub fn code(source: impl Into<String>) -> Self {
        Self::new(CellType::Code, source)
    }

    /// Create markdown cell
    pub fn markdown(source: impl Into<String>) -> Self {
        Self::new(CellType::Markdown, source)
    }

    /// Create cell with auto-detected type
    pub fn auto(source: impl Into<String>) -> Self {
        let source = source.into();
        let cell_type = CellType::detect(&source);
        Self::new(cell_type, source)
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: CellMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// Check if cell can be executed
    pub fn can_execute(&self) -> bool {
        self.cell_type.is_executable() && !self.state.is_running()
    }

    /// Check if cell has outputs
    pub fn has_outputs(&self) -> bool {
        !self.outputs.is_empty()
    }

    /// Check if cell has errors
    pub fn has_errors(&self) -> bool {
        self.outputs.iter().any(|o| o.is_error())
    }

    /// Clear outputs
    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
        self.state = CellState::Idle;
        self.executed_at = None;
        self.duration_ms = None;
    }

    /// Add output
    pub fn add_output(&mut self, output: CellOutput) {
        self.outputs.push(output);
    }

    /// Mark as running
    pub fn mark_running(&mut self) {
        self.state = CellState::Running;
        self.outputs.clear();
    }

    /// Mark as success
    pub fn mark_success(&mut self, duration_ms: u64) {
        self.state = CellState::Success;
        self.executed_at = Some(Utc::now());
        self.duration_ms = Some(duration_ms);
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.state = CellState::Failed;
        self.executed_at = Some(Utc::now());
        self.add_output(CellOutput::error(error));
    }

    /// Mark as cancelled
    pub fn mark_cancelled(&mut self) {
        self.state = CellState::Cancelled;
    }

    /// Get preview of source (first line, truncated)
    pub fn source_preview(&self, max_len: usize) -> String {
        let first_line = self.source.lines().next().unwrap_or("");
        if first_line.len() > max_len {
            format!("{}...", &first_line[..max_len])
        } else {
            first_line.to_string()
        }
    }
}

// ============================================================================
// Notebook Metadata
// ============================================================================

/// Notebook metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NotebookMetadata {
    /// Notebook description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Author name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Tags for organization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Language kernel (e.g., "shell", "python")
    #[serde(default = "default_kernel")]
    pub kernel: String,

    /// Format version
    #[serde(default = "default_version")]
    pub format_version: String,

    /// Custom key-value pairs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

fn default_kernel() -> String {
    "shell".to_string()
}

fn default_version() -> String {
    "2.0.0-alpha.1".to_string()
}

impl NotebookMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set kernel
    pub fn with_kernel(mut self, kernel: impl Into<String>) -> Self {
        self.kernel = kernel.into();
        self
    }
}

// ============================================================================
// Notebook
// ============================================================================

/// A notebook containing cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    /// Unique notebook identifier
    pub id: Uuid,

    /// Notebook name
    pub name: String,

    /// Cells in order
    pub cells: Vec<Cell>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,

    /// Notebook metadata
    #[serde(default)]
    pub metadata: NotebookMetadata,

    /// Global execution counter
    #[serde(default)]
    execution_counter: u32,
}

impl Notebook {
    /// Create new notebook
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            cells: Vec::new(),
            created_at: now,
            modified_at: now,
            metadata: NotebookMetadata::default(),
            execution_counter: 0,
        }
    }

    /// Create notebook from parts (used for deserialization)
    pub fn from_parts(
        id: Uuid,
        name: String,
        cells: Vec<Cell>,
        created_at: DateTime<Utc>,
        modified_at: DateTime<Utc>,
        metadata: NotebookMetadata,
    ) -> Self {
        // Calculate execution counter from cells
        let execution_counter = cells
            .iter()
            .filter_map(|c| c.execution_count)
            .max()
            .unwrap_or(0);

        Self {
            id,
            name,
            cells,
            created_at,
            modified_at,
            metadata,
            execution_counter,
        }
    }

    /// Create with metadata
    pub fn with_metadata(mut self, metadata: NotebookMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add a cell to the end
    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
        self.mark_modified();
    }

    /// Insert cell at index
    pub fn insert_cell(&mut self, index: usize, cell: Cell) {
        if index <= self.cells.len() {
            self.cells.insert(index, cell);
            self.mark_modified();
        }
    }

    /// Remove cell by id
    pub fn remove_cell(&mut self, cell_id: Uuid) -> Option<Cell> {
        if let Some(pos) = self.cells.iter().position(|c| c.id == cell_id) {
            self.mark_modified();
            Some(self.cells.remove(pos))
        } else {
            None
        }
    }

    /// Get cell by id
    pub fn get_cell(&self, cell_id: Uuid) -> Option<&Cell> {
        self.cells.iter().find(|c| c.id == cell_id)
    }

    /// Get mutable cell by id
    pub fn get_cell_mut(&mut self, cell_id: Uuid) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| c.id == cell_id)
    }

    /// Move cell to new position
    pub fn move_cell(&mut self, cell_id: Uuid, new_index: usize) -> bool {
        if let Some(old_pos) = self.cells.iter().position(|c| c.id == cell_id) {
            if new_index < self.cells.len() && old_pos != new_index {
                let cell = self.cells.remove(old_pos);
                let insert_pos = if new_index > old_pos {
                    new_index - 1
                } else {
                    new_index
                };
                self.cells.insert(insert_pos.min(self.cells.len()), cell);
                self.mark_modified();
                return true;
            }
        }
        false
    }

    /// Get next execution count
    pub fn next_execution_count(&mut self) -> u32 {
        self.execution_counter += 1;
        self.execution_counter
    }

    /// Count cells
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Count executed cells
    pub fn executed_cell_count(&self) -> usize {
        self.cells.iter().filter(|c| c.state.is_executed()).count()
    }

    /// Check if notebook is empty
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Clear all cells
    pub fn clear(&mut self) {
        self.cells.clear();
        self.execution_counter = 0;
        self.mark_modified();
    }

    /// Clear all outputs
    pub fn clear_all_outputs(&mut self) {
        for cell in &mut self.cells {
            cell.clear_outputs();
        }
        self.mark_modified();
    }

    /// Mark as modified
    fn mark_modified(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Get cells by type
    pub fn cells_by_type(&self, cell_type: CellType) -> Vec<&Cell> {
        self.cells.iter().filter(|c| c.cell_type == cell_type).collect()
    }

    /// Get cells by state
    pub fn cells_by_state(&self, state: CellState) -> Vec<&Cell> {
        self.cells.iter().filter(|c| c.state == state).collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_type_detect() {
        assert_eq!(CellType::detect("/help"), CellType::Command);
        assert_eq!(CellType::detect("/trace"), CellType::Command);
        assert_eq!(CellType::detect("!ls -la"), CellType::Code);
        assert_eq!(CellType::detect("```python\nprint('hi')```"), CellType::Code);
        assert_eq!(CellType::detect("# Title"), CellType::Markdown);
        assert_eq!(CellType::detect("---"), CellType::Markdown);
        assert_eq!(CellType::detect("Hello, how are you?"), CellType::Natural);
    }

    #[test]
    fn test_cell_type_properties() {
        assert!(CellType::Natural.is_executable());
        assert!(CellType::Command.is_executable());
        assert!(CellType::Code.is_executable());
        assert!(!CellType::Markdown.is_executable());
    }

    #[test]
    fn test_cell_state_properties() {
        assert!(!CellState::Idle.is_executed());
        assert!(CellState::Pending.is_running());
        assert!(CellState::Running.is_running());
        assert!(CellState::Success.is_executed());
        assert!(CellState::Success.is_success());
        assert!(CellState::Failed.is_executed());
        assert!(!CellState::Failed.is_success());
    }

    #[test]
    fn test_cell_output_creation() {
        let text = CellOutput::text("Hello");
        assert_eq!(text.type_name(), "text");
        assert!(!text.is_error());

        let error = CellOutput::error("Something went wrong");
        assert_eq!(error.type_name(), "error");
        assert!(error.is_error());

        let code = CellOutput::code("python", "print('hi')");
        assert_eq!(code.type_name(), "code");

        let table = CellOutput::table(
            vec!["Name".to_string(), "Age".to_string()],
            vec![vec!["Alice".to_string(), "30".to_string()]],
        );
        assert_eq!(table.type_name(), "table");
    }

    #[test]
    fn test_cell_creation() {
        let cell = Cell::natural("Hello world");
        assert_eq!(cell.cell_type, CellType::Natural);
        assert_eq!(cell.source, "Hello world");
        assert_eq!(cell.state, CellState::Idle);
        assert!(cell.outputs.is_empty());
        assert!(cell.can_execute());
    }

    #[test]
    fn test_cell_auto_detection() {
        let natural = Cell::auto("Tell me a joke");
        assert_eq!(natural.cell_type, CellType::Natural);

        let command = Cell::auto("/help");
        assert_eq!(command.cell_type, CellType::Command);

        let code = Cell::auto("!echo hello");
        assert_eq!(code.cell_type, CellType::Code);

        let markdown = Cell::auto("# Title");
        assert_eq!(markdown.cell_type, CellType::Markdown);
    }

    #[test]
    fn test_cell_execution_lifecycle() {
        let mut cell = Cell::natural("test");

        // Initial state
        assert_eq!(cell.state, CellState::Idle);
        assert!(cell.can_execute());

        // Mark running
        cell.mark_running();
        assert_eq!(cell.state, CellState::Running);
        assert!(!cell.can_execute());

        // Mark success
        cell.add_output(CellOutput::text("Result"));
        cell.mark_success(100);
        assert_eq!(cell.state, CellState::Success);
        assert!(cell.has_outputs());
        assert_eq!(cell.duration_ms, Some(100));
    }

    #[test]
    fn test_cell_failure() {
        let mut cell = Cell::code("invalid command");
        cell.mark_failed("Command not found");

        assert_eq!(cell.state, CellState::Failed);
        assert!(cell.has_errors());
    }

    #[test]
    fn test_cell_metadata() {
        let metadata = CellMetadata::new()
            .with_language("python")
            .with_tag("test")
            .with_collapsed(true);

        let cell = Cell::code("print('hi')").with_metadata(metadata);
        assert_eq!(cell.metadata.language, Some("python".to_string()));
        assert!(cell.metadata.tags.contains(&"test".to_string()));
        assert!(cell.metadata.collapsed);
    }

    #[test]
    fn test_notebook_creation() {
        let notebook = Notebook::new("Test Notebook");
        assert_eq!(notebook.name, "Test Notebook");
        assert!(notebook.is_empty());
        assert_eq!(notebook.cell_count(), 0);
    }

    #[test]
    fn test_notebook_add_cells() {
        let mut notebook = Notebook::new("Test");

        notebook.add_cell(Cell::natural("First"));
        notebook.add_cell(Cell::command("/help"));
        notebook.add_cell(Cell::code("!ls"));

        assert_eq!(notebook.cell_count(), 3);
        assert!(!notebook.is_empty());
    }

    #[test]
    fn test_notebook_get_cell() {
        let mut notebook = Notebook::new("Test");
        let cell = Cell::natural("Test cell");
        let cell_id = cell.id;
        notebook.add_cell(cell);

        let found = notebook.get_cell(cell_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().source, "Test cell");
    }

    #[test]
    fn test_notebook_remove_cell() {
        let mut notebook = Notebook::new("Test");
        let cell = Cell::natural("Test");
        let cell_id = cell.id;
        notebook.add_cell(cell);

        let removed = notebook.remove_cell(cell_id);
        assert!(removed.is_some());
        assert!(notebook.is_empty());
    }

    #[test]
    fn test_notebook_insert_cell() {
        let mut notebook = Notebook::new("Test");
        notebook.add_cell(Cell::natural("First"));
        notebook.add_cell(Cell::natural("Third"));
        notebook.insert_cell(1, Cell::natural("Second"));

        assert_eq!(notebook.cells[0].source, "First");
        assert_eq!(notebook.cells[1].source, "Second");
        assert_eq!(notebook.cells[2].source, "Third");
    }

    #[test]
    fn test_notebook_execution_counter() {
        let mut notebook = Notebook::new("Test");

        assert_eq!(notebook.next_execution_count(), 1);
        assert_eq!(notebook.next_execution_count(), 2);
        assert_eq!(notebook.next_execution_count(), 3);
    }

    #[test]
    fn test_notebook_clear_outputs() {
        let mut notebook = Notebook::new("Test");
        let mut cell = Cell::natural("Test");
        cell.add_output(CellOutput::text("Output"));
        cell.mark_success(100);
        notebook.add_cell(cell);

        notebook.clear_all_outputs();

        assert!(!notebook.cells[0].has_outputs());
        assert_eq!(notebook.cells[0].state, CellState::Idle);
    }

    #[test]
    fn test_notebook_cells_by_type() {
        let mut notebook = Notebook::new("Test");
        notebook.add_cell(Cell::natural("Natural 1"));
        notebook.add_cell(Cell::command("/help"));
        notebook.add_cell(Cell::natural("Natural 2"));

        let natural_cells = notebook.cells_by_type(CellType::Natural);
        assert_eq!(natural_cells.len(), 2);

        let command_cells = notebook.cells_by_type(CellType::Command);
        assert_eq!(command_cells.len(), 1);
    }

    #[test]
    fn test_notebook_metadata() {
        let metadata = NotebookMetadata::new()
            .with_description("Test notebook")
            .with_author("Test Author")
            .with_kernel("python");

        let notebook = Notebook::new("Test").with_metadata(metadata);

        assert_eq!(notebook.metadata.description, Some("Test notebook".to_string()));
        assert_eq!(notebook.metadata.author, Some("Test Author".to_string()));
        assert_eq!(notebook.metadata.kernel, "python");
    }

    #[test]
    fn test_cell_serialization() {
        let cell = Cell::natural("Hello")
            .with_tag("test");

        let json = serde_json::to_string(&cell).unwrap();
        let deserialized: Cell = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.source, "Hello");
        assert_eq!(deserialized.cell_type, CellType::Natural);
        assert!(deserialized.metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_notebook_serialization() {
        let mut notebook = Notebook::new("Test");
        notebook.add_cell(Cell::natural("Cell 1"));
        notebook.add_cell(Cell::code("!ls"));

        let json = serde_json::to_string_pretty(&notebook).unwrap();
        let deserialized: Notebook = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Test");
        assert_eq!(deserialized.cell_count(), 2);
    }

    #[test]
    fn test_cell_source_preview() {
        let cell = Cell::natural("This is a very long line that should be truncated for preview purposes");
        let preview = cell.source_preview(20);
        assert!(preview.len() <= 23); // 20 + "..."
    }

    #[test]
    fn test_cell_output_stream() {
        let stdout = CellOutput::stdout("Hello\n");
        let stderr = CellOutput::stderr("Error\n");

        match stdout {
            CellOutput::Stream { name, content } => {
                assert_eq!(name, "stdout");
                assert_eq!(content, "Hello\n");
            }
            _ => panic!("Expected Stream"),
        }

        match stderr {
            CellOutput::Stream { name, content } => {
                assert_eq!(name, "stderr");
                assert_eq!(content, "Error\n");
            }
            _ => panic!("Expected Stream"),
        }
    }
}
