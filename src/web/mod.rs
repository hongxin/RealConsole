//! Web 终端模块
//!
//! 提供浏览器访问的 Web 终端界面，通过 WebSocket 与 RealConsole Agent 交互
//!
//! # 架构设计（一分为三）
//! - **表现层**：Browser + xterm.js（用户交互）
//! - **通信层**：WebSocket Handler（消息路由）
//! - **执行层**：Agent（业务逻辑复用）
//!
//! # 使用方式
//! ```bash
//! # 启动 Web 服务
//! realconsole web
//!
//! # 自定义端口
//! realconsole web --port 9000
//!
//! # 指定绑定地址
//! realconsole web --bind 0.0.0.0
//! ```

pub mod server;
pub mod session;
pub mod session_manager; // v1.40.0: 会话持久化管理
pub mod session_persistence; // v1.94.0: 会话持久化与恢复（自动保存、Token、重连）
pub mod websocket;
pub mod frontend;
pub mod uploaded_files; // v1.46.0: 文件上传管理
pub mod metadata_extractor; // v1.53.0: 元数据提取器统一架构
pub mod memory; // v1.54.0: Memory 2.0 WebUI - 智能上下文编排器
pub mod mobile; // v1.95.0: 移动端增强（触控、手势、虚拟键盘）
pub mod keyboard; // v1.96.0: 键盘快捷键系统（帮助覆盖层、命令面板）
pub mod theme; // v1.97.0: 主题系统（亮/暗/跟随系统、自定义主题）
pub mod virtual_scroll; // v1.98.0: 虚拟滚动（大输出性能优化、DOM回收）
pub mod ws_optimize; // v1.99.0: WebSocket 优化（消息批处理、心跳、重连）
pub mod asset_optimize; // v1.100.0: 资源优化（缓存策略、压缩、懒加载、代码分割）
pub mod tabs; // v1.101.0: 多标签支持（标签管理、状态切换、会话隔离）

pub use server::WebServer;
