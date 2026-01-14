//! v2.0.0-alpha.4: Notebook Collaboration System
//!
//! Provides collaborative editing support for notebooks using Operational Transformation (OT).
//!
//! # Features
//!
//! - **Operation Transform**: Concurrent edit resolution
//! - **Cell Operations**: Insert, update, delete, move cells
//! - **Cursor Sync**: Real-time cursor position sharing
//! - **Presence**: Track collaborator status
//! - **Conflict Resolution**: Automatic merge of concurrent changes
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Collaboration Session                         │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  Client A ─────┐                      ┌───── Client B            │
//! │       │        │                      │        │                 │
//! │       ▼        │                      │        ▼                 │
//! │  [Local Op] ───┼──────► Server ◄─────┼─── [Local Op]            │
//! │       │        │          │           │        │                 │
//! │       │        │     [Transform]      │        │                 │
//! │       │        │          │           │        │                 │
//! │       ▼        └───── [Broadcast] ────┘        ▼                 │
//! │  [Apply] ◄────────────────┴──────────────► [Apply]              │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use realconsole::notebook::collaboration::{
//!     CollaborationSession, CellOperation, OperationTransform,
//! };
//!
//! let mut session = CollaborationSession::new("notebook-123");
//!
//! // Client A inserts a cell
//! let op_a = CellOperation::insert(0, Cell::natural("Hello"));
//!
//! // Client B inserts at same position concurrently
//! let op_b = CellOperation::insert(0, Cell::code("!ls"));
//!
//! // Transform to resolve conflict
//! let (op_a_prime, op_b_prime) = OperationTransform::transform(&op_a, &op_b);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;
use uuid::Uuid;

use super::types::{Cell, CellType};

// ============================================================================
// Error Types
// ============================================================================

/// Collaboration error types
#[derive(Debug, Error)]
pub enum CollaborationError {
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Collaborator not found: {0}")]
    CollaboratorNotFound(Uuid),

    #[error("Cell not found: {0}")]
    CellNotFound(Uuid),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    #[error("Session full: max {max} collaborators")]
    SessionFull { max: usize },
}

// ============================================================================
// Operation Types
// ============================================================================

/// Cell operation for collaborative editing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CellOperation {
    /// Insert a new cell at index
    Insert {
        index: usize,
        cell: Cell,
    },
    /// Delete cell by ID
    Delete {
        cell_id: Uuid,
        /// Original index for transform
        index: usize,
    },
    /// Update cell source
    Update {
        cell_id: Uuid,
        old_source: String,
        new_source: String,
        /// Edit position in the source
        position: usize,
    },
    /// Move cell to new index
    Move {
        cell_id: Uuid,
        from_index: usize,
        to_index: usize,
    },
    /// Update cell metadata
    UpdateMetadata {
        cell_id: Uuid,
        key: String,
        value: serde_json::Value,
    },
    /// No operation (used for transform results)
    Noop,
}

impl CellOperation {
    /// Create insert operation
    pub fn insert(index: usize, cell: Cell) -> Self {
        Self::Insert { index, cell }
    }

    /// Create delete operation
    pub fn delete(cell_id: Uuid, index: usize) -> Self {
        Self::Delete { cell_id, index }
    }

    /// Create update operation
    pub fn update(cell_id: Uuid, old_source: String, new_source: String, position: usize) -> Self {
        Self::Update {
            cell_id,
            old_source,
            new_source,
            position,
        }
    }

    /// Create move operation
    pub fn move_cell(cell_id: Uuid, from: usize, to: usize) -> Self {
        Self::Move {
            cell_id,
            from_index: from,
            to_index: to,
        }
    }

    /// Check if this is a noop
    pub fn is_noop(&self) -> bool {
        matches!(self, CellOperation::Noop)
    }

    /// Get affected cell ID (if any)
    pub fn cell_id(&self) -> Option<Uuid> {
        match self {
            CellOperation::Insert { cell, .. } => Some(cell.id),
            CellOperation::Delete { cell_id, .. } => Some(*cell_id),
            CellOperation::Update { cell_id, .. } => Some(*cell_id),
            CellOperation::Move { cell_id, .. } => Some(*cell_id),
            CellOperation::UpdateMetadata { cell_id, .. } => Some(*cell_id),
            CellOperation::Noop => None,
        }
    }

    /// Get operation type name
    pub fn type_name(&self) -> &'static str {
        match self {
            CellOperation::Insert { .. } => "insert",
            CellOperation::Delete { .. } => "delete",
            CellOperation::Update { .. } => "update",
            CellOperation::Move { .. } => "move",
            CellOperation::UpdateMetadata { .. } => "update_metadata",
            CellOperation::Noop => "noop",
        }
    }
}

// ============================================================================
// Text Operation (for cell source editing)
// ============================================================================

/// Text operation for character-level edits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextOperation {
    /// Insert text at position
    Insert { position: usize, text: String },
    /// Delete text range
    Delete { position: usize, length: usize },
    /// Retain (skip) characters
    Retain { count: usize },
}

impl TextOperation {
    /// Apply operation to a string
    pub fn apply(&self, s: &str) -> String {
        match self {
            TextOperation::Insert { position, text } => {
                let len = s.len();
                let pos = (*position).min(len);
                format!("{}{}{}", &s[..pos], text, &s[pos..])
            }
            TextOperation::Delete { position, length } => {
                let len = s.len();
                let start = (*position).min(len);
                let end = (*position + *length).min(len);
                format!("{}{}", &s[..start], &s[end..])
            }
            TextOperation::Retain { .. } => s.to_string(),
        }
    }
}

// ============================================================================
// Operational Transformation
// ============================================================================

/// Operational Transformation engine
#[derive(Debug, Default)]
pub struct OperationTransform;

impl OperationTransform {
    /// Transform two concurrent operations
    ///
    /// Given operations A and B applied to the same state:
    /// - Returns (A', B') such that:
    /// - State.apply(A).apply(B') = State.apply(B).apply(A')
    pub fn transform(op_a: &CellOperation, op_b: &CellOperation) -> (CellOperation, CellOperation) {
        match (op_a, op_b) {
            // Insert vs Insert
            (
                CellOperation::Insert { index: idx_a, cell: cell_a },
                CellOperation::Insert { index: idx_b, cell: cell_b },
            ) => {
                if idx_a < idx_b {
                    // A comes strictly before B, B needs to shift
                    (
                        op_a.clone(),
                        CellOperation::Insert {
                            index: idx_b + 1,
                            cell: cell_b.clone(),
                        },
                    )
                } else {
                    // B comes first or at same position (B wins tie), A needs to shift
                    (
                        CellOperation::Insert {
                            index: idx_a + 1,
                            cell: cell_a.clone(),
                        },
                        op_b.clone(),
                    )
                }
            }

            // Insert vs Delete
            (
                CellOperation::Insert { index: ins_idx, cell },
                CellOperation::Delete { cell_id, index: del_idx },
            ) => {
                if ins_idx <= del_idx {
                    // Insert before delete, delete shifts
                    (
                        op_a.clone(),
                        CellOperation::Delete {
                            cell_id: *cell_id,
                            index: del_idx + 1,
                        },
                    )
                } else {
                    // Delete before insert, insert shifts back
                    (
                        CellOperation::Insert {
                            index: ins_idx.saturating_sub(1),
                            cell: cell.clone(),
                        },
                        op_b.clone(),
                    )
                }
            }

            // Delete vs Insert (symmetric)
            (
                CellOperation::Delete { cell_id, index: del_idx },
                CellOperation::Insert { index: ins_idx, cell },
            ) => {
                if del_idx < ins_idx {
                    // Delete before insert, insert shifts back
                    (
                        op_a.clone(),
                        CellOperation::Insert {
                            index: ins_idx.saturating_sub(1),
                            cell: cell.clone(),
                        },
                    )
                } else {
                    // Insert before delete, delete shifts
                    (
                        CellOperation::Delete {
                            cell_id: *cell_id,
                            index: del_idx + 1,
                        },
                        op_b.clone(),
                    )
                }
            }

            // Delete vs Delete
            (
                CellOperation::Delete { cell_id: id_a, index: idx_a },
                CellOperation::Delete { cell_id: id_b, index: idx_b },
            ) => {
                if id_a == id_b {
                    // Same cell deleted - both become noop
                    (CellOperation::Noop, CellOperation::Noop)
                } else if idx_a < idx_b {
                    // A before B, B shifts back
                    (
                        op_a.clone(),
                        CellOperation::Delete {
                            cell_id: *id_b,
                            index: idx_b.saturating_sub(1),
                        },
                    )
                } else if idx_b < idx_a {
                    // B before A, A shifts back
                    (
                        CellOperation::Delete {
                            cell_id: *id_a,
                            index: idx_a.saturating_sub(1),
                        },
                        op_b.clone(),
                    )
                } else {
                    // Same index, different cells (shouldn't happen in valid state)
                    (op_a.clone(), op_b.clone())
                }
            }

            // Update vs Update (same cell)
            (
                CellOperation::Update { cell_id: id_a, position: pos_a, .. },
                CellOperation::Update { cell_id: id_b, position: pos_b, .. },
            ) if id_a == id_b => {
                // For same cell updates, we use position to determine order
                // This is a simplified version - full text OT would be more complex
                if pos_a <= pos_b {
                    (op_a.clone(), op_b.clone())
                } else {
                    (op_b.clone(), op_a.clone())
                }
            }

            // Update vs other operations (usually independent)
            (CellOperation::Update { .. }, _) => (op_a.clone(), op_b.clone()),
            (_, CellOperation::Update { .. }) => (op_a.clone(), op_b.clone()),

            // Move operations
            (CellOperation::Move { .. }, _) | (_, CellOperation::Move { .. }) => {
                // Move operations are complex - simplified handling
                (op_a.clone(), op_b.clone())
            }

            // Metadata updates are independent
            (CellOperation::UpdateMetadata { .. }, _) | (_, CellOperation::UpdateMetadata { .. }) => {
                (op_a.clone(), op_b.clone())
            }

            // Noop handling
            (CellOperation::Noop, _) => (CellOperation::Noop, op_b.clone()),
            (_, CellOperation::Noop) => (op_a.clone(), CellOperation::Noop),
        }
    }

    /// Transform a list of operations against another operation
    pub fn transform_list(ops: &[CellOperation], against: &CellOperation) -> Vec<CellOperation> {
        let mut result = Vec::with_capacity(ops.len());
        let mut current_against = against.clone();

        for op in ops {
            let (transformed_op, new_against) = Self::transform(op, &current_against);
            result.push(transformed_op);
            current_against = new_against;
        }

        result
    }
}

// ============================================================================
// Cursor Position
// ============================================================================

/// Cursor position in the notebook
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CursorPosition {
    /// Cell ID where cursor is located
    pub cell_id: Option<Uuid>,
    /// Line number within cell (0-based)
    pub line: usize,
    /// Column number (0-based)
    pub column: usize,
    /// Selection end (if selecting)
    pub selection_end: Option<(usize, usize)>,
}

impl CursorPosition {
    /// Create cursor at cell
    pub fn at_cell(cell_id: Uuid, line: usize, column: usize) -> Self {
        Self {
            cell_id: Some(cell_id),
            line,
            column,
            selection_end: None,
        }
    }

    /// Create cursor with selection
    pub fn with_selection(mut self, end_line: usize, end_column: usize) -> Self {
        self.selection_end = Some((end_line, end_column));
        self
    }

    /// Check if cursor has selection
    pub fn has_selection(&self) -> bool {
        self.selection_end.is_some()
    }
}

// ============================================================================
// Collaborator
// ============================================================================

/// Collaborator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    /// Unique collaborator ID
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// Color for cursor/highlights (hex)
    pub color: String,
    /// Current cursor position
    pub cursor: CursorPosition,
    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
    /// Online status
    pub is_online: bool,
}

impl Collaborator {
    /// Create new collaborator
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            color: generate_color(),
            cursor: CursorPosition::default(),
            last_active: Utc::now(),
            is_online: true,
        }
    }

    /// Update cursor position
    pub fn update_cursor(&mut self, position: CursorPosition) {
        self.cursor = position;
        self.last_active = Utc::now();
    }

    /// Mark as offline
    pub fn go_offline(&mut self) {
        self.is_online = false;
    }

    /// Mark as online
    pub fn go_online(&mut self) {
        self.is_online = true;
        self.last_active = Utc::now();
    }
}

/// Generate a random color for collaborator
fn generate_color() -> String {
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4",
        "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9", "#F8B500", "#00CED1",
    ];
    let index = (Uuid::new_v4().as_u128() % colors.len() as u128) as usize;
    colors[index].to_string()
}

// ============================================================================
// Operation Entry
// ============================================================================

/// Operation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationEntry {
    /// Operation ID
    pub id: Uuid,
    /// Sequence number
    pub seq: u64,
    /// The operation
    pub operation: CellOperation,
    /// Author (collaborator ID)
    pub author: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether this operation has been acknowledged by server
    pub acknowledged: bool,
}

impl OperationEntry {
    /// Create new entry
    pub fn new(seq: u64, operation: CellOperation, author: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            seq,
            operation,
            author,
            timestamp: Utc::now(),
            acknowledged: false,
        }
    }

    /// Mark as acknowledged
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

// ============================================================================
// Collaboration Session
// ============================================================================

/// Notebook collaboration session
#[derive(Debug)]
pub struct CollaborationSession {
    /// Session ID
    id: Uuid,
    /// Notebook ID being collaborated on
    notebook_id: Uuid,
    /// Collaborators
    collaborators: HashMap<Uuid, Collaborator>,
    /// Owner ID
    owner_id: Uuid,
    /// Operation history
    history: VecDeque<OperationEntry>,
    /// Current sequence number
    sequence: u64,
    /// Maximum history size
    max_history: usize,
    /// Created timestamp
    created_at: DateTime<Utc>,
    /// Pending operations (not yet acknowledged)
    pending: Vec<OperationEntry>,
}

impl CollaborationSession {
    /// Create new collaboration session
    pub fn new(notebook_id: Uuid, owner: Collaborator) -> Self {
        let owner_id = owner.id;
        let mut collaborators = HashMap::new();
        collaborators.insert(owner.id, owner);

        Self {
            id: Uuid::new_v4(),
            notebook_id,
            collaborators,
            owner_id,
            history: VecDeque::new(),
            sequence: 0,
            max_history: 1000,
            created_at: Utc::now(),
            pending: Vec::new(),
        }
    }

    /// Get session ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get notebook ID
    pub fn notebook_id(&self) -> Uuid {
        self.notebook_id
    }

    /// Get owner ID
    pub fn owner_id(&self) -> Uuid {
        self.owner_id
    }

    /// Get current sequence number
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Add a collaborator
    pub fn add_collaborator(&mut self, collaborator: Collaborator) {
        self.collaborators.insert(collaborator.id, collaborator);
    }

    /// Remove a collaborator
    pub fn remove_collaborator(&mut self, collaborator_id: Uuid) -> Option<Collaborator> {
        if collaborator_id == self.owner_id {
            return None; // Cannot remove owner
        }
        self.collaborators.remove(&collaborator_id)
    }

    /// Get collaborator by ID
    pub fn get_collaborator(&self, id: Uuid) -> Option<&Collaborator> {
        self.collaborators.get(&id)
    }

    /// Get mutable collaborator by ID
    pub fn get_collaborator_mut(&mut self, id: Uuid) -> Option<&mut Collaborator> {
        self.collaborators.get_mut(&id)
    }

    /// Get all collaborators
    pub fn collaborators(&self) -> impl Iterator<Item = &Collaborator> {
        self.collaborators.values()
    }

    /// Get online collaborators
    pub fn online_collaborators(&self) -> impl Iterator<Item = &Collaborator> {
        self.collaborators.values().filter(|c| c.is_online)
    }

    /// Get collaborator count
    pub fn collaborator_count(&self) -> usize {
        self.collaborators.len()
    }

    /// Update collaborator cursor
    pub fn update_cursor(&mut self, collaborator_id: Uuid, position: CursorPosition) -> bool {
        if let Some(collaborator) = self.collaborators.get_mut(&collaborator_id) {
            collaborator.update_cursor(position);
            true
        } else {
            false
        }
    }

    /// Apply a local operation (from a collaborator)
    pub fn apply_local(&mut self, operation: CellOperation, author: Uuid) -> OperationEntry {
        self.sequence += 1;
        let entry = OperationEntry::new(self.sequence, operation, author);

        // Add to pending
        self.pending.push(entry.clone());

        entry
    }

    /// Apply a remote operation (from server)
    pub fn apply_remote(&mut self, mut entry: OperationEntry) -> CellOperation {
        // Transform against pending operations
        let mut transformed = entry.operation.clone();

        for pending in &self.pending {
            let (_, new_transformed) = OperationTransform::transform(&pending.operation, &transformed);
            transformed = new_transformed;
        }

        // Add to history
        entry.acknowledge();
        self.add_to_history(entry);

        transformed
    }

    /// Acknowledge local operation (server confirmed)
    pub fn acknowledge(&mut self, operation_id: Uuid) {
        if let Some(pos) = self.pending.iter().position(|e| e.id == operation_id) {
            let mut entry = self.pending.remove(pos);
            entry.acknowledge();
            self.add_to_history(entry);
        }
    }

    /// Add operation to history
    fn add_to_history(&mut self, entry: OperationEntry) {
        self.history.push_back(entry);

        // Trim history if needed
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Get operation history
    pub fn history(&self) -> impl Iterator<Item = &OperationEntry> {
        self.history.iter()
    }

    /// Get operations since sequence number
    pub fn operations_since(&self, seq: u64) -> Vec<&OperationEntry> {
        self.history.iter().filter(|e| e.seq > seq).collect()
    }

    /// Get pending operations count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Check if session is empty (only owner)
    pub fn is_empty(&self) -> bool {
        self.collaborators.len() <= 1
    }

    /// Get session age
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }
}

// ============================================================================
// Collaboration Manager
// ============================================================================

/// Manages multiple collaboration sessions
#[derive(Debug, Default)]
pub struct CollaborationManager {
    /// Active sessions (notebook_id -> session)
    sessions: HashMap<Uuid, CollaborationSession>,
    /// Collaborator to session mapping
    collaborator_sessions: HashMap<Uuid, Uuid>,
}

impl CollaborationManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create or get session for notebook
    pub fn get_or_create_session(
        &mut self,
        notebook_id: Uuid,
        owner: Collaborator,
    ) -> &mut CollaborationSession {
        self.sessions
            .entry(notebook_id)
            .or_insert_with(|| CollaborationSession::new(notebook_id, owner))
    }

    /// Get session by notebook ID
    pub fn get_session(&self, notebook_id: Uuid) -> Option<&CollaborationSession> {
        self.sessions.get(&notebook_id)
    }

    /// Get mutable session by notebook ID
    pub fn get_session_mut(&mut self, notebook_id: Uuid) -> Option<&mut CollaborationSession> {
        self.sessions.get_mut(&notebook_id)
    }

    /// Join collaborator to session
    pub fn join_session(
        &mut self,
        notebook_id: Uuid,
        collaborator: Collaborator,
    ) -> Result<(), &'static str> {
        let collaborator_id = collaborator.id;

        if let Some(session) = self.sessions.get_mut(&notebook_id) {
            session.add_collaborator(collaborator);
            self.collaborator_sessions.insert(collaborator_id, notebook_id);
            Ok(())
        } else {
            Err("Session not found")
        }
    }

    /// Leave session
    pub fn leave_session(&mut self, collaborator_id: Uuid) -> Option<Collaborator> {
        if let Some(notebook_id) = self.collaborator_sessions.remove(&collaborator_id) {
            if let Some(session) = self.sessions.get_mut(&notebook_id) {
                return session.remove_collaborator(collaborator_id);
            }
        }
        None
    }

    /// Get session for collaborator
    pub fn get_collaborator_session(&self, collaborator_id: Uuid) -> Option<&CollaborationSession> {
        self.collaborator_sessions
            .get(&collaborator_id)
            .and_then(|notebook_id| self.sessions.get(notebook_id))
    }

    /// Remove empty sessions
    pub fn cleanup_empty_sessions(&mut self) {
        let empty: Vec<Uuid> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_empty())
            .map(|(id, _)| *id)
            .collect();

        for id in empty {
            self.sessions.remove(&id);
        }
    }

    /// Get active session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get total collaborator count
    pub fn total_collaborators(&self) -> usize {
        self.sessions.values().map(|s| s.collaborator_count()).sum()
    }
}

// ============================================================================
// Sync Message Types
// ============================================================================

/// Message types for collaboration sync
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollabMessage {
    /// Join session request
    Join {
        notebook_id: String,
        collaborator_name: String,
    },
    /// Leave session
    Leave {
        notebook_id: String,
    },
    /// Operation from client
    Operation {
        notebook_id: String,
        operation: CellOperation,
        client_seq: u64,
    },
    /// Cursor update
    CursorUpdate {
        notebook_id: String,
        position: CursorPosition,
    },
    /// Acknowledge operation
    Ack {
        operation_id: String,
        server_seq: u64,
    },
    /// Broadcast operation to other clients
    Broadcast {
        notebook_id: String,
        operation: CellOperation,
        author_id: String,
        server_seq: u64,
    },
    /// Collaborator joined
    CollaboratorJoined {
        notebook_id: String,
        collaborator: Collaborator,
    },
    /// Collaborator left
    CollaboratorLeft {
        notebook_id: String,
        collaborator_id: String,
    },
    /// Cursor broadcast
    CursorBroadcast {
        notebook_id: String,
        collaborator_id: String,
        position: CursorPosition,
    },
    /// Session state (for initial sync)
    SessionState {
        notebook_id: String,
        collaborators: Vec<Collaborator>,
        sequence: u64,
    },
    /// Error
    Error {
        code: String,
        message: String,
    },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cell(source: &str) -> Cell {
        Cell::natural(source)
    }

    #[test]
    fn test_cell_operation_insert() {
        let cell = make_cell("test");
        let op = CellOperation::insert(0, cell.clone());

        assert_eq!(op.type_name(), "insert");
        assert_eq!(op.cell_id(), Some(cell.id));
    }

    #[test]
    fn test_cell_operation_delete() {
        let id = Uuid::new_v4();
        let op = CellOperation::delete(id, 5);

        assert_eq!(op.type_name(), "delete");
        assert_eq!(op.cell_id(), Some(id));
    }

    #[test]
    fn test_cell_operation_update() {
        let id = Uuid::new_v4();
        let op = CellOperation::update(id, "old".into(), "new".into(), 0);

        assert_eq!(op.type_name(), "update");
        assert_eq!(op.cell_id(), Some(id));
    }

    #[test]
    fn test_cell_operation_noop() {
        let op = CellOperation::Noop;
        assert!(op.is_noop());
        assert_eq!(op.cell_id(), None);
    }

    #[test]
    fn test_text_operation_insert() {
        let op = TextOperation::Insert {
            position: 5,
            text: "world".into(),
        };
        let result = op.apply("hello ");
        assert_eq!(result, "helloworld ");
    }

    #[test]
    fn test_text_operation_delete() {
        let op = TextOperation::Delete {
            position: 5,
            length: 6,
        };
        let result = op.apply("hello world");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_ot_insert_insert_same_index() {
        let cell_a = make_cell("A");
        let cell_b = make_cell("B");

        let op_a = CellOperation::insert(0, cell_a.clone());
        let op_b = CellOperation::insert(0, cell_b.clone());

        let (op_a_prime, op_b_prime) = OperationTransform::transform(&op_a, &op_b);

        // B wins tie (server precedence), A shifts to index 1
        match (&op_a_prime, &op_b_prime) {
            (
                CellOperation::Insert { index: idx_a, .. },
                CellOperation::Insert { index: idx_b, .. },
            ) => {
                assert_eq!(*idx_a, 1); // A shifted
                assert_eq!(*idx_b, 0); // B kept position
            }
            _ => panic!("Expected Insert operations"),
        }
    }

    #[test]
    fn test_ot_insert_delete() {
        let cell = make_cell("A");
        let del_id = Uuid::new_v4();

        let op_insert = CellOperation::insert(2, cell);
        let op_delete = CellOperation::delete(del_id, 5);

        let (ins_prime, del_prime) = OperationTransform::transform(&op_insert, &op_delete);

        // Insert at 2, delete at 5 -> delete shifts to 6
        match del_prime {
            CellOperation::Delete { index, .. } => assert_eq!(index, 6),
            _ => panic!("Expected Delete"),
        }
        match ins_prime {
            CellOperation::Insert { index, .. } => assert_eq!(index, 2),
            _ => panic!("Expected Insert"),
        }
    }

    #[test]
    fn test_ot_delete_delete_same_cell() {
        let id = Uuid::new_v4();
        let op_a = CellOperation::delete(id, 3);
        let op_b = CellOperation::delete(id, 3);

        let (op_a_prime, op_b_prime) = OperationTransform::transform(&op_a, &op_b);

        // Both become noop
        assert!(op_a_prime.is_noop());
        assert!(op_b_prime.is_noop());
    }

    #[test]
    fn test_cursor_position() {
        let id = Uuid::new_v4();
        let cursor = CursorPosition::at_cell(id, 5, 10);

        assert_eq!(cursor.cell_id, Some(id));
        assert_eq!(cursor.line, 5);
        assert_eq!(cursor.column, 10);
        assert!(!cursor.has_selection());

        let with_sel = cursor.with_selection(7, 15);
        assert!(with_sel.has_selection());
    }

    #[test]
    fn test_collaborator() {
        let mut collab = Collaborator::new("Alice");

        assert_eq!(collab.name, "Alice");
        assert!(collab.is_online);
        assert!(!collab.color.is_empty());

        collab.go_offline();
        assert!(!collab.is_online);

        collab.go_online();
        assert!(collab.is_online);
    }

    #[test]
    fn test_collaboration_session() {
        let owner = Collaborator::new("Owner");
        let notebook_id = Uuid::new_v4();
        let session = CollaborationSession::new(notebook_id, owner.clone());

        assert_eq!(session.notebook_id(), notebook_id);
        assert_eq!(session.owner_id(), owner.id);
        assert_eq!(session.collaborator_count(), 1);
    }

    #[test]
    fn test_session_add_remove_collaborator() {
        let owner = Collaborator::new("Owner");
        let notebook_id = Uuid::new_v4();
        let mut session = CollaborationSession::new(notebook_id, owner.clone());

        let guest = Collaborator::new("Guest");
        let guest_id = guest.id;

        session.add_collaborator(guest);
        assert_eq!(session.collaborator_count(), 2);

        let removed = session.remove_collaborator(guest_id);
        assert!(removed.is_some());
        assert_eq!(session.collaborator_count(), 1);

        // Cannot remove owner
        let owner_remove = session.remove_collaborator(owner.id);
        assert!(owner_remove.is_none());
    }

    #[test]
    fn test_session_apply_local() {
        let owner = Collaborator::new("Owner");
        let owner_id = owner.id;
        let notebook_id = Uuid::new_v4();
        let mut session = CollaborationSession::new(notebook_id, owner);

        let cell = make_cell("test");
        let op = CellOperation::insert(0, cell);

        let entry = session.apply_local(op, owner_id);

        assert_eq!(entry.seq, 1);
        assert_eq!(entry.author, owner_id);
        assert!(!entry.acknowledged);
        assert_eq!(session.pending_count(), 1);
    }

    #[test]
    fn test_session_acknowledge() {
        let owner = Collaborator::new("Owner");
        let owner_id = owner.id;
        let notebook_id = Uuid::new_v4();
        let mut session = CollaborationSession::new(notebook_id, owner);

        let cell = make_cell("test");
        let op = CellOperation::insert(0, cell);

        let entry = session.apply_local(op, owner_id);
        let op_id = entry.id;

        session.acknowledge(op_id);

        assert_eq!(session.pending_count(), 0);
        assert_eq!(session.history().count(), 1);
    }

    #[test]
    fn test_collaboration_manager() {
        let mut manager = CollaborationManager::new();

        let owner = Collaborator::new("Owner");
        let notebook_id = Uuid::new_v4();

        manager.get_or_create_session(notebook_id, owner);
        assert_eq!(manager.session_count(), 1);

        let guest = Collaborator::new("Guest");
        manager.join_session(notebook_id, guest).unwrap();

        let session = manager.get_session(notebook_id).unwrap();
        assert_eq!(session.collaborator_count(), 2);
    }

    #[test]
    fn test_operation_entry() {
        let op = CellOperation::Noop;
        let author = Uuid::new_v4();
        let mut entry = OperationEntry::new(1, op, author);

        assert_eq!(entry.seq, 1);
        assert!(!entry.acknowledged);

        entry.acknowledge();
        assert!(entry.acknowledged);
    }

    #[test]
    fn test_ot_transform_list() {
        let cell1 = make_cell("1");
        let cell2 = make_cell("2");

        let ops = vec![
            CellOperation::insert(0, cell1),
            CellOperation::insert(1, cell2),
        ];

        let against = CellOperation::insert(0, make_cell("X"));

        let transformed = OperationTransform::transform_list(&ops, &against);

        assert_eq!(transformed.len(), 2);
        // First op should shift to 1
        match &transformed[0] {
            CellOperation::Insert { index, .. } => assert_eq!(*index, 1),
            _ => panic!("Expected Insert"),
        }
    }
}
