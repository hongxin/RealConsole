//! WebSocket Optimization - v1.99.0
//!
//! Provides performance optimizations for WebSocket communication:
//! - Message batching for high-frequency updates
//! - Compression for large payloads
//! - Heartbeat for connection health
//! - Message prioritization

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for WebSocket optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsOptimizeConfig {
    /// Enable message batching
    pub batch_enabled: bool,
    /// Maximum messages per batch
    pub batch_max_size: usize,
    /// Maximum wait time for batching (ms)
    pub batch_max_wait_ms: u64,
    /// Enable compression for large messages
    pub compression_enabled: bool,
    /// Minimum size for compression (bytes)
    pub compression_threshold: usize,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval_secs: u64,
    /// Heartbeat timeout (seconds)
    pub heartbeat_timeout_secs: u64,
    /// Enable message prioritization
    pub priority_enabled: bool,
}

impl Default for WsOptimizeConfig {
    fn default() -> Self {
        Self {
            batch_enabled: true,
            batch_max_size: 10,
            batch_max_wait_ms: 16, // ~60fps
            compression_enabled: true,
            compression_threshold: 1024, // 1KB
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 60,
            priority_enabled: true,
        }
    }
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(Default)]
pub enum MessagePriority {
    /// Low priority (can be batched/delayed)
    Low = 0,
    /// Normal priority
    #[default]
    Normal = 1,
    /// High priority (bypass batching)
    High = 2,
    /// Critical priority (immediate send)
    Critical = 3,
}


/// Message wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedMessage {
    /// Message payload
    pub payload: String,
    /// Message priority
    pub priority: MessagePriority,
    /// Message type for categorization
    pub msg_type: String,
    /// Timestamp when queued
    #[serde(skip)]
    pub queued_at: Option<Instant>,
    /// Original size before compression
    pub original_size: usize,
    /// Whether message is compressed
    pub compressed: bool,
}

impl OptimizedMessage {
    /// Create new message with normal priority
    pub fn new(payload: String, msg_type: &str) -> Self {
        let original_size = payload.len();
        Self {
            payload,
            priority: MessagePriority::Normal,
            msg_type: msg_type.to_string(),
            queued_at: Some(Instant::now()),
            original_size,
            compressed: false,
        }
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Check if message should bypass batching
    pub fn should_bypass_batch(&self) -> bool {
        self.priority >= MessagePriority::High
    }
}

/// Message batch for grouped sending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    /// Batched messages
    pub messages: Vec<OptimizedMessage>,
    /// Batch creation time
    #[serde(skip)]
    pub created_at: Option<Instant>,
    /// Total payload size
    pub total_size: usize,
}

impl MessageBatch {
    /// Create new empty batch
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            created_at: Some(Instant::now()),
            total_size: 0,
        }
    }

    /// Add message to batch
    pub fn add(&mut self, msg: OptimizedMessage) {
        self.total_size += msg.payload.len();
        self.messages.push(msg);
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if batch should be flushed
    pub fn should_flush(&self, config: &WsOptimizeConfig) -> bool {
        if self.messages.is_empty() {
            return false;
        }

        // Size limit reached
        if self.messages.len() >= config.batch_max_size {
            return true;
        }

        // Time limit reached
        if let Some(created_at) = self.created_at {
            if created_at.elapsed().as_millis() >= config.batch_max_wait_ms as u128 {
                return true;
            }
        }

        false
    }

    /// Serialize batch for sending
    pub fn to_wire_format(&self) -> String {
        if self.messages.len() == 1 {
            // Single message, no wrapper needed
            self.messages[0].payload.clone()
        } else {
            // Multiple messages, wrap in batch
            serde_json::json!({
                "type": "batch",
                "messages": self.messages.iter().map(|m| {
                    serde_json::json!({
                        "type": m.msg_type,
                        "payload": m.payload,
                        "compressed": m.compressed
                    })
                }).collect::<Vec<_>>()
            })
            .to_string()
        }
    }
}

impl Default for MessageBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Heartbeat state for connection health
#[derive(Debug)]
pub struct HeartbeatState {
    /// Last ping sent time
    last_ping: AtomicU64,
    /// Last pong received time
    last_pong: AtomicU64,
    /// Connection is healthy
    is_healthy: AtomicBool,
    /// Missed heartbeats count
    missed_count: AtomicU64,
}

impl Default for HeartbeatState {
    fn default() -> Self {
        Self::new()
    }
}

impl HeartbeatState {
    /// Create new heartbeat state
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            last_ping: AtomicU64::new(now),
            last_pong: AtomicU64::new(now),
            is_healthy: AtomicBool::new(true),
            missed_count: AtomicU64::new(0),
        }
    }

    /// Record ping sent
    pub fn record_ping(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_ping.store(now, Ordering::SeqCst);
    }

    /// Record pong received
    pub fn record_pong(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_pong.store(now, Ordering::SeqCst);
        self.missed_count.store(0, Ordering::SeqCst);
        self.is_healthy.store(true, Ordering::SeqCst);
    }

    /// Check connection health
    pub fn check_health(&self, timeout_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let last_pong = self.last_pong.load(Ordering::SeqCst);
        let elapsed_secs = (now - last_pong) / 1000;

        if elapsed_secs > timeout_secs {
            self.is_healthy.store(false, Ordering::SeqCst);
            self.missed_count.fetch_add(1, Ordering::SeqCst);
            false
        } else {
            true
        }
    }

    /// Get if connection is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_healthy.load(Ordering::SeqCst)
    }

    /// Get missed heartbeat count
    pub fn missed_count(&self) -> u64 {
        self.missed_count.load(Ordering::SeqCst)
    }

    /// Get latency in milliseconds
    pub fn latency_ms(&self) -> u64 {
        let ping = self.last_ping.load(Ordering::SeqCst);
        let pong = self.last_pong.load(Ordering::SeqCst);
        pong.saturating_sub(ping)
    }
}

/// Message queue with priority support
#[derive(Debug)]
pub struct PriorityQueue {
    /// Critical priority messages
    critical: VecDeque<OptimizedMessage>,
    /// High priority messages
    high: VecDeque<OptimizedMessage>,
    /// Normal priority messages
    normal: VecDeque<OptimizedMessage>,
    /// Low priority messages
    low: VecDeque<OptimizedMessage>,
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityQueue {
    /// Create new priority queue
    pub fn new() -> Self {
        Self {
            critical: VecDeque::new(),
            high: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
        }
    }

    /// Add message to appropriate queue
    pub fn push(&mut self, msg: OptimizedMessage) {
        match msg.priority {
            MessagePriority::Critical => self.critical.push_back(msg),
            MessagePriority::High => self.high.push_back(msg),
            MessagePriority::Normal => self.normal.push_back(msg),
            MessagePriority::Low => self.low.push_back(msg),
        }
    }

    /// Pop highest priority message
    pub fn pop(&mut self) -> Option<OptimizedMessage> {
        self.critical
            .pop_front()
            .or_else(|| self.high.pop_front())
            .or_else(|| self.normal.pop_front())
            .or_else(|| self.low.pop_front())
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.critical.is_empty()
            && self.high.is_empty()
            && self.normal.is_empty()
            && self.low.is_empty()
    }

    /// Get total message count
    pub fn len(&self) -> usize {
        self.critical.len() + self.high.len() + self.normal.len() + self.low.len()
    }

    /// Clear all queues
    pub fn clear(&mut self) {
        self.critical.clear();
        self.high.clear();
        self.normal.clear();
        self.low.clear();
    }
}

/// Statistics for WebSocket optimization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WsOptimizeStats {
    /// Messages sent
    pub messages_sent: u64,
    /// Batches sent
    pub batches_sent: u64,
    /// Messages per batch (average)
    pub avg_batch_size: f64,
    /// Bytes saved by compression
    pub bytes_saved: u64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Heartbeats sent
    pub heartbeats_sent: u64,
    /// Heartbeats missed
    pub heartbeats_missed: u64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
}

impl WsOptimizeStats {
    /// Update batch statistics
    pub fn record_batch(&mut self, batch_size: usize) {
        self.batches_sent += 1;
        self.messages_sent += batch_size as u64;

        // Update average batch size
        let total_messages = self.messages_sent as f64;
        let total_batches = self.batches_sent as f64;
        self.avg_batch_size = total_messages / total_batches;
    }

    /// Update compression statistics
    pub fn record_compression(&mut self, original: usize, compressed: usize) {
        if original > compressed {
            self.bytes_saved += (original - compressed) as u64;
        }
        if original > 0 {
            self.compression_ratio = compressed as f64 / original as f64;
        }
    }

    /// Update heartbeat statistics
    pub fn record_heartbeat(&mut self, missed: bool) {
        self.heartbeats_sent += 1;
        if missed {
            self.heartbeats_missed += 1;
        }
    }

    /// Update latency
    pub fn update_latency(&mut self, latency_ms: u64) {
        // Simple moving average
        self.avg_latency_ms = (self.avg_latency_ms * 0.9) + (latency_ms as f64 * 0.1);
    }
}

/// WebSocket optimization CSS (for connection status UI)
pub const WS_OPTIMIZE_CSS: &str = r#"
/* ============================================
   WebSocket Status UI v1.99.0
   ============================================ */

/* Connection status indicator */
.ws-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    background: var(--bg-secondary, #0d1117);
    border: 1px solid var(--border, #30363d);
}

.ws-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    transition: background-color 0.3s ease;
}

.ws-status.connected .ws-status-dot {
    background: var(--success, #0ecb81);
    box-shadow: 0 0 6px var(--success, #0ecb81);
}

.ws-status.disconnected .ws-status-dot {
    background: var(--error, #f85149);
}

.ws-status.reconnecting .ws-status-dot {
    background: var(--warning, #f0b90b);
    animation: wsPulse 1s ease-in-out infinite;
}

@keyframes wsPulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
}

.ws-status-text {
    color: var(--text-secondary, #8b949e);
}

.ws-latency {
    color: var(--text-secondary, #8b949e);
    font-size: 11px;
}

/* Light theme */
[data-theme="light"] .ws-status {
    background: #ffffff;
    border-color: #d0d7de;
}
"#;

/// WebSocket optimization JavaScript
pub const WS_OPTIMIZE_JS: &str = r#"
// ============================================
// WebSocket Optimization v1.99.0
// ============================================

(function() {
    'use strict';

    // Default configuration
    const DEFAULT_CONFIG = {
        batchEnabled: true,
        batchMaxSize: 10,
        batchMaxWaitMs: 16,
        compressionEnabled: true,
        compressionThreshold: 1024,
        heartbeatIntervalMs: 30000,
        heartbeatTimeoutMs: 60000,
        priorityEnabled: true,
        reconnectEnabled: true,
        reconnectMaxAttempts: 5,
        reconnectBaseDelay: 1000
    };

    // Message priorities
    const PRIORITY = {
        LOW: 0,
        NORMAL: 1,
        HIGH: 2,
        CRITICAL: 3
    };

    /**
     * Optimized WebSocket wrapper
     */
    class OptimizedWebSocket {
        constructor(url, config = {}) {
            this.url = url;
            this.config = { ...DEFAULT_CONFIG, ...config };
            this.ws = null;
            this.messageQueue = [];
            this.batchQueue = [];
            this.batchTimeout = null;
            this.heartbeatInterval = null;
            this.lastPong = Date.now();
            this.reconnectAttempts = 0;
            this.stats = {
                messagesSent: 0,
                batchesSent: 0,
                bytesSaved: 0,
                heartbeatsSent: 0,
                latencyMs: 0
            };
            this.listeners = new Map();

            this.connect();
        }

        connect() {
            this.ws = new WebSocket(this.url);

            this.ws.onopen = () => {
                this.reconnectAttempts = 0;
                this.startHeartbeat();
                this.flushQueue();
                this.emit('open');
            };

            this.ws.onclose = (event) => {
                this.stopHeartbeat();
                this.emit('close', event);

                if (this.config.reconnectEnabled && this.reconnectAttempts < this.config.reconnectMaxAttempts) {
                    this.scheduleReconnect();
                }
            };

            this.ws.onerror = (error) => {
                this.emit('error', error);
            };

            this.ws.onmessage = (event) => {
                this.handleMessage(event.data);
            };
        }

        handleMessage(data) {
            // Handle pong
            if (data === 'pong' || data === '{"type":"pong"}') {
                this.lastPong = Date.now();
                this.stats.latencyMs = this.lastPong - this.lastPing;
                return;
            }

            // Parse message
            try {
                const msg = JSON.parse(data);

                // Handle batch
                if (msg.type === 'batch' && Array.isArray(msg.messages)) {
                    msg.messages.forEach(m => this.emit('message', m));
                } else {
                    this.emit('message', msg);
                }
            } catch (e) {
                this.emit('message', { type: 'raw', data });
            }
        }

        send(data, priority = PRIORITY.NORMAL) {
            const message = {
                data: typeof data === 'string' ? data : JSON.stringify(data),
                priority,
                timestamp: Date.now()
            };

            // Critical messages bypass queue
            if (priority >= PRIORITY.CRITICAL) {
                this.sendImmediate(message.data);
                return;
            }

            // High priority bypasses batching
            if (priority >= PRIORITY.HIGH || !this.config.batchEnabled) {
                this.sendImmediate(message.data);
                return;
            }

            // Add to batch queue
            this.batchQueue.push(message);
            this.scheduleBatchFlush();
        }

        sendImmediate(data) {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(data);
                this.stats.messagesSent++;
            } else {
                this.messageQueue.push(data);
            }
        }

        scheduleBatchFlush() {
            if (this.batchTimeout) return;

            this.batchTimeout = setTimeout(() => {
                this.flushBatch();
                this.batchTimeout = null;
            }, this.config.batchMaxWaitMs);

            // Also check size limit
            if (this.batchQueue.length >= this.config.batchMaxSize) {
                clearTimeout(this.batchTimeout);
                this.batchTimeout = null;
                this.flushBatch();
            }
        }

        flushBatch() {
            if (this.batchQueue.length === 0) return;

            // Sort by priority
            this.batchQueue.sort((a, b) => b.priority - a.priority);

            if (this.batchQueue.length === 1) {
                this.sendImmediate(this.batchQueue[0].data);
            } else {
                const batch = {
                    type: 'batch',
                    messages: this.batchQueue.map(m => JSON.parse(m.data))
                };
                this.sendImmediate(JSON.stringify(batch));
                this.stats.batchesSent++;
            }

            this.batchQueue = [];
        }

        flushQueue() {
            while (this.messageQueue.length > 0) {
                const msg = this.messageQueue.shift();
                this.sendImmediate(msg);
            }
        }

        startHeartbeat() {
            if (this.heartbeatInterval) return;

            this.heartbeatInterval = setInterval(() => {
                if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                    this.lastPing = Date.now();
                    this.ws.send('ping');
                    this.stats.heartbeatsSent++;

                    // Check for timeout
                    if (Date.now() - this.lastPong > this.config.heartbeatTimeoutMs) {
                        this.emit('timeout');
                        this.ws.close();
                    }
                }
            }, this.config.heartbeatIntervalMs);
        }

        stopHeartbeat() {
            if (this.heartbeatInterval) {
                clearInterval(this.heartbeatInterval);
                this.heartbeatInterval = null;
            }
        }

        scheduleReconnect() {
            this.reconnectAttempts++;
            const delay = this.config.reconnectBaseDelay * Math.pow(2, this.reconnectAttempts - 1);

            this.emit('reconnecting', { attempt: this.reconnectAttempts, delay });

            setTimeout(() => {
                this.connect();
            }, delay);
        }

        on(event, callback) {
            if (!this.listeners.has(event)) {
                this.listeners.set(event, []);
            }
            this.listeners.get(event).push(callback);
        }

        off(event, callback) {
            const listeners = this.listeners.get(event);
            if (listeners) {
                const index = listeners.indexOf(callback);
                if (index > -1) {
                    listeners.splice(index, 1);
                }
            }
        }

        emit(event, data) {
            const listeners = this.listeners.get(event);
            if (listeners) {
                listeners.forEach(cb => cb(data));
            }
        }

        getStats() {
            return { ...this.stats };
        }

        close() {
            this.config.reconnectEnabled = false;
            this.stopHeartbeat();
            if (this.ws) {
                this.ws.close();
            }
        }
    }

    // Expose globally
    window.OptimizedWebSocket = OptimizedWebSocket;
    window.WS_PRIORITY = PRIORITY;
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = WsOptimizeConfig::default();
        assert!(config.batch_enabled);
        assert_eq!(config.batch_max_size, 10);
        assert_eq!(config.batch_max_wait_ms, 16);
        assert!(config.compression_enabled);
    }

    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn test_optimized_message_new() {
        let msg = OptimizedMessage::new("test".to_string(), "output");
        assert_eq!(msg.payload, "test");
        assert_eq!(msg.msg_type, "output");
        assert_eq!(msg.priority, MessagePriority::Normal);
        assert!(!msg.compressed);
    }

    #[test]
    fn test_message_with_priority() {
        let msg = OptimizedMessage::new("test".to_string(), "output")
            .with_priority(MessagePriority::High);
        assert_eq!(msg.priority, MessagePriority::High);
    }

    #[test]
    fn test_message_should_bypass_batch() {
        let normal = OptimizedMessage::new("test".to_string(), "output");
        let high = normal.clone().with_priority(MessagePriority::High);
        let critical = normal.clone().with_priority(MessagePriority::Critical);

        assert!(!normal.should_bypass_batch());
        assert!(high.should_bypass_batch());
        assert!(critical.should_bypass_batch());
    }

    #[test]
    fn test_batch_new() {
        let batch = MessageBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.total_size, 0);
    }

    #[test]
    fn test_batch_add() {
        let mut batch = MessageBatch::new();
        let msg = OptimizedMessage::new("hello".to_string(), "output");
        batch.add(msg);

        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 1);
        assert_eq!(batch.total_size, 5);
    }

    #[test]
    fn test_batch_should_flush_by_size() {
        let config = WsOptimizeConfig {
            batch_max_size: 2,
            ..Default::default()
        };
        let mut batch = MessageBatch::new();

        assert!(!batch.should_flush(&config));

        batch.add(OptimizedMessage::new("a".to_string(), "o"));
        assert!(!batch.should_flush(&config));

        batch.add(OptimizedMessage::new("b".to_string(), "o"));
        assert!(batch.should_flush(&config));
    }

    #[test]
    fn test_batch_wire_format_single() {
        let mut batch = MessageBatch::new();
        batch.add(OptimizedMessage::new(r#"{"type":"test"}"#.to_string(), "output"));

        let wire = batch.to_wire_format();
        assert_eq!(wire, r#"{"type":"test"}"#);
    }

    #[test]
    fn test_batch_wire_format_multiple() {
        let mut batch = MessageBatch::new();
        batch.add(OptimizedMessage::new(r#"{"a":1}"#.to_string(), "output"));
        batch.add(OptimizedMessage::new(r#"{"b":2}"#.to_string(), "output"));

        let wire = batch.to_wire_format();
        assert!(wire.contains("batch"));
        assert!(wire.contains("messages"));
    }

    #[test]
    fn test_heartbeat_state_new() {
        let state = HeartbeatState::new();
        assert!(state.is_healthy());
        assert_eq!(state.missed_count(), 0);
    }

    #[test]
    fn test_heartbeat_record_pong() {
        let state = HeartbeatState::new();
        state.record_ping();
        std::thread::sleep(std::time::Duration::from_millis(10));
        state.record_pong();

        assert!(state.latency_ms() >= 10);
    }

    #[test]
    fn test_priority_queue_new() {
        let queue = PriorityQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_priority_queue_push_pop() {
        let mut queue = PriorityQueue::new();

        queue.push(OptimizedMessage::new("low".to_string(), "o").with_priority(MessagePriority::Low));
        queue.push(OptimizedMessage::new("critical".to_string(), "o").with_priority(MessagePriority::Critical));
        queue.push(OptimizedMessage::new("normal".to_string(), "o").with_priority(MessagePriority::Normal));

        // Should pop in priority order
        assert_eq!(queue.pop().unwrap().payload, "critical");
        assert_eq!(queue.pop().unwrap().payload, "normal");
        assert_eq!(queue.pop().unwrap().payload, "low");
        assert!(queue.pop().is_none());
    }

    #[test]
    fn test_priority_queue_clear() {
        let mut queue = PriorityQueue::new();
        queue.push(OptimizedMessage::new("a".to_string(), "o"));
        queue.push(OptimizedMessage::new("b".to_string(), "o"));

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_stats_default() {
        let stats = WsOptimizeStats::default();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.batches_sent, 0);
    }

    #[test]
    fn test_stats_record_batch() {
        let mut stats = WsOptimizeStats::default();
        stats.record_batch(5);
        stats.record_batch(3);

        assert_eq!(stats.messages_sent, 8);
        assert_eq!(stats.batches_sent, 2);
        assert_eq!(stats.avg_batch_size, 4.0);
    }

    #[test]
    fn test_stats_record_compression() {
        let mut stats = WsOptimizeStats::default();
        stats.record_compression(1000, 400);

        assert_eq!(stats.bytes_saved, 600);
        assert!((stats.compression_ratio - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_css_not_empty() {
        assert!(!WS_OPTIMIZE_CSS.is_empty());
        assert!(WS_OPTIMIZE_CSS.contains("ws-status"));
    }

    #[test]
    fn test_js_not_empty() {
        assert!(!WS_OPTIMIZE_JS.is_empty());
        assert!(WS_OPTIMIZE_JS.contains("OptimizedWebSocket"));
    }

    #[test]
    fn test_js_has_batching() {
        assert!(WS_OPTIMIZE_JS.contains("flushBatch"));
        assert!(WS_OPTIMIZE_JS.contains("batchQueue"));
    }

    #[test]
    fn test_js_has_heartbeat() {
        assert!(WS_OPTIMIZE_JS.contains("startHeartbeat"));
        assert!(WS_OPTIMIZE_JS.contains("stopHeartbeat"));
    }

    #[test]
    fn test_js_has_reconnect() {
        assert!(WS_OPTIMIZE_JS.contains("scheduleReconnect"));
        assert!(WS_OPTIMIZE_JS.contains("reconnectAttempts"));
    }
}
