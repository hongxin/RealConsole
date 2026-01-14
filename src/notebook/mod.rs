//! v2.0.0-alpha.1: Notebook System
//!
//! The Notebook module provides an interactive computational environment
//! that combines natural language, commands, and code in a single document.
//!
//! # Core Concepts
//!
//! - **Cell**: The basic unit of content and execution
//! - **Notebook**: A container of cells with metadata
//! - **CellType**: Classification (Natural, Command, Code, Markdown)
//! - **CellOutput**: Rich output types (Text, Code, Chart, Image, Table)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        Notebook                              │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │ Cell 1 (Natural): "分析这段代码的性能"                   ││
//! │  │ → Output: [Text: "这段代码有以下性能问题..."]            ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │ Cell 2 (Code): "!cargo bench"                            ││
//! │  │ → Output: [Code: "test bench_sort ... 1,234 ns/iter"]    ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │ Cell 3 (Command): "/memory save performance-analysis"    ││
//! │  │ → Output: [Text: "已保存到记忆系统"]                      ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Quick Start
//!
//! ```rust
//! use realconsole::notebook::{Notebook, Cell, CellType, CellOutput};
//!
//! // Create a new notebook
//! let mut notebook = Notebook::new("My Analysis");
//!
//! // Add cells
//! notebook.add_cell(Cell::natural("分析当前目录的文件结构"));
//! notebook.add_cell(Cell::code("!tree -L 2"));
//! notebook.add_cell(Cell::markdown("## 结果分析\n\n以下是分析结果..."));
//!
//! // Check notebook state
//! println!("Cells: {}", notebook.cell_count());
//! println!("Executed: {}", notebook.executed_cell_count());
//! ```
//!
//! # File Format (.rcnb)
//!
//! Notebooks are stored in `.rcnb` format (RealConsole Notebook):
//! - Line 1: Notebook metadata (JSON)
//! - Lines 2+: One cell per line (JSON)
//!
//! This format is designed for:
//! - Git-friendly diffs (one line per cell)
//! - Streaming reads (no need to load entire file)
//! - Easy parsing and generation
//!
//! # Modules
//!
//! - [`types`]: Core data structures (Cell, Notebook, CellOutput)
//! - [`storage`]: Storage abstraction for notebooks
//! - [`execution`]: Cell execution engine
//! - [`persistence`]: .rcnb file format handling

pub mod types;
pub mod storage;
pub mod execution;
pub mod persistence;

// Re-export core types
pub use types::{
    Cell, CellMetadata, CellOutput, CellState, CellType,
    Notebook, NotebookMetadata,
};

pub use storage::{
    NotebookStorage, NotebookStorageError, FileNotebookStorage,
    MemoryNotebookStorage, NotebookIndex, NotebookIndexEntry,
};

pub use execution::{
    CellExecutor, ExecutionConfig, ExecutionResult, ExecutionError,
};

pub use persistence::{
    RcnbFormat, RcnbError, RCNB_EXTENSION, RCNB_VERSION,
};

/// Notebook format version
pub const NOTEBOOK_VERSION: &str = "2.0.0-alpha.1";

/// Default notebook file extension
pub const NOTEBOOK_EXTENSION: &str = "rcnb";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that core types are exported
        let _cell_type = CellType::Natural;
        let _cell_state = CellState::Idle;
        let _output = CellOutput::text("test");
        let _cell = Cell::natural("test");
        let _notebook = Notebook::new("test");
    }

    #[test]
    fn test_version_constants() {
        assert!(!NOTEBOOK_VERSION.is_empty());
        assert_eq!(NOTEBOOK_EXTENSION, "rcnb");
    }
}
