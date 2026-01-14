//! v2.0.0-alpha.1: Notebook Storage
//!
//! Provides storage abstraction for notebooks with multiple backend support.
//!
//! # Backends
//!
//! - **FileNotebookStorage**: Persistent file-based storage
//! - **MemoryNotebookStorage**: In-memory storage (for testing)
//!
//! # Example
//!
//! ```ignore
//! use realconsole::notebook::{NotebookStorage, FileNotebookStorage, Notebook};
//!
//! let storage = FileNotebookStorage::new("~/.realconsole/notebooks");
//!
//! // Save notebook
//! let notebook = Notebook::new("My Notebook");
//! storage.save(&notebook).await?;
//!
//! // Load notebook
//! let loaded = storage.load(notebook.id).await?;
//!
//! // List all notebooks
//! let list = storage.list().await?;
//! ```

use super::types::{Notebook, NotebookMetadata};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

/// Notebook storage error
#[derive(Debug, thiserror::Error)]
pub enum NotebookStorageError {
    #[error("Notebook not found: {0}")]
    NotFound(Uuid),

    #[error("Notebook already exists: {0}")]
    AlreadyExists(Uuid),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid notebook format: {0}")]
    InvalidFormat(String),

    #[error("Storage unavailable: {0}")]
    Unavailable(String),
}

/// Storage result type
pub type StorageResult<T> = Result<T, NotebookStorageError>;

// ============================================================================
// Notebook Index
// ============================================================================

/// Index entry for a notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookIndexEntry {
    /// Notebook ID
    pub id: Uuid,
    /// Notebook name
    pub name: String,
    /// Cell count
    pub cell_count: usize,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last modified time
    pub modified_at: DateTime<Utc>,
    /// Tags
    pub tags: Vec<String>,
    /// File path (for file storage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl NotebookIndexEntry {
    /// Create from notebook
    pub fn from_notebook(notebook: &Notebook) -> Self {
        Self {
            id: notebook.id,
            name: notebook.name.clone(),
            cell_count: notebook.cell_count(),
            created_at: notebook.created_at,
            modified_at: notebook.modified_at,
            tags: notebook.metadata.tags.clone(),
            path: None,
        }
    }

    /// Set file path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
}

/// Notebook index (lightweight listing)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotebookIndex {
    /// Indexed notebooks
    pub entries: Vec<NotebookIndexEntry>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl NotebookIndex {
    /// Create new index
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: NotebookIndexEntry) {
        // Remove existing entry with same ID
        self.entries.retain(|e| e.id != entry.id);
        self.entries.push(entry);
        self.updated_at = Utc::now();
    }

    /// Remove entry by ID
    pub fn remove(&mut self, id: Uuid) -> Option<NotebookIndexEntry> {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            self.updated_at = Utc::now();
            Some(self.entries.remove(pos))
        } else {
            None
        }
    }

    /// Get entry by ID
    pub fn get(&self, id: Uuid) -> Option<&NotebookIndexEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Count entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Sort by modified time (newest first)
    pub fn sort_by_modified(&mut self) {
        self.entries.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    }

    /// Sort by name
    pub fn sort_by_name(&mut self) {
        self.entries.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Filter by tag
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&NotebookIndexEntry> {
        self.entries
            .iter()
            .filter(|e| e.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Search by name
    pub fn search(&self, query: &str) -> Vec<&NotebookIndexEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&query_lower))
            .collect()
    }
}

// ============================================================================
// Storage Trait
// ============================================================================

/// Notebook storage trait
#[async_trait]
pub trait NotebookStorage: Send + Sync {
    /// Save a notebook
    async fn save(&self, notebook: &Notebook) -> StorageResult<()>;

    /// Load a notebook by ID
    async fn load(&self, id: Uuid) -> StorageResult<Notebook>;

    /// Delete a notebook by ID
    async fn delete(&self, id: Uuid) -> StorageResult<()>;

    /// Check if notebook exists
    async fn exists(&self, id: Uuid) -> StorageResult<bool>;

    /// List all notebooks (index only)
    async fn list(&self) -> StorageResult<NotebookIndex>;

    /// Get storage statistics
    fn stats(&self) -> NotebookStorageStats;
}

/// Storage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotebookStorageStats {
    /// Number of notebooks
    pub notebook_count: usize,
    /// Total cells across all notebooks
    pub total_cells: usize,
    /// Storage size in bytes (if applicable)
    pub size_bytes: Option<u64>,
    /// Storage backend name
    pub backend: String,
}

// ============================================================================
// Memory Storage
// ============================================================================

/// In-memory notebook storage (for testing)
pub struct MemoryNotebookStorage {
    notebooks: RwLock<HashMap<Uuid, Notebook>>,
}

impl MemoryNotebookStorage {
    /// Create new memory storage
    pub fn new() -> Self {
        Self {
            notebooks: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryNotebookStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotebookStorage for MemoryNotebookStorage {
    async fn save(&self, notebook: &Notebook) -> StorageResult<()> {
        let mut notebooks = self.notebooks.write().await;
        notebooks.insert(notebook.id, notebook.clone());
        Ok(())
    }

    async fn load(&self, id: Uuid) -> StorageResult<Notebook> {
        let notebooks = self.notebooks.read().await;
        notebooks
            .get(&id)
            .cloned()
            .ok_or(NotebookStorageError::NotFound(id))
    }

    async fn delete(&self, id: Uuid) -> StorageResult<()> {
        let mut notebooks = self.notebooks.write().await;
        if notebooks.remove(&id).is_some() {
            Ok(())
        } else {
            Err(NotebookStorageError::NotFound(id))
        }
    }

    async fn exists(&self, id: Uuid) -> StorageResult<bool> {
        let notebooks = self.notebooks.read().await;
        Ok(notebooks.contains_key(&id))
    }

    async fn list(&self) -> StorageResult<NotebookIndex> {
        let notebooks = self.notebooks.read().await;
        let mut index = NotebookIndex::new();

        for notebook in notebooks.values() {
            index.add(NotebookIndexEntry::from_notebook(notebook));
        }

        index.sort_by_modified();
        Ok(index)
    }

    fn stats(&self) -> NotebookStorageStats {
        // Note: This blocks briefly to get the count
        let count = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.notebooks.read().await.len()
            })
        });

        NotebookStorageStats {
            notebook_count: count,
            total_cells: 0, // Would need to iterate
            size_bytes: None,
            backend: "memory".to_string(),
        }
    }
}

// ============================================================================
// File Storage
// ============================================================================

/// File-based notebook storage
pub struct FileNotebookStorage {
    /// Base directory for notebooks
    base_path: PathBuf,
    /// Index cache
    index: RwLock<NotebookIndex>,
    /// Index file path
    index_path: PathBuf,
}

impl FileNotebookStorage {
    /// Create new file storage
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        let base_path = base_path.as_ref().to_path_buf();
        let index_path = base_path.join("index.json");

        Self {
            base_path,
            index: RwLock::new(NotebookIndex::new()),
            index_path,
        }
    }

    /// Initialize storage (create directories, load index)
    pub async fn init(&self) -> StorageResult<()> {
        // Create base directory
        tokio::fs::create_dir_all(&self.base_path).await?;

        // Load or create index
        if self.index_path.exists() {
            let content = tokio::fs::read_to_string(&self.index_path).await?;
            let index: NotebookIndex = serde_json::from_str(&content)
                .map_err(|e| NotebookStorageError::SerializationError(e.to_string()))?;
            *self.index.write().await = index;
        }

        Ok(())
    }

    /// Get notebook file path
    fn notebook_path(&self, id: Uuid) -> PathBuf {
        self.base_path.join(format!("{}.rcnb", id))
    }

    /// Save index to disk
    async fn save_index(&self) -> StorageResult<()> {
        let index = self.index.read().await;
        let json = serde_json::to_string_pretty(&*index)
            .map_err(|e| NotebookStorageError::SerializationError(e.to_string()))?;
        tokio::fs::write(&self.index_path, json).await?;
        Ok(())
    }

    /// Get base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[async_trait]
impl NotebookStorage for FileNotebookStorage {
    async fn save(&self, notebook: &Notebook) -> StorageResult<()> {
        let path = self.notebook_path(notebook.id);

        // Serialize notebook
        let json = serde_json::to_string_pretty(notebook)
            .map_err(|e| NotebookStorageError::SerializationError(e.to_string()))?;

        // Write to file
        tokio::fs::write(&path, json).await?;

        // Update index
        let mut index = self.index.write().await;
        let entry = NotebookIndexEntry::from_notebook(notebook).with_path(path);
        index.add(entry);
        drop(index);

        // Persist index
        self.save_index().await?;

        Ok(())
    }

    async fn load(&self, id: Uuid) -> StorageResult<Notebook> {
        let path = self.notebook_path(id);

        if !path.exists() {
            return Err(NotebookStorageError::NotFound(id));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let notebook: Notebook = serde_json::from_str(&content)
            .map_err(|e| NotebookStorageError::SerializationError(e.to_string()))?;

        Ok(notebook)
    }

    async fn delete(&self, id: Uuid) -> StorageResult<()> {
        let path = self.notebook_path(id);

        if !path.exists() {
            return Err(NotebookStorageError::NotFound(id));
        }

        // Remove file
        tokio::fs::remove_file(&path).await?;

        // Update index
        let mut index = self.index.write().await;
        index.remove(id);
        drop(index);

        // Persist index
        self.save_index().await?;

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> StorageResult<bool> {
        let path = self.notebook_path(id);
        Ok(path.exists())
    }

    async fn list(&self) -> StorageResult<NotebookIndex> {
        let index = self.index.read().await;
        Ok(index.clone())
    }

    fn stats(&self) -> NotebookStorageStats {
        NotebookStorageStats {
            notebook_count: 0, // Would need async access
            total_cells: 0,
            size_bytes: None,
            backend: "file".to_string(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notebook::Cell;

    #[test]
    fn test_notebook_index_entry() {
        let notebook = Notebook::new("Test");
        let entry = NotebookIndexEntry::from_notebook(&notebook);

        assert_eq!(entry.id, notebook.id);
        assert_eq!(entry.name, "Test");
        assert_eq!(entry.cell_count, 0);
    }

    #[test]
    fn test_notebook_index() {
        let mut index = NotebookIndex::new();
        assert!(index.is_empty());

        let notebook = Notebook::new("Test");
        let entry = NotebookIndexEntry::from_notebook(&notebook);
        index.add(entry);

        assert_eq!(index.len(), 1);
        assert!(index.get(notebook.id).is_some());
    }

    #[test]
    fn test_notebook_index_remove() {
        let mut index = NotebookIndex::new();
        let notebook = Notebook::new("Test");
        let id = notebook.id;

        index.add(NotebookIndexEntry::from_notebook(&notebook));
        assert_eq!(index.len(), 1);

        let removed = index.remove(id);
        assert!(removed.is_some());
        assert!(index.is_empty());
    }

    #[test]
    fn test_notebook_index_search() {
        let mut index = NotebookIndex::new();

        let nb1 = Notebook::new("Analysis Report");
        let nb2 = Notebook::new("Data Processing");
        let nb3 = Notebook::new("Analysis Summary");

        index.add(NotebookIndexEntry::from_notebook(&nb1));
        index.add(NotebookIndexEntry::from_notebook(&nb2));
        index.add(NotebookIndexEntry::from_notebook(&nb3));

        let results = index.search("analysis");
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_storage_save_load() {
        let storage = MemoryNotebookStorage::new();
        let mut notebook = Notebook::new("Test");
        notebook.add_cell(Cell::natural("Hello"));

        // Save
        storage.save(&notebook).await.unwrap();

        // Load
        let loaded = storage.load(notebook.id).await.unwrap();
        assert_eq!(loaded.name, "Test");
        assert_eq!(loaded.cell_count(), 1);
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = MemoryNotebookStorage::new();
        let notebook = Notebook::new("Test");
        let id = notebook.id;

        storage.save(&notebook).await.unwrap();
        assert!(storage.exists(id).await.unwrap());

        storage.delete(id).await.unwrap();
        assert!(!storage.exists(id).await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_list() {
        let storage = MemoryNotebookStorage::new();

        storage.save(&Notebook::new("Notebook 1")).await.unwrap();
        storage.save(&Notebook::new("Notebook 2")).await.unwrap();

        let index = storage.list().await.unwrap();
        assert_eq!(index.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_storage_not_found() {
        let storage = MemoryNotebookStorage::new();
        let result = storage.load(Uuid::new_v4()).await;
        assert!(matches!(result, Err(NotebookStorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_file_storage_new() {
        let temp_dir = std::env::temp_dir().join("realconsole_test_notebooks");
        let storage = FileNotebookStorage::new(&temp_dir);

        assert_eq!(storage.base_path(), temp_dir);
    }

    #[test]
    fn test_storage_stats_default() {
        let stats = NotebookStorageStats::default();
        assert_eq!(stats.notebook_count, 0);
        assert!(stats.backend.is_empty());
    }

    #[test]
    fn test_notebook_storage_error_display() {
        let err = NotebookStorageError::NotFound(Uuid::nil());
        assert!(err.to_string().contains("not found"));

        let err = NotebookStorageError::AlreadyExists(Uuid::nil());
        assert!(err.to_string().contains("exists"));
    }

    #[test]
    fn test_index_sort_by_name() {
        let mut index = NotebookIndex::new();

        index.add(NotebookIndexEntry::from_notebook(&Notebook::new("Zebra")));
        index.add(NotebookIndexEntry::from_notebook(&Notebook::new("Apple")));
        index.add(NotebookIndexEntry::from_notebook(&Notebook::new("Mango")));

        index.sort_by_name();

        assert_eq!(index.entries[0].name, "Apple");
        assert_eq!(index.entries[1].name, "Mango");
        assert_eq!(index.entries[2].name, "Zebra");
    }

    #[test]
    fn test_index_filter_by_tag() {
        let mut index = NotebookIndex::new();

        let mut nb1 = Notebook::new("NB1");
        nb1.metadata.tags.push("rust".to_string());
        let mut nb2 = Notebook::new("NB2");
        nb2.metadata.tags.push("python".to_string());
        let mut nb3 = Notebook::new("NB3");
        nb3.metadata.tags.push("rust".to_string());

        index.add(NotebookIndexEntry::from_notebook(&nb1));
        index.add(NotebookIndexEntry::from_notebook(&nb2));
        index.add(NotebookIndexEntry::from_notebook(&nb3));

        let rust_nbs = index.filter_by_tag("rust");
        assert_eq!(rust_nbs.len(), 2);
    }
}
