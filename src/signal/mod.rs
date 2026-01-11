//! v1.93.0: Signal Handling System
//!
//! Provides unified signal handling and graceful shutdown capabilities:
//! - **SignalHandler**: Intercept SIGINT/SIGTERM signals
//! - **ShutdownCoordinator**: Coordinate graceful shutdown across components
//! - **ShutdownGuard**: RAII guard for cleanup registration
//!
//! ## Architecture Design
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │            SignalHandler                     │
//! ├─────────────────────────────────────────────┤
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐     │
//! │  │ SIGINT  │  │ SIGTERM │  │Shutdown │     │
//! │  │ Handler │  │ Handler │  │Broadcast│     │
//! │  └─────────┘  └─────────┘  └─────────┘     │
//! │         │          │            │           │
//! │         └──────────┴────────────┘           │
//! │                    │                        │
//! │         ┌──────────▼──────────┐             │
//! │         │ ShutdownCoordinator │             │
//! │         │  - Cleanup hooks    │             │
//! │         │  - State saving     │             │
//! │         │  - Resource release │             │
//! │         └─────────────────────┘             │
//! └─────────────────────────────────────────────┘
//! ```

pub mod handler;
pub mod shutdown;

pub use handler::{SignalConfig, SignalHandler, SignalStats, SignalType};
pub use shutdown::{
    CleanupHook, ShutdownCoordinator, ShutdownConfig, ShutdownGuard, ShutdownPhase,
    ShutdownResult, ShutdownStats,
};
