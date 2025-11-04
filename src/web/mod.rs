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
pub mod websocket;

pub use server::WebServer;
