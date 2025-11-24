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
pub mod websocket;
pub mod frontend;
pub mod uploaded_files; // v1.46.0: 文件上传管理
pub mod metadata_extractor; // v1.53.0: 元数据提取器统一架构
pub mod memory; // v1.54.0: Memory 2.0 WebUI - 智能上下文编排器

pub use server::WebServer;
