//! v2.0.0-alpha.2: Notebook WebSocket Integration
//!
//! Provides WebSocket protocol for real-time Notebook operations:
//! - Create/Open/Save/Delete notebooks
//! - Add/Edit/Delete/Execute cells
//! - Real-time execution streaming
//! - Notebook listing and search

use crate::notebook::{
    Cell, CellExecutor, CellOutput, CellState, CellType, ExecutionConfig,
    ExecutionResult, FileNotebookStorage, IpynbConverter, MemoryNotebookStorage, Notebook,
    NotebookStorage, NotebookStorageError, RcnbFormat,
};
use axum::extract::ws::{Message, WebSocket};
use chrono::{DateTime, Utc};
use futures::stream::SplitSink;
use futures::SinkExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Client → Server Messages
// ============================================================================

/// Client messages for Notebook operations
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotebookClientMessage {
    // Notebook CRUD
    /// Create a new notebook
    CreateNotebook { name: String },
    /// Open an existing notebook
    OpenNotebook { notebook_id: String },
    /// Save current notebook
    SaveNotebook { notebook_id: String },
    /// Close notebook (cleanup)
    CloseNotebook { notebook_id: String },
    /// Delete notebook
    DeleteNotebook { notebook_id: String },
    /// List all notebooks
    ListNotebooks,
    /// Rename notebook
    RenameNotebook { notebook_id: String, new_name: String },

    // Cell Operations
    /// Add a new cell
    AddCell {
        notebook_id: String,
        cell_type: String, // "natural", "command", "code", "markdown"
        source: String,
        #[serde(default)]
        index: Option<usize>, // Insert position, None = append
    },
    /// Update cell source
    UpdateCell {
        notebook_id: String,
        cell_id: String,
        source: String,
    },
    /// Delete cell
    DeleteCell {
        notebook_id: String,
        cell_id: String,
    },
    /// Move cell to new position
    MoveCell {
        notebook_id: String,
        cell_id: String,
        new_index: usize,
    },
    /// Execute a cell
    ExecuteCell {
        notebook_id: String,
        cell_id: String,
    },
    /// Execute all cells in order
    ExecuteAll { notebook_id: String },
    /// Cancel cell execution
    CancelExecution {
        notebook_id: String,
        cell_id: String,
    },
    /// Clear cell outputs
    ClearOutputs {
        notebook_id: String,
        cell_id: String,
    },

    // Import/Export
    /// Export notebook to .rcnb format
    ExportNotebook {
        notebook_id: String,
        format: String, // "rcnb", "json", "markdown"
    },
    /// Import notebook from content
    ImportNotebook {
        name: String,
        content: String,
        format: String,
    },
}

// ============================================================================
// Server → Client Messages
// ============================================================================

/// Server messages for Notebook responses
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotebookServerMessage {
    // Notebook responses
    /// Notebook created
    NotebookCreated { notebook: NotebookSummary },
    /// Notebook opened (full data)
    NotebookOpened { notebook: NotebookData },
    /// Notebook saved
    NotebookSaved { notebook_id: String, modified_at: DateTime<Utc> },
    /// Notebook closed
    NotebookClosed { notebook_id: String },
    /// Notebook deleted
    NotebookDeleted { notebook_id: String },
    /// Notebook list
    NotebookList { notebooks: Vec<NotebookSummary> },
    /// Notebook renamed
    NotebookRenamed { notebook_id: String, new_name: String },

    // Cell responses
    /// Cell added
    CellAdded {
        notebook_id: String,
        cell: CellData,
        index: usize,
    },
    /// Cell updated
    CellUpdated {
        notebook_id: String,
        cell_id: String,
        source: String,
    },
    /// Cell deleted
    CellDeleted {
        notebook_id: String,
        cell_id: String,
    },
    /// Cell moved
    CellMoved {
        notebook_id: String,
        cell_id: String,
        new_index: usize,
    },
    /// Cell execution started
    CellExecutionStarted {
        notebook_id: String,
        cell_id: String,
    },
    /// Cell execution output (streaming)
    CellOutput {
        notebook_id: String,
        cell_id: String,
        output: CellOutputData,
    },
    /// Cell execution completed
    CellExecutionCompleted {
        notebook_id: String,
        cell_id: String,
        state: String, // "success", "failed", "cancelled"
        duration_ms: u64,
        execution_count: u32,
    },
    /// Cell outputs cleared
    CellOutputsCleared {
        notebook_id: String,
        cell_id: String,
    },

    // Import/Export responses
    /// Notebook exported
    NotebookExported {
        notebook_id: String,
        format: String,
        content: String,
        filename: String,
    },
    /// Notebook imported
    NotebookImported { notebook: NotebookSummary },

    // Error
    /// Error message
    Error { code: String, message: String },
}

// ============================================================================
// Data Transfer Objects
// ============================================================================

/// Notebook summary for listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookSummary {
    pub id: String,
    pub name: String,
    pub cell_count: usize,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl From<&Notebook> for NotebookSummary {
    fn from(nb: &Notebook) -> Self {
        Self {
            id: nb.id.to_string(),
            name: nb.name.clone(),
            cell_count: nb.cells.len(),
            created_at: nb.created_at,
            modified_at: nb.modified_at,
            tags: nb.metadata.tags.clone(),
        }
    }
}

/// Full notebook data for opening
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookData {
    pub id: String,
    pub name: String,
    pub cells: Vec<CellData>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub metadata: NotebookMetadataData,
}

impl From<&Notebook> for NotebookData {
    fn from(nb: &Notebook) -> Self {
        Self {
            id: nb.id.to_string(),
            name: nb.name.clone(),
            cells: nb.cells.iter().map(CellData::from).collect(),
            created_at: nb.created_at,
            modified_at: nb.modified_at,
            metadata: NotebookMetadataData {
                description: nb.metadata.description.clone(),
                author: nb.metadata.author.clone(),
                tags: nb.metadata.tags.clone(),
                kernel: nb.metadata.kernel.clone(),
            },
        }
    }
}

/// Notebook metadata for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadataData {
    pub description: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub kernel: String,
}

/// Cell data for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    pub id: String,
    pub cell_type: String,
    pub source: String,
    pub state: String,
    pub outputs: Vec<CellOutputData>,
    pub execution_count: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

impl From<&Cell> for CellData {
    fn from(cell: &Cell) -> Self {
        Self {
            id: cell.id.to_string(),
            cell_type: match cell.cell_type {
                CellType::Natural => "natural",
                CellType::Command => "command",
                CellType::Code => "code",
                CellType::Markdown => "markdown",
            }.to_string(),
            source: cell.source.clone(),
            state: match cell.state {
                CellState::Idle => "idle",
                CellState::Pending => "pending",
                CellState::Running => "running",
                CellState::Success => "success",
                CellState::Failed => "failed",
                CellState::Cancelled => "cancelled",
            }.to_string(),
            outputs: cell.outputs.iter().map(CellOutputData::from).collect(),
            execution_count: cell.execution_count,
            created_at: cell.created_at,
            executed_at: cell.executed_at,
            duration_ms: cell.duration_ms,
        }
    }
}

/// Cell output data for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CellOutputData {
    Text { content: String },
    Code { language: String, content: String },
    Chart { chart_type: String, title: Option<String>, data: serde_json::Value },
    Image { mime_type: String, data: String, alt: Option<String> },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Error { message: String, traceback: Option<String> },
    Stream { name: String, content: String },
}

impl From<&CellOutput> for CellOutputData {
    fn from(output: &CellOutput) -> Self {
        match output {
            CellOutput::Text { content } => CellOutputData::Text { content: content.clone() },
            CellOutput::Code { language, content } => CellOutputData::Code {
                language: language.clone(),
                content: content.clone(),
            },
            CellOutput::Chart { chart_type, title, data } => CellOutputData::Chart {
                chart_type: chart_type.clone(),
                title: title.clone(),
                data: data.clone(),
            },
            CellOutput::Image { mime_type, data, alt } => CellOutputData::Image {
                mime_type: mime_type.clone(),
                data: data.clone(),
                alt: alt.clone(),
            },
            CellOutput::Table { headers, rows } => CellOutputData::Table {
                headers: headers.clone(),
                rows: rows.clone(),
            },
            CellOutput::Error { message, traceback } => CellOutputData::Error {
                message: message.clone(),
                traceback: traceback.clone(),
            },
            CellOutput::Stream { name, content } => CellOutputData::Stream {
                name: name.clone(),
                content: content.clone(),
            },
        }
    }
}

// ============================================================================
// Notebook Session State
// ============================================================================

/// Manages notebook state for a WebSocket session
pub struct NotebookSession {
    /// Currently open notebooks (id -> notebook)
    notebooks: RwLock<HashMap<Uuid, Notebook>>,
    /// Cell executor
    executor: CellExecutor,
    /// Storage backend
    storage: Arc<dyn NotebookStorage>,
    /// Base directory for file storage
    base_dir: PathBuf,
}

impl NotebookSession {
    /// Create new notebook session
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            notebooks: RwLock::new(HashMap::new()),
            executor: CellExecutor::new(ExecutionConfig::default()),
            storage: Arc::new(MemoryNotebookStorage::new()),
            base_dir,
        }
    }

    /// Create with file storage
    pub fn with_file_storage(base_dir: PathBuf) -> Self {
        let storage_dir = base_dir.join("notebooks");
        Self {
            notebooks: RwLock::new(HashMap::new()),
            executor: CellExecutor::new(ExecutionConfig::default()),
            storage: Arc::new(FileNotebookStorage::new(storage_dir)),
            base_dir,
        }
    }

    /// Handle notebook message
    pub async fn handle_message(
        &self,
        msg: NotebookClientMessage,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> anyhow::Result<()> {
        let response = match msg {
            NotebookClientMessage::CreateNotebook { name } => {
                self.create_notebook(name).await
            }
            NotebookClientMessage::OpenNotebook { notebook_id } => {
                self.open_notebook(&notebook_id).await
            }
            NotebookClientMessage::SaveNotebook { notebook_id } => {
                self.save_notebook(&notebook_id).await
            }
            NotebookClientMessage::CloseNotebook { notebook_id } => {
                self.close_notebook(&notebook_id).await
            }
            NotebookClientMessage::DeleteNotebook { notebook_id } => {
                self.delete_notebook(&notebook_id).await
            }
            NotebookClientMessage::ListNotebooks => {
                self.list_notebooks().await
            }
            NotebookClientMessage::RenameNotebook { notebook_id, new_name } => {
                self.rename_notebook(&notebook_id, new_name).await
            }
            NotebookClientMessage::AddCell { notebook_id, cell_type, source, index } => {
                self.add_cell(&notebook_id, &cell_type, source, index).await
            }
            NotebookClientMessage::UpdateCell { notebook_id, cell_id, source } => {
                self.update_cell(&notebook_id, &cell_id, source).await
            }
            NotebookClientMessage::DeleteCell { notebook_id, cell_id } => {
                self.delete_cell(&notebook_id, &cell_id).await
            }
            NotebookClientMessage::MoveCell { notebook_id, cell_id, new_index } => {
                self.move_cell(&notebook_id, &cell_id, new_index).await
            }
            NotebookClientMessage::ExecuteCell { notebook_id, cell_id } => {
                return self.execute_cell(&notebook_id, &cell_id, sender).await;
            }
            NotebookClientMessage::ExecuteAll { notebook_id } => {
                return self.execute_all(&notebook_id, sender).await;
            }
            NotebookClientMessage::CancelExecution { notebook_id, cell_id } => {
                self.cancel_execution(&notebook_id, &cell_id).await
            }
            NotebookClientMessage::ClearOutputs { notebook_id, cell_id } => {
                self.clear_outputs(&notebook_id, &cell_id).await
            }
            NotebookClientMessage::ExportNotebook { notebook_id, format } => {
                self.export_notebook(&notebook_id, &format).await
            }
            NotebookClientMessage::ImportNotebook { name, content, format } => {
                self.import_notebook(name, content, &format).await
            }
        };

        self.send_message(sender, response).await
    }

    // Notebook operations

    async fn create_notebook(&self, name: String) -> NotebookServerMessage {
        let notebook = Notebook::new(&name);
        let summary = NotebookSummary::from(&notebook);

        // Save to storage
        if let Err(e) = self.storage.save(&notebook).await {
            return NotebookServerMessage::Error {
                code: "STORAGE_ERROR".to_string(),
                message: format!("Failed to save notebook: {}", e),
            };
        }

        // Add to open notebooks
        self.notebooks.write().await.insert(notebook.id, notebook);

        NotebookServerMessage::NotebookCreated { notebook: summary }
    }

    async fn open_notebook(&self, notebook_id: &str) -> NotebookServerMessage {
        let id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        // Check if already open
        if let Some(nb) = self.notebooks.read().await.get(&id) {
            return NotebookServerMessage::NotebookOpened {
                notebook: NotebookData::from(nb),
            };
        }

        // Load from storage
        match self.storage.load(id).await {
            Ok(notebook) => {
                let data = NotebookData::from(&notebook);
                self.notebooks.write().await.insert(id, notebook);
                NotebookServerMessage::NotebookOpened { notebook: data }
            }
            Err(NotebookStorageError::NotFound(_)) => {
                NotebookServerMessage::Error {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Notebook not found: {}", notebook_id),
                }
            }
            Err(e) => NotebookServerMessage::Error {
                code: "STORAGE_ERROR".to_string(),
                message: format!("Failed to load notebook: {}", e),
            },
        }
    }

    async fn save_notebook(&self, notebook_id: &str) -> NotebookServerMessage {
        let id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let notebooks = self.notebooks.read().await;
        let notebook = match notebooks.get(&id) {
            Some(nb) => nb,
            None => return NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            },
        };

        if let Err(e) = self.storage.save(notebook).await {
            return NotebookServerMessage::Error {
                code: "STORAGE_ERROR".to_string(),
                message: format!("Failed to save notebook: {}", e),
            };
        }

        NotebookServerMessage::NotebookSaved {
            notebook_id: notebook_id.to_string(),
            modified_at: notebook.modified_at,
        }
    }

    async fn close_notebook(&self, notebook_id: &str) -> NotebookServerMessage {
        let id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        self.notebooks.write().await.remove(&id);

        NotebookServerMessage::NotebookClosed {
            notebook_id: notebook_id.to_string(),
        }
    }

    async fn delete_notebook(&self, notebook_id: &str) -> NotebookServerMessage {
        let id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        // Remove from open notebooks
        self.notebooks.write().await.remove(&id);

        // Delete from storage
        if let Err(e) = self.storage.delete(id).await {
            if !matches!(e, NotebookStorageError::NotFound(_)) {
                return NotebookServerMessage::Error {
                    code: "STORAGE_ERROR".to_string(),
                    message: format!("Failed to delete notebook: {}", e),
                };
            }
        }

        NotebookServerMessage::NotebookDeleted {
            notebook_id: notebook_id.to_string(),
        }
    }

    async fn list_notebooks(&self) -> NotebookServerMessage {
        match self.storage.list().await {
            Ok(index) => {
                let notebooks: Vec<NotebookSummary> = index
                    .entries
                    .iter()
                    .map(|e| NotebookSummary {
                        id: e.id.to_string(),
                        name: e.name.clone(),
                        cell_count: e.cell_count,
                        created_at: e.created_at,
                        modified_at: e.modified_at,
                        tags: e.tags.clone(),
                    })
                    .collect();
                NotebookServerMessage::NotebookList { notebooks }
            }
            Err(e) => NotebookServerMessage::Error {
                code: "STORAGE_ERROR".to_string(),
                message: format!("Failed to list notebooks: {}", e),
            },
        }
    }

    async fn rename_notebook(&self, notebook_id: &str, new_name: String) -> NotebookServerMessage {
        let id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&id) {
            notebook.name = new_name.clone();

            if let Err(e) = self.storage.save(notebook).await {
                return NotebookServerMessage::Error {
                    code: "STORAGE_ERROR".to_string(),
                    message: format!("Failed to save renamed notebook: {}", e),
                };
            }

            NotebookServerMessage::NotebookRenamed {
                notebook_id: notebook_id.to_string(),
                new_name,
            }
        } else {
            NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }
        }
    }

    // Cell operations

    async fn add_cell(
        &self,
        notebook_id: &str,
        cell_type: &str,
        source: String,
        index: Option<usize>,
    ) -> NotebookServerMessage {
        let id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let cell_type = match cell_type {
            "natural" => CellType::Natural,
            "command" => CellType::Command,
            "code" => CellType::Code,
            "markdown" => CellType::Markdown,
            _ => return NotebookServerMessage::Error {
                code: "INVALID_CELL_TYPE".to_string(),
                message: format!("Unknown cell type: {}", cell_type),
            },
        };

        let cell = Cell::new(cell_type, source);
        let cell_data = CellData::from(&cell);

        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&id) {
            let actual_index = match index {
                Some(idx) if idx <= notebook.cells.len() => {
                    notebook.insert_cell(idx, cell);
                    idx
                }
                _ => {
                    let idx = notebook.cells.len();
                    notebook.add_cell(cell);
                    idx
                }
            };

            NotebookServerMessage::CellAdded {
                notebook_id: notebook_id.to_string(),
                cell: cell_data,
                index: actual_index,
            }
        } else {
            NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }
        }
    }

    async fn update_cell(
        &self,
        notebook_id: &str,
        cell_id: &str,
        source: String,
    ) -> NotebookServerMessage {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let cell_uuid = match Uuid::parse_str(cell_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_CELL_ID".to_string(),
                message: "Invalid cell ID format".to_string(),
            },
        };

        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&nb_id) {
            if let Some(cell) = notebook.get_cell_mut(cell_uuid) {
                cell.source = source.clone();
                cell.state = CellState::Idle; // Reset state when edited

                NotebookServerMessage::CellUpdated {
                    notebook_id: notebook_id.to_string(),
                    cell_id: cell_id.to_string(),
                    source,
                }
            } else {
                NotebookServerMessage::Error {
                    code: "CELL_NOT_FOUND".to_string(),
                    message: "Cell not found".to_string(),
                }
            }
        } else {
            NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }
        }
    }

    async fn delete_cell(&self, notebook_id: &str, cell_id: &str) -> NotebookServerMessage {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let cell_uuid = match Uuid::parse_str(cell_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_CELL_ID".to_string(),
                message: "Invalid cell ID format".to_string(),
            },
        };

        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&nb_id) {
            if notebook.remove_cell(cell_uuid).is_some() {
                NotebookServerMessage::CellDeleted {
                    notebook_id: notebook_id.to_string(),
                    cell_id: cell_id.to_string(),
                }
            } else {
                NotebookServerMessage::Error {
                    code: "CELL_NOT_FOUND".to_string(),
                    message: "Cell not found".to_string(),
                }
            }
        } else {
            NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }
        }
    }

    async fn move_cell(
        &self,
        notebook_id: &str,
        cell_id: &str,
        new_index: usize,
    ) -> NotebookServerMessage {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let cell_uuid = match Uuid::parse_str(cell_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_CELL_ID".to_string(),
                message: "Invalid cell ID format".to_string(),
            },
        };

        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&nb_id) {
            // Find current position
            let current_pos = notebook.cells.iter().position(|c| c.id == cell_uuid);

            if let Some(pos) = current_pos {
                if new_index < notebook.cells.len() {
                    let cell = notebook.cells.remove(pos);
                    let actual_index = if new_index > pos { new_index - 1 } else { new_index };
                    notebook.cells.insert(actual_index.min(notebook.cells.len()), cell);

                    NotebookServerMessage::CellMoved {
                        notebook_id: notebook_id.to_string(),
                        cell_id: cell_id.to_string(),
                        new_index: actual_index,
                    }
                } else {
                    NotebookServerMessage::Error {
                        code: "INVALID_INDEX".to_string(),
                        message: "Invalid target index".to_string(),
                    }
                }
            } else {
                NotebookServerMessage::Error {
                    code: "CELL_NOT_FOUND".to_string(),
                    message: "Cell not found".to_string(),
                }
            }
        } else {
            NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }
        }
    }

    async fn execute_cell(
        &self,
        notebook_id: &str,
        cell_id: &str,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> anyhow::Result<()> {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => {
                self.send_message(sender, NotebookServerMessage::Error {
                    code: "INVALID_ID".to_string(),
                    message: "Invalid notebook ID format".to_string(),
                }).await?;
                return Ok(());
            }
        };

        let cell_uuid = match Uuid::parse_str(cell_id) {
            Ok(id) => id,
            Err(_) => {
                self.send_message(sender, NotebookServerMessage::Error {
                    code: "INVALID_CELL_ID".to_string(),
                    message: "Invalid cell ID format".to_string(),
                }).await?;
                return Ok(());
            }
        };

        // Send execution started
        self.send_message(sender, NotebookServerMessage::CellExecutionStarted {
            notebook_id: notebook_id.to_string(),
            cell_id: cell_id.to_string(),
        }).await?;

        // Get cell and execute
        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&nb_id) {
            if let Some(cell) = notebook.get_cell_mut(cell_uuid) {
                let result = self.executor.execute(cell).await;

                match result {
                    Ok(exec_result) => {
                        // Send outputs
                        for output in &cell.outputs {
                            self.send_message(sender, NotebookServerMessage::CellOutput {
                                notebook_id: notebook_id.to_string(),
                                cell_id: cell_id.to_string(),
                                output: CellOutputData::from(output),
                            }).await?;
                        }

                        // Send completion
                        self.send_message(sender, NotebookServerMessage::CellExecutionCompleted {
                            notebook_id: notebook_id.to_string(),
                            cell_id: cell_id.to_string(),
                            state: if exec_result.success { "success" } else { "failed" }.to_string(),
                            duration_ms: exec_result.duration_ms,
                            execution_count: cell.execution_count.unwrap_or(0),
                        }).await?;
                    }
                    Err(e) => {
                        self.send_message(sender, NotebookServerMessage::CellExecutionCompleted {
                            notebook_id: notebook_id.to_string(),
                            cell_id: cell_id.to_string(),
                            state: "failed".to_string(),
                            duration_ms: 0,
                            execution_count: 0,
                        }).await?;

                        self.send_message(sender, NotebookServerMessage::Error {
                            code: "EXECUTION_ERROR".to_string(),
                            message: format!("Execution failed: {}", e),
                        }).await?;
                    }
                }
            } else {
                self.send_message(sender, NotebookServerMessage::Error {
                    code: "CELL_NOT_FOUND".to_string(),
                    message: "Cell not found".to_string(),
                }).await?;
            }
        } else {
            self.send_message(sender, NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }).await?;
        }

        Ok(())
    }

    async fn execute_all(
        &self,
        notebook_id: &str,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> anyhow::Result<()> {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => {
                self.send_message(sender, NotebookServerMessage::Error {
                    code: "INVALID_ID".to_string(),
                    message: "Invalid notebook ID format".to_string(),
                }).await?;
                return Ok(());
            }
        };

        let cell_ids: Vec<String> = {
            let notebooks = self.notebooks.read().await;
            if let Some(notebook) = notebooks.get(&nb_id) {
                notebook.cells.iter().map(|c| c.id.to_string()).collect()
            } else {
                self.send_message(sender, NotebookServerMessage::Error {
                    code: "NOT_OPEN".to_string(),
                    message: "Notebook not open".to_string(),
                }).await?;
                return Ok(());
            }
        };

        for cell_id in cell_ids {
            self.execute_cell(notebook_id, &cell_id, sender).await?;
        }

        Ok(())
    }

    async fn cancel_execution(
        &self,
        notebook_id: &str,
        cell_id: &str,
    ) -> NotebookServerMessage {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let cell_uuid = match Uuid::parse_str(cell_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_CELL_ID".to_string(),
                message: "Invalid cell ID format".to_string(),
            },
        };

        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&nb_id) {
            if let Some(cell) = notebook.get_cell_mut(cell_uuid) {
                cell.mark_cancelled();

                NotebookServerMessage::CellExecutionCompleted {
                    notebook_id: notebook_id.to_string(),
                    cell_id: cell_id.to_string(),
                    state: "cancelled".to_string(),
                    duration_ms: 0,
                    execution_count: cell.execution_count.unwrap_or(0),
                }
            } else {
                NotebookServerMessage::Error {
                    code: "CELL_NOT_FOUND".to_string(),
                    message: "Cell not found".to_string(),
                }
            }
        } else {
            NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }
        }
    }

    async fn clear_outputs(&self, notebook_id: &str, cell_id: &str) -> NotebookServerMessage {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let cell_uuid = match Uuid::parse_str(cell_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_CELL_ID".to_string(),
                message: "Invalid cell ID format".to_string(),
            },
        };

        let mut notebooks = self.notebooks.write().await;
        if let Some(notebook) = notebooks.get_mut(&nb_id) {
            if let Some(cell) = notebook.get_cell_mut(cell_uuid) {
                cell.clear_outputs();

                NotebookServerMessage::CellOutputsCleared {
                    notebook_id: notebook_id.to_string(),
                    cell_id: cell_id.to_string(),
                }
            } else {
                NotebookServerMessage::Error {
                    code: "CELL_NOT_FOUND".to_string(),
                    message: "Cell not found".to_string(),
                }
            }
        } else {
            NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            }
        }
    }

    // Import/Export

    async fn export_notebook(&self, notebook_id: &str, format: &str) -> NotebookServerMessage {
        let nb_id = match Uuid::parse_str(notebook_id) {
            Ok(id) => id,
            Err(_) => return NotebookServerMessage::Error {
                code: "INVALID_ID".to_string(),
                message: "Invalid notebook ID format".to_string(),
            },
        };

        let notebooks = self.notebooks.read().await;
        let notebook = match notebooks.get(&nb_id) {
            Some(nb) => nb,
            None => return NotebookServerMessage::Error {
                code: "NOT_OPEN".to_string(),
                message: "Notebook not open".to_string(),
            },
        };

        let (content, extension) = match format {
            "rcnb" => {
                let rcnb = RcnbFormat::new();
                match rcnb.write_to_string(notebook) {
                    Ok(content) => (content, "rcnb"),
                    Err(e) => return NotebookServerMessage::Error {
                        code: "EXPORT_ERROR".to_string(),
                        message: format!("Failed to export: {}", e),
                    },
                }
            }
            "json" => {
                match serde_json::to_string_pretty(notebook) {
                    Ok(content) => (content, "json"),
                    Err(e) => return NotebookServerMessage::Error {
                        code: "EXPORT_ERROR".to_string(),
                        message: format!("Failed to export: {}", e),
                    },
                }
            }
            "markdown" => {
                let content = export_to_markdown(notebook);
                (content, "md")
            }
            // v2.2.0: Jupyter Notebook export
            "ipynb" => {
                match IpynbConverter::to_ipynb_str(notebook) {
                    Ok(content) => (content, "ipynb"),
                    Err(e) => return NotebookServerMessage::Error {
                        code: "EXPORT_ERROR".to_string(),
                        message: format!("Failed to export .ipynb: {}", e),
                    },
                }
            }
            _ => return NotebookServerMessage::Error {
                code: "INVALID_FORMAT".to_string(),
                message: format!("Unknown format: {}", format),
            },
        };

        let filename = format!("{}.{}", sanitize_filename(&notebook.name), extension);

        NotebookServerMessage::NotebookExported {
            notebook_id: notebook_id.to_string(),
            format: format.to_string(),
            content,
            filename,
        }
    }

    async fn import_notebook(
        &self,
        name: String,
        content: String,
        format: &str,
    ) -> NotebookServerMessage {
        let notebook = match format {
            "rcnb" => {
                let rcnb = RcnbFormat::new();
                match rcnb.read_from_string(&content) {
                    Ok(nb) => nb,
                    Err(e) => return NotebookServerMessage::Error {
                        code: "IMPORT_ERROR".to_string(),
                        message: format!("Failed to parse .rcnb: {}", e),
                    },
                }
            }
            "json" => {
                match serde_json::from_str::<Notebook>(&content) {
                    Ok(nb) => nb,
                    Err(e) => return NotebookServerMessage::Error {
                        code: "IMPORT_ERROR".to_string(),
                        message: format!("Failed to parse JSON: {}", e),
                    },
                }
            }
            // v2.2.0-beta.1: Jupyter Notebook import
            "ipynb" => {
                match IpynbConverter::from_ipynb_str(&content, &name) {
                    Ok(nb) => nb,
                    Err(e) => return NotebookServerMessage::Error {
                        code: "IMPORT_ERROR".to_string(),
                        message: format!("Failed to parse .ipynb: {}", e),
                    },
                }
            }
            _ => return NotebookServerMessage::Error {
                code: "INVALID_FORMAT".to_string(),
                message: format!("Unknown import format: {}", format),
            },
        };

        // Create new notebook with imported content but new ID
        let mut new_notebook = Notebook::new(name);
        for cell in notebook.cells {
            new_notebook.add_cell(cell);
        }
        new_notebook.metadata = notebook.metadata;

        let summary = NotebookSummary::from(&new_notebook);

        // Save to storage
        if let Err(e) = self.storage.save(&new_notebook).await {
            return NotebookServerMessage::Error {
                code: "STORAGE_ERROR".to_string(),
                message: format!("Failed to save imported notebook: {}", e),
            };
        }

        // Add to open notebooks
        self.notebooks.write().await.insert(new_notebook.id, new_notebook);

        NotebookServerMessage::NotebookImported { notebook: summary }
    }

    // Helper methods

    async fn send_message(
        &self,
        sender: &mut SplitSink<WebSocket, Message>,
        msg: NotebookServerMessage,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string(&msg)?;
        sender.send(Message::Text(json)).await?;
        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Export notebook to Markdown format
fn export_to_markdown(notebook: &Notebook) -> String {
    let mut md = String::new();

    // Title
    md.push_str(&format!("# {}\n\n", notebook.name));

    // Metadata
    if let Some(desc) = &notebook.metadata.description {
        md.push_str(&format!("> {}\n\n", desc));
    }
    if let Some(author) = &notebook.metadata.author {
        md.push_str(&format!("*Author: {}*\n\n", author));
    }

    md.push_str("---\n\n");

    // Cells
    for (i, cell) in notebook.cells.iter().enumerate() {
        md.push_str(&format!("## Cell {} ({})\n\n", i + 1, format!("{:?}", cell.cell_type).to_lowercase()));

        // Source
        match cell.cell_type {
            CellType::Code | CellType::Command => {
                md.push_str(&format!("```\n{}\n```\n\n", cell.source));
            }
            CellType::Markdown => {
                md.push_str(&format!("{}\n\n", cell.source));
            }
            CellType::Natural => {
                md.push_str(&format!("> {}\n\n", cell.source));
            }
        }

        // Outputs
        if !cell.outputs.is_empty() {
            md.push_str("**Output:**\n\n");
            for output in &cell.outputs {
                match output {
                    CellOutput::Text { content } => {
                        md.push_str(&format!("{}\n\n", content));
                    }
                    CellOutput::Code { language, content } => {
                        md.push_str(&format!("```{}\n{}\n```\n\n", language, content));
                    }
                    CellOutput::Error { message, .. } => {
                        md.push_str(&format!("❌ Error: {}\n\n", message));
                    }
                    CellOutput::Table { headers, rows } => {
                        md.push_str(&format!("| {} |\n", headers.join(" | ")));
                        md.push_str(&format!("| {} |\n", headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")));
                        for row in rows {
                            md.push_str(&format!("| {} |\n", row.join(" | ")));
                        }
                        md.push('\n');
                    }
                    _ => {}
                }
            }
        }

        md.push_str("---\n\n");
    }

    md
}

/// Sanitize filename for safe file system use
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notebook_summary_from_notebook() {
        let notebook = Notebook::new("Test Notebook");
        let summary = NotebookSummary::from(&notebook);

        assert_eq!(summary.name, "Test Notebook");
        assert_eq!(summary.cell_count, 0);
    }

    #[test]
    fn test_cell_data_from_cell() {
        let cell = Cell::natural("Hello, world!");
        let data = CellData::from(&cell);

        assert_eq!(data.cell_type, "natural");
        assert_eq!(data.source, "Hello, world!");
        assert_eq!(data.state, "idle");
    }

    #[test]
    fn test_cell_output_data_from_output() {
        let output = CellOutput::text("Test output");
        let data = CellOutputData::from(&output);

        match data {
            CellOutputData::Text { content } => assert_eq!(content, "Test output"),
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file:name*"), "test_file_name_");
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
    }

    #[test]
    fn test_export_to_markdown() {
        let mut notebook = Notebook::new("Test Export");
        notebook.add_cell(Cell::natural("What is 2+2?"));
        notebook.add_cell(Cell::code("!echo 4"));

        let md = export_to_markdown(&notebook);

        assert!(md.contains("# Test Export"));
        assert!(md.contains("Cell 1 (natural)"));
        assert!(md.contains("What is 2+2?"));
        assert!(md.contains("Cell 2 (code)"));
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"create_notebook","name":"My Notebook"}"#;
        let msg: NotebookClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            NotebookClientMessage::CreateNotebook { name } => {
                assert_eq!(name, "My Notebook");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = NotebookServerMessage::NotebookCreated {
            notebook: NotebookSummary {
                id: "test-id".to_string(),
                name: "Test".to_string(),
                cell_count: 0,
                created_at: Utc::now(),
                modified_at: Utc::now(),
                tags: vec![],
            },
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("notebook_created"));
        assert!(json.contains("Test"));
    }
}
