//! Web 服务器实现
//!
//! 基于 axum 框架，提供：
//! - WebSocket 连接（/ws）
//! - 静态文件服务（/static/*）
//! - 健康检查（/health）
//! - 主页重定向（/）

use crate::command::CommandRegistry;
use crate::config::{Config, WebConfig};
use crate::i18n;
use crate::web::frontend;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use colored::Colorize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

/// Web 服务器状态
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub registry: CommandRegistry,
}

/// Web 服务器
pub struct WebServer {
    config: WebConfig,
    app_state: AppState,
}

impl WebServer {
    /// 创建新的 Web 服务器
    pub fn new(web_config: WebConfig, config: Config, registry: CommandRegistry) -> Self {
        let app_state = AppState { config, registry };
        Self {
            config: web_config,
            app_state,
        }
    }

    /// 启动 Web 服务器
    pub async fn serve(self) -> anyhow::Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.bind, self.config.port)
            .parse()
            .expect("Invalid bind address");

        // 配置 CORS
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // 构建路由
        let app = Router::new()
            .route("/", get(index_handler))
            .route("/health", get(health_handler))
            .route("/ws", get(ws_handler))
            .route("/static/*path", get(static_handler))
            .layer(ServiceBuilder::new().layer(cors))
            .with_state(Arc::new(self.app_state));

        // 启动提示
        println!("\n{}", i18n::t("web.server.startup").cyan().bold());
        println!("   {} http://{}", i18n::t("web.server.address_label").green(), addr);
        println!("   {} {}\n", i18n::t("web.server.tip_label").yellow(), i18n::t("web.server.stop_hint"));

        // 启动服务
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// 主页处理
async fn index_handler() -> impl IntoResponse {
    Html(frontend::get_index_html())
}

/// 健康检查
async fn health_handler() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "realconsole-web"
    }))
}

/// WebSocket 连接处理
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 处理 WebSocket 连接
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    use crate::web::websocket::WebSocketSession;

    let session = WebSocketSession::new(socket, state.config.clone(), state.registry.clone()).await;
    if let Err(e) = session.run().await {
        eprintln!("WebSocket session error: {}", e);
    }
}

/// 静态文件处理
async fn static_handler(uri: Uri) -> impl IntoResponse {
    use axum::body::Body;

    let path = uri.path().trim_start_matches("/static/");

    match path {
        "terminal.js" => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/javascript")
            .body(Body::from(frontend::get_terminal_js()))
            .unwrap(),
        "style.css" => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/css")
            .body(Body::from(frontend::get_style_css()))
            .unwrap(),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

