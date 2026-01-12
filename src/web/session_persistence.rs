//! v1.94.0: Session Persistence System
//!
//! Provides automatic session persistence and recovery:
//! - Auto-save on disconnect
//! - Session token for reconnection
//! - Session recovery on reconnect
//! - Active session tracking

use super::session::{ConversationRound, Session};
use super::session_manager::{SerializableSession, SessionManager};
use crate::command::CommandRegistry;
use crate::config::Config;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Session Token
// ============================================================================

/// Session token for reconnection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    /// Token string (UUID-based)
    pub token: String,
    /// Session ID
    pub session_id: String,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Connection count (reconnections)
    pub connection_count: u32,
}

impl SessionToken {
    /// Create a new session token
    pub fn new(session_id: &str, ttl_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            token: format!("st_{}", Uuid::new_v4().to_string().replace('-', "")),
            session_id: session_id.to_string(),
            created_at: now,
            expires_at: now + Duration::hours(ttl_hours),
            last_activity: now,
            connection_count: 1,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Update activity and increment connection count
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
        self.connection_count += 1;
    }

    /// Extend expiration
    pub fn extend(&mut self, hours: i64) {
        self.expires_at = Utc::now() + Duration::hours(hours);
    }

    /// Time until expiration
    pub fn time_until_expiry(&self) -> std::time::Duration {
        let remaining = self.expires_at - Utc::now();
        if remaining.num_seconds() > 0 {
            std::time::Duration::from_secs(remaining.num_seconds() as u64)
        } else {
            std::time::Duration::ZERO
        }
    }
}

// ============================================================================
// Persistence Configuration
// ============================================================================

/// Session persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Enable auto-save on disconnect
    pub auto_save_on_disconnect: bool,
    /// Token TTL in hours
    pub token_ttl_hours: i64,
    /// Maximum active sessions
    pub max_active_sessions: usize,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval_secs: u64,
    /// Cleanup expired tokens interval in seconds
    pub cleanup_interval_secs: u64,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            auto_save_on_disconnect: true,
            token_ttl_hours: 24, // 24 hours
            max_active_sessions: 100,
            auto_save_interval_secs: 300, // 5 minutes
            cleanup_interval_secs: 3600,  // 1 hour
        }
    }
}

// ============================================================================
// Active Session Info
// ============================================================================

/// Active session information
#[derive(Debug, Clone)]
pub struct ActiveSessionInfo {
    /// Session token
    pub token: SessionToken,
    /// Serializable session data (for quick recovery)
    pub session_data: Option<SerializableSession>,
    /// Is currently connected
    pub is_connected: bool,
    /// Last saved time
    pub last_saved: Option<DateTime<Utc>>,
}

impl ActiveSessionInfo {
    fn new(token: SessionToken) -> Self {
        Self {
            token,
            session_data: None,
            is_connected: true,
            last_saved: None,
        }
    }
}

// ============================================================================
// Persistence Statistics
// ============================================================================

/// Persistence statistics
#[derive(Debug, Default)]
pub struct PersistenceStats {
    /// Sessions created
    sessions_created: AtomicU64,
    /// Sessions recovered
    sessions_recovered: AtomicU64,
    /// Auto-saves performed
    auto_saves: AtomicU64,
    /// Manual saves performed
    manual_saves: AtomicU64,
    /// Tokens expired
    tokens_expired: AtomicU64,
    /// Recovery failures
    recovery_failures: AtomicU64,
}

/// Statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceStatsSnapshot {
    pub sessions_created: u64,
    pub sessions_recovered: u64,
    pub auto_saves: u64,
    pub manual_saves: u64,
    pub tokens_expired: u64,
    pub recovery_failures: u64,
}

impl PersistenceStats {
    fn snapshot(&self) -> PersistenceStatsSnapshot {
        PersistenceStatsSnapshot {
            sessions_created: self.sessions_created.load(Ordering::Relaxed),
            sessions_recovered: self.sessions_recovered.load(Ordering::Relaxed),
            auto_saves: self.auto_saves.load(Ordering::Relaxed),
            manual_saves: self.manual_saves.load(Ordering::Relaxed),
            tokens_expired: self.tokens_expired.load(Ordering::Relaxed),
            recovery_failures: self.recovery_failures.load(Ordering::Relaxed),
        }
    }
}

// ============================================================================
// Session Recovery Result
// ============================================================================

/// Session recovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    /// Whether recovery was successful
    pub success: bool,
    /// Session ID
    pub session_id: Option<String>,
    /// Number of rounds recovered
    pub rounds_recovered: usize,
    /// Error message if failed
    pub error: Option<String>,
    /// Recovery source
    pub source: RecoverySource,
}

/// Where the session was recovered from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoverySource {
    /// From in-memory cache
    Memory,
    /// From disk storage
    Disk,
    /// No recovery (new session)
    None,
}

impl RecoveryResult {
    fn success_from_memory(session_id: &str, rounds: usize) -> Self {
        Self {
            success: true,
            session_id: Some(session_id.to_string()),
            rounds_recovered: rounds,
            error: None,
            source: RecoverySource::Memory,
        }
    }

    fn success_from_disk(session_id: &str, rounds: usize) -> Self {
        Self {
            success: true,
            session_id: Some(session_id.to_string()),
            rounds_recovered: rounds,
            error: None,
            source: RecoverySource::Disk,
        }
    }

    fn new_session() -> Self {
        Self {
            success: true,
            session_id: None,
            rounds_recovered: 0,
            error: None,
            source: RecoverySource::None,
        }
    }

    fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            session_id: None,
            rounds_recovered: 0,
            error: Some(error.into()),
            source: RecoverySource::None,
        }
    }
}

// ============================================================================
// Session Persistence Service
// ============================================================================

/// Session persistence service
pub struct SessionPersistence {
    /// Configuration
    config: PersistenceConfig,
    /// Session manager for disk operations
    session_manager: SessionManager,
    /// Active sessions by token
    active_sessions: RwLock<HashMap<String, ActiveSessionInfo>>,
    /// Token to session ID mapping
    token_to_session: RwLock<HashMap<String, String>>,
    /// Statistics
    stats: Arc<PersistenceStats>,
}

impl SessionPersistence {
    /// Create new session persistence service
    pub fn new(config: PersistenceConfig) -> Result<Self> {
        let session_manager = SessionManager::new()?;

        Ok(Self {
            config,
            session_manager,
            active_sessions: RwLock::new(HashMap::new()),
            token_to_session: RwLock::new(HashMap::new()),
            stats: Arc::new(PersistenceStats::default()),
        })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(PersistenceConfig::default())
    }

    /// Register a new session and get a token
    pub async fn register_session(&self, session: &Session) -> SessionToken {
        let token = SessionToken::new(&session.id, self.config.token_ttl_hours);

        let mut active = self.active_sessions.write().await;
        let mut mapping = self.token_to_session.write().await;

        active.insert(token.token.clone(), ActiveSessionInfo::new(token.clone()));
        mapping.insert(token.token.clone(), session.id.clone());

        self.stats.sessions_created.fetch_add(1, Ordering::Relaxed);

        eprintln!(
            "[SessionPersistence] Registered session {} with token {}",
            session.id, token.token
        );

        token
    }

    /// Update session data (for recovery)
    pub async fn update_session_data(&self, token: &str, data: SerializableSession) {
        let mut active = self.active_sessions.write().await;
        if let Some(info) = active.get_mut(token) {
            info.session_data = Some(data);
        }
    }

    /// Mark session as disconnected and auto-save
    pub async fn on_disconnect(&self, token: &str) -> Result<()> {
        let mut active = self.active_sessions.write().await;

        if let Some(info) = active.get_mut(token) {
            info.is_connected = false;

            if self.config.auto_save_on_disconnect {
                if let Some(ref data) = info.session_data {
                    self.session_manager.save_session(data)?;
                    info.last_saved = Some(Utc::now());
                    self.stats.auto_saves.fetch_add(1, Ordering::Relaxed);
                    eprintln!(
                        "[SessionPersistence] Auto-saved session {} on disconnect",
                        data.id
                    );
                }
            }
        }

        Ok(())
    }

    /// Try to recover session by token
    pub async fn recover_session(
        &self,
        token: &str,
        config: Config,
        registry: CommandRegistry,
    ) -> Result<(Session, RecoveryResult)> {
        // Check if token is valid
        let active = self.active_sessions.read().await;

        if let Some(info) = active.get(token) {
            if info.token.is_expired() {
                self.stats.tokens_expired.fetch_add(1, Ordering::Relaxed);
                return Err(anyhow::anyhow!("Session token expired"));
            }

            // Try to recover from memory cache first
            if let Some(ref data) = info.session_data {
                let session = Session::from_serializable(data.clone(), config, registry).await;
                self.stats.sessions_recovered.fetch_add(1, Ordering::Relaxed);

                eprintln!(
                    "[SessionPersistence] Recovered session {} from memory ({} rounds)",
                    data.id,
                    data.rounds.len()
                );

                let result = RecoveryResult::success_from_memory(&data.id, data.rounds.len());
                return Ok((session, result));
            }

            // Try to recover from disk
            let session_id = info.token.session_id.clone();
            drop(active); // Release lock before disk I/O

            match self.session_manager.load_session(&session_id) {
                Ok(data) => {
                    let rounds_count = data.rounds.len();
                    let session = Session::from_serializable(data.clone(), config, registry).await;
                    self.stats.sessions_recovered.fetch_add(1, Ordering::Relaxed);

                    // Update memory cache
                    let mut active = self.active_sessions.write().await;
                    if let Some(info) = active.get_mut(token) {
                        info.session_data = Some(data.clone());
                        info.is_connected = true;
                        info.token.touch();
                    }

                    eprintln!(
                        "[SessionPersistence] Recovered session {} from disk ({} rounds)",
                        data.id, rounds_count
                    );

                    let result = RecoveryResult::success_from_disk(&data.id, rounds_count);
                    Ok((session, result))
                }
                Err(e) => {
                    self.stats.recovery_failures.fetch_add(1, Ordering::Relaxed);
                    Err(e.context("Failed to load session from disk"))
                }
            }
        } else {
            // Token not found, try disk-based recovery using session ID
            drop(active);

            // Extract session ID from token if possible
            let mapping = self.token_to_session.read().await;
            if let Some(session_id) = mapping.get(token) {
                let session_id = session_id.clone();
                drop(mapping);

                match self.session_manager.load_session(&session_id) {
                    Ok(data) => {
                        let rounds_count = data.rounds.len();
                        let session =
                            Session::from_serializable(data.clone(), config, registry).await;
                        self.stats.sessions_recovered.fetch_add(1, Ordering::Relaxed);

                        let result = RecoveryResult::success_from_disk(&data.id, rounds_count);
                        Ok((session, result))
                    }
                    Err(_) => {
                        self.stats.recovery_failures.fetch_add(1, Ordering::Relaxed);
                        Err(anyhow::anyhow!("Token not found and session not recoverable"))
                    }
                }
            } else {
                self.stats.recovery_failures.fetch_add(1, Ordering::Relaxed);
                Err(anyhow::anyhow!("Invalid session token"))
            }
        }
    }

    /// Save session manually
    pub async fn save_session(&self, token: &str) -> Result<()> {
        let active = self.active_sessions.read().await;

        if let Some(info) = active.get(token) {
            if let Some(ref data) = info.session_data {
                self.session_manager.save_session(data)?;
                self.stats.manual_saves.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Session not found or no data to save"))
    }

    /// Cleanup expired tokens
    pub async fn cleanup_expired(&self) -> usize {
        let mut active = self.active_sessions.write().await;
        let mut mapping = self.token_to_session.write().await;

        let expired: Vec<String> = active
            .iter()
            .filter(|(_, info)| info.token.is_expired())
            .map(|(token, _)| token.clone())
            .collect();

        let count = expired.len();

        for token in expired {
            if let Some(info) = active.remove(&token) {
                mapping.remove(&token);

                // Save before removing if configured
                if self.config.auto_save_on_disconnect {
                    if let Some(ref data) = info.session_data {
                        let _ = self.session_manager.save_session(data);
                    }
                }
            }
        }

        if count > 0 {
            self.stats
                .tokens_expired
                .fetch_add(count as u64, Ordering::Relaxed);
            eprintln!("[SessionPersistence] Cleaned up {} expired tokens", count);
        }

        count
    }

    /// Get active session count
    pub async fn active_count(&self) -> usize {
        let active = self.active_sessions.read().await;
        active.len()
    }

    /// Get connected session count
    pub async fn connected_count(&self) -> usize {
        let active = self.active_sessions.read().await;
        active.values().filter(|info| info.is_connected).count()
    }

    /// Get statistics
    pub fn stats(&self) -> PersistenceStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get configuration
    pub fn config(&self) -> &PersistenceConfig {
        &self.config
    }

    /// Generate status report
    pub async fn report(&self) -> String {
        let stats = self.stats();
        let active_count = self.active_count().await;
        let connected_count = self.connected_count().await;

        let mut report = String::new();
        report.push_str("=== Session Persistence Report ===\n\n");

        report.push_str("Active Sessions:\n");
        report.push_str(&format!("  Total: {}\n", active_count));
        report.push_str(&format!("  Connected: {}\n", connected_count));
        report.push_str(&format!(
            "  Disconnected: {}\n\n",
            active_count - connected_count
        ));

        report.push_str("Statistics:\n");
        report.push_str(&format!("  Created: {}\n", stats.sessions_created));
        report.push_str(&format!("  Recovered: {}\n", stats.sessions_recovered));
        report.push_str(&format!("  Auto-saves: {}\n", stats.auto_saves));
        report.push_str(&format!("  Manual saves: {}\n", stats.manual_saves));
        report.push_str(&format!("  Tokens expired: {}\n", stats.tokens_expired));
        report.push_str(&format!("  Recovery failures: {}\n", stats.recovery_failures));

        report
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_token_new() {
        let token = SessionToken::new("session-123", 24);
        assert!(token.token.starts_with("st_"));
        assert_eq!(token.session_id, "session-123");
        assert!(!token.is_expired());
        assert_eq!(token.connection_count, 1);
    }

    #[test]
    fn test_session_token_expired() {
        let mut token = SessionToken::new("session-123", 24);
        // Manually set expiration to past
        token.expires_at = Utc::now() - Duration::hours(1);
        assert!(token.is_expired());
    }

    #[test]
    fn test_session_token_touch() {
        let mut token = SessionToken::new("session-123", 24);
        assert_eq!(token.connection_count, 1);
        token.touch();
        assert_eq!(token.connection_count, 2);
    }

    #[test]
    fn test_session_token_extend() {
        let mut token = SessionToken::new("session-123", 1);
        let original_expiry = token.expires_at;
        token.extend(24);
        assert!(token.expires_at > original_expiry);
    }

    #[test]
    fn test_persistence_config_default() {
        let config = PersistenceConfig::default();
        assert!(config.auto_save_on_disconnect);
        assert_eq!(config.token_ttl_hours, 24);
        assert_eq!(config.max_active_sessions, 100);
    }

    #[test]
    fn test_recovery_result_success() {
        let result = RecoveryResult::success_from_memory("session-123", 5);
        assert!(result.success);
        assert_eq!(result.session_id, Some("session-123".to_string()));
        assert_eq!(result.rounds_recovered, 5);
        assert!(matches!(result.source, RecoverySource::Memory));
    }

    #[test]
    fn test_recovery_result_failure() {
        let result = RecoveryResult::failure("Token expired");
        assert!(!result.success);
        assert!(result.session_id.is_none());
        assert_eq!(result.error, Some("Token expired".to_string()));
    }

    #[test]
    fn test_recovery_result_new_session() {
        let result = RecoveryResult::new_session();
        assert!(result.success);
        assert!(result.session_id.is_none());
        assert_eq!(result.rounds_recovered, 0);
        assert!(matches!(result.source, RecoverySource::None));
    }

    #[tokio::test]
    async fn test_persistence_stats() {
        let stats = PersistenceStats::default();
        stats.sessions_created.fetch_add(5, Ordering::Relaxed);
        stats.sessions_recovered.fetch_add(3, Ordering::Relaxed);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.sessions_created, 5);
        assert_eq!(snapshot.sessions_recovered, 3);
    }

    #[test]
    fn test_active_session_info() {
        let token = SessionToken::new("session-123", 24);
        let info = ActiveSessionInfo::new(token.clone());

        assert!(info.is_connected);
        assert!(info.session_data.is_none());
        assert!(info.last_saved.is_none());
    }
}
