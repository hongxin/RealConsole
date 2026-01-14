//! .rcnb File Format Persistence
//!
//! This module handles reading and writing notebooks in the .rcnb format.
//! The format is designed for:
//! - Git-friendly diffs (one line per cell)
//! - Streaming reads (no need to load entire file)
//! - Easy parsing and generation
//!
//! # Format Specification
//!
//! ```text
//! Line 1: {"version":"2.0.0-alpha.1","id":"...","name":"...","created_at":"...","modified_at":"...","metadata":{...}}
//! Line 2: {"id":"...","cell_type":"natural","source":"...","state":"idle","outputs":[],...}
//! Line 3: {"id":"...","cell_type":"code","source":"...","state":"success","outputs":[...],...}
//! ...
//! ```
//!
//! Each line is a valid JSON object, making the file easy to:
//! - Parse line-by-line for streaming
//! - Diff in version control
//! - Validate independently

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::fs::File;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::types::{Cell, Notebook, NotebookMetadata};

/// .rcnb file extension
pub const RCNB_EXTENSION: &str = "rcnb";

/// .rcnb format version
pub const RCNB_VERSION: &str = "2.0.0-alpha.1";

/// Errors that can occur during .rcnb operations
#[derive(Debug, Error)]
pub enum RcnbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    #[error("Missing header line")]
    MissingHeader,

    #[error("Empty file")]
    EmptyFile,

    #[error("Invalid cell at line {line}: {message}")]
    InvalidCell { line: usize, message: String },
}

/// Result type for .rcnb operations
pub type RcnbResult<T> = Result<T, RcnbError>;

/// Header structure for .rcnb files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcnbHeader {
    pub version: String,
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    #[serde(default)]
    pub metadata: NotebookMetadata,
}

impl RcnbHeader {
    /// Create header from notebook
    pub fn from_notebook(notebook: &Notebook) -> Self {
        Self {
            version: RCNB_VERSION.to_string(),
            id: notebook.id,
            name: notebook.name.clone(),
            created_at: notebook.created_at,
            modified_at: notebook.modified_at,
            metadata: notebook.metadata.clone(),
        }
    }
}

/// .rcnb format handler
#[derive(Debug, Default)]
pub struct RcnbFormat;

impl RcnbFormat {
    /// Create a new RcnbFormat instance
    pub fn new() -> Self {
        Self
    }

    /// Write a notebook to a file
    pub fn write<P: AsRef<Path>>(&self, path: P, notebook: &Notebook) -> RcnbResult<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write header
        let header = RcnbHeader::from_notebook(notebook);
        let header_json = serde_json::to_string(&header)
            .map_err(|e| RcnbError::InvalidFormat(format!("Failed to serialize header: {}", e)))?;
        writeln!(writer, "{}", header_json)?;

        // Write each cell on its own line
        for cell in &notebook.cells {
            let cell_json = serde_json::to_string(cell)
                .map_err(|e| RcnbError::InvalidFormat(format!("Failed to serialize cell: {}", e)))?;
            writeln!(writer, "{}", cell_json)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Write notebook to a string
    pub fn write_to_string(&self, notebook: &Notebook) -> RcnbResult<String> {
        let mut output = String::new();

        // Write header
        let header = RcnbHeader::from_notebook(notebook);
        let header_json = serde_json::to_string(&header)
            .map_err(|e| RcnbError::InvalidFormat(format!("Failed to serialize header: {}", e)))?;
        output.push_str(&header_json);
        output.push('\n');

        // Write each cell
        for cell in &notebook.cells {
            let cell_json = serde_json::to_string(cell)
                .map_err(|e| RcnbError::InvalidFormat(format!("Failed to serialize cell: {}", e)))?;
            output.push_str(&cell_json);
            output.push('\n');
        }

        Ok(output)
    }

    /// Read a notebook from a file
    pub fn read<P: AsRef<Path>>(&self, path: P) -> RcnbResult<Notebook> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.read_from_reader(reader)
    }

    /// Read notebook from a string
    pub fn read_from_string(&self, content: &str) -> RcnbResult<Notebook> {
        let reader = BufReader::new(content.as_bytes());
        self.read_from_reader(reader)
    }

    /// Read notebook from a reader
    fn read_from_reader<R: BufRead>(&self, reader: R) -> RcnbResult<Notebook> {
        let mut lines = reader.lines();

        // Read header (first line)
        let header_line = lines.next().ok_or(RcnbError::EmptyFile)??;
        if header_line.is_empty() {
            return Err(RcnbError::MissingHeader);
        }

        let header: RcnbHeader = serde_json::from_str(&header_line)
            .map_err(|e| RcnbError::ParseError {
                line: 1,
                message: format!("Invalid header: {}", e),
            })?;

        // Version compatibility check (warn but continue for minor versions)
        if !header.version.starts_with("2.") {
            return Err(RcnbError::VersionMismatch {
                expected: RCNB_VERSION.to_string(),
                actual: header.version,
            });
        }

        // Read cells
        let mut cells = Vec::new();
        for (line_num, line_result) in lines.enumerate() {
            let line = line_result?;
            if line.is_empty() {
                continue; // Skip empty lines
            }

            let cell: Cell = serde_json::from_str(&line)
                .map_err(|e| RcnbError::InvalidCell {
                    line: line_num + 2, // +2 because header is line 1, and enumerate starts at 0
                    message: e.to_string(),
                })?;
            cells.push(cell);
        }

        Ok(Notebook::from_parts(
            header.id,
            header.name,
            cells,
            header.created_at,
            header.modified_at,
            header.metadata,
        ))
    }

    /// Validate a .rcnb file without fully loading it
    pub fn validate<P: AsRef<Path>>(&self, path: P) -> RcnbResult<ValidationResult> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Check header
        let header_line = lines.next().ok_or(RcnbError::EmptyFile)??;
        if header_line.is_empty() {
            return Err(RcnbError::MissingHeader);
        }

        let header: RcnbHeader = serde_json::from_str(&header_line)
            .map_err(|e| RcnbError::ParseError {
                line: 1,
                message: format!("Invalid header: {}", e),
            })?;

        // Validate each cell line
        let mut cell_count = 0;
        let mut errors = Vec::new();

        for (line_num, line_result) in lines.enumerate() {
            match line_result {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<Cell>(&line) {
                        Ok(_) => cell_count += 1,
                        Err(e) => errors.push(ValidationError {
                            line: line_num + 2,
                            message: e.to_string(),
                        }),
                    }
                }
                Err(e) => errors.push(ValidationError {
                    line: line_num + 2,
                    message: format!("IO error: {}", e),
                }),
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            version: header.version,
            notebook_id: header.id,
            notebook_name: header.name,
            cell_count,
            errors,
        })
    }

    /// Append a cell to an existing .rcnb file
    pub fn append_cell<P: AsRef<Path>>(&self, path: P, cell: &Cell) -> RcnbResult<()> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(path)?;

        let cell_json = serde_json::to_string(cell)
            .map_err(|e| RcnbError::InvalidFormat(format!("Failed to serialize cell: {}", e)))?;
        writeln!(file, "{}", cell_json)?;

        Ok(())
    }

    /// Update the header of an existing .rcnb file (updates modified_at, etc.)
    pub fn update_header<P: AsRef<Path>>(&self, path: P, notebook: &Notebook) -> RcnbResult<()> {
        // Read the entire file
        let content = std::fs::read_to_string(&path)?;
        let mut lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err(RcnbError::EmptyFile);
        }

        // Create new header
        let header = RcnbHeader::from_notebook(notebook);
        let header_json = serde_json::to_string(&header)
            .map_err(|e| RcnbError::InvalidFormat(format!("Failed to serialize header: {}", e)))?;

        // Replace first line
        lines[0] = &header_json;

        // Write back
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for line in lines {
            writeln!(writer, "{}", line)?;
        }
        writer.flush()?;

        Ok(())
    }

    /// Get file statistics without loading the full notebook
    pub fn file_stats<P: AsRef<Path>>(&self, path: P) -> RcnbResult<FileStats> {
        let metadata = std::fs::metadata(&path)?;
        let file_size = metadata.len();

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Read header
        let header_line = lines.next().ok_or(RcnbError::EmptyFile)??;
        let header: RcnbHeader = serde_json::from_str(&header_line)
            .map_err(|e| RcnbError::ParseError {
                line: 1,
                message: e.to_string(),
            })?;

        // Count cells
        let mut cell_count = 0;
        for line_result in lines {
            if let Ok(line) = line_result {
                if !line.is_empty() {
                    cell_count += 1;
                }
            }
        }

        Ok(FileStats {
            file_size,
            cell_count,
            notebook_id: header.id,
            notebook_name: header.name,
            version: header.version,
            created_at: header.created_at,
            modified_at: header.modified_at,
        })
    }
}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub line: usize,
    pub message: String,
}

/// Result of validating a .rcnb file
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub version: String,
    pub notebook_id: Uuid,
    pub notebook_name: String,
    pub cell_count: usize,
    pub errors: Vec<ValidationError>,
}

/// Statistics about a .rcnb file
#[derive(Debug)]
pub struct FileStats {
    pub file_size: u64,
    pub cell_count: usize,
    pub notebook_id: Uuid,
    pub notebook_name: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::types::{CellOutput, CellType};
    use tempfile::tempdir;

    fn create_test_notebook() -> Notebook {
        let mut notebook = Notebook::new("Test Notebook");
        notebook.add_cell(Cell::natural("Hello, world!"));
        notebook.add_cell(Cell::code("!echo test"));
        notebook.add_cell(Cell::markdown("## Section 1"));
        notebook
    }

    #[test]
    fn test_rcnb_format_constants() {
        assert_eq!(RCNB_EXTENSION, "rcnb");
        assert!(RCNB_VERSION.starts_with("2."));
    }

    #[test]
    fn test_write_and_read_roundtrip() {
        let format = RcnbFormat::new();
        let notebook = create_test_notebook();

        // Write to string
        let content = format.write_to_string(&notebook).unwrap();

        // Read back
        let loaded = format.read_from_string(&content).unwrap();

        assert_eq!(loaded.id, notebook.id);
        assert_eq!(loaded.name, notebook.name);
        assert_eq!(loaded.cells.len(), notebook.cells.len());
        assert_eq!(loaded.cells[0].source, "Hello, world!");
        assert_eq!(loaded.cells[1].source, "!echo test");
    }

    #[test]
    fn test_write_and_read_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rcnb");

        let format = RcnbFormat::new();
        let notebook = create_test_notebook();

        // Write to file
        format.write(&path, &notebook).unwrap();

        // Verify file exists
        assert!(path.exists());

        // Read back
        let loaded = format.read(&path).unwrap();

        assert_eq!(loaded.id, notebook.id);
        assert_eq!(loaded.name, notebook.name);
        assert_eq!(loaded.cells.len(), 3);
    }

    #[test]
    fn test_format_is_json_lines() {
        let format = RcnbFormat::new();
        let notebook = create_test_notebook();

        let content = format.write_to_string(&notebook).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Should have header + 3 cells = 4 lines
        assert_eq!(lines.len(), 4);

        // Each line should be valid JSON
        for line in &lines {
            assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
        }
    }

    #[test]
    fn test_header_contains_version() {
        let format = RcnbFormat::new();
        let notebook = create_test_notebook();

        let content = format.write_to_string(&notebook).unwrap();
        let first_line = content.lines().next().unwrap();

        let header: RcnbHeader = serde_json::from_str(first_line).unwrap();
        assert_eq!(header.version, RCNB_VERSION);
        assert_eq!(header.name, "Test Notebook");
    }

    #[test]
    fn test_validate_valid_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("valid.rcnb");

        let format = RcnbFormat::new();
        let notebook = create_test_notebook();
        format.write(&path, &notebook).unwrap();

        let result = format.validate(&path).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.cell_count, 3);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_cell() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.rcnb");

        // Write valid header + invalid cell
        let mut file = File::create(&path).unwrap();
        let header = RcnbHeader {
            version: RCNB_VERSION.to_string(),
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            metadata: NotebookMetadata::default(),
        };
        writeln!(file, "{}", serde_json::to_string(&header).unwrap()).unwrap();
        writeln!(file, "{{\"invalid\": \"cell\"}}").unwrap();

        let format = RcnbFormat::new();
        let result = format.validate(&path).unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].line, 2);
    }

    #[test]
    fn test_file_stats() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stats.rcnb");

        let format = RcnbFormat::new();
        let notebook = create_test_notebook();
        format.write(&path, &notebook).unwrap();

        let stats = format.file_stats(&path).unwrap();
        assert_eq!(stats.cell_count, 3);
        assert_eq!(stats.notebook_name, "Test Notebook");
        assert!(stats.file_size > 0);
    }

    #[test]
    fn test_append_cell() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("append.rcnb");

        let format = RcnbFormat::new();
        let notebook = create_test_notebook();
        format.write(&path, &notebook).unwrap();

        // Append a new cell
        let new_cell = Cell::natural("Appended cell");
        format.append_cell(&path, &new_cell).unwrap();

        // Read back and verify
        let loaded = format.read(&path).unwrap();
        assert_eq!(loaded.cells.len(), 4);
        assert_eq!(loaded.cells[3].source, "Appended cell");
    }

    #[test]
    fn test_empty_file_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.rcnb");

        // Create empty file
        File::create(&path).unwrap();

        let format = RcnbFormat::new();
        let result = format.read(&path);

        assert!(matches!(result, Err(RcnbError::EmptyFile)));
    }

    #[test]
    fn test_version_mismatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("old.rcnb");

        // Write file with old version
        let mut file = File::create(&path).unwrap();
        writeln!(file, "{{\"version\":\"1.0.0\",\"id\":\"00000000-0000-0000-0000-000000000000\",\"name\":\"Old\",\"created_at\":\"2024-01-01T00:00:00Z\",\"modified_at\":\"2024-01-01T00:00:00Z\"}}").unwrap();

        let format = RcnbFormat::new();
        let result = format.read(&path);

        assert!(matches!(result, Err(RcnbError::VersionMismatch { .. })));
    }

    #[test]
    fn test_cell_with_outputs() {
        let format = RcnbFormat::new();
        let mut notebook = Notebook::new("Output Test");

        let mut cell = Cell::code("!echo hello");
        cell.outputs.push(CellOutput::text("hello"));
        cell.outputs.push(CellOutput::code("bash", "$ echo hello\nhello"));
        notebook.add_cell(cell);

        let content = format.write_to_string(&notebook).unwrap();
        let loaded = format.read_from_string(&content).unwrap();

        assert_eq!(loaded.cells[0].outputs.len(), 2);
    }

    #[test]
    fn test_skip_empty_lines() {
        let content = r#"{"version":"2.0.0-alpha.1","id":"00000000-0000-0000-0000-000000000001","name":"Test","created_at":"2024-01-01T00:00:00Z","modified_at":"2024-01-01T00:00:00Z","metadata":{"tags":[],"kernel":"shell","format_version":"2.0.0-alpha.1"}}
{"id":"00000000-0000-0000-0000-000000000002","cell_type":"natural","source":"test","state":"idle","outputs":[],"execution_count":null,"created_at":"2024-01-01T00:00:00Z","executed_at":null,"duration_ms":null,"metadata":{"collapsed":false,"editable":true,"tags":[]}}

{"id":"00000000-0000-0000-0000-000000000003","cell_type":"code","source":"!ls","state":"idle","outputs":[],"execution_count":null,"created_at":"2024-01-01T00:00:00Z","executed_at":null,"duration_ms":null,"metadata":{"collapsed":false,"editable":true,"tags":[]}}
"#;

        let format = RcnbFormat::new();
        let notebook = format.read_from_string(content).unwrap();

        // Should have 2 cells (empty line skipped)
        assert_eq!(notebook.cells.len(), 2);
    }

    #[test]
    fn test_git_friendly_format() {
        let format = RcnbFormat::new();
        let notebook = create_test_notebook();

        let content = format.write_to_string(&notebook).unwrap();

        // Each cell should be on its own line (git-friendly)
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 4); // 1 header + 3 cells

        // No line should be excessively long (reasonable for diff viewing)
        for line in &lines {
            assert!(line.len() < 10000, "Line too long for git diff");
        }
    }

    #[test]
    fn test_rcnb_header_from_notebook() {
        let notebook = Notebook::new("Header Test");
        let header = RcnbHeader::from_notebook(&notebook);

        assert_eq!(header.version, RCNB_VERSION);
        assert_eq!(header.name, "Header Test");
        assert_eq!(header.id, notebook.id);
    }

    #[test]
    fn test_special_characters_in_source() {
        let format = RcnbFormat::new();
        let mut notebook = Notebook::new("Special Chars");

        // Add cell with special characters (quotes, newlines, unicode)
        let cell = Cell::natural("Hello \"world\"\nLine 2\n中文测试 🎉");
        notebook.add_cell(cell);

        let content = format.write_to_string(&notebook).unwrap();
        let loaded = format.read_from_string(&content).unwrap();

        assert_eq!(loaded.cells[0].source, "Hello \"world\"\nLine 2\n中文测试 🎉");
    }

    #[test]
    fn test_all_cell_types_roundtrip() {
        let format = RcnbFormat::new();
        let mut notebook = Notebook::new("All Types");

        notebook.add_cell(Cell::natural("Natural language"));
        notebook.add_cell(Cell::code("!shell command"));
        notebook.add_cell(Cell::command("/help"));
        notebook.add_cell(Cell::markdown("# Heading"));

        let content = format.write_to_string(&notebook).unwrap();
        let loaded = format.read_from_string(&content).unwrap();

        assert_eq!(loaded.cells.len(), 4);
        assert_eq!(loaded.cells[0].cell_type, CellType::Natural);
        assert_eq!(loaded.cells[1].cell_type, CellType::Code);
        assert_eq!(loaded.cells[2].cell_type, CellType::Command);
        assert_eq!(loaded.cells[3].cell_type, CellType::Markdown);
    }
}
