//! Web 服务器实现
//!
//! 基于 axum 框架，提供：
//! - WebSocket 连接（/ws）
//! - 静态文件服务（/static/*）
//! - 健康检查（/health）
//! - 主页重定向（/）

use crate::command::CommandRegistry;
use crate::config::{Config, WebConfig};
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
        println!("\n{}", "🌐 RealConsole Web 终端启动".cyan().bold());
        println!("   {} http://{}", "地址:".green(), addr);
        println!("   {} 按 Ctrl+C 停止服务\n", "提示:".yellow());

        // 启动服务
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// 主页处理
async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
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
            .body(Body::from(TERMINAL_JS))
            .unwrap(),
        "style.css" => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/css")
            .body(Body::from(STYLE_CSS))
            .unwrap(),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

/// 内嵌的主页 HTML
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RealConsole Web 终端</title>
    <link rel="stylesheet" href="/static/style.css">
    <!-- xterm.js CDN -->
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/xterm@5.3.0/css/xterm.min.css">
    <script src="https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/xterm-addon-fit@0.8.0/lib/xterm-addon-fit.min.js"></script>
</head>
<body>
    <div id="header">
        <h1>🌟 RealConsole Web 终端</h1>
        <p>融合东方哲学智慧的智能 CLI Agent</p>
    </div>
    <div id="terminal-container">
        <div id="terminal"></div>
    </div>
    <div id="status">
        <span id="connection-status">连接中...</span>
    </div>
    <script src="/static/terminal.js"></script>
</body>
</html>
"#;

/// 内嵌的终端 JavaScript
const TERMINAL_JS: &str = r#"
// RealConsole Web Terminal
(function() {
    'use strict';

    // 创建终端
    const term = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: 'Menlo, Monaco, "Courier New", monospace',
        theme: {
            background: '#1e1e1e',
            foreground: '#d4d4d4',
            cursor: '#00ff00',
            selection: '#264f78',
        },
        scrollback: 10000,
        convertEol: true,  // 自动转换行尾符
        wordSeparator: ' ()[]{}\'"`',  // 单词分隔符
        allowProposedApi: true,  // 允许使用提议的 API
    });

    // 适配插件
    const fitAddon = new FitAddon.FitAddon();
    term.loadAddon(fitAddon);

    // 挂载到 DOM
    term.open(document.getElementById('terminal'));

    // 延迟执行 fit，确保 DOM 完全渲染
    // 使用 requestAnimationFrame 确保在下一帧执行
    const doFit = () => {
        try {
            fitAddon.fit();
        } catch (e) {
            console.warn('Fit failed:', e);
        }
    };

    // 初始 fit（延迟执行）
    setTimeout(() => {
        doFit();
        // 再次 fit 确保准确
        setTimeout(doFit, 100);
    }, 0);

    // 窗口大小调整
    window.addEventListener('resize', () => {
        doFit();
    });

    // WebSocket 连接
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    const ws = new WebSocket(wsUrl);

    const statusEl = document.getElementById('connection-status');

    // 输入缓冲和光标位置
    let inputBuffer = '';
    let cursorPosition = 0;  // 光标在输入缓冲中的位置

    // 历史命令管理
    let commandHistory = [];  // 历史命令数组
    let historyIndex = -1;    // 当前浏览的历史索引（-1 表示当前输入）
    let tempInput = '';       // 临时保存正在编辑的命令

    // 飞轮动画（橙色旋转符号，与命令行版本一致）
    const SPINNER_FRAMES = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinnerInterval = null;
    let spinnerFrame = 0;

    function startSpinner(modelName = '') {
        if (spinnerInterval) return; // 已经在运行

        spinnerFrame = 0;
        spinnerInterval = setInterval(() => {
            // 清除当前行并显示橙色飞轮 + 模型名称
            const frame = SPINNER_FRAMES[spinnerFrame];
            const label = modelName ? ` ${modelName} ` : ' ';
            term.write('\r\x1b[38;2;255;165;0m' + frame + '\x1b[0m\x1b[1m' + label + '\x1b[0m');

            spinnerFrame = (spinnerFrame + 1) % SPINNER_FRAMES.length;
        }, 80); // 80ms 每帧，与命令行版本一致
    }

    function stopSpinner() {
        if (spinnerInterval) {
            clearInterval(spinnerInterval);
            spinnerInterval = null;
            // 清除飞轮行
            term.write('\r\x1b[K');
        }
    }

    // 辅助函数：重新渲染当前输入行
    function redrawLine() {
        // 清除当前行
        term.write('\r\x1b[K');
        // 显示提示符
        term.write('\x1b[33m% \x1b[0m');
        // 显示输入内容
        term.write(inputBuffer);
        // 移动光标到正确位置
        if (cursorPosition < inputBuffer.length) {
            const offset = inputBuffer.length - cursorPosition;
            term.write('\x1b[' + offset + 'D');
        }
    }

    // 辅助函数：加载历史命令
    function loadHistory(index) {
        if (index < 0 || index >= commandHistory.length) {
            return;
        }
        inputBuffer = commandHistory[index];
        cursorPosition = inputBuffer.length;
        redrawLine();
    }

    // 辅助函数：添加到历史
    function addToHistory(cmd) {
        if (cmd.trim()) {
            // 避免重复的连续命令
            if (commandHistory.length === 0 || commandHistory[commandHistory.length - 1] !== cmd) {
                commandHistory.push(cmd);
                // 限制历史数量为 1000
                if (commandHistory.length > 1000) {
                    commandHistory.shift();
                }
            }
        }
        historyIndex = -1;
        tempInput = '';
    }

    ws.onopen = () => {
        statusEl.textContent = '已连接';
        statusEl.style.color = '#4CAF50';
        term.writeln('\x1b[32m欢迎使用 RealConsole Web 终端！\x1b[0m');
        term.writeln('\x1b[36m输入命令开始使用，输入 /help 查看帮助\x1b[0m');
        term.write('\n\x1b[33m% \x1b[0m');

        // 连接建立后重新 fit，确保尺寸正确
        setTimeout(doFit, 50);
    };

    ws.onclose = () => {
        statusEl.textContent = '已断开';
        statusEl.style.color = '#f44336';
        term.writeln('\n\x1b[31m连接已断开\x1b[0m');
    };

    ws.onerror = (err) => {
        statusEl.textContent = '连接错误';
        statusEl.style.color = '#f44336';
        console.error('WebSocket error:', err);
    };

    ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);

        switch (msg.type) {
            case 'thinking':
                // 开始思考，显示飞轮
                const modelName = msg.model || '';
                startSpinner(modelName);
                break;
            case 'output':
                // 停止飞轮
                stopSpinner();

                // 格式化输出：将 \n 转换为 \r\n，确保正确换行
                let formattedContent = msg.content
                    .replace(/\r\n/g, '\n')  // 统一换行符
                    .replace(/\n/g, '\r\n'); // 转换为终端格式

                term.write('\r\n\r\n' + formattedContent);
                term.write('\r\n\r\n\x1b[33m% \x1b[0m');
                inputBuffer = '';
                cursorPosition = 0;
                break;
            case 'stream':
                // 第一次流式输出时停止飞轮
                stopSpinner();

                // 流式输出也需要格式化
                let streamContent = msg.content
                    .replace(/\r\n/g, '\n')
                    .replace(/\n/g, '\r\n');
                term.write(streamContent);
                break;
            case 'error':
                // 停止飞轮
                stopSpinner();

                let errorContent = msg.content
                    .replace(/\r\n/g, '\n')
                    .replace(/\n/g, '\r\n');
                term.write('\r\n\r\n\x1b[31m' + errorContent + '\x1b[0m');
                term.write('\r\n\r\n\x1b[33m% \x1b[0m');
                inputBuffer = '';
                cursorPosition = 0;
                break;
            case 'clear':
                // 停止飞轮
                stopSpinner();

                term.clear();
                term.write('\x1b[33m% \x1b[0m');
                inputBuffer = '';
                cursorPosition = 0;
                break;
        }
    };

    // 处理用户输入
    term.onData((data) => {
        // 处理方向键和特殊键（ESC 序列）
        if (data.startsWith('\x1b[')) {
            // 上箭头：历史命令（上一条）
            if (data === '\x1b[A') {
                if (commandHistory.length === 0) return;

                // 第一次按上箭头，保存当前输入
                if (historyIndex === -1) {
                    tempInput = inputBuffer;
                    historyIndex = commandHistory.length - 1;
                } else if (historyIndex > 0) {
                    historyIndex--;
                }

                loadHistory(historyIndex);
                return;
            }

            // 下箭头：历史命令（下一条）
            if (data === '\x1b[B') {
                if (historyIndex === -1) return;

                historyIndex++;
                if (historyIndex >= commandHistory.length) {
                    // 恢复临时输入
                    historyIndex = -1;
                    inputBuffer = tempInput;
                    cursorPosition = inputBuffer.length;
                } else {
                    loadHistory(historyIndex);
                }
                redrawLine();
                return;
            }

            // 右箭头：光标右移
            if (data === '\x1b[C') {
                if (cursorPosition < inputBuffer.length) {
                    cursorPosition++;
                    term.write('\x1b[C');
                }
                return;
            }

            // 左箭头：光标左移
            if (data === '\x1b[D') {
                if (cursorPosition > 0) {
                    cursorPosition--;
                    term.write('\x1b[D');
                }
                return;
            }

            // Home 键：移到行首
            if (data === '\x1b[H' || data === '\x1b[1~') {
                if (cursorPosition > 0) {
                    term.write('\x1b[' + cursorPosition + 'D');
                    cursorPosition = 0;
                }
                return;
            }

            // End 键：移到行尾
            if (data === '\x1b[F' || data === '\x1b[4~') {
                if (cursorPosition < inputBuffer.length) {
                    const offset = inputBuffer.length - cursorPosition;
                    term.write('\x1b[' + offset + 'C');
                    cursorPosition = inputBuffer.length;
                }
                return;
            }

            // Delete 键：删除光标处字符
            if (data === '\x1b[3~') {
                if (cursorPosition < inputBuffer.length) {
                    inputBuffer = inputBuffer.slice(0, cursorPosition) +
                                  inputBuffer.slice(cursorPosition + 1);
                    redrawLine();
                }
                return;
            }

            // 其他 ESC 序列忽略
            return;
        }

        const code = data.charCodeAt(0);

        // Enter 键
        if (code === 13) {
            if (inputBuffer.trim()) {
                // 添加到历史
                addToHistory(inputBuffer);

                // 显示换行
                term.write('\r\n');

                // 发送命令
                ws.send(JSON.stringify({
                    type: 'input',
                    content: inputBuffer
                }));

                // 清空缓冲区
                inputBuffer = '';
                cursorPosition = 0;
            }
            return;
        }

        // Backspace 键
        if (code === 127) {
            if (cursorPosition > 0) {
                // 删除光标前的字符
                inputBuffer = inputBuffer.slice(0, cursorPosition - 1) +
                              inputBuffer.slice(cursorPosition);
                cursorPosition--;
                redrawLine();
            }
            return;
        }

        // Ctrl+C
        if (code === 3) {
            ws.send(JSON.stringify({
                type: 'interrupt',
                content: ''
            }));
            inputBuffer = '';
            cursorPosition = 0;
            historyIndex = -1;
            term.write('^C\r\n\x1b[33m% \x1b[0m');
            return;
        }

        // Ctrl+L (清屏)
        if (code === 12) {
            term.clear();
            term.write('\x1b[33m% \x1b[0m');
            inputBuffer = '';
            cursorPosition = 0;
            return;
        }

        // Ctrl+U (清除整行)
        if (code === 21) {
            inputBuffer = '';
            cursorPosition = 0;
            redrawLine();
            return;
        }

        // 支持所有字符（包括中文）
        // 排除控制字符但允许 UTF-8
        if (code >= 32 || data.length > 1) {
            // 在光标位置插入字符
            inputBuffer = inputBuffer.slice(0, cursorPosition) +
                          data +
                          inputBuffer.slice(cursorPosition);
            cursorPosition += data.length;
            redrawLine();
        }
    });

})();
"#;

/// 内嵌的样式 CSS
const STYLE_CSS: &str = r#"
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

html, body {
    width: 100%;
    height: 100%;
    overflow: hidden;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    display: flex;
    flex-direction: column;
    padding: 10px;
    margin: 0;
}

#header {
    text-align: center;
    color: white;
    margin-bottom: 10px;
    padding: 8px 0;
    flex-shrink: 0;
}

#header h1 {
    font-size: 1.5em;
    margin: 0 0 5px 0;
    text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
}

#header p {
    font-size: 0.9em;
    opacity: 0.9;
    margin: 0;
}

#terminal-container {
    flex: 1;
    background: #1e1e1e;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
    overflow: hidden;
    padding: 8px;
    max-width: 1400px;
    width: 100%;
    margin: 0 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
}

#terminal {
    width: 100%;
    height: 100%;
    overflow: hidden;
}

/* xterm.js 内部容器 */
.xterm {
    height: 100%;
}

.xterm-viewport {
    overflow-y: auto !important;
}

#status {
    text-align: center;
    margin-top: 8px;
    color: white;
    font-size: 0.85em;
    flex-shrink: 0;
}

#connection-status {
    padding: 5px 15px;
    background: rgba(255,255,255,0.2);
    border-radius: 20px;
    display: inline-block;
}

@media (max-width: 768px) {
    body {
        padding: 10px;
    }

    #header h1 {
        font-size: 1.5em;
    }

    #header p {
        font-size: 0.9em;
    }
}
"#;
