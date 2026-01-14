//! Distributed Tracing Support (v1.111.0)
//!
//! Provides W3C Trace Context compatible distributed tracing capabilities
//! for cross-service trace correlation and propagation.
//!
//! # Features
//!
//! - **W3C Trace Context**: Standard traceparent/tracestate headers
//! - **Trace Propagation**: Extract/inject context from/to carriers
//! - **Sampling**: Head-based and tail-based sampling strategies
//! - **Export Formats**: Zipkin, OTLP, and custom formats
//!
//! # W3C Trace Context Format
//!
//! `traceparent`: version-trace_id-parent_id-flags
//! Example: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`
//!
//! # Example
//!
//! ```rust
//! use realconsole::tracer::distributed::{
//!     DistributedContext, TraceParent, Propagator, HeaderCarrier,
//!     Sampler, SamplingDecision, AlwaysOnSampler,
//! };
//!
//! // Parse incoming trace context
//! let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
//! let parent = TraceParent::parse(traceparent).unwrap();
//!
//! // Create distributed context
//! let ctx = DistributedContext::from_parent(&parent);
//!
//! // Inject into outgoing headers
//! let mut headers = HeaderCarrier::new();
//! Propagator::inject(&ctx, &mut headers);
//!
//! println!("traceparent: {}", headers.get("traceparent").unwrap());
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// W3C Trace Context Types
// ============================================================================

/// W3C Trace Context version
pub const TRACE_CONTEXT_VERSION: u8 = 0x00;

/// Trace flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TraceFlags(u8);

impl TraceFlags {
    /// Sampled flag (bit 0)
    pub const SAMPLED: u8 = 0x01;

    /// Create new trace flags
    pub fn new(flags: u8) -> Self {
        Self(flags)
    }

    /// Check if sampled
    pub fn is_sampled(&self) -> bool {
        self.0 & Self::SAMPLED != 0
    }

    /// Set sampled flag
    pub fn set_sampled(&mut self, sampled: bool) {
        if sampled {
            self.0 |= Self::SAMPLED;
        } else {
            self.0 &= !Self::SAMPLED;
        }
    }

    /// Get raw value
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for TraceFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

/// 128-bit Trace ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId([u8; 16]);

impl TraceId {
    /// Create new random trace ID
    pub fn generate() -> Self {
        let uuid = Uuid::new_v4();
        Self(*uuid.as_bytes())
    }

    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, TraceError> {
        if hex.len() != 32 {
            return Err(TraceError::InvalidTraceId(hex.to_string()));
        }

        let mut bytes = [0u8; 16];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let s = std::str::from_utf8(chunk)
                .map_err(|_| TraceError::InvalidTraceId(hex.to_string()))?;
            bytes[i] = u8::from_str_radix(s, 16)
                .map_err(|_| TraceError::InvalidTraceId(hex.to_string()))?;
        }

        Ok(Self(bytes))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Check if valid (non-zero)
    pub fn is_valid(&self) -> bool {
        self.0.iter().any(|&b| b != 0)
    }

    /// Convert from UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(*uuid.as_bytes())
    }

    /// Convert to UUID
    pub fn to_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0)
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// 64-bit Span ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId([u8; 8]);

impl SpanId {
    /// Create new random span ID
    pub fn generate() -> Self {
        let uuid = Uuid::new_v4();
        let bytes = uuid.as_bytes();
        let mut span_bytes = [0u8; 8];
        span_bytes.copy_from_slice(&bytes[..8]);
        Self(span_bytes)
    }

    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, TraceError> {
        if hex.len() != 16 {
            return Err(TraceError::InvalidSpanId(hex.to_string()));
        }

        let mut bytes = [0u8; 8];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let s = std::str::from_utf8(chunk)
                .map_err(|_| TraceError::InvalidSpanId(hex.to_string()))?;
            bytes[i] = u8::from_str_radix(s, 16)
                .map_err(|_| TraceError::InvalidSpanId(hex.to_string()))?;
        }

        Ok(Self(bytes))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    /// Check if valid (non-zero)
    pub fn is_valid(&self) -> bool {
        self.0.iter().any(|&b| b != 0)
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// W3C traceparent header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceParent {
    /// Version (always 00 currently)
    pub version: u8,
    /// Trace ID (128-bit)
    pub trace_id: TraceId,
    /// Parent Span ID (64-bit)
    pub parent_id: SpanId,
    /// Trace flags
    pub flags: TraceFlags,
}

impl TraceParent {
    /// Create new traceparent
    pub fn new(trace_id: TraceId, parent_id: SpanId, sampled: bool) -> Self {
        let mut flags = TraceFlags::default();
        flags.set_sampled(sampled);
        Self {
            version: TRACE_CONTEXT_VERSION,
            trace_id,
            parent_id,
            flags,
        }
    }

    /// Create new root trace
    pub fn new_root(sampled: bool) -> Self {
        Self::new(TraceId::generate(), SpanId::generate(), sampled)
    }

    /// Parse from traceparent header value
    ///
    /// Format: version-trace_id-parent_id-flags
    /// Example: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
    pub fn parse(value: &str) -> Result<Self, TraceError> {
        let parts: Vec<&str> = value.trim().split('-').collect();
        if parts.len() != 4 {
            return Err(TraceError::InvalidTraceParent(value.to_string()));
        }

        // Parse version
        let version = u8::from_str_radix(parts[0], 16)
            .map_err(|_| TraceError::InvalidTraceParent(value.to_string()))?;

        // Parse trace ID
        let trace_id = TraceId::from_hex(parts[1])?;
        if !trace_id.is_valid() {
            return Err(TraceError::InvalidTraceId(parts[1].to_string()));
        }

        // Parse parent ID
        let parent_id = SpanId::from_hex(parts[2])?;
        if !parent_id.is_valid() {
            return Err(TraceError::InvalidSpanId(parts[2].to_string()));
        }

        // Parse flags
        let flags_value = u8::from_str_radix(parts[3], 16)
            .map_err(|_| TraceError::InvalidTraceParent(value.to_string()))?;

        Ok(Self {
            version,
            trace_id,
            parent_id,
            flags: TraceFlags::new(flags_value),
        })
    }

    /// Convert to traceparent header value
    pub fn to_header(&self) -> String {
        format!(
            "{:02x}-{}-{}-{}",
            self.version, self.trace_id, self.parent_id, self.flags
        )
    }

    /// Check if sampled
    pub fn is_sampled(&self) -> bool {
        self.flags.is_sampled()
    }
}

impl fmt::Display for TraceParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_header())
    }
}

/// W3C tracestate header value
#[derive(Debug, Clone, Default)]
pub struct TraceState {
    /// Key-value pairs (vendor-specific data)
    entries: Vec<(String, String)>,
}

impl TraceState {
    /// Create empty tracestate
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse from tracestate header value
    pub fn parse(value: &str) -> Result<Self, TraceError> {
        let mut entries = Vec::new();

        for part in value.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some((key, val)) = part.split_once('=') {
                entries.push((key.to_string(), val.to_string()));
            }
        }

        Ok(Self { entries })
    }

    /// Get value by key
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Set value (adds or updates)
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();

        if let Some((_, v)) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            *v = value;
        } else {
            // New entries go to the front (per W3C spec)
            self.entries.insert(0, (key, value));
        }
    }

    /// Remove by key
    pub fn remove(&mut self, key: &str) {
        self.entries.retain(|(k, _)| k != key);
    }

    /// Convert to tracestate header value
    pub fn to_header(&self) -> String {
        self.entries
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_header())
    }
}

// ============================================================================
// Distributed Context
// ============================================================================

/// Distributed tracing context
#[derive(Debug, Clone)]
pub struct DistributedContext {
    /// Trace ID
    pub trace_id: TraceId,
    /// Current span ID
    pub span_id: SpanId,
    /// Parent span ID
    pub parent_span_id: Option<SpanId>,
    /// Trace state (vendor data)
    pub trace_state: TraceState,
    /// Trace flags
    pub flags: TraceFlags,
    /// Remote indicator (true if from another service)
    pub is_remote: bool,
    /// Service name
    pub service_name: Option<String>,
    /// Additional baggage items
    pub baggage: HashMap<String, String>,
}

impl DistributedContext {
    /// Create new root context
    pub fn new_root(sampled: bool) -> Self {
        let mut flags = TraceFlags::default();
        flags.set_sampled(sampled);

        Self {
            trace_id: TraceId::generate(),
            span_id: SpanId::generate(),
            parent_span_id: None,
            trace_state: TraceState::new(),
            flags,
            is_remote: false,
            service_name: None,
            baggage: HashMap::new(),
        }
    }

    /// Create context from incoming traceparent
    pub fn from_parent(parent: &TraceParent) -> Self {
        Self {
            trace_id: parent.trace_id,
            span_id: SpanId::generate(),
            parent_span_id: Some(parent.parent_id),
            trace_state: TraceState::new(),
            flags: parent.flags,
            is_remote: true,
            service_name: None,
            baggage: HashMap::new(),
        }
    }

    /// Create child context
    pub fn create_child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: SpanId::generate(),
            parent_span_id: Some(self.span_id),
            trace_state: self.trace_state.clone(),
            flags: self.flags,
            is_remote: false,
            service_name: self.service_name.clone(),
            baggage: self.baggage.clone(),
        }
    }

    /// Convert to traceparent
    pub fn to_traceparent(&self) -> TraceParent {
        TraceParent {
            version: TRACE_CONTEXT_VERSION,
            trace_id: self.trace_id,
            parent_id: self.span_id,
            flags: self.flags,
        }
    }

    /// Check if sampled
    pub fn is_sampled(&self) -> bool {
        self.flags.is_sampled()
    }

    /// Set service name
    pub fn with_service(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// Add baggage item
    pub fn add_baggage(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.baggage.insert(key.into(), value.into());
    }
}

// ============================================================================
// Carrier Trait & Implementations
// ============================================================================

/// Carrier for trace context propagation
pub trait Carrier {
    /// Get value by key
    fn get(&self, key: &str) -> Option<&str>;
    /// Set value by key
    fn set(&mut self, key: &str, value: String);
    /// Get all keys
    fn keys(&self) -> Vec<&str>;
}

/// Header-based carrier (HTTP headers)
#[derive(Debug, Clone, Default)]
pub struct HeaderCarrier {
    headers: HashMap<String, String>,
}

impl HeaderCarrier {
    /// Create new header carrier
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from hashmap
    pub fn from_headers(headers: HashMap<String, String>) -> Self {
        Self { headers }
    }

    /// Get all headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

impl Carrier for HeaderCarrier {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(&key.to_lowercase()).map(|s| s.as_str())
    }

    fn set(&mut self, key: &str, value: String) {
        self.headers.insert(key.to_lowercase(), value);
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|s| s.as_str()).collect()
    }
}

// ============================================================================
// Propagator
// ============================================================================

/// Header names
pub const TRACEPARENT_HEADER: &str = "traceparent";
pub const TRACESTATE_HEADER: &str = "tracestate";
pub const BAGGAGE_HEADER: &str = "baggage";

/// Trace context propagator
pub struct Propagator;

impl Propagator {
    /// Extract context from carrier
    pub fn extract<C: Carrier>(carrier: &C) -> Option<DistributedContext> {
        // Extract traceparent
        let traceparent_value = carrier.get(TRACEPARENT_HEADER)?;
        let traceparent = TraceParent::parse(traceparent_value).ok()?;

        let mut ctx = DistributedContext::from_parent(&traceparent);

        // Extract tracestate
        if let Some(tracestate_value) = carrier.get(TRACESTATE_HEADER) {
            if let Ok(tracestate) = TraceState::parse(tracestate_value) {
                ctx.trace_state = tracestate;
            }
        }

        // Extract baggage
        if let Some(baggage_value) = carrier.get(BAGGAGE_HEADER) {
            for part in baggage_value.split(',') {
                if let Some((key, value)) = part.trim().split_once('=') {
                    ctx.baggage.insert(key.to_string(), value.to_string());
                }
            }
        }

        Some(ctx)
    }

    /// Inject context into carrier
    pub fn inject<C: Carrier>(ctx: &DistributedContext, carrier: &mut C) {
        // Inject traceparent
        carrier.set(TRACEPARENT_HEADER, ctx.to_traceparent().to_header());

        // Inject tracestate if not empty
        if !ctx.trace_state.is_empty() {
            carrier.set(TRACESTATE_HEADER, ctx.trace_state.to_header());
        }

        // Inject baggage if not empty
        if !ctx.baggage.is_empty() {
            let baggage = ctx
                .baggage
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            carrier.set(BAGGAGE_HEADER, baggage);
        }
    }
}

// ============================================================================
// Sampling
// ============================================================================

/// Sampling decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingDecision {
    /// Drop this trace
    Drop,
    /// Record but don't sample
    RecordOnly,
    /// Record and sample
    RecordAndSample,
}

impl SamplingDecision {
    /// Check if should record
    pub fn should_record(&self) -> bool {
        !matches!(self, SamplingDecision::Drop)
    }

    /// Check if should sample
    pub fn should_sample(&self) -> bool {
        matches!(self, SamplingDecision::RecordAndSample)
    }
}

/// Sampling parameters
#[derive(Debug, Clone)]
pub struct SamplingParameters {
    /// Trace ID
    pub trace_id: TraceId,
    /// Span name
    pub name: String,
    /// Span kind
    pub kind: SpanKind,
    /// Attributes
    pub attributes: HashMap<String, String>,
    /// Parent context
    pub parent: Option<DistributedContext>,
}

/// Span kind (for sampling decisions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// Internal operation
    Internal,
    /// Incoming request (server)
    Server,
    /// Outgoing request (client)
    Client,
    /// Producer (messaging)
    Producer,
    /// Consumer (messaging)
    Consumer,
}

/// Sampler trait
pub trait Sampler: Send + Sync {
    /// Get sampling decision
    fn should_sample(&self, params: &SamplingParameters) -> SamplingDecision;

    /// Get description
    fn description(&self) -> String;
}

/// Always-on sampler (sample everything)
#[derive(Debug, Clone, Default)]
pub struct AlwaysOnSampler;

impl Sampler for AlwaysOnSampler {
    fn should_sample(&self, _params: &SamplingParameters) -> SamplingDecision {
        SamplingDecision::RecordAndSample
    }

    fn description(&self) -> String {
        "AlwaysOnSampler".to_string()
    }
}

/// Always-off sampler (sample nothing)
#[derive(Debug, Clone, Default)]
pub struct AlwaysOffSampler;

impl Sampler for AlwaysOffSampler {
    fn should_sample(&self, _params: &SamplingParameters) -> SamplingDecision {
        SamplingDecision::Drop
    }

    fn description(&self) -> String {
        "AlwaysOffSampler".to_string()
    }
}

/// Probability-based sampler
#[derive(Debug, Clone)]
pub struct ProbabilitySampler {
    /// Sampling probability (0.0 to 1.0)
    probability: f64,
    /// Threshold for trace ID comparison
    threshold: u64,
}

impl ProbabilitySampler {
    /// Create new probability sampler
    pub fn new(probability: f64) -> Self {
        let probability = probability.clamp(0.0, 1.0);
        let threshold = (probability * u64::MAX as f64) as u64;
        Self {
            probability,
            threshold,
        }
    }

    /// Get probability
    pub fn probability(&self) -> f64 {
        self.probability
    }
}

impl Sampler for ProbabilitySampler {
    fn should_sample(&self, params: &SamplingParameters) -> SamplingDecision {
        // Use trace ID for deterministic sampling
        // Use first 8 bytes (bytes 0-7) which are fully random in UUID v4
        // Note: bytes 6-7 have version bits but bytes 0-5 are fully random
        let bytes = params.trace_id.as_bytes();
        let hash = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);

        if hash < self.threshold {
            SamplingDecision::RecordAndSample
        } else {
            SamplingDecision::Drop
        }
    }

    fn description(&self) -> String {
        format!("ProbabilitySampler{{probability={}}}", self.probability)
    }
}

/// Rate-limiting sampler
pub struct RateLimitingSampler {
    /// Maximum samples per second
    max_per_second: u64,
    /// Sample counter
    counter: AtomicU64,
    /// Last reset time (unix timestamp)
    last_reset: AtomicU64,
}

impl RateLimitingSampler {
    /// Create new rate-limiting sampler
    pub fn new(max_per_second: u64) -> Self {
        Self {
            max_per_second,
            counter: AtomicU64::new(0),
            last_reset: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        }
    }

    fn maybe_reset(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let last = self.last_reset.load(Ordering::Relaxed);
        if now > last {
            if self
                .last_reset
                .compare_exchange(last, now, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                self.counter.store(0, Ordering::Relaxed);
            }
        }
    }
}

impl Sampler for RateLimitingSampler {
    fn should_sample(&self, _params: &SamplingParameters) -> SamplingDecision {
        self.maybe_reset();

        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        if count < self.max_per_second {
            SamplingDecision::RecordAndSample
        } else {
            SamplingDecision::Drop
        }
    }

    fn description(&self) -> String {
        format!("RateLimitingSampler{{max_per_second={}}}", self.max_per_second)
    }
}

/// Parent-based sampler (follows parent's decision)
pub struct ParentBasedSampler {
    /// Root sampler (when no parent)
    root: Arc<dyn Sampler>,
}

impl ParentBasedSampler {
    /// Create new parent-based sampler
    pub fn new(root: Arc<dyn Sampler>) -> Self {
        Self { root }
    }
}

impl Sampler for ParentBasedSampler {
    fn should_sample(&self, params: &SamplingParameters) -> SamplingDecision {
        match &params.parent {
            Some(parent) if parent.is_sampled() => SamplingDecision::RecordAndSample,
            Some(_) => SamplingDecision::Drop,
            None => self.root.should_sample(params),
        }
    }

    fn description(&self) -> String {
        format!("ParentBasedSampler{{root={}}}", self.root.description())
    }
}

// ============================================================================
// Export Formats
// ============================================================================

/// Span for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSpan {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_span_id: Option<String>,
    /// Operation name
    pub operation_name: String,
    /// Service name
    pub service_name: String,
    /// Start time (unix timestamp in microseconds)
    pub start_time_us: i64,
    /// Duration (microseconds)
    pub duration_us: i64,
    /// Tags/attributes
    pub tags: HashMap<String, String>,
    /// Logs/events
    pub logs: Vec<ExportLog>,
    /// Kind
    pub kind: String,
}

/// Log/event for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportLog {
    /// Timestamp (unix microseconds)
    pub timestamp_us: i64,
    /// Fields
    pub fields: HashMap<String, String>,
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Zipkin JSON format
    Zipkin,
    /// OTLP JSON format
    Otlp,
    /// RealConsole native format
    Native,
}

/// Span exporter
pub struct SpanExporter;

impl SpanExporter {
    /// Export spans to Zipkin format
    pub fn to_zipkin(spans: &[ExportSpan]) -> String {
        let zipkin_spans: Vec<serde_json::Value> = spans
            .iter()
            .map(|span| {
                serde_json::json!({
                    "traceId": span.trace_id,
                    "id": span.span_id,
                    "parentId": span.parent_span_id,
                    "name": span.operation_name,
                    "timestamp": span.start_time_us,
                    "duration": span.duration_us,
                    "localEndpoint": {
                        "serviceName": span.service_name
                    },
                    "tags": span.tags,
                    "annotations": span.logs.iter().map(|log| {
                        serde_json::json!({
                            "timestamp": log.timestamp_us,
                            "value": log.fields.get("message").unwrap_or(&"event".to_string())
                        })
                    }).collect::<Vec<_>>()
                })
            })
            .collect();

        serde_json::to_string_pretty(&zipkin_spans).unwrap_or_default()
    }

    /// Export spans to OTLP JSON format
    pub fn to_otlp(spans: &[ExportSpan], service_name: &str) -> String {
        let resource_spans = serde_json::json!({
            "resourceSpans": [{
                "resource": {
                    "attributes": [{
                        "key": "service.name",
                        "value": { "stringValue": service_name }
                    }]
                },
                "scopeSpans": [{
                    "scope": {
                        "name": "realconsole",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "spans": spans.iter().map(|span| {
                        serde_json::json!({
                            "traceId": span.trace_id,
                            "spanId": span.span_id,
                            "parentSpanId": span.parent_span_id.as_deref().unwrap_or(""),
                            "name": span.operation_name,
                            "kind": match span.kind.as_str() {
                                "server" => 2,
                                "client" => 3,
                                "producer" => 4,
                                "consumer" => 5,
                                _ => 1 // internal
                            },
                            "startTimeUnixNano": span.start_time_us * 1000,
                            "endTimeUnixNano": (span.start_time_us + span.duration_us) * 1000,
                            "attributes": span.tags.iter().map(|(k, v)| {
                                serde_json::json!({
                                    "key": k,
                                    "value": { "stringValue": v }
                                })
                            }).collect::<Vec<_>>(),
                            "events": span.logs.iter().map(|log| {
                                serde_json::json!({
                                    "timeUnixNano": log.timestamp_us * 1000,
                                    "name": log.fields.get("name").unwrap_or(&"event".to_string()),
                                    "attributes": log.fields.iter().map(|(k, v)| {
                                        serde_json::json!({
                                            "key": k,
                                            "value": { "stringValue": v }
                                        })
                                    }).collect::<Vec<_>>()
                                })
                            }).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>()
                }]
            }]
        });

        serde_json::to_string_pretty(&resource_spans).unwrap_or_default()
    }

    /// Export spans to native format
    pub fn to_native(spans: &[ExportSpan]) -> String {
        serde_json::to_string_pretty(&spans).unwrap_or_default()
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Trace error
#[derive(Debug, Clone, thiserror::Error)]
pub enum TraceError {
    #[error("Invalid traceparent header: {0}")]
    InvalidTraceParent(String),

    #[error("Invalid trace ID: {0}")]
    InvalidTraceId(String),

    #[error("Invalid span ID: {0}")]
    InvalidSpanId(String),

    #[error("Invalid tracestate: {0}")]
    InvalidTraceState(String),
}

// ============================================================================
// Stats
// ============================================================================

/// Distributed tracing statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistributedTracingStats {
    /// Total spans created
    pub spans_created: u64,
    /// Spans sampled
    pub spans_sampled: u64,
    /// Spans dropped
    pub spans_dropped: u64,
    /// Remote contexts received
    pub remote_contexts: u64,
    /// Contexts propagated
    pub contexts_propagated: u64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id() {
        let id = TraceId::generate();
        assert!(id.is_valid());

        let hex = id.to_hex();
        assert_eq!(hex.len(), 32);

        let parsed = TraceId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_span_id() {
        let id = SpanId::generate();
        assert!(id.is_valid());

        let hex = id.to_hex();
        assert_eq!(hex.len(), 16);

        let parsed = SpanId::from_hex(&hex).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_trace_flags() {
        let mut flags = TraceFlags::default();
        assert!(!flags.is_sampled());

        flags.set_sampled(true);
        assert!(flags.is_sampled());
        assert_eq!(flags.value(), 0x01);
    }

    #[test]
    fn test_traceparent_parse() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let parent = TraceParent::parse(value).unwrap();

        assert_eq!(parent.version, 0x00);
        assert!(parent.is_sampled());
        assert_eq!(parent.trace_id.to_hex(), "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(parent.parent_id.to_hex(), "00f067aa0ba902b7");
    }

    #[test]
    fn test_traceparent_roundtrip() {
        let original = TraceParent::new_root(true);
        let header = original.to_header();
        let parsed = TraceParent::parse(&header).unwrap();

        assert_eq!(original.trace_id, parsed.trace_id);
        assert_eq!(original.parent_id, parsed.parent_id);
        assert_eq!(original.is_sampled(), parsed.is_sampled());
    }

    #[test]
    fn test_tracestate() {
        let value = "vendor1=value1,vendor2=value2";
        let state = TraceState::parse(value).unwrap();

        assert_eq!(state.get("vendor1"), Some("value1"));
        assert_eq!(state.get("vendor2"), Some("value2"));
        assert_eq!(state.get("vendor3"), None);
    }

    #[test]
    fn test_distributed_context() {
        let root = DistributedContext::new_root(true);
        assert!(root.is_sampled());
        assert!(root.parent_span_id.is_none());
        assert!(!root.is_remote);

        let child = root.create_child();
        assert_eq!(child.trace_id, root.trace_id);
        assert_eq!(child.parent_span_id, Some(root.span_id));
    }

    #[test]
    fn test_propagator_inject_extract() {
        let ctx = DistributedContext::new_root(true).with_service("test-service");

        let mut carrier = HeaderCarrier::new();
        Propagator::inject(&ctx, &mut carrier);

        let extracted = Propagator::extract(&carrier).unwrap();
        assert_eq!(extracted.trace_id, ctx.trace_id);
        assert!(extracted.is_remote);
    }

    #[test]
    fn test_always_on_sampler() {
        let sampler = AlwaysOnSampler;
        let params = SamplingParameters {
            trace_id: TraceId::generate(),
            name: "test".to_string(),
            kind: SpanKind::Internal,
            attributes: HashMap::new(),
            parent: None,
        };

        assert_eq!(
            sampler.should_sample(&params),
            SamplingDecision::RecordAndSample
        );
    }

    #[test]
    fn test_always_off_sampler() {
        let sampler = AlwaysOffSampler;
        let params = SamplingParameters {
            trace_id: TraceId::generate(),
            name: "test".to_string(),
            kind: SpanKind::Internal,
            attributes: HashMap::new(),
            parent: None,
        };

        assert_eq!(sampler.should_sample(&params), SamplingDecision::Drop);
    }

    #[test]
    fn test_probability_sampler() {
        let sampler = ProbabilitySampler::new(0.5);

        let mut sampled = 0;
        let trials = 1000;

        for _ in 0..trials {
            // Use random trace IDs for proper distribution
            let trace_id = TraceId::generate();

            let params = SamplingParameters {
                trace_id,
                name: "test".to_string(),
                kind: SpanKind::Internal,
                attributes: HashMap::new(),
                parent: None,
            };

            if sampler.should_sample(&params) == SamplingDecision::RecordAndSample {
                sampled += 1;
            }
        }

        // Should be roughly 50% (with some variance)
        // With 1000 trials at 50%, we expect ~500 samples
        // Allow ±15% variance for random sampling
        let ratio = sampled as f64 / trials as f64;
        assert!(ratio > 0.35 && ratio < 0.65, "ratio: {}, expected ~0.5", ratio);
    }

    #[test]
    fn test_parent_based_sampler() {
        let root_sampler = Arc::new(AlwaysOnSampler);
        let sampler = ParentBasedSampler::new(root_sampler);

        // No parent - uses root sampler
        let params = SamplingParameters {
            trace_id: TraceId::generate(),
            name: "test".to_string(),
            kind: SpanKind::Internal,
            attributes: HashMap::new(),
            parent: None,
        };
        assert_eq!(
            sampler.should_sample(&params),
            SamplingDecision::RecordAndSample
        );

        // Parent sampled - follows parent
        let sampled_parent = DistributedContext::new_root(true);
        let params_with_parent = SamplingParameters {
            trace_id: sampled_parent.trace_id,
            name: "test".to_string(),
            kind: SpanKind::Internal,
            attributes: HashMap::new(),
            parent: Some(sampled_parent),
        };
        assert_eq!(
            sampler.should_sample(&params_with_parent),
            SamplingDecision::RecordAndSample
        );

        // Parent not sampled - follows parent
        let unsampled_parent = DistributedContext::new_root(false);
        let params_unsampled = SamplingParameters {
            trace_id: unsampled_parent.trace_id,
            name: "test".to_string(),
            kind: SpanKind::Internal,
            attributes: HashMap::new(),
            parent: Some(unsampled_parent),
        };
        assert_eq!(
            sampler.should_sample(&params_unsampled),
            SamplingDecision::Drop
        );
    }

    #[test]
    fn test_export_zipkin() {
        let span = ExportSpan {
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            span_id: "00f067aa0ba902b7".to_string(),
            parent_span_id: None,
            operation_name: "test-operation".to_string(),
            service_name: "test-service".to_string(),
            start_time_us: 1704067200000000,
            duration_us: 1000,
            tags: HashMap::from([("key".to_string(), "value".to_string())]),
            logs: vec![],
            kind: "server".to_string(),
        };

        let json = SpanExporter::to_zipkin(&[span]);
        assert!(json.contains("traceId"));
        assert!(json.contains("test-operation"));
        assert!(json.contains("test-service"));
    }

    #[test]
    fn test_export_otlp() {
        let span = ExportSpan {
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            span_id: "00f067aa0ba902b7".to_string(),
            parent_span_id: None,
            operation_name: "test-operation".to_string(),
            service_name: "test-service".to_string(),
            start_time_us: 1704067200000000,
            duration_us: 1000,
            tags: HashMap::new(),
            logs: vec![],
            kind: "internal".to_string(),
        };

        let json = SpanExporter::to_otlp(&[span], "my-service");
        assert!(json.contains("resourceSpans"));
        assert!(json.contains("scopeSpans"));
        assert!(json.contains("realconsole"));
    }

    #[test]
    fn test_baggage_propagation() {
        let mut ctx = DistributedContext::new_root(true);
        ctx.add_baggage("user_id", "12345");
        ctx.add_baggage("request_id", "abc-def");

        let mut carrier = HeaderCarrier::new();
        Propagator::inject(&ctx, &mut carrier);

        let extracted = Propagator::extract(&carrier).unwrap();
        assert_eq!(extracted.baggage.get("user_id"), Some(&"12345".to_string()));
        assert_eq!(
            extracted.baggage.get("request_id"),
            Some(&"abc-def".to_string())
        );
    }

    #[test]
    fn test_trace_id_uuid_conversion() {
        let uuid = Uuid::new_v4();
        let trace_id = TraceId::from_uuid(uuid);
        let back = trace_id.to_uuid();
        assert_eq!(uuid, back);
    }
}
