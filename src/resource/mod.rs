//! v1.92.0: Resource Management System
//!
//! Provides unified resource monitoring and automatic cleanup capabilities:
//! - **ResourceMonitor**: Track memory and resource usage
//! - **CleanupManager**: Automatic cleanup when thresholds exceeded
//! - **ResourceStats**: Comprehensive resource statistics
//!
//! ## Architecture Design
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │            ResourceMonitor                   │
//! ├─────────────────────────────────────────────┤
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐     │
//! │  │ Memory  │  │Component│  │ Cleanup │     │
//! │  │ Tracker │  │ Tracker │  │ Manager │     │
//! │  └─────────┘  └─────────┘  └─────────┘     │
//! └─────────────────────────────────────────────┘
//! ```

pub mod cleanup;
pub mod monitor;

pub use cleanup::{
    CleanupAction, CleanupConfig, CleanupManager, CleanupResult, CleanupStats, CleanupTrigger,
};
pub use monitor::{
    ComponentUsage, MemorySnapshot, ResourceConfig, ResourceMonitor, ResourceStats,
    ResourceThreshold, ResourceUsage,
};
