//! Voice 模块
//!
//! 提供语音播报功能，支持多平台 TTS（Text-to-Speech）
//!
//! 特性：
//! - macOS: 使用 `say` 命令
//! - Linux: 支持 `espeak` 或 `festival`（可选）
//! - Windows: 支持 PowerShell TTS（可选）
//! - 异步播报，不阻塞主线程
//! - 队列管理，避免语音重叠

pub mod voice_broadcaster;
pub mod platform;
pub mod filter;

pub use voice_broadcaster::{VoiceBroadcaster, BroadcastConfig};
pub use platform::PlatformVoice;
pub use filter::{filter_for_voice, FilterConfig};
