//! v2.2.0-beta.1: Jupyter Notebook (.ipynb) Import/Export
//!
//! Provides bidirectional conversion between Jupyter Notebook format (.ipynb)
//! and RealConsole Notebook format (.rcnb).
//!
//! # Format Mapping
//!
//! ## Import: .ipynb → RealConsole
//! | .ipynb cell_type | RealConsole CellType |
//! |------------------|----------------------|
//! | "code"           | Code (Shell)         |
//! | "markdown"       | Markdown             |
//! | "raw"            | Natural              |
//!
//! ## Export: RealConsole → .ipynb
//! | RealConsole CellType | .ipynb cell_type |
//! |----------------------|------------------|
//! | Natural              | "raw"            |
//! | Command              | "raw"            |
//! | Code                 | "code"           |
//! | Markdown             | "markdown"       |

use crate::notebook::{Cell, CellMetadata, CellOutput, CellState, CellType, Notebook, NotebookMetadata};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Jupyter Notebook format version
pub const IPYNB_FORMAT_VERSION: u32 = 4;
pub const IPYNB_FORMAT_MINOR: u32 = 5;

// ============================================================================
// Jupyter Notebook JSON Structures
// ============================================================================

/// Jupyter Notebook root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterNotebook {
    pub metadata: JupyterMetadata,
    pub nbformat: u32,
    pub nbformat_minor: u32,
    pub cells: Vec<JupyterCell>,
}

/// Jupyter Notebook metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JupyterMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernelspec: Option<KernelSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_info: Option<LanguageInfo>,
    /// RealConsole extension: original notebook name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realconsole_name: Option<String>,
    /// RealConsole extension: original notebook ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realconsole_id: Option<String>,
    /// Additional metadata fields
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Kernel specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelSpec {
    pub display_name: String,
    pub language: String,
    pub name: String,
}

/// Language information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,
}

/// Jupyter Cell structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterCell {
    pub cell_type: String,
    pub source: JupyterSource,
    #[serde(default)]
    pub metadata: JupyterCellMetadata,
    /// Only for code cells
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<JupyterOutput>>,
    /// Only for code cells
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_count: Option<u32>,
}

/// Jupyter source can be string or array of strings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JupyterSource {
    String(String),
    Lines(Vec<String>),
}

impl JupyterSource {
    pub fn as_string(&self) -> String {
        match self {
            JupyterSource::String(s) => s.clone(),
            JupyterSource::Lines(lines) => lines.join(""),
        }
    }

    pub fn from_string(s: String) -> Self {
        // Split into lines preserving newlines for compatibility
        let lines: Vec<String> = s.lines().map(|l| format!("{}\n", l)).collect();
        if lines.is_empty() {
            JupyterSource::String(String::new())
        } else {
            // Remove trailing newline from last line if original didn't have it
            let mut lines = lines;
            if !s.ends_with('\n') && !lines.is_empty() {
                if let Some(last) = lines.last_mut() {
                    *last = last.trim_end_matches('\n').to_string();
                }
            }
            JupyterSource::Lines(lines)
        }
    }
}

/// Jupyter cell metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JupyterCellMetadata {
    /// RealConsole extension: original cell type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realconsole_type: Option<String>,
    /// RealConsole extension: original cell ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realconsole_id: Option<String>,
    /// Tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Jupyter output types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "output_type")]
pub enum JupyterOutput {
    #[serde(rename = "stream")]
    Stream {
        name: String,
        text: JupyterSource,
    },
    #[serde(rename = "execute_result")]
    ExecuteResult {
        data: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
        execution_count: Option<u32>,
    },
    #[serde(rename = "display_data")]
    DisplayData {
        data: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
    },
    #[serde(rename = "error")]
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
}

// ============================================================================
// Converter
// ============================================================================

/// Jupyter Notebook converter
pub struct IpynbConverter;

impl IpynbConverter {
    /// Import a .ipynb file into RealConsole Notebook
    pub fn import(path: &Path) -> Result<Notebook> {
        let content = std::fs::read_to_string(path)?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Notebook")
            .to_string();
        Self::from_ipynb_str(&content, &name)
    }

    /// Convert .ipynb JSON string to RealConsole Notebook
    pub fn from_ipynb_str(content: &str, name: &str) -> Result<Notebook> {
        let jupyter: JupyterNotebook = serde_json::from_str(content)
            .map_err(|e| anyhow!("Invalid .ipynb format: {}", e))?;

        // Create notebook with metadata
        let mut notebook = Notebook::new(name);

        // Preserve original ID if present
        if let Some(ref id) = jupyter.metadata.realconsole_id {
            if let Ok(uuid) = Uuid::parse_str(id) {
                notebook.id = uuid;
            }
        }

        // Convert metadata
        notebook.metadata = Self::convert_jupyter_metadata(&jupyter.metadata);

        // Convert cells
        for jcell in jupyter.cells {
            let cell = Self::convert_jupyter_cell(jcell)?;
            notebook.add_cell(cell);
        }

        Ok(notebook)
    }

    /// Export RealConsole Notebook to .ipynb file
    pub fn export(notebook: &Notebook, path: &Path) -> Result<()> {
        let content = Self::to_ipynb_str(notebook)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Convert RealConsole Notebook to .ipynb JSON string
    pub fn to_ipynb_str(notebook: &Notebook) -> Result<String> {
        let jupyter = Self::to_jupyter_notebook(notebook);
        serde_json::to_string_pretty(&jupyter)
            .map_err(|e| anyhow!("Failed to serialize .ipynb: {}", e))
    }

    /// Convert RealConsole Notebook to Jupyter Notebook structure
    fn to_jupyter_notebook(notebook: &Notebook) -> JupyterNotebook {
        JupyterNotebook {
            metadata: Self::convert_to_jupyter_metadata(notebook),
            nbformat: IPYNB_FORMAT_VERSION,
            nbformat_minor: IPYNB_FORMAT_MINOR,
            cells: notebook.cells.iter().map(Self::convert_to_jupyter_cell).collect(),
        }
    }

    /// Convert Jupyter metadata to RealConsole metadata
    fn convert_jupyter_metadata(meta: &JupyterMetadata) -> NotebookMetadata {
        let kernel = meta
            .kernelspec
            .as_ref()
            .map(|k| k.language.clone())
            .or_else(|| meta.language_info.as_ref().map(|l| l.name.clone()))
            .unwrap_or_else(|| "shell".to_string());

        NotebookMetadata {
            description: None,
            author: None,
            tags: Vec::new(),
            kernel,
            format_version: "2.2.0".to_string(),
            custom: HashMap::new(),
        }
    }

    /// Convert RealConsole metadata to Jupyter metadata
    fn convert_to_jupyter_metadata(notebook: &Notebook) -> JupyterMetadata {
        let language = &notebook.metadata.kernel;

        JupyterMetadata {
            kernelspec: Some(KernelSpec {
                display_name: format!("RealConsole ({})", language),
                language: language.clone(),
                name: "realconsole".to_string(),
            }),
            language_info: Some(LanguageInfo {
                name: language.clone(),
                version: None,
                mimetype: None,
                file_extension: Some(".rcnb".to_string()),
            }),
            realconsole_name: Some(notebook.name.clone()),
            realconsole_id: Some(notebook.id.to_string()),
            extra: HashMap::new(),
        }
    }

    /// Convert Jupyter cell to RealConsole cell
    fn convert_jupyter_cell(jcell: JupyterCell) -> Result<Cell> {
        // Determine cell type
        let cell_type = match jcell.cell_type.as_str() {
            "code" => CellType::Code,
            "markdown" => CellType::Markdown,
            "raw" => {
                // Check if it was originally a RealConsole cell
                if let Some(ref rc_type) = jcell.metadata.realconsole_type {
                    match rc_type.as_str() {
                        "natural" => CellType::Natural,
                        "command" => CellType::Command,
                        _ => CellType::Natural,
                    }
                } else {
                    CellType::Natural
                }
            }
            _ => CellType::Natural,
        };

        let source = jcell.source.as_string();
        let mut cell = Cell::new(cell_type, source);

        // Restore original ID if present
        if let Some(ref id) = jcell.metadata.realconsole_id {
            if let Ok(uuid) = Uuid::parse_str(id) {
                cell.id = uuid;
            }
        }

        // Convert execution count
        cell.execution_count = jcell.execution_count;

        // Convert metadata
        cell.metadata = CellMetadata {
            language: None,
            tags: jcell.metadata.tags,
            collapsed: false,
            editable: true,
            custom: HashMap::new(),
        };

        // Convert outputs
        if let Some(outputs) = jcell.outputs {
            for output in outputs {
                if let Some(cell_output) = Self::convert_jupyter_output(output) {
                    cell.outputs.push(cell_output);
                }
            }
            // Mark as executed if has outputs
            if !cell.outputs.is_empty() {
                cell.state = CellState::Success;
                cell.executed_at = Some(Utc::now());
            }
        }

        Ok(cell)
    }

    /// Convert RealConsole cell to Jupyter cell
    fn convert_to_jupyter_cell(cell: &Cell) -> JupyterCell {
        let (cell_type, realconsole_type) = match cell.cell_type {
            CellType::Natural => ("raw".to_string(), Some("natural".to_string())),
            CellType::Command => ("raw".to_string(), Some("command".to_string())),
            CellType::Code => ("code".to_string(), None),
            CellType::Markdown => ("markdown".to_string(), None),
        };

        let outputs = if cell.cell_type == CellType::Code {
            Some(cell.outputs.iter().filter_map(Self::convert_to_jupyter_output).collect())
        } else {
            None
        };

        let execution_count = if cell.cell_type == CellType::Code {
            cell.execution_count
        } else {
            None
        };

        JupyterCell {
            cell_type,
            source: JupyterSource::from_string(cell.source.clone()),
            metadata: JupyterCellMetadata {
                realconsole_type,
                realconsole_id: Some(cell.id.to_string()),
                tags: cell.metadata.tags.clone(),
                extra: HashMap::new(),
            },
            outputs,
            execution_count,
        }
    }

    /// Convert Jupyter output to RealConsole output
    fn convert_jupyter_output(output: JupyterOutput) -> Option<CellOutput> {
        match output {
            JupyterOutput::Stream { name, text } => {
                Some(CellOutput::Stream {
                    name,
                    content: text.as_string(),
                })
            }
            JupyterOutput::ExecuteResult { data, .. } | JupyterOutput::DisplayData { data, .. } => {
                // Try to extract text/plain first
                if let Some(Value::String(text)) = data.get("text/plain") {
                    return Some(CellOutput::text(text.clone()));
                }
                if let Some(Value::Array(lines)) = data.get("text/plain") {
                    let text: String = lines
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("");
                    return Some(CellOutput::text(text));
                }
                // Try image/png
                if let Some(Value::String(base64)) = data.get("image/png") {
                    return Some(CellOutput::image_base64("image/png", base64.clone()));
                }
                // Try text/html (convert to text for now)
                if let Some(Value::String(html)) = data.get("text/html") {
                    return Some(CellOutput::text(html.clone()));
                }
                None
            }
            JupyterOutput::Error { ename, evalue, traceback } => {
                Some(CellOutput::error_with_traceback(
                    format!("{}: {}", ename, evalue),
                    traceback.join("\n"),
                ))
            }
        }
    }

    /// Convert RealConsole output to Jupyter output
    fn convert_to_jupyter_output(output: &CellOutput) -> Option<JupyterOutput> {
        match output {
            CellOutput::Stream { name, content } => {
                Some(JupyterOutput::Stream {
                    name: name.clone(),
                    text: JupyterSource::from_string(content.clone()),
                })
            }
            CellOutput::Text { content } => {
                let mut data = HashMap::new();
                data.insert("text/plain".to_string(), Value::String(content.clone()));
                Some(JupyterOutput::ExecuteResult {
                    data,
                    metadata: HashMap::new(),
                    execution_count: None,
                })
            }
            CellOutput::Code { content, language } => {
                let mut data = HashMap::new();
                data.insert("text/plain".to_string(), Value::String(format!("```{}\n{}\n```", language, content)));
                Some(JupyterOutput::ExecuteResult {
                    data,
                    metadata: HashMap::new(),
                    execution_count: None,
                })
            }
            CellOutput::Image { mime_type, data: image_data, .. } => {
                let mut data = HashMap::new();
                data.insert(mime_type.clone(), Value::String(image_data.clone()));
                Some(JupyterOutput::DisplayData {
                    data,
                    metadata: HashMap::new(),
                })
            }
            CellOutput::Error { message, traceback } => {
                Some(JupyterOutput::Error {
                    ename: "Error".to_string(),
                    evalue: message.clone(),
                    traceback: traceback
                        .as_ref()
                        .map(|t| t.lines().map(String::from).collect())
                        .unwrap_or_default(),
                })
            }
            CellOutput::Table { headers, rows } => {
                // Convert table to text representation
                let mut lines = vec![headers.join("\t")];
                for row in rows {
                    lines.push(row.join("\t"));
                }
                let mut data = HashMap::new();
                data.insert("text/plain".to_string(), Value::String(lines.join("\n")));
                Some(JupyterOutput::ExecuteResult {
                    data,
                    metadata: HashMap::new(),
                    execution_count: None,
                })
            }
            CellOutput::Chart { .. } => {
                // Charts need special handling - skip for now
                None
            }
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
    fn test_jupyter_source_string() {
        let source = JupyterSource::String("hello".to_string());
        assert_eq!(source.as_string(), "hello");
    }

    #[test]
    fn test_jupyter_source_lines() {
        let source = JupyterSource::Lines(vec!["line1\n".to_string(), "line2".to_string()]);
        assert_eq!(source.as_string(), "line1\nline2");
    }

    #[test]
    fn test_source_from_string() {
        let source = JupyterSource::from_string("line1\nline2".to_string());
        match source {
            JupyterSource::Lines(lines) => {
                assert_eq!(lines.len(), 2);
            }
            _ => panic!("Expected Lines"),
        }
    }

    #[test]
    fn test_convert_code_cell() {
        let jcell = JupyterCell {
            cell_type: "code".to_string(),
            source: JupyterSource::String("print('hello')".to_string()),
            metadata: JupyterCellMetadata::default(),
            outputs: Some(vec![]),
            execution_count: Some(1),
        };

        let cell = IpynbConverter::convert_jupyter_cell(jcell).unwrap();
        assert_eq!(cell.cell_type, CellType::Code);
        assert_eq!(cell.source, "print('hello')");
        assert_eq!(cell.execution_count, Some(1));
    }

    #[test]
    fn test_convert_markdown_cell() {
        let jcell = JupyterCell {
            cell_type: "markdown".to_string(),
            source: JupyterSource::String("# Title".to_string()),
            metadata: JupyterCellMetadata::default(),
            outputs: None,
            execution_count: None,
        };

        let cell = IpynbConverter::convert_jupyter_cell(jcell).unwrap();
        assert_eq!(cell.cell_type, CellType::Markdown);
    }

    #[test]
    fn test_convert_raw_cell() {
        let jcell = JupyterCell {
            cell_type: "raw".to_string(),
            source: JupyterSource::String("Some text".to_string()),
            metadata: JupyterCellMetadata::default(),
            outputs: None,
            execution_count: None,
        };

        let cell = IpynbConverter::convert_jupyter_cell(jcell).unwrap();
        assert_eq!(cell.cell_type, CellType::Natural);
    }

    #[test]
    fn test_convert_raw_cell_with_realconsole_type() {
        let mut metadata = JupyterCellMetadata::default();
        metadata.realconsole_type = Some("command".to_string());

        let jcell = JupyterCell {
            cell_type: "raw".to_string(),
            source: JupyterSource::String("/help".to_string()),
            metadata,
            outputs: None,
            execution_count: None,
        };

        let cell = IpynbConverter::convert_jupyter_cell(jcell).unwrap();
        assert_eq!(cell.cell_type, CellType::Command);
    }

    #[test]
    fn test_roundtrip_cell() {
        let original = Cell::natural("Hello world");
        let jupyter = IpynbConverter::convert_to_jupyter_cell(&original);
        let converted = IpynbConverter::convert_jupyter_cell(jupyter).unwrap();

        assert_eq!(converted.cell_type, original.cell_type);
        assert_eq!(converted.source, original.source);
        assert_eq!(converted.id, original.id);
    }

    #[test]
    fn test_roundtrip_notebook() {
        let mut notebook = Notebook::new("Test Notebook");
        notebook.add_cell(Cell::natural("Natural cell"));
        notebook.add_cell(Cell::command("/help"));
        notebook.add_cell(Cell::code("!ls"));
        notebook.add_cell(Cell::markdown("# Title"));

        let json = IpynbConverter::to_ipynb_str(&notebook).unwrap();
        let imported = IpynbConverter::from_ipynb_str(&json, "Test Notebook").unwrap();

        assert_eq!(imported.name, notebook.name);
        assert_eq!(imported.cell_count(), notebook.cell_count());

        for (orig, imp) in notebook.cells.iter().zip(imported.cells.iter()) {
            assert_eq!(orig.cell_type, imp.cell_type);
            assert_eq!(orig.source, imp.source);
        }
    }

    #[test]
    fn test_parse_simple_ipynb() {
        let ipynb = r#"{
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5,
            "cells": [
                {
                    "cell_type": "code",
                    "source": "print('hello')",
                    "metadata": {},
                    "outputs": [],
                    "execution_count": 1
                }
            ]
        }"#;

        let notebook = IpynbConverter::from_ipynb_str(ipynb, "Test").unwrap();
        assert_eq!(notebook.cell_count(), 1);
        assert_eq!(notebook.cells[0].cell_type, CellType::Code);
    }

    #[test]
    fn test_convert_stream_output() {
        let output = JupyterOutput::Stream {
            name: "stdout".to_string(),
            text: JupyterSource::String("Hello\n".to_string()),
        };

        let cell_output = IpynbConverter::convert_jupyter_output(output).unwrap();
        match cell_output {
            CellOutput::Stream { name, content } => {
                assert_eq!(name, "stdout");
                assert_eq!(content, "Hello\n");
            }
            _ => panic!("Expected Stream output"),
        }
    }

    #[test]
    fn test_convert_error_output() {
        let output = JupyterOutput::Error {
            ename: "ValueError".to_string(),
            evalue: "invalid value".to_string(),
            traceback: vec!["line 1".to_string(), "line 2".to_string()],
        };

        let cell_output = IpynbConverter::convert_jupyter_output(output).unwrap();
        match cell_output {
            CellOutput::Error { message, traceback } => {
                assert!(message.contains("ValueError"));
                assert!(traceback.is_some());
            }
            _ => panic!("Expected Error output"),
        }
    }
}
