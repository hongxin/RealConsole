//! Web 前端资源模块
//!
//! 本模块包含 Web 终端的所有前端资源：
//! - HTML 页面模板
//! - CSS 样式表
//! - JavaScript 代码
//!
//! # 设计理念
//!
//! 将前端代码从 server.rs 中解耦，便于：
//! 1. 前端代码的独立维护和更新
//! 2. 降低 server.rs 的复杂度
//! 3. 为未来的前端增强打好基础
//!
//! # 版本历史
//!
//! - v1.28.3: 初始创建，从 server.rs 分离
//!
//! # 未来规划
//!
//! - v1.29.0+: 考虑使用外部文件或模板引擎
//! - v1.30.0+: 支持自定义主题和样式

/// 获取主页 HTML
pub fn get_index_html() -> &'static str {
    INDEX_HTML
}

/// 获取终端 JavaScript
pub fn get_terminal_js() -> &'static str {
    TERMINAL_JS
}

/// 获取样式表 CSS
pub fn get_style_css() -> &'static str {
    STYLE_CSS
}

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
        <div id="header-controls">
            <div id="lang-switcher">
                <button onclick="setLanguage('zh-CN')" id="btn-zh" class="active">中文</button>
                <button onclick="setLanguage('en-US')" id="btn-en">English</button>
            </div>
            <button id="view-mode-toggle" class="view-mode-btn" title="切换到传统流式输出">📊 回合模式</button>
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

    // ========== 常量定义 ==========

    // 回合类型常量（与后端 RoundType 枚举对应）
    const RoundType = {
        LLM: 'llm',
        SHELL: 'shell',
        SYSTEM: 'system'
    };

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
            this.isComposing = false;  // 输入法组合状态

            // ===== v1.28.0: 对话回合管理 =====
            this.rounds = [];           // 回合列表
            this.currentRound = null;   // 当前执行的回合
            this.viewMode = 'round';    // 视图模式: 'round' (回合卡片) 或 'stream' (流式输出)

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

            // 监听输入法组合状态（中文、日文等输入法）
            input.addEventListener('compositionstart', () => {
                this.isComposing = true;
            });

            input.addEventListener('compositionend', () => {
                this.isComposing = false;
            });

            input.addEventListener('keydown', (e) => {
                switch (e.key) {
                    case 'Enter':
                        // 如果正在使用输入法组合，不提交命令
                        if (!this.isComposing) {
                            this.handleSubmit();
                            e.preventDefault();
                        }
                        break;
                    case 'ArrowUp':
                        // 输入法组合时，方向键由输入法处理
                        if (!this.isComposing) {
                            this.historyPrev();
                            e.preventDefault();
                        }
                        break;
                    case 'ArrowDown':
                        if (!this.isComposing) {
                            this.historyNext();
                            e.preventDefault();
                        }
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

            // 用户提交新命令时，强制滚动到底部
            this.forceScrollToBottom();

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
            // 回合模式下跳过命令回显（已在回合卡片中显示）
            if (this.viewMode === 'round') return;

            const line = document.createElement('div');
            line.className = 'terminal-line line-command';
            line.innerHTML = `<span class="prompt">% </span><span class="command">${this.escapeHtml(command)}</span>`;
            this.appendToOutput(line);
        }

        writeOutput(content) {
            // 回合模式下跳过输出（已在回合卡片中显示）
            if (this.viewMode === 'round') return;

            // 自动检测 Markdown
            if (this.markdownRenderer.isMarkdown(content)) {
                this.writeMarkdown(content);
            } else {
                this.writePlainText(content);
            }
        }

        writePlainText(content, isSystemMessage = false) {
            // 回合模式下跳过（除非是系统消息）
            if (this.viewMode === 'round' && !isSystemMessage) return;

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
            // 回合模式下跳过
            if (this.viewMode === 'round') return;

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
            // 回合模式下在当前回合卡片中显示 Spinner
            if (this.viewMode === 'round') {
                if (this.currentRound && this.currentRound.element) {
                    const statusSpan = this.currentRound.element.querySelector('.round-status');
                    if (statusSpan) {
                        // 添加飞轮动画类
                        statusSpan.classList.add('spinner-active');
                        // 显示模型名称（如果有）
                        const timeSpan = this.currentRound.element.querySelector('.round-time');
                        if (timeSpan && modelName) {
                            timeSpan.textContent = `${modelName} ...`;
                        }
                    }
                }
                return;
            }

            this.removeSpinner();

            const line = document.createElement('div');
            line.className = 'terminal-line line-spinner';

            const icon = document.createElement('span');
            icon.className = 'spinner-icon';
            icon.textContent = '⠋';

            const text = document.createElement('span');
            text.className = 'spinner-text';
            text.textContent = modelName || '...';

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
                // 流式输出完成时，确保滚动到底部
                this.forceScrollToBottom();
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
            // 智能自动滚动：只在用户位于底部附近时滚动
            requestAnimationFrame(() => {
                const { scrollTop, scrollHeight, clientHeight } = this.outputArea;
                const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

                // 如果用户在底部 100px 范围内，自动滚动到底部
                // 如果用户向上滚动查看历史，不打断
                if (distanceFromBottom < 100) {
                    this.outputArea.scrollTop = this.outputArea.scrollHeight;
                }
            });
        }

        // 强制滚动到底部（用于用户提交命令等场景）
        forceScrollToBottom() {
            requestAnimationFrame(() => {
                this.outputArea.scrollTop = this.outputArea.scrollHeight;
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

        // ===== v1.28.0: 对话回合管理方法 =====

        createRound(round) {
            const roundData = {
                id: round.id,
                index: round.index,
                roundType: round.round_type || RoundType.LLM, // 默认为 LLM 类型
                userInput: round.user_input,
                aiResponse: round.ai_response || '',
                toolsUsed: round.tools_used || [],
                executionTime: round.execution_time || 0,
                status: this.normalizeStatus(round.status),
                timestamp: round.timestamp,
                model: round.model,
                element: null,
                expanded: true
            };

            // 创建回合 DOM 元素
            roundData.element = this.createRoundElement(roundData);
            this.outputArea.appendChild(roundData.element);

            this.rounds.push(roundData);
            this.currentRound = roundData;

            // ===== 关键修复：根据当前视图模式设置初始显示状态 =====
            if (this.viewMode === 'stream') {
                // 传统模式下，立即隐藏新创建的回合卡片
                roundData.element.style.display = 'none';
            }

            return roundData;
        }

        // 标准化状态值（处理 Rust enum 序列化格式）
        normalizeStatus(status) {
            // 如果是字符串，直接返回（pending, running, success）
            if (typeof status === 'string') {
                return status;
            }
            // 如果是对象，提取键名（error）
            if (typeof status === 'object' && status !== null) {
                return Object.keys(status)[0] || 'unknown';
            }
            return 'unknown';
        }

        createRoundElement(round) {
            const roundDiv = document.createElement('div');
            roundDiv.className = 'conversation-round expanded';
            roundDiv.dataset.roundId = round.id;

            // 根据类型确定标签和图标
            const typeConfig = this.getRoundTypeConfig(round.roundType);

            // 回合头部
            const header = document.createElement('div');
            header.className = 'round-header';

            // 对于 Shell/System 命令，不显示工具使用
            const toolsHtml = (round.roundType === 'llm')
                ? `<span class="round-tools">${this.renderTools(round.toolsUsed)}</span>`
                : '';

            header.innerHTML = `
                <div class="round-info">
                    <span class="round-number">${typeConfig.badge} #${round.index}</span>
                    <span class="round-status ${round.status}">${this.getStatusIcon(round.status)}</span>
                    <span class="round-time">${round.executionTime.toFixed(2)}s</span>
                    ${toolsHtml}
                    <span class="round-summary">${this.escapeHtml(round.userInput.substring(0, 50))}${round.userInput.length > 50 ? '...' : ''}</span>
                </div>
                <button class="round-toggle" data-action="collapse">▼</button>
            `;

            // 回合内容
            const content = document.createElement('div');
            content.className = 'round-content';

            // 用户输入
            const inputDiv = document.createElement('div');
            inputDiv.className = 'round-input';
            inputDiv.innerHTML = `
                <span class="round-input-label">${typeConfig.inputLabel}</span>
                <div class="round-input-content">${this.escapeHtml(round.userInput)}</div>
            `;

            // 输出
            const outputDiv = document.createElement('div');
            outputDiv.className = 'round-output';
            outputDiv.innerHTML = `
                <span class="round-output-label">📤 Output:</span>
                <div class="output-content"></div>
            `;

            content.appendChild(inputDiv);
            content.appendChild(outputDiv);

            roundDiv.appendChild(header);
            roundDiv.appendChild(content);

            // 折叠/展开事件
            const toggleBtn = header.querySelector('.round-toggle');
            toggleBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                this.toggleRound(round.id);
            });

            return roundDiv;
        }

        // 获取回合类型配置
        getRoundTypeConfig(roundType) {
            const configs = {
                [RoundType.LLM]: {
                    badge: 'Round',
                    inputLabel: '📥 Input:',
                    outputLabel: '📤 Output:'
                },
                [RoundType.SHELL]: {
                    badge: '💻 Shell',
                    inputLabel: '💻 Command:',
                    outputLabel: '📤 Output:'
                },
                [RoundType.SYSTEM]: {
                    badge: '⚙️ System',
                    inputLabel: '⚙️ Command:',
                    outputLabel: '📤 Output:'
                }
            };

            const config = configs[roundType];
            if (!config) {
                console.warn(`Unknown round type: ${roundType}, falling back to LLM`);
                return configs[RoundType.LLM];
            }
            return config;
        }

        updateRoundStatus(roundId, status) {
            const round = this.rounds.find(r => r.id === roundId);
            if (!round) return;

            const normalizedStatus = this.normalizeStatus(status);
            round.status = normalizedStatus;
            const statusSpan = round.element.querySelector('.round-status');
            statusSpan.textContent = this.getStatusIcon(normalizedStatus);
            statusSpan.className = `round-status ${normalizedStatus}`;
        }

        completeRound(roundData) {
            const round = this.rounds.find(r => r.id === roundData.id);
            if (!round) return;

            const normalizedStatus = this.normalizeStatus(roundData.status);
            round.aiResponse = roundData.ai_response || '';
            round.executionTime = roundData.execution_time || 0;
            round.toolsUsed = roundData.tools_used || [];
            round.status = normalizedStatus;

            // 更新 UI
            const statusSpan = round.element.querySelector('.round-status');
            statusSpan.textContent = this.getStatusIcon(normalizedStatus);
            statusSpan.className = `round-status ${normalizedStatus}`;  // 移除 spinner-active

            const timeSpan = round.element.querySelector('.round-time');
            timeSpan.textContent = `${roundData.execution_time.toFixed(2)}s`;

            const toolsSpan = round.element.querySelector('.round-tools');
            if (toolsSpan) {  // Shell/System 命令可能没有 toolsSpan
                toolsSpan.innerHTML = this.renderTools(roundData.tools_used);
            }

            // 渲染输出内容
            const outputContent = round.element.querySelector('.output-content');
            if (round.aiResponse) {
                // 根据回合类型选择渲染方式
                if (round.roundType === RoundType.LLM) {
                    // LLM 对话：使用 Markdown 渲染
                    outputContent.innerHTML = this.markdownRenderer.render(round.aiResponse);
                } else {
                    // Shell/System 命令：使用 <pre> 保留格式
                    const pre = document.createElement('pre');
                    pre.className = 'terminal-text';
                    pre.textContent = round.aiResponse;
                    outputContent.innerHTML = '';
                    outputContent.appendChild(pre);
                }
            }

            // 滚动到底部
            this.scrollToBottom();
        }

        toggleRound(roundId) {
            const round = this.rounds.find(r => r.id === roundId);
            if (!round) return;

            round.expanded = !round.expanded;
            round.element.classList.toggle('collapsed');
            round.element.classList.toggle('expanded');

            const toggleBtn = round.element.querySelector('.round-toggle');
            toggleBtn.textContent = round.expanded ? '▼' : '▶';
        }

        getStatusIcon(status) {
            const icons = {
                success: '✓',
                running: '⏳',
                error: '✗',
                pending: '⏸'
            };
            return icons[status] || '?';
        }

        renderTools(tools) {
            if (!tools || tools.length === 0) return '';
            return tools.map(tool =>
                `<span class="tool-badge">${this.escapeHtml(tool)}</span>`
            ).join('');
        }

        escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // ===== v1.28.0: 视图模式切换 =====

        toggleViewMode() {
            this.viewMode = this.viewMode === 'round' ? 'stream' : 'round';

            // 更新按钮文字和样式
            const button = document.getElementById('view-mode-toggle');
            if (button) {
                if (this.viewMode === 'round') {
                    button.textContent = '📊 回合模式';
                    button.title = '切换到传统流式输出';
                } else {
                    button.textContent = '📜 传统模式';
                    button.title = '切换到回合卡片视图';
                }
            }

            // 根据模式显示/隐藏内容
            this.applyViewMode();
        }

        applyViewMode() {
            if (this.viewMode === 'round') {
                // 回合模式：隐藏传统输出，显示回合卡片

                // 隐藏所有传统输出行
                this.lines.forEach(line => {
                    line.style.display = 'none';
                });

                // 显示所有回合卡片（使用已保存的元素引用）
                this.rounds.forEach(round => {
                    if (round.element) {
                        round.element.style.display = 'block';
                    }
                });
            } else {
                // 传统模式：显示传统输出，隐藏回合卡片

                // 隐藏所有回合卡片（使用已保存的元素引用）
                this.rounds.forEach(round => {
                    if (round.element) {
                        round.element.style.display = 'none';
                    }
                });

                // 显示所有传统输出行（只显示非回合卡片的行）
                this.lines.forEach(line => {
                    // 确保不是回合卡片元素
                    if (!line.classList.contains('conversation-round')) {
                        line.style.display = 'block';
                    }
                });
            }
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

    // ===== v1.28.0: 绑定视图模式切换按钮 =====
    const viewModeToggle = document.getElementById('view-mode-toggle');
    if (viewModeToggle) {
        viewModeToggle.addEventListener('click', () => {
            terminal.toggleViewMode();
        });
    }

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

        // 显示欢迎消息（系统消息，在回合模式下也显示）
        terminal.writePlainText('\x1b[32m' + t('web.terminal.welcome') + '\x1b[0m\n' +
                                '\x1b[36m' + t('web.terminal.usage_hint') + '\x1b[0m', true);

        // 应用初始语言设置
        updatePageText();
    };

    ws.onclose = () => {
        statusEl.textContent = t('web.status.disconnected');
        statusEl.style.color = '#f44336';
        terminal.writePlainText('\x1b[31m' + t('web.terminal.disconnected_message') + '\x1b[0m', true);
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

            // ===== v1.28.0: 对话回合消息 =====
            case 'round_start':
                // 回合开始：总是创建回合（维护数据）
                // 注意：不需要在这里显示命令，handleSubmit() 已经显示过了
                terminal.createRound(msg.round);
                break;

            case 'round_update':
                // 回合状态更新
                terminal.updateRoundStatus(msg.round_id, msg.status);
                break;

            case 'round_complete':
                // 回合完成：总是完成回合（维护数据），传统模式下额外显示输出
                terminal.completeRound(msg.round);
                if (terminal.viewMode === 'stream') {
                    // 传统模式：只有 Shell/System 命令才额外显示输出
                    // LLM 对话已经通过 stream 消息显示过了，不需要重复显示
                    if (msg.round.round_type !== RoundType.LLM && msg.round.ai_response) {
                        terminal.writeOutput(msg.round.ai_response);
                    }
                }
                break;

            case 'round_history':
                // 历史回合列表（重连时）
                msg.rounds.forEach(round => {
                    terminal.createRound(round);
                    terminal.completeRound(round);
                });
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
/* ============================================
   🌃 Cyberpunk Theme - RealConsole v1.26.0
   未来赛博朋克风格
   ============================================ */

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
    /* 赛博朋克深色背景 + 动态网格 */
    background:
        repeating-linear-gradient(
            0deg,
            rgba(0, 240, 255, 0.03) 0px,
            transparent 1px,
            transparent 40px,
            rgba(0, 240, 255, 0.03) 41px
        ),
        repeating-linear-gradient(
            90deg,
            rgba(0, 240, 255, 0.03) 0px,
            transparent 1px,
            transparent 40px,
            rgba(0, 240, 255, 0.03) 41px
        ),
        linear-gradient(135deg, #0a0e27 0%, #0d1117 50%, #1a0b2e 100%);
    background-attachment: fixed;
    display: flex;
    flex-direction: column;
    padding: 10px;
    margin: 0;
    position: relative;
}

/* 扫描线效果 */
body::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: repeating-linear-gradient(
        0deg,
        rgba(0, 0, 0, 0.15) 0px,
        transparent 1px,
        transparent 2px,
        rgba(0, 0, 0, 0.15) 3px
    );
    pointer-events: none;
    z-index: 9999;
    opacity: 0.3;
    animation: scanlines 8s linear infinite;
}

@keyframes scanlines {
    0% { transform: translateY(0); }
    100% { transform: translateY(10px); }
}

#header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
    padding: 8px 20px;
    flex-shrink: 0;
    background: rgba(10, 14, 39, 0.6);
    border: 1px solid rgba(0, 240, 255, 0.3);
    border-radius: 8px;
    backdrop-filter: blur(10px);
    box-shadow: 0 0 20px rgba(0, 240, 255, 0.2);
    /* 与终端容器等宽对齐 */
    max-width: 1400px;
    width: 100%;
    margin-left: auto;
    margin-right: auto;
}

#header-content {
    text-align: center;
    flex: 1;
}

#header h1 {
    font-size: 1.5em;
    margin: 0 0 5px 0;
    /* 霓虹发光效果 - 青色到粉色渐变 */
    background: linear-gradient(90deg, #00f0ff 0%, #ff006e 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    text-shadow:
        0 0 10px rgba(0, 240, 255, 0.5),
        0 0 20px rgba(0, 240, 255, 0.3),
        0 0 30px rgba(0, 240, 255, 0.2);
    animation: neon-pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes neon-pulse {
    0%, 100% {
        filter: brightness(1);
        text-shadow:
            0 0 10px rgba(0, 240, 255, 0.5),
            0 0 20px rgba(0, 240, 255, 0.3),
            0 0 30px rgba(0, 240, 255, 0.2);
    }
    25% {
        filter: brightness(1.05);
        text-shadow:
            0 0 12px rgba(0, 240, 255, 0.6),
            0 0 22px rgba(0, 240, 255, 0.4),
            0 0 32px rgba(0, 240, 255, 0.25);
    }
    50% {
        filter: brightness(1.15);
        text-shadow:
            0 0 15px rgba(0, 240, 255, 0.7),
            0 0 25px rgba(0, 240, 255, 0.5),
            0 0 35px rgba(0, 240, 255, 0.3);
    }
    75% {
        filter: brightness(1.05);
        text-shadow:
            0 0 12px rgba(0, 240, 255, 0.6),
            0 0 22px rgba(0, 240, 255, 0.4),
            0 0 32px rgba(0, 240, 255, 0.25);
    }
}

#header p {
    font-size: 0.9em;
    margin: 0;
    color: #00f0ff;
    text-shadow: 0 0 10px rgba(0, 240, 255, 0.4);
}

#lang-switcher {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
}

#lang-switcher button {
    padding: 6px 12px;
    /* 霓虹边框 */
    border: 2px solid rgba(0, 240, 255, 0.4);
    background: rgba(10, 14, 39, 0.5);
    color: #00f0ff;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.3s ease;
    backdrop-filter: blur(10px);
    box-shadow: 0 0 10px rgba(0, 240, 255, 0.2);
    text-shadow: 0 0 5px rgba(0, 240, 255, 0.3);
}

#lang-switcher button:hover {
    background: rgba(0, 240, 255, 0.1);
    border-color: rgba(0, 240, 255, 0.8);
    transform: translateY(-2px);
    box-shadow:
        0 0 15px rgba(0, 240, 255, 0.4),
        0 0 25px rgba(0, 240, 255, 0.2);
}

#lang-switcher button.active {
    background: rgba(0, 240, 255, 0.15);
    border-color: #00f0ff;
    font-weight: 600;
    box-shadow:
        0 0 20px rgba(0, 240, 255, 0.5),
        0 0 30px rgba(0, 240, 255, 0.3),
        inset 0 0 10px rgba(0, 240, 255, 0.2);
}

/* ===== v1.28.0: 视图模式切换按钮 ===== */

#header-controls {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
}

.view-mode-btn {
    padding: 6px 12px;
    border: 2px solid rgba(255, 0, 110, 0.4);
    background: rgba(10, 14, 39, 0.5);
    color: #ff006e;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.3s ease;
    backdrop-filter: blur(10px);
    box-shadow: 0 0 10px rgba(255, 0, 110, 0.2);
    text-shadow: 0 0 5px rgba(255, 0, 110, 0.3);
    white-space: nowrap;
}

.view-mode-btn:hover {
    background: rgba(255, 0, 110, 0.1);
    border-color: rgba(255, 0, 110, 0.8);
    transform: translateY(-2px);
    box-shadow:
        0 0 15px rgba(255, 0, 110, 0.4),
        0 0 25px rgba(255, 0, 110, 0.2);
}

#terminal-container {
    flex: 1;
    /* 深色背景 */
    background: rgba(5, 8, 20, 0.85);
    border-radius: 8px;
    /* 霓虹青色边框 + 发光 */
    border: 2px solid rgba(0, 240, 255, 0.5);
    box-shadow:
        0 0 20px rgba(0, 240, 255, 0.3),
        0 0 40px rgba(0, 240, 255, 0.2),
        inset 0 0 60px rgba(0, 240, 255, 0.05);
    overflow: hidden;
    padding: 8px;
    max-width: 1400px;
    width: 100%;
    margin: 0 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    backdrop-filter: blur(10px);
}

/* 终端容器脉动效果 - 已移除（极简主义） */
/* #terminal-container::before {
    content: '';
    position: absolute;
    top: -2px;
    left: -2px;
    right: -2px;
    bottom: -2px;
    background: linear-gradient(45deg,
        rgba(0, 240, 255, 0.3) 0%,
        rgba(255, 0, 110, 0.3) 50%,
        rgba(0, 240, 255, 0.3) 100%);
    border-radius: 8px;
    z-index: -1;
    opacity: 0;
    animation: border-glow 6s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes border-glow {
    0%, 100% {
        opacity: 0;
    }
    25% {
        opacity: 0.15;
    }
    50% {
        opacity: 0.35;
    }
    75% {
        opacity: 0.15;
    }
} */

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

/* 命令回显行 - 赛博朋克风格 */
.line-command {
    color: rgba(0, 240, 255, 0.6);
}

.line-command .prompt {
    color: #39ff14;
    font-weight: bold;
    text-shadow:
        0 0 10px rgba(57, 255, 20, 0.6),
        0 0 20px rgba(57, 255, 20, 0.3);
}

.line-command .command {
    color: #00f0ff;
    font-weight: 600;
    text-shadow: 0 0 8px rgba(0, 240, 255, 0.4);
}

/* Markdown 行 - 融入终端的赛博朋克 Markdown 格式化 */
.line-markdown {
    padding: 8px 0 8px 12px;
    /* 霓虹粉色左边框 + 发光 */
    border-left: 3px solid #ff006e;
    background: rgba(255, 0, 110, 0.05);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.6;
    white-space: normal;
    box-shadow: -3px 0 10px rgba(255, 0, 110, 0.2);
}

/* Spinner 行 - 赛博朋克闪烁 */
.line-spinner {
    color: #39ff14;
    font-style: italic;
    text-shadow:
        0 0 10px rgba(57, 255, 20, 0.8),
        0 0 20px rgba(57, 255, 20, 0.4);
    animation: spinner-glow 1s ease-in-out infinite;
}

@keyframes spinner-glow {
    0%, 100% {
        text-shadow:
            0 0 10px rgba(57, 255, 20, 0.8),
            0 0 20px rgba(57, 255, 20, 0.4);
    }
    50% {
        text-shadow:
            0 0 15px rgba(57, 255, 20, 1),
            0 0 25px rgba(57, 255, 20, 0.6),
            0 0 35px rgba(57, 255, 20, 0.3);
    }
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

/* 输入字段 - 赛博朋克风格 */
.terminal-input-field {
    display: flex;
    align-items: center;
    padding: 8px 10px;
    /* 霓虹青色分割线 + 发光 */
    border-top: 2px solid rgba(0, 240, 255, 0.3);
    background: rgba(0, 240, 255, 0.03);
    box-shadow: 0 -2px 10px rgba(0, 240, 255, 0.1);
}

.terminal-input-field .prompt {
    /* 霓虹绿色 Prompt */
    color: #39ff14;
    font-weight: bold;
    margin-right: 8px;
    flex-shrink: 0;
    text-shadow:
        0 0 10px rgba(57, 255, 20, 0.8),
        0 0 20px rgba(57, 255, 20, 0.4);
    animation: prompt-blink 1.5s ease-in-out infinite;
}

@keyframes prompt-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
}

.terminal-input-field input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    /* 霓虹青色文字 */
    color: #00f0ff;
    font-family: inherit;
    font-size: inherit;
    text-shadow: 0 0 5px rgba(0, 240, 255, 0.5);
}

.terminal-input-field input::placeholder {
    color: rgba(0, 240, 255, 0.3);
}

/* ANSI 颜色类 - 赛博朋克霓虹色系 */
.ansi-reset {
    color: #00f0ff;
    font-weight: normal;
}

.ansi-bold {
    font-weight: bold;
    text-shadow: 0 0 5px currentColor;
}

.ansi-red {
    color: #ff0055;
    text-shadow: 0 0 10px rgba(255, 0, 85, 0.5);
}

.ansi-green {
    color: #39ff14;
    text-shadow: 0 0 10px rgba(57, 255, 20, 0.5);
}

.ansi-yellow {
    color: #ffea00;
    text-shadow: 0 0 10px rgba(255, 234, 0, 0.5);
}

.ansi-blue {
    color: #00f0ff;
    text-shadow: 0 0 10px rgba(0, 240, 255, 0.5);
}

.ansi-cyan {
    color: #00ffff;
    text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

.ansi-white {
    color: #ffffff;
    text-shadow: 0 0 8px rgba(255, 255, 255, 0.3);
}

/* 滚动条样式 - 赛博朋克霓虹 */
.terminal-output-area::-webkit-scrollbar {
    width: 8px;
}

.terminal-output-area::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.3);
    border-radius: 4px;
    box-shadow: inset 0 0 5px rgba(0, 240, 255, 0.1);
}

.terminal-output-area::-webkit-scrollbar-thumb {
    background: rgba(0, 240, 255, 0.3);
    border-radius: 4px;
    box-shadow:
        0 0 5px rgba(0, 240, 255, 0.5),
        inset 0 0 3px rgba(0, 240, 255, 0.3);
}

.terminal-output-area::-webkit-scrollbar-thumb:hover {
    background: rgba(0, 240, 255, 0.5);
    box-shadow:
        0 0 10px rgba(0, 240, 255, 0.8),
        inset 0 0 5px rgba(0, 240, 255, 0.5);
}

#status {
    text-align: center;
    margin-top: 8px;
    font-size: 0.85em;
    flex-shrink: 0;
}

#connection-status {
    padding: 5px 15px;
    /* 霓虹青色状态指示器 */
    background: rgba(0, 240, 255, 0.15);
    border: 1px solid rgba(0, 240, 255, 0.4);
    border-radius: 20px;
    display: inline-block;
    color: #00f0ff;
    text-shadow: 0 0 5px rgba(0, 240, 255, 0.5);
    box-shadow: 0 0 10px rgba(0, 240, 255, 0.3);
    animation: status-pulse 2s ease-in-out infinite;
}

@keyframes status-pulse {
    0%, 100% {
        box-shadow: 0 0 10px rgba(0, 240, 255, 0.3);
    }
    50% {
        box-shadow: 0 0 15px rgba(0, 240, 255, 0.5);
    }
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
   对话回合样式 (v1.28.0)
   Jupyter-like 卡片式回合显示
   ============================================ */

/* 回合容器 - 卡片风格 */
.conversation-round {
    margin: 12px 0;
    padding: 0;
    background: rgba(10, 14, 39, 0.6);
    border: 1px solid rgba(0, 240, 255, 0.3);
    border-radius: 8px;
    backdrop-filter: blur(10px);
    box-shadow: 0 0 15px rgba(0, 240, 255, 0.15);
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.conversation-round:hover {
    border-color: rgba(0, 240, 255, 0.5);
    box-shadow: 0 0 20px rgba(0, 240, 255, 0.25);
}

/* 回合头部 */
.round-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: rgba(0, 240, 255, 0.05);
    border-bottom: 1px solid rgba(0, 240, 255, 0.2);
    cursor: pointer;
    transition: background 0.2s;
}

.round-header:hover {
    background: rgba(0, 240, 255, 0.08);
}

/* 回合信息容器 */
.round-info {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    flex-wrap: wrap;
}

/* 回合编号 */
.round-number {
    font-weight: 600;
    color: #00f0ff;
    font-size: 0.9em;
    text-shadow: 0 0 8px rgba(0, 240, 255, 0.4);
}

/* 回合状态指示器 */
.round-status {
    font-size: 1.1em;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    transition: all 0.3s;
}

.round-status.pending {
    color: #888;
    background: rgba(136, 136, 136, 0.1);
}

.round-status.running {
    color: #00f0ff;
    background: rgba(0, 240, 255, 0.15);
    animation: status-pulse 1.5s ease-in-out infinite;
}

.round-status.success {
    color: #39ff14;
    background: rgba(57, 255, 20, 0.15);
    text-shadow: 0 0 8px rgba(57, 255, 20, 0.6);
}

.round-status.error {
    color: #ff006e;
    background: rgba(255, 0, 110, 0.15);
    text-shadow: 0 0 8px rgba(255, 0, 110, 0.6);
}

/* 飞轮动画（回合模式思考中） */
.round-status.spinner-active::before {
    content: '⠋';
    display: inline-block;
    animation: spinner-rotate 1s steps(10) infinite;
}

@keyframes spinner-rotate {
    0% { content: '⠋'; }
    10% { content: '⠙'; }
    20% { content: '⠹'; }
    30% { content: '⠸'; }
    40% { content: '⠼'; }
    50% { content: '⠴'; }
    60% { content: '⠦'; }
    70% { content: '⠧'; }
    80% { content: '⠇'; }
    90% { content: '⠏'; }
    100% { content: '⠋'; }
}

/* 执行时间 */
.round-time {
    color: #888;
    font-size: 0.85em;
    font-family: "Consolas", monospace;
}

/* 工具容器 */
.round-tools {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
}

/* 工具标签 */
.tool-badge {
    display: inline-block;
    padding: 2px 8px;
    background: rgba(255, 0, 110, 0.15);
    border: 1px solid rgba(255, 0, 110, 0.3);
    border-radius: 12px;
    font-size: 0.75em;
    color: #ff006e;
    text-shadow: 0 0 5px rgba(255, 0, 110, 0.4);
    box-shadow: 0 0 8px rgba(255, 0, 110, 0.2);
}

/* 回合摘要 */
.round-summary {
    color: rgba(240, 240, 240, 0.7);
    font-size: 0.85em;
    font-style: italic;
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
}

/* 折叠按钮 */
.round-toggle {
    background: none;
    border: none;
    color: #00f0ff;
    font-size: 1.2em;
    cursor: pointer;
    padding: 4px 8px;
    transition: all 0.2s;
    text-shadow: 0 0 8px rgba(0, 240, 255, 0.4);
}

.round-toggle:hover {
    color: #39ff14;
    text-shadow: 0 0 12px rgba(57, 255, 20, 0.6);
    transform: scale(1.1);
}

/* 回合内容区域 */
.round-content {
    padding: 12px;
    max-height: 10000px;
    overflow: hidden;
    transition: max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1),
                opacity 0.3s ease,
                padding 0.3s ease;
    opacity: 1;
}

/* 折叠状态 */
.conversation-round.collapsed .round-content {
    max-height: 0;
    padding-top: 0;
    padding-bottom: 0;
    opacity: 0;
}

.conversation-round.collapsed .round-toggle {
    transform: rotate(-90deg);
}

/* 输入区域 */
.round-input {
    margin-bottom: 8px;
}

.round-input-label {
    display: block;
    color: #00f0ff;
    font-size: 0.85em;
    font-weight: 600;
    margin-bottom: 4px;
    text-shadow: 0 0 5px rgba(0, 240, 255, 0.3);
}

.round-input-content {
    padding: 8px 12px;
    background: rgba(0, 240, 255, 0.05);
    border-left: 3px solid #00f0ff;
    border-radius: 4px;
    color: rgba(240, 240, 240, 0.9);
    font-family: "Consolas", monospace;
    font-size: 0.9em;
    white-space: pre-wrap;
    word-wrap: break-word;
    box-shadow: -3px 0 10px rgba(0, 240, 255, 0.1);
}

/* 输出区域 */
.round-output {
    margin-top: 8px;
}

.round-output-label {
    display: block;
    color: #39ff14;
    font-size: 0.85em;
    font-weight: 600;
    margin-bottom: 4px;
    text-shadow: 0 0 5px rgba(57, 255, 20, 0.3);
}

.output-content {
    padding: 8px 12px;
    background: rgba(57, 255, 20, 0.03);
    border-left: 3px solid #39ff14;
    border-radius: 4px;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.6;
    box-shadow: -3px 0 10px rgba(57, 255, 20, 0.1);
}

/* 输出内容继承 Markdown 样式 */
.output-content h1,
.output-content h2,
.output-content h3,
.output-content h4,
.output-content h5,
.output-content h6 {
    background: linear-gradient(90deg, #00f0ff 0%, #ff006e 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    font-weight: 600;
    margin: 0.8em 0 0.4em 0;
    filter: drop-shadow(0 0 5px rgba(0, 240, 255, 0.4));
}

.output-content code {
    color: #00f0ff;
    background-color: rgba(0, 240, 255, 0.08);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    border: 1px solid rgba(0, 240, 255, 0.3);
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.9em;
}

.output-content pre {
    background: rgba(10, 14, 39, 0.8);
    border: 1px solid rgba(0, 240, 255, 0.3);
    border-radius: 6px;
    padding: 12px;
    overflow-x: auto;
    margin: 8px 0;
    box-shadow: inset 0 0 15px rgba(0, 240, 255, 0.1);
}

.output-content pre code {
    background: none;
    border: none;
    padding: 0;
    color: rgba(240, 240, 240, 0.9);
}

/* 响应式调整 */
@media (max-width: 768px) {
    .round-info {
        gap: 8px;
    }

    .round-summary {
        max-width: 200px;
    }

    .conversation-round {
        margin: 8px 0;
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
    /* 霓虹青色到粉色渐变标题 */
    background: linear-gradient(90deg, #00f0ff 0%, #ff006e 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    font-weight: 600;
    margin: 0.8em 0 0.4em 0;
    text-shadow: 0 0 10px rgba(0, 240, 255, 0.3);
    filter: drop-shadow(0 0 5px rgba(0, 240, 255, 0.4));
}

.line-markdown h1 { font-size: 1.8em; }
.line-markdown h2 { font-size: 1.5em; }
.line-markdown h3 { font-size: 1.3em; }
.line-markdown h4 { font-size: 1.1em; }
.line-markdown h5 { font-size: 1.0em; }
.line-markdown h6 { font-size: 0.9em; }

/* 粗体 - 霓虹白色发光 */
.line-markdown strong {
    color: #ffffff;
    font-weight: 700;
    text-shadow: 0 0 8px rgba(255, 255, 255, 0.4);
}

/* 斜体 - 霓虹青色 */
.line-markdown em {
    color: #00f0ff;
    font-style: italic;
    text-shadow: 0 0 5px rgba(0, 240, 255, 0.3);
}

/* 内联代码 - 霓虹青色 + 发光边框 */
.line-markdown code {
    color: #00f0ff;
    background-color: rgba(0, 240, 255, 0.08);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    border: 1px solid rgba(0, 240, 255, 0.3);
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.9em;
    box-shadow:
        0 0 5px rgba(0, 240, 255, 0.2),
        inset 0 0 5px rgba(0, 240, 255, 0.1);
}

/* 代码块 - 霓虹绿色 + 发光边框 */
.line-markdown pre {
    background-color: rgba(0, 0, 0, 0.5);
    padding: 1em;
    border-radius: 5px;
    border: 1px solid rgba(57, 255, 20, 0.3);
    overflow-x: auto;
    margin: 0.5em 0;
    box-shadow:
        0 0 10px rgba(57, 255, 20, 0.2),
        inset 0 0 20px rgba(57, 255, 20, 0.05);
}

.line-markdown pre code {
    color: #39ff14;
    background: none;
    border: none;
    padding: 0;
    font-size: 0.95em;
    box-shadow: none;
    text-shadow: 0 0 5px rgba(57, 255, 20, 0.3);
}

/* 段落 - 霓虹青色 */
.line-markdown p {
    margin: 0.5em 0;
    color: rgba(0, 240, 255, 0.9);
}

/* 列表 - 霓虹粉色 bullet */
.line-markdown ul,
.line-markdown ol {
    margin: 0.5em 0;
    padding-left: 1.5em;
}

.line-markdown ul li::marker {
    color: #ff006e;
    text-shadow: 0 0 5px rgba(255, 0, 110, 0.5);
}

.line-markdown ol li::marker {
    color: #ff006e;
    font-weight: 600;
    text-shadow: 0 0 5px rgba(255, 0, 110, 0.5);
}

.line-markdown li {
    margin: 0.3em 0;
    color: rgba(0, 240, 255, 0.9);
}

/* 引用块 - 霓虹紫色边框 */
.line-markdown blockquote {
    border-left: 3px solid rgba(162, 57, 234, 0.6);
    padding-left: 1em;
    color: rgba(0, 240, 255, 0.7);
    margin: 0.5em 0;
    font-style: italic;
    background: rgba(162, 57, 234, 0.05);
    box-shadow: -3px 0 10px rgba(162, 57, 234, 0.2);
}

/* 链接 - 霓虹粉色 + 悬停发光 */
.line-markdown a {
    color: #ff006e;
    text-decoration: none;
    border-bottom: 1px solid rgba(255, 0, 110, 0.5);
    text-shadow: 0 0 5px rgba(255, 0, 110, 0.3);
    transition: all 0.3s ease;
}

.line-markdown a:hover {
    color: #ff3399;
    border-bottom-color: #ff006e;
    text-shadow:
        0 0 10px rgba(255, 0, 110, 0.6),
        0 0 20px rgba(255, 0, 110, 0.3);
}

/* 分隔线 - 霓虹发光 */
.line-markdown hr {
    border: none;
    height: 1px;
    background: linear-gradient(90deg,
        transparent 0%,
        rgba(0, 240, 255, 0.5) 20%,
        rgba(255, 0, 110, 0.5) 50%,
        rgba(0, 240, 255, 0.5) 80%,
        transparent 100%);
    margin: 1em 0;
    box-shadow: 0 0 10px rgba(0, 240, 255, 0.3);
}

/* 表格 - 赛博朋克网格 */
.line-markdown table {
    border-collapse: collapse;
    width: 100%;
    margin: 0.5em 0;
}

.line-markdown th,
.line-markdown td {
    border: 1px solid rgba(0, 240, 255, 0.3);
    padding: 0.4em 0.8em;
    text-align: left;
    color: rgba(0, 240, 255, 0.9);
}

.line-markdown th {
    background-color: rgba(0, 240, 255, 0.1);
    color: #00f0ff;
    font-weight: 600;
    text-shadow: 0 0 5px rgba(0, 240, 255, 0.4);
    box-shadow: inset 0 0 10px rgba(0, 240, 255, 0.1);
}

.line-markdown td {
    background-color: rgba(0, 240, 255, 0.03);
}

/* 图片 - 霓虹边框 */
.line-markdown img {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
    border: 2px solid rgba(0, 240, 255, 0.4);
    margin: 0.5em 0;
    box-shadow: 0 0 15px rgba(0, 240, 255, 0.3);
}
"#;
