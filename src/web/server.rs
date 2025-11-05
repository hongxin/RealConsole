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
<html lang="zh-CN" id="html-root">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title data-i18n="web.page.title">RealConsole Web 终端</title>
    <link rel="stylesheet" href="/static/style.css">
    <!-- Markdown 渲染支持 (v1.26.0) -->
    <script src="https://cdn.jsdelivr.net/npm/marked@11.1.1/marked.min.js"></script>
</head>
<body>
    <div id="header">
        <div id="header-content">
            <h1 data-i18n="web.header.title">🌟 RealConsole Web 终端</h1>
            <p data-i18n="web.header.tagline">融合东方哲学智慧的智能 CLI Agent</p>
        </div>
        <div id="lang-switcher">
            <button onclick="setLanguage('zh-CN')" id="btn-zh" class="active">中文</button>
            <button onclick="setLanguage('en-US')" id="btn-en">English</button>
        </div>
    </div>
    <div id="terminal-container">
        <!-- 混合终端：单一容器，统一滚动 -->
    </div>
    <div id="status">
        <span id="connection-status" data-i18n="web.status.connecting">连接中...</span>
    </div>
    <script src="/static/terminal.js"></script>
</body>
</html>
"#;

/// 内嵌的终端 JavaScript
const TERMINAL_JS: &str = r#"
// RealConsole Web Hybrid Terminal (v1.26.0)
// 融合终端 + Markdown 的统一体验
(function() {
    'use strict';

    // ========== ANSI 颜色解析器 ==========
    class AnsiParser {
        constructor() {
            this.ansiMap = {
                '0': 'reset',
                '1': 'bold',
                '31': 'red',
                '32': 'green',
                '33': 'yellow',
                '34': 'blue',
                '36': 'cyan',
                '37': 'white',
            };
        }

        parse(text) {
            // 解析 ANSI 转义序列为 HTML
            const regex = /\x1b\[([0-9;]+)m/g;
            let html = '';
            let lastIndex = 0;
            let currentClasses = [];

            text.replace(regex, (match, codes, offset) => {
                // 添加前面的文本
                if (offset > lastIndex) {
                    const content = text.slice(lastIndex, offset);
                    html += this.wrapWithClasses(content, currentClasses);
                }

                // 处理 ANSI 代码
                const codeList = codes.split(';');
                for (const code of codeList) {
                    if (code === '0') {
                        currentClasses = [];
                    } else if (this.ansiMap[code]) {
                        currentClasses.push(`ansi-${this.ansiMap[code]}`);
                    }
                }

                lastIndex = offset + match.length;
                return '';
            });

            // 添加剩余文本
            if (lastIndex < text.length) {
                html += this.wrapWithClasses(text.slice(lastIndex), currentClasses);
            }

            return html || this.escapeHtml(text);
        }

        wrapWithClasses(text, classes) {
            const escaped = this.escapeHtml(text);
            if (classes.length > 0) {
                return `<span class="${classes.join(' ')}">${escaped}</span>`;
            }
            return escaped;
        }

        escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
    }

    // ========== Markdown 渲染器 ==========
    class MarkdownRenderer {
        constructor(enabled = true) {
            this.enabled = enabled;
            this.streamBuffer = '';
            this.renderTimeout = null;
            this.RENDER_INTERVAL = 300; // 300ms 渲染一次（流式输出时）
            this.overlayElement = document.getElementById('markdown-overlay');
            this.configureMarked();
        }

        configureMarked() {
            if (typeof marked !== 'undefined') {
                marked.setOptions({
                    breaks: true,        // 支持 GFM 换行
                    gfm: true,          // GitHub Flavored Markdown
                    headerIds: false,   // 不生成 header ID
                    mangle: false       // 不混淆邮箱
                });
            }
        }

        // 检测文本是否包含 Markdown 标记
        isMarkdown(text) {
            if (!text || typeof text !== 'string') return false;

            // 启发式检测常见 Markdown 模式
            const patterns = [
                /^#{1,6}\s+/m,          // 标题 # ## ###
                /\*\*[^*]+\*\*/,        // 粗体 **text**
                /\*[^*]+\*/,            // 斜体 *text*
                /`[^`]+`/,              // 内联代码 `code`
                /^```/m,                // 代码块 ```
                /^\s*[-*+]\s+/m,        // 无序列表
                /^\s*\d+\.\s+/m,        // 有序列表
                /\[.+\]\(.+\)/,         // 链接 [text](url)
                /^>\s+/m                // 引用块 >
            ];

            return patterns.some(pattern => pattern.test(text));
        }

        // 渲染 Markdown 文本
        render(text, isStream = false) {
            if (!this.enabled || typeof marked === 'undefined' || !this.isMarkdown(text)) {
                return null; // 返回 null 表示不渲染为 Markdown
            }

            try {
                const html = marked.parse(text);
                return `<div class="markdown-content">${html}</div>`;
            } catch (error) {
                console.error('Markdown render error:', error);
                return null;
            }
        }

        // 显示 Markdown 内容（覆盖层模式）
        show(html) {
            if (!this.overlayElement || !html) return;

            this.overlayElement.innerHTML = html;
            this.overlayElement.classList.add('active');

            // 滚动到底部
            setTimeout(() => {
                this.overlayElement.scrollTop = this.overlayElement.scrollHeight;
            }, 50);
        }

        // 隐藏 Markdown 覆盖层
        hide() {
            if (!this.overlayElement) return;

            this.overlayElement.classList.remove('active');
            this.overlayElement.innerHTML = '';
        }

        // 处理流式输出
        handleStream(chunk) {
            if (!this.enabled) return false;

            this.streamBuffer += chunk;

            // 防抖渲染
            if (this.renderTimeout) {
                clearTimeout(this.renderTimeout);
            }

            this.renderTimeout = setTimeout(() => {
                const html = this.render(this.streamBuffer, true);
                if (html) {
                    this.show(html);
                }
            }, this.RENDER_INTERVAL);

            return this.isMarkdown(this.streamBuffer);
        }

        // 完成流式输出
        finishStream() {
            if (this.renderTimeout) {
                clearTimeout(this.renderTimeout);
                this.renderTimeout = null;
            }

            if (this.streamBuffer) {
                const html = this.render(this.streamBuffer, false);
                if (html) {
                    this.show(html);
                }
            }

            this.streamBuffer = '';
        }

        // 重置状态
        reset() {
            this.streamBuffer = '';
            if (this.renderTimeout) {
                clearTimeout(this.renderTimeout);
                this.renderTimeout = null;
            }
            this.hide();
        }

        // 设置启用状态
        setEnabled(enabled) {
            this.enabled = enabled;
            if (!enabled) {
                this.reset();
            }
        }
    }

    // ========== 混合终端核心 ==========
    class HybridTerminal {
        constructor(container) {
            this.container = container;
            this.lines = [];
            this.currentInput = null;
            this.history = [];
            this.historyIndex = -1;
            this.tempInput = '';

            this.ansiParser = new AnsiParser();
            this.markdownRenderer = new MarkdownRenderer(true);

            this.spinnerLine = null;
            this.streamBuffer = '';
            this.isStreaming = false;

            this.init();
        }

        init() {
            this.container.innerHTML = '';
            this.container.className = 'hybrid-terminal';

            // 创建输出区
            this.outputArea = document.createElement('div');
            this.outputArea.className = 'terminal-output-area';
            this.container.appendChild(this.outputArea);

            // 创建输入行
            this.createInputLine();

            // 点击容器时聚焦输入
            this.container.addEventListener('click', (e) => {
                if (e.target === this.container || e.target === this.outputArea) {
                    this.focusInput();
                }
            });
        }

        createInputLine() {
            const line = document.createElement('div');
            line.className = 'terminal-input-field';

            const prompt = document.createElement('span');
            prompt.className = 'prompt';
            prompt.textContent = '% ';

            const input = document.createElement('input');
            input.type = 'text';
            input.autocomplete = 'off';
            input.spellcheck = false;

            line.appendChild(prompt);
            line.appendChild(input);

            this.currentInput = { line, input };
            this.container.appendChild(line);

            this.setupInputHandlers();
            this.focusInput();
        }

        setupInputHandlers() {
            const input = this.currentInput.input;

            input.addEventListener('keydown', (e) => {
                switch (e.key) {
                    case 'Enter':
                        this.handleSubmit();
                        break;
                    case 'ArrowUp':
                        this.historyPrev();
                        e.preventDefault();
                        break;
                    case 'ArrowDown':
                        this.historyNext();
                        e.preventDefault();
                        break;
                    case 'l':
                        if (e.ctrlKey) {
                            this.clear();
                            e.preventDefault();
                        }
                        break;
                    case 'c':
                        if (e.ctrlKey) {
                            this.handleInterrupt();
                            e.preventDefault();
                        }
                        break;
                }
            });
        }

        handleSubmit() {
            const command = this.currentInput.input.value.trim();
            if (!command) return;

            // 添加到历史
            if (command && (this.history.length === 0 || this.history[this.history.length - 1] !== command)) {
                this.history.push(command);
                if (this.history.length > 1000) {
                    this.history.shift();
                }
            }
            this.historyIndex = this.history.length;
            this.tempInput = '';

            // 显示命令
            this.writeCommand(command);

            // 清空输入
            this.currentInput.input.value = '';

            // 发送命令
            if (this.onCommand) {
                this.onCommand(command);
            }
        }

        handleInterrupt() {
            this.writeOutput('\x1b[36m^C\x1b[0m');
            this.currentInput.input.value = '';
            if (this.onInterrupt) {
                this.onInterrupt();
            }
        }

        historyPrev() {
            if (this.history.length === 0) return;

            if (this.historyIndex === this.history.length) {
                this.tempInput = this.currentInput.input.value;
            }

            if (this.historyIndex > 0) {
                this.historyIndex--;
                this.currentInput.input.value = this.history[this.historyIndex];
            }
        }

        historyNext() {
            if (this.historyIndex === this.history.length) return;

            this.historyIndex++;
            if (this.historyIndex >= this.history.length) {
                this.historyIndex = this.history.length;
                this.currentInput.input.value = this.tempInput;
            } else {
                this.currentInput.input.value = this.history[this.historyIndex];
            }
        }

        focusInput() {
            this.currentInput.input.focus();
        }

        // ========== 输出方法 ==========

        writeCommand(command) {
            const line = document.createElement('div');
            line.className = 'terminal-line line-command';
            line.innerHTML = `<span class="prompt">% </span><span class="command">${this.escapeHtml(command)}</span>`;
            this.appendToOutput(line);
        }

        writeOutput(content) {
            // 自动检测 Markdown
            if (this.markdownRenderer.isMarkdown(content)) {
                this.writeMarkdown(content);
            } else {
                this.writePlainText(content);
            }
        }

        writePlainText(content) {
            const line = document.createElement('div');
            line.className = 'terminal-line line-output';

            // 解析 ANSI 颜色
            const html = this.ansiParser.parse(content);

            const pre = document.createElement('pre');
            pre.className = 'terminal-text';
            pre.innerHTML = html;

            line.appendChild(pre);
            this.appendToOutput(line);
        }

        writeMarkdown(content) {
            const line = document.createElement('div');
            line.className = 'terminal-line line-markdown';

            try {
                const html = marked.parse(content);
                line.innerHTML = html;
            } catch (error) {
                console.error('Markdown parse error:', error);
                this.writePlainText(content);
                return;
            }

            this.appendToOutput(line);
        }

        writeSpinner(modelName = '') {
            this.removeSpinner();

            const line = document.createElement('div');
            line.className = 'terminal-line line-spinner';

            const icon = document.createElement('span');
            icon.className = 'spinner-icon';
            icon.textContent = '⠋';

            const text = document.createElement('span');
            text.className = 'spinner-text';
            text.textContent = modelName || '思考中...';

            line.appendChild(icon);
            line.appendChild(text);

            this.spinnerLine = line;
            this.appendToOutput(line);

            this.startSpinnerAnimation();
        }

        removeSpinner() {
            if (this.spinnerLine) {
                this.spinnerLine.remove();
                this.spinnerLine = null;
            }
            this.stopSpinnerAnimation();
        }

        startSpinnerAnimation() {
            const frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let frame = 0;

            this.spinnerInterval = setInterval(() => {
                if (this.spinnerLine) {
                    const icon = this.spinnerLine.querySelector('.spinner-icon');
                    if (icon) {
                        icon.textContent = frames[frame];
                        frame = (frame + 1) % frames.length;
                    }
                }
            }, 80);
        }

        stopSpinnerAnimation() {
            if (this.spinnerInterval) {
                clearInterval(this.spinnerInterval);
                this.spinnerInterval = null;
            }
        }

        // ========== 流式输出 ==========

        startStream() {
            this.isStreaming = true;
            this.streamBuffer = '';
            this.removeSpinner();
        }

        writeStream(chunk) {
            this.streamBuffer += chunk;
        }

        finishStream() {
            if (this.streamBuffer) {
                this.writeOutput(this.streamBuffer);
            }
            this.streamBuffer = '';
            this.isStreaming = false;
        }

        // ========== 辅助方法 ==========

        appendToOutput(element) {
            this.outputArea.appendChild(element);
            this.lines.push(element);
            this.scrollToBottom();
        }

        scrollToBottom() {
            requestAnimationFrame(() => {
                this.container.scrollTop = this.container.scrollHeight;
            });
        }

        clear() {
            this.outputArea.innerHTML = '';
            this.lines = [];
            this.removeSpinner();
            this.streamBuffer = '';
            this.isStreaming = false;
            this.focusInput();
        }

        escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
    }

    // ========== i18n 国际化支持 ==========
    const I18N_TRANSLATIONS = {
        'zh-CN': {
            'web.page.title': 'RealConsole Web 终端',
            'web.header.title': '🌟 RealConsole Web 终端',
            'web.header.tagline': '融合东方哲学智慧的智能 CLI Agent',
            'web.status.connecting': '连接中...',
            'web.status.connected': '已连接',
            'web.status.disconnected': '已断开',
            'web.status.error': '连接错误',
            'web.terminal.welcome': '欢迎使用 RealConsole Web 终端！',
            'web.terminal.usage_hint': '输入命令开始使用，输入 /help 查看帮助',
            'web.terminal.disconnected_message': '连接已断开',
        },
        'en-US': {
            'web.page.title': 'RealConsole Web Terminal',
            'web.header.title': '🌟 RealConsole Web Terminal',
            'web.header.tagline': 'Intelligent CLI Agent Blending Eastern Philosophy Wisdom',
            'web.status.connecting': 'Connecting...',
            'web.status.connected': 'Connected',
            'web.status.disconnected': 'Disconnected',
            'web.status.error': 'Connection error',
            'web.terminal.welcome': 'Welcome to RealConsole Web Terminal!',
            'web.terminal.usage_hint': 'Enter commands to start, type /help for help',
            'web.terminal.disconnected_message': 'Connection closed',
        }
    };

    // 当前语言
    let currentLanguage = 'zh-CN';

    // 获取浏览器语言
    function getBrowserLanguage() {
        const lang = navigator.language || navigator.userLanguage;
        if (lang.startsWith('zh')) {
            return 'zh-CN';
        } else if (lang.startsWith('en')) {
            return 'en-US';
        }
        return 'zh-CN'; // 默认中文
    }

    // 翻译函数
    function t(key) {
        const translations = I18N_TRANSLATIONS[currentLanguage] || I18N_TRANSLATIONS['zh-CN'];
        return translations[key] || key;
    }

    // 显示欢迎消息
    function showWelcomeMessage() {
        term.clear();
        term.writeln('\x1b[32m' + t('web.terminal.welcome') + '\x1b[0m');
        term.writeln('\x1b[36m' + t('web.terminal.usage_hint') + '\x1b[0m');
        term.write('\x1b[33m% \x1b[0m');
    }

    // 设置语言
    function setLanguage(lang) {
        if (!I18N_TRANSLATIONS[lang]) return;
        currentLanguage = lang;
        updatePageText();
        // 更新按钮状态
        document.getElementById('btn-zh').classList.toggle('active', lang === 'zh-CN');
        document.getElementById('btn-en').classList.toggle('active', lang === 'en-US');
        // 刷新终端欢迎消息
        if (ws && ws.readyState === WebSocket.OPEN) {
            showWelcomeMessage();
        }
    }

    // 更新页面文本
    function updatePageText() {
        // 更新所有 data-i18n 元素
        document.querySelectorAll('[data-i18n]').forEach(el => {
            const key = el.getAttribute('data-i18n');
            el.textContent = t(key);
        });

        // 更新 title
        const title = document.querySelector('title');
        if (title) {
            const key = title.getAttribute('data-i18n');
            if (key) title.textContent = t(key);
        }
    }

    // 暴露到全局作用域，供 HTML 按钮调用
    window.setLanguage = setLanguage;

    // 初始化语言（从浏览器检测）
    currentLanguage = getBrowserLanguage();

    // ========== 终端核心 ==========

    // 创建混合终端
    const terminal = new HybridTerminal(document.getElementById('terminal-container'));

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

    // 辅助函数：计算字符串的显示宽度（考虑中文字符）
    function getDisplayWidth(str) {
        let width = 0;
        for (const char of str) {
            const code = char.charCodeAt(0);
            // 判断是否为宽字符（中文、日文、韩文等）
            // 基本规则：Unicode >= 0x1100 的大部分字符为宽字符
            if (code >= 0x1100 && (
                (code <= 0x115f) ||  // Hangul Jamo
                (code >= 0x2e80 && code <= 0x9fff) ||  // CJK
                (code >= 0xac00 && code <= 0xd7a3) ||  // Hangul Syllables
                (code >= 0xf900 && code <= 0xfaff) ||  // CJK Compatibility
                (code >= 0xfe10 && code <= 0xfe19) ||  // Vertical forms
                (code >= 0xfe30 && code <= 0xfe6f) ||  // CJK Compatibility Forms
                (code >= 0xff00 && code <= 0xff60) ||  // Fullwidth Forms
                (code >= 0xffe0 && code <= 0xffe6) ||  // Fullwidth Forms
                (code >= 0x20000 && code <= 0x2fffd) ||  // CJK Extension
                (code >= 0x30000 && code <= 0x3fffd)     // CJK Extension
            )) {
                width += 2;
            } else {
                width += 1;
            }
        }
        return width;
    }

    // 辅助函数：获取从字符串开始到某个字符位置的显示宽度
    function getWidthUpToPosition(str, pos) {
        return getDisplayWidth(str.slice(0, pos));
    }

    // 辅助函数：重新渲染当前输入行
    function redrawLine() {
        // 清除当前行
        term.write('\r\x1b[K');
        // 显示提示符
        term.write('\x1b[33m% \x1b[0m');
        // 显示输入内容
        term.write(inputBuffer);
        // 移动光标到正确位置（基于显示宽度，不是字符数）
        if (cursorPosition < inputBuffer.length) {
            const widthAfterCursor = getDisplayWidth(inputBuffer.slice(cursorPosition));
            if (widthAfterCursor > 0) {
                term.write('\x1b[' + widthAfterCursor + 'D');
            }
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
        statusEl.textContent = t('web.status.connected');
        statusEl.style.color = '#4CAF50';

        // 显示欢迎消息
        terminal.writePlainText('\x1b[32m' + t('web.terminal.welcome') + '\x1b[0m\n' +
                                '\x1b[36m' + t('web.terminal.usage_hint') + '\x1b[0m');

        // 应用初始语言设置
        updatePageText();
    };

    ws.onclose = () => {
        statusEl.textContent = t('web.status.disconnected');
        statusEl.style.color = '#f44336';
        terminal.writePlainText('\x1b[31m' + t('web.terminal.disconnected_message') + '\x1b[0m');
    };

    ws.onerror = (err) => {
        statusEl.textContent = t('web.status.error');
        statusEl.style.color = '#f44336';
        console.error('WebSocket error:', err);
    };

    ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);

        switch (msg.type) {
            case 'thinking':
                // 显示思考状态
                const modelName = msg.model || '思考中...';
                terminal.writeSpinner(modelName);
                break;

            case 'output':
                // 完整输出（自动检测 Markdown）
                terminal.removeSpinner();
                terminal.writeOutput(msg.content);
                break;

            case 'stream':
                // 流式输出
                if (!terminal.isStreaming) {
                    terminal.startStream();
                }
                terminal.writeStream(msg.content);
                break;

            case 'stream_end':
                // 流式输出结束
                terminal.finishStream();
                break;

            case 'error':
                // 错误输出（红色）
                terminal.removeSpinner();
                terminal.writePlainText('\x1b[31m' + msg.content + '\x1b[0m');
                break;

            case 'clear':
                // 清屏
                terminal.clear();
                break;
        }
    };

    // 设置命令处理回调
    terminal.onCommand = (command) => {
        ws.send(JSON.stringify({
            type: 'input',
            content: command
        }));
    };

    // 设置中断处理回调
    terminal.onInterrupt = () => {
        ws.send(JSON.stringify({
            type: 'interrupt',
            content: ''
        }));
    };

    // ========== 初始化 i18n ==========
    // 页面加载完成后立即应用语言设置
    window.addEventListener('DOMContentLoaded', () => {
        updatePageText();
        // 根据初始语言设置按钮状态
        document.getElementById('btn-zh').classList.toggle('active', currentLanguage === 'zh-CN');
        document.getElementById('btn-en').classList.toggle('active', currentLanguage === 'en-US');
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
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: white;
    margin-bottom: 10px;
    padding: 8px 20px;
    flex-shrink: 0;
}

#header-content {
    text-align: center;
    flex: 1;
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

#lang-switcher {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
}

#lang-switcher button {
    padding: 6px 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.1);
    color: white;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.3s ease;
    backdrop-filter: blur(10px);
}

#lang-switcher button:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.5);
    transform: translateY(-2px);
}

#lang-switcher button.active {
    background: rgba(255, 255, 255, 0.3);
    border-color: rgba(255, 255, 255, 0.8);
    font-weight: 600;
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

/* ============================================
   混合终端样式 (v1.26.0)
   统一的终端 + Markdown 融合体验
   ============================================ */

.hybrid-terminal {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 14px;
    color: rgb(240, 240, 240);
}

.terminal-output-area {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    line-height: 1.5;
}

.terminal-line {
    margin: 4px 0;
    word-wrap: break-word;
    white-space: pre-wrap;
}

/* 输出行 */
.line-output {
    color: rgb(240, 240, 240);
}

.line-output .terminal-text {
    margin: 0;
    font-family: inherit;
    white-space: pre-wrap;
    word-wrap: break-word;
}

/* 命令回显行 */
.line-command {
    color: rgb(180, 180, 180);
}

.line-command .prompt {
    color: rgb(100, 255, 100);
    font-weight: bold;
}

.line-command .command {
    color: rgb(100, 180, 255);
    font-weight: 600;
}

/* Markdown 行 - 融入终端的 Markdown 格式化 */
.line-markdown {
    padding: 8px 0 8px 12px;
    border-left: 3px solid rgb(100, 180, 255);
    background: rgba(100, 180, 255, 0.03);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.6;
    white-space: normal;
}

/* Spinner 行 */
.line-spinner {
    color: rgb(100, 255, 100);
    font-style: italic;
}

.spinner {
    display: inline-block;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    0% { content: '⠋'; }
    12.5% { content: '⠙'; }
    25% { content: '⠹'; }
    37.5% { content: '⠸'; }
    50% { content: '⠼'; }
    62.5% { content: '⠴'; }
    75% { content: '⠦'; }
    87.5% { content: '⠧'; }
    100% { content: '⠇'; }
}

/* 输入字段 */
.terminal-input-field {
    display: flex;
    align-items: center;
    padding: 8px 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(0, 0, 0, 0.2);
}

.terminal-input-field .prompt {
    color: rgb(100, 255, 100);
    font-weight: bold;
    margin-right: 8px;
    flex-shrink: 0;
}

.terminal-input-field input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: rgb(255, 255, 255);
    font-family: inherit;
    font-size: inherit;
}

/* ANSI 颜色类 */
.ansi-reset {
    color: rgb(240, 240, 240);
    font-weight: normal;
}

.ansi-bold {
    font-weight: bold;
}

.ansi-red {
    color: rgb(255, 100, 100);
}

.ansi-green {
    color: rgb(100, 255, 100);
}

.ansi-yellow {
    color: rgb(255, 255, 100);
}

.ansi-blue {
    color: rgb(100, 180, 255);
}

.ansi-cyan {
    color: rgb(100, 255, 255);
}

.ansi-white {
    color: rgb(255, 255, 255);
}

/* 滚动条样式 */
.terminal-output-area::-webkit-scrollbar {
    width: 8px;
}

.terminal-output-area::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 4px;
}

.terminal-output-area::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
}

.terminal-output-area::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
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

    #header {
        flex-direction: column;
        gap: 10px;
        padding: 8px 10px;
    }

    #header-content h1 {
        font-size: 1.3em;
    }

    #header-content p {
        font-size: 0.85em;
    }

    #lang-switcher {
        width: 100%;
        justify-content: center;
    }

    #lang-switcher button {
        flex: 1;
        max-width: 120px;
    }
}

/* ============================================
   Markdown 内容样式 (v1.26.0)
   Claude Code 风格 - 融入终端体验
   ============================================ */

.line-markdown h1,
.line-markdown h2,
.line-markdown h3,
.line-markdown h4,
.line-markdown h5,
.line-markdown h6 {
    color: rgb(100, 180, 255);
    font-weight: 600;
    margin: 0.8em 0 0.4em 0;
}

.line-markdown h1 { font-size: 1.8em; }
.line-markdown h2 { font-size: 1.5em; }
.line-markdown h3 { font-size: 1.3em; }
.line-markdown h4 { font-size: 1.1em; }
.line-markdown h5 { font-size: 1.0em; }
.line-markdown h6 { font-size: 0.9em; }

/* 粗体 - 明亮白色 */
.line-markdown strong {
    color: rgb(255, 255, 255);
    font-weight: 700;
}

/* 斜体 - 浅灰色 */
.line-markdown em {
    color: rgb(180, 180, 180);
    font-style: italic;
}

/* 内联代码 - 浅蓝色 */
.line-markdown code {
    color: rgb(130, 200, 255);
    background-color: rgba(40, 40, 40, 0.6);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.9em;
}

/* 代码块 - 柔和绿色 + 深灰背景 */
.line-markdown pre {
    background-color: rgb(40, 40, 40);
    padding: 1em;
    border-radius: 5px;
    overflow-x: auto;
    margin: 0.5em 0;
}

.line-markdown pre code {
    color: rgb(150, 220, 150);
    background: none;
    padding: 0;
    font-size: 0.95em;
}

/* 段落 */
.line-markdown p {
    margin: 0.5em 0;
    color: rgb(240, 240, 240);
}

/* 列表 - 柔和蓝色 bullet */
.line-markdown ul,
.line-markdown ol {
    margin: 0.5em 0;
    padding-left: 1.5em;
}

.line-markdown ul li::marker {
    color: rgb(100, 180, 255);
}

.line-markdown ol li::marker {
    color: rgb(100, 180, 255);
    font-weight: 600;
}

.line-markdown li {
    margin: 0.3em 0;
}

/* 引用块 - 中等灰色 */
.line-markdown blockquote {
    border-left: 3px solid rgb(120, 120, 120);
    padding-left: 1em;
    color: rgb(180, 180, 180);
    margin: 0.5em 0;
    font-style: italic;
}

/* 链接 - 与标题一致的蓝色 */
.line-markdown a {
    color: rgb(100, 180, 255);
    text-decoration: underline;
}

.line-markdown a:hover {
    color: rgb(130, 200, 255);
}

/* 分隔线 */
.line-markdown hr {
    border: none;
    border-top: 1px solid rgb(80, 80, 80);
    margin: 1em 0;
}

/* 表格 */
.line-markdown table {
    border-collapse: collapse;
    width: 100%;
    margin: 0.5em 0;
}

.line-markdown th,
.line-markdown td {
    border: 1px solid rgb(80, 80, 80);
    padding: 0.4em 0.8em;
    text-align: left;
}

.line-markdown th {
    background-color: rgba(100, 180, 255, 0.2);
    color: rgb(100, 180, 255);
    font-weight: 600;
}

/* 图片 */
.line-markdown img {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
    margin: 0.5em 0;
}
"#;
