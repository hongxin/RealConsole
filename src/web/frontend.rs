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
    <!-- 可视化支持 (v1.44.0): Apache ECharts -->
    <script src="https://cdn.jsdelivr.net/npm/echarts@5.4.3/dist/echarts.min.js"></script>
</head>
<body>
    <div id="header">
        <div id="header-left-controls">
            <div id="lang-switcher">
                <select id="lang-select" class="lang-dropdown">
                    <option value="zh-CN">🌐 中文</option>
                    <option value="en-US">🌐 English</option>
                </select>
            </div>
            <button id="theme-toggle-btn" class="theme-toggle-btn" title="切换主题">🌙 深色</button>
        </div>
        <div id="header-content">
            <h1 data-i18n="web.header.title">🌟 RealConsole 睿境</h1>
            <p data-i18n="web.header.tagline">融合东方哲学智慧的智能 CLI Agent</p>
        </div>
        <div id="header-right-controls">
            <button id="session-menu-btn" class="session-btn" title="会话管理">💾 会话</button>
            <button id="view-mode-toggle" class="view-mode-btn" title="切换到传统流式输出">📊 回合</button>
            <button id="clear-screen-btn" class="clear-btn" title="清空当前对话">🗑️ 清空</button>
        </div>
    </div>
    <!-- v1.40.0: 会话管理面板 -->
    <div id="session-panel" class="session-panel hidden">
        <div class="session-panel-overlay"></div>
        <div class="session-panel-dialog">
            <div class="session-panel-header">
                <h3>💾 会话管理</h3>
                <button id="session-panel-close" class="close-btn" title="关闭">×</button>
            </div>
            <div class="session-panel-content">
                <div class="session-actions">
                    <button id="save-session-btn" class="session-action-btn">💾 保存当前会话</button>
                    <button id="refresh-sessions-btn" class="session-action-btn">🔄 刷新列表</button>
                    <button id="clear-history-btn" class="session-action-btn session-clear-btn">🗑️ 清空历史</button>
                </div>
                <!-- v1.40.0: 搜索和筛选 -->
                <div class="session-filters">
                    <input type="text" id="session-search" class="session-search-input" placeholder="🔍 搜索会话名称...">
                    <select id="session-sort" class="session-sort-select">
                        <option value="updated_desc">最近更新</option>
                        <option value="updated_asc">最早更新</option>
                        <option value="created_desc">最新创建</option>
                        <option value="created_asc">最早创建</option>
                        <option value="rounds_desc">回合数多→少</option>
                        <option value="rounds_asc">回合数少→多</option>
                    </select>
                </div>
                <div id="session-list" class="session-list">
                    <div class="session-list-empty">加载中...</div>
                </div>
            </div>
        </div>
    </div>
    <!-- v1.46.0: 文件上传区域 -->
    <div id="file-upload-section" class="file-upload-section">
        <div id="upload-area" class="upload-area">
            <input type="file" id="file-input" accept=".csv" style="display: none;">
            <div class="upload-prompt">
                <svg class="upload-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                    <polyline points="17 8 12 3 7 8"></polyline>
                    <line x1="12" y1="3" x2="12" y2="15"></line>
                </svg>
                <p class="upload-text">拖拽 CSV 文件到此处或点击上传</p>
                <p class="upload-hint">支持最大 1MB，建议使用小文件进行测试</p>
            </div>
            <div id="upload-status" class="upload-status hidden"></div>
        </div>
        <!-- 已上传文件列表 -->
        <div id="uploaded-files" class="uploaded-files hidden">
            <h4>📁 已上传文件</h4>
            <div id="files-list" class="files-list"></div>
        </div>
    </div>
    <div id="terminal-container">
        <!-- 混合终端：单一容器，统一滚动 -->
    </div>
    <!-- v1.40.0: Toast 通知容器 -->
    <div id="toast-container" class="toast-container"></div>
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
                '90': 'dimmed',
            };
        }

        parse(text) {
            // 解析 ANSI 转义序列为 HTML
            // 支持两种格式：\x1b[XXm 和 [XXm（缺少 ESC 字符的情况）
            const regex = /(?:\x1b)?\[([0-9;]+)m/g;
            let html = '';
            let lastIndex = 0;
            let currentClasses = [];

            text.replace(regex, (match, codes, offset) => {
                // 添加前面的文本
                if (offset > lastIndex) {
                    const content = text.slice(lastIndex, offset);
                    html += this.wrapWithClasses(content, currentClasses);
                }

                // 处理 ANSI 代码（支持组合代码如 1;36）
                const codeList = codes.split(';');
                for (const code of codeList) {
                    if (code === '0') {
                        currentClasses = [];
                    } else if (this.ansiMap[code]) {
                        const className = `ansi-${this.ansiMap[code]}`;
                        // 避免重复添加
                        if (!currentClasses.includes(className)) {
                            currentClasses.push(className);
                        }
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

    // ========== Toast 通知管理器 (v1.40.0) ==========
    class ToastManager {
        constructor() {
            this.container = document.getElementById('toast-container');
            this.toasts = [];
            this.maxToasts = 5;
            this.defaultDuration = 4000; // 4 秒
        }

        /**
         * 显示 Toast 通知
         * @param {string} type - 类型: success, error, info, warning
         * @param {string} title - 标题
         * @param {string} message - 消息内容（可选）
         * @param {number} duration - 持续时间（毫秒），0 表示不自动关闭
         */
        show(type, title, message = '', duration = this.defaultDuration) {
            // 限制最大数量
            if (this.toasts.length >= this.maxToasts) {
                this.remove(this.toasts[0]);
            }

            const toast = this.createToast(type, title, message);
            this.container.appendChild(toast);
            this.toasts.push(toast);

            // 自动关闭
            if (duration > 0) {
                setTimeout(() => this.remove(toast), duration);
            }

            return toast;
        }

        createToast(type, title, message) {
            const toast = document.createElement('div');
            toast.className = `toast toast-${type}`;

            // 图标映射
            const icons = {
                success: '✓',
                error: '✕',
                info: 'ℹ',
                warning: '⚠'
            };

            const icon = icons[type] || 'ℹ';

            toast.innerHTML = `
                <div class="toast-icon">${icon}</div>
                <div class="toast-content">
                    <div class="toast-title">${this.escapeHtml(title)}</div>
                    ${message ? `<div class="toast-message">${this.escapeHtml(message)}</div>` : ''}
                </div>
                <button class="toast-close" aria-label="关闭">×</button>
            `;

            // 关闭按钮事件
            toast.querySelector('.toast-close').addEventListener('click', () => {
                this.remove(toast);
            });

            return toast;
        }

        remove(toast) {
            if (!toast || !this.toasts.includes(toast)) return;

            toast.classList.add('toast-exit');
            setTimeout(() => {
                if (toast.parentNode) {
                    toast.parentNode.removeChild(toast);
                }
                this.toasts = this.toasts.filter(t => t !== toast);
            }, 300); // 等待退出动画完成
        }

        // 便捷方法
        success(title, message = '', duration) {
            return this.show('success', title, message, duration);
        }

        error(title, message = '', duration) {
            return this.show('error', title, message, duration);
        }

        info(title, message = '', duration) {
            return this.show('info', title, message, duration);
        }

        warning(title, message = '', duration) {
            return this.show('warning', title, message, duration);
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

    // ========== v1.36.0: 意图占卜动画系统 ==========
    class DivinationAnimation {
        constructor(container) {
            this.container = container;
            this.animationDiv = null;
        }

        async start(planId) {
            // 创建动画容器
            this.animationDiv = document.createElement('div');
            this.animationDiv.className = 'divination-animation';
            this.animationDiv.innerHTML = `
                <div class="divination-stage qigua">
                    <div class="dots-container">
                        <span class="dot">⚪</span>
                        <span class="dot">⚪</span>
                        <span class="dot">⚪</span>
                        <span class="dot">⚪</span>
                        <span class="dot">⚪</span>
                        <span class="dot">⚪</span>
                    </div>
                    <div class="stage-label">起卦</div>
                </div>
            `;
            this.container.appendChild(this.animationDiv);

            // 起卦动画：圆点旋转闪烁
            await this.animateQiGua();
        }

        async animateQiGua() {
            const dots = this.animationDiv.querySelectorAll('.dot');
            let count = 0;

            return new Promise(resolve => {
                const interval = setInterval(() => {
                    dots.forEach((dot, i) => {
                        if (i <= count % 6) {
                            dot.textContent = '⚫';
                            dot.classList.add('active');
                        } else {
                            dot.textContent = '⚪';
                            dot.classList.remove('active');
                        }
                    });

                    count++;

                    if (count > 12) {  // 两轮动画
                        clearInterval(interval);
                        resolve();
                    }
                }, 100);
            });
        }

        async showYarrowStep(step) {
            // 切换到演算阶段
            this.animationDiv.innerHTML = `
                <div class="divination-stage yansuan">
                    <div class="operation-name">${step.operation}</div>
                    <div class="stalk-count">${step.value}</div>
                    <div class="operation-desc">${step.description}</div>
                    <div class="yarrow-visual">${'|'.repeat(Math.min(step.value, 49))}</div>
                </div>
            `;

            // 数字变化动画
            const countEl = this.animationDiv.querySelector('.stalk-count');
            countEl.classList.add('changing');

            await this.sleep(100);

            countEl.classList.remove('changing');
        }

        async showHexagram(hexagram) {
            // 切换到成卦阶段
            this.animationDiv.innerHTML = `
                <div class="divination-stage chenggua">
                    <div class="hexagram-forming">
                        <div class="hexagram-symbol"></div>
                        <div class="hexagram-name">【${hexagram.name}】</div>
                    </div>
                </div>
            `;

            // 卦象生成动画（爻画逐个显示，从下往上）
            const symbolEl = this.animationDiv.querySelector('.hexagram-symbol');
            const lines = hexagram.symbol.split('\n');

            for (const line of lines.reverse()) {
                await this.sleep(100);
                const lineDiv = document.createElement('div');
                lineDiv.className = 'yao-line fade-in';
                lineDiv.textContent = line;
                symbolEl.insertBefore(lineDiv, symbolEl.firstChild);
            }
        }

        complete(divinationResult) {
            // 移除动画，显示最终卦象
            this.animationDiv.remove();

            // 在意图卡片顶部插入卦象信息
            const hexagramCard = document.createElement('div');
            hexagramCard.className = 'hexagram-card';
            hexagramCard.innerHTML = `
                <div class="hexagram-display">
                    <div class="hexagram-symbol-large">${divinationResult.hexagram.symbol.replace(/\n/g, '<br>')}</div>
                    <div class="hexagram-info">
                        <div class="hexagram-name-large">【${divinationResult.hexagram.name}】</div>
                        <div class="hexagram-judgement">${divinationResult.hexagram.judgement}</div>
                    </div>
                </div>
            `;

            return hexagramCard;
        }

        sleep(ms) {
            return new Promise(resolve => setTimeout(resolve, ms));
        }
    }

    // ========== v1.40.0: 浏览器端会话持久化 ==========
    /**
     * LocalStorageManager - 浏览器端会话持久化
     *
     * 功能：
     * 1. 自动保存当前会话到 LocalStorage
     * 2. 页面刷新后自动恢复会话
     * 3. 管理历史会话（列表、删除、清理）
     * 4. 配置保留策略（数量、时间限制）
     */
    class LocalStorageManager {
        constructor() {
            this.currentSessionKey = 'realconsole_current_session';
            this.historyKey = 'realconsole_session_history';
            this.configKey = 'realconsole_session_config';

            // 默认配置
            this.defaultConfig = {
                auto_save: true,           // 自动保存
                max_history: 10,           // 最大历史数量
                max_age_days: 30,          // 最大保留天数
                save_on_exit: true,        // 退出时保存
                auto_restore: true         // 自动恢复
            };

            this.config = this.loadConfig();
            this.init();
        }

        init() {
            // 页面加载时清理过期会话
            this.cleanupOldSessions();
            console.log('[LocalStorage] Initialized, config:', this.config);
        }

        // ========== 配置管理 ==========

        loadConfig() {
            try {
                const json = localStorage.getItem(this.configKey);
                if (json) {
                    return { ...this.defaultConfig, ...JSON.parse(json) };
                }
            } catch (e) {
                console.error('[LocalStorage] Failed to load config:', e);
            }
            return this.defaultConfig;
        }

        saveConfig(config) {
            try {
                this.config = { ...this.config, ...config };
                localStorage.setItem(this.configKey, JSON.stringify(this.config));
                console.log('[LocalStorage] Config saved:', this.config);
            } catch (e) {
                console.error('[LocalStorage] Failed to save config:', e);
            }
        }

        // ========== 当前会话管理 ==========

        saveCurrentSession(session) {
            try {
                const json = JSON.stringify(session);
                localStorage.setItem(this.currentSessionKey, json);
                console.log('[LocalStorage] Current session saved:', session.name, this.formatSize(json.length));
                return true;
            } catch (e) {
                console.error('[LocalStorage] Failed to save current session:', e);
                return false;
            }
        }

        loadCurrentSession() {
            try {
                const json = localStorage.getItem(this.currentSessionKey);
                if (json) {
                    const session = JSON.parse(json);
                    console.log('[LocalStorage] Current session loaded:', session.name);
                    return session;
                }
            } catch (e) {
                console.error('[LocalStorage] Failed to load current session:', e);
            }
            return null;
        }

        clearCurrentSession() {
            try {
                localStorage.removeItem(this.currentSessionKey);
                console.log('[LocalStorage] Current session cleared');
            } catch (e) {
                console.error('[LocalStorage] Failed to clear current session:', e);
            }
        }

        // ========== 历史会话管理 ==========

        addToHistory(session) {
            try {
                const history = this.getHistory();

                // 创建历史项
                const historyItem = {
                    id: session.id,
                    name: session.name,
                    created_at: session.created_at,
                    updated_at: session.updated_at,
                    round_count: session.rounds ? session.rounds.length : 0,
                    preview: this.generatePreview(session.rounds),
                    size: JSON.stringify(session).length
                };

                // 检查是否已存在
                const existingIndex = history.findIndex(h => h.id === session.id);
                if (existingIndex >= 0) {
                    // 更新已有项
                    history[existingIndex] = historyItem;
                } else {
                    // 添加新项
                    history.push(historyItem);
                }

                // 保存历史列表
                localStorage.setItem(this.historyKey, JSON.stringify(history));

                // 保存完整会话数据
                const sessionKey = `realconsole_session_${session.id}`;
                localStorage.setItem(sessionKey, JSON.stringify(session));

                // 执行清理策略
                this.enforceMaxHistory();

                console.log('[LocalStorage] Added to history:', session.name);
                return true;
            } catch (e) {
                console.error('[LocalStorage] Failed to add to history:', e);
                return false;
            }
        }

        getHistory() {
            try {
                const json = localStorage.getItem(this.historyKey);
                if (json) {
                    const history = JSON.parse(json);
                    // 按更新时间倒序排列
                    return history.sort((a, b) =>
                        new Date(b.updated_at) - new Date(a.updated_at)
                    );
                }
            } catch (e) {
                console.error('[LocalStorage] Failed to get history:', e);
            }
            return [];
        }

        getHistoryItem(id) {
            try {
                const sessionKey = `realconsole_session_${id}`;
                const json = localStorage.getItem(sessionKey);
                if (json) {
                    return JSON.parse(json);
                }
            } catch (e) {
                console.error('[LocalStorage] Failed to get history item:', e);
            }
            return null;
        }

        deleteHistoryItem(id) {
            try {
                // 删除历史列表项
                const history = this.getHistory();
                const newHistory = history.filter(h => h.id !== id);
                localStorage.setItem(this.historyKey, JSON.stringify(newHistory));

                // 删除完整会话数据
                const sessionKey = `realconsole_session_${id}`;
                localStorage.removeItem(sessionKey);

                console.log('[LocalStorage] Deleted history item:', id);
                return true;
            } catch (e) {
                console.error('[LocalStorage] Failed to delete history item:', e);
                return false;
            }
        }

        clearHistory() {
            try {
                const history = this.getHistory();

                // 删除所有会话数据
                history.forEach(item => {
                    const sessionKey = `realconsole_session_${item.id}`;
                    localStorage.removeItem(sessionKey);
                });

                // 清空历史列表
                localStorage.removeItem(this.historyKey);

                console.log('[LocalStorage] History cleared');
                return true;
            } catch (e) {
                console.error('[LocalStorage] Failed to clear history:', e);
                return false;
            }
        }

        // ========== 清理策略 ==========

        enforceMaxHistory() {
            const history = this.getHistory();
            if (history.length <= this.config.max_history) {
                return;
            }

            // 删除最旧的项
            const toDelete = history.length - this.config.max_history;
            for (let i = 0; i < toDelete; i++) {
                const item = history[history.length - 1 - i];
                this.deleteHistoryItem(item.id);
                console.log('[LocalStorage] Deleted old session (max_history):', item.name);
            }
        }

        cleanupOldSessions() {
            if (this.config.max_age_days <= 0) {
                return;
            }

            const now = new Date();
            const maxAge = this.config.max_age_days * 24 * 60 * 60 * 1000;
            const history = this.getHistory();

            history.forEach(item => {
                const age = now - new Date(item.updated_at);
                if (age > maxAge) {
                    this.deleteHistoryItem(item.id);
                    console.log('[LocalStorage] Deleted expired session:', item.name);
                }
            });
        }

        // ========== 辅助方法 ==========

        generatePreview(rounds) {
            if (!rounds || rounds.length === 0) {
                return 'Empty session';
            }

            const firstRound = rounds[0];
            const input = firstRound.user_input || '';

            // 安全截取前 50 个字符
            if (input.length > 50) {
                return input.substring(0, 50) + '...';
            }
            return input;
        }

        formatSize(bytes) {
            if (bytes < 1024) return bytes + ' B';
            if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
            return (bytes / 1024 / 1024).toFixed(1) + ' MB';
        }

        generateUUID() {
            return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
                var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
                return v.toString(16);
            });
        }

        checkStorageQuota() {
            try {
                let total = 0;
                for (let key in localStorage) {
                    if (key.startsWith('realconsole_')) {
                        total += localStorage[key].length;
                    }
                }

                const totalMB = total / 1024 / 1024;
                console.log('[LocalStorage] Storage usage:', totalMB.toFixed(2), 'MB');

                // 如果超过 8MB，清理最旧的会话
                if (totalMB > 8) {
                    console.warn('[LocalStorage] Storage quota warning, cleaning up...');
                    this.enforceMaxHistory();
                }

                return totalMB;
            } catch (e) {
                console.error('[LocalStorage] Failed to check storage quota:', e);
                return 0;
            }
        }
    }

    // ========== v1.40.0: 服务器端会话管理器 ==========
    /**
     * ServerSessionManager - 服务器端会话管理
     *
     * 功能：通过 WebSocket 与服务器通信，管理服务器端存储的会话
     * 存储：Rust 服务器端文件系统（~/.realconsole/sessions/）
     * 结构：使用 session-card HTML 结构
     * 特点：多设备同步、持久化存储、支持导出
     *
     * 注意：这是主要使用的会话管理系统，与 BrowserSessionManager 独立
     */
    class ServerSessionManager {
        constructor(terminal, websocket) {
            this.terminal = terminal;
            this.ws = websocket;
            this.panel = null;
            this.sessions = [];
            this.currentSessionId = null;

            this.init();
        }

        init() {
            this.panel = document.getElementById('session-panel');
            if (!this.panel) {
                console.error('[SessionManager] 无法找到 session-panel 元素');
                return;
            }
            this.bindEvents();
        }

        bindEvents() {
            const menuBtn = document.getElementById('session-menu-btn');
            const closeBtn = document.getElementById('session-panel-close');
            const overlay = document.querySelector('.session-panel-overlay');
            const saveBtn = document.getElementById('save-session-btn');
            const refreshBtn = document.getElementById('refresh-sessions-btn');

            if (menuBtn) {
                menuBtn.onclick = () => this.show();
            }

            if (closeBtn) {
                closeBtn.onclick = () => this.hide();
            }

            if (overlay) {
                overlay.onclick = () => this.hide();
            }

            if (saveBtn) {
                saveBtn.onclick = () => this.saveSession();
            }

            if (refreshBtn) {
                refreshBtn.onclick = () => this.loadSessions();
            }
        }

        show() {
            this.panel.classList.remove('hidden');
            this.loadSessions();
        }

        hide() {
            this.panel.classList.add('hidden');
        }

        saveSession(name = null) {
            const message = {
                type: 'save_session',
                name: name
            };
            this.ws.send(JSON.stringify(message));
        }

        loadSession(sessionId) {
            if (!confirm('加载会话将替换当前内容，是否继续？')) {
                return;
            }

            const message = {
                type: 'load_session',
                session_id: sessionId
            };
            this.ws.send(JSON.stringify(message));
            this.hide();
        }

        loadSessions() {
            const message = { type: 'list_sessions' };
            this.ws.send(JSON.stringify(message));
        }

        renameSession(sessionId, currentName) {
            const newName = prompt('请输入新的会话名称:', currentName);

            if (!newName || newName.trim() === '') {
                return;
            }

            const trimmedName = newName.trim();
            if (trimmedName === currentName) {
                this.showNotification('名称未改变', 'info');
                return;
            }

            const message = {
                type: 'rename_session',
                session_id: sessionId,
                new_name: trimmedName
            };
            this.ws.send(JSON.stringify(message));
        }

        deleteSession(sessionId, sessionName) {
            if (!confirm(`确定删除会话 "${sessionName}"？`)) {
                return;
            }

            const message = {
                type: 'delete_session',
                session_id: sessionId
            };
            this.ws.send(JSON.stringify(message));
        }

        exportSession(sessionId, format = 'markdown') {
            const message = {
                type: 'export_session',
                session_id: sessionId,
                format: format
            };
            this.ws.send(JSON.stringify(message));
        }

        handleSessionSaved(data) {
            this.showNotification(`✅ 会话已保存: ${data.name}`);
            this.loadSessions();
        }

        handleSessionLoaded(data) {
            this.currentSessionId = data.session.id;
            this.showNotification(`✅ 会话已加载: ${data.session.name}`);

            this.terminal.clearAll();
            if (data.session.rounds && data.session.rounds.length > 0) {
                data.session.rounds.forEach(round => {
                    this.terminal.createRound(round);
                    this.terminal.completeRound(round);
                });
            }
        }

        handleSessionList(data) {
            this.sessions = data.sessions;
            this.renderSessionList();
        }

        handleSessionDeleted(data) {
            this.showNotification(`✅ 会话已删除: ${data.session_id}`);
            this.loadSessions();
        }

        handleSessionExported(data) {
            // 从路径中提取文件名
            const filename = data.export_path.split('/').pop() || `session-${data.session_id}.${data.format}`;
            this.showNotification(`✅ 会话已导出: ${filename}`);
            this.downloadFile(filename, data.content);
        }

        handleSessionError(data) {
            this.showNotification(`❌ 错误: ${data.message}`, 'error');
        }

        renderSessionList() {
            const listContainer = document.getElementById('session-list');

            if (this.sessions.length === 0) {
                listContainer.innerHTML = `
                    <div class="session-list-empty">
                        暂无保存的会话
                    </div>
                `;
                return;
            }

            const html = this.sessions.map(session => this.renderSessionCard(session)).join('');
            listContainer.innerHTML = html;

            this.bindSessionCardEvents();
        }

        renderSessionCard(session) {
            const date = new Date(session.created_at).toLocaleString('zh-CN');
            const isCurrent = session.id === this.currentSessionId;
            // 后端返回的是 SessionListItem，使用 round_count 字段
            const roundCount = session.round_count || 0;

            return `
                <div class="session-card ${isCurrent ? 'current' : ''}" data-session-id="${session.id}">
                    <div class="session-card-header">
                        <h4 class="session-name">${this.escapeHtml(session.name)}</h4>
                        ${isCurrent ? '<span class="current-badge">当前</span>' : ''}
                    </div>
                    <div class="session-card-info">
                        <span class="session-date">📅 ${date}</span>
                        <span class="session-rounds">💬 ${roundCount} 回合</span>
                    </div>
                    <div class="session-card-actions">
                        <button class="session-card-btn load-btn" data-action="load">
                            📂 加载
                        </button>
                        <button class="session-card-btn rename-btn" data-action="rename">
                            ✏️ 重命名
                        </button>
                        <button class="session-card-btn export-btn" data-action="export">
                            📤 导出
                        </button>
                        <button class="session-card-btn delete-btn" data-action="delete">
                            🗑️ 删除
                        </button>
                    </div>
                </div>
            `;
        }

        bindSessionCardEvents() {
            document.querySelectorAll('.session-card-btn').forEach(btn => {
                btn.onclick = (e) => {
                    e.stopPropagation();
                    const card = btn.closest('.session-card');
                    const sessionId = card.dataset.sessionId;
                    const action = btn.dataset.action;
                    const session = this.sessions.find(s => s.id === sessionId);

                    switch (action) {
                        case 'load':
                            this.loadSession(sessionId);
                            break;
                        case 'rename':
                            this.renameSession(sessionId, session.name);
                            break;
                        case 'export':
                            this.exportSession(sessionId);
                            break;
                        case 'delete':
                            this.deleteSession(sessionId, session.name);
                            break;
                    }
                };
            });
        }

        showNotification(message, type = 'success') {
            const statusEl = document.getElementById('connection-status');
            const originalText = statusEl.textContent;
            const originalClass = statusEl.className;

            statusEl.textContent = message;
            statusEl.className = `notification ${type}`;

            setTimeout(() => {
                statusEl.textContent = originalText;
                statusEl.className = originalClass;
            }, 3000);
        }

        escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        downloadFile(filename, content) {
            const blob = new Blob([content], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            a.click();
            URL.revokeObjectURL(url);
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

            // ===== v1.29.2: 意图拆解交互编辑 =====
            this.intentPlans = new Map();  // 存储计划数据: planId -> {understanding, steps, metadata}
            this.editMode = new Map();     // 存储编辑状态: planId -> {editing: boolean, originalSteps: [...]}

            // ===== v1.29.3: 执行计划回调 =====
            this.onExecutePlan = null;     // 执行计划回调: (planId, enabledSteps) => void

            // ===== v1.36.0: 态势测算分析动画 =====
            this.currentDivination = null;  // 当前的态势分析动画实例

            // ===== v1.40.0: 会话管理 =====
            this.sessionManager = null;  // WebSocket 连接后初始化

            // ===== v1.40.0 Phase 2: 浏览器端会话持久化 =====
            this.localStorage = new LocalStorageManager();
            this.sessionId = null;           // 会话 ID (UUID)
            this.sessionCreatedAt = null;    // 会话创建时间
            this.conversationId = null;      // 对话 ID (从服务器获取)

            this.init();

            // 设置自动保存
            this.setupAutoSave();
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

            // v1.44.0: 语音输入按钮
            const voiceBtn = document.createElement('button');
            voiceBtn.className = 'voice-input-btn';
            voiceBtn.innerHTML = '🎤';
            voiceBtn.title = '点击开始语音输入 (Click to start voice input)';
            voiceBtn.setAttribute('aria-label', 'Voice Input');

            line.appendChild(prompt);
            line.appendChild(input);
            line.appendChild(voiceBtn);

            this.currentInput = { line, input, voiceBtn };
            this.container.appendChild(line);

            this.setupInputHandlers();
            this.setupVoiceInput();
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

        // v1.44.0: 语音输入功能
        setupVoiceInput() {
            const voiceBtn = this.currentInput.voiceBtn;
            const input = this.currentInput.input;

            // 检查浏览器是否支持语音识别
            const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
            if (!SpeechRecognition) {
                voiceBtn.disabled = true;
                voiceBtn.title = '您的浏览器不支持语音识别 (Voice recognition not supported)';
                voiceBtn.style.opacity = '0.3';
                voiceBtn.style.cursor = 'not-allowed';
                return;
            }

            // 初始化语音识别
            const recognition = new SpeechRecognition();
            recognition.continuous = false;  // 单次识别
            recognition.interimResults = true;  // 显示临时结果
            recognition.lang = 'zh-CN';  // 默认中文，会自动检测语言

            let isRecording = false;
            let finalTranscript = '';

            // 点击按钮开始/停止录音
            voiceBtn.addEventListener('click', () => {
                if (isRecording) {
                    recognition.stop();
                } else {
                    finalTranscript = input.value;  // 保存当前输入
                    recognition.start();
                }
            });

            // 开始录音
            recognition.onstart = () => {
                isRecording = true;
                voiceBtn.classList.add('recording');
                voiceBtn.innerHTML = '🔴';
                voiceBtn.title = '点击停止录音 (Click to stop recording)';
            };

            // 识别结果
            recognition.onresult = (event) => {
                let interimTranscript = '';

                for (let i = event.resultIndex; i < event.results.length; i++) {
                    const transcript = event.results[i][0].transcript;
                    if (event.results[i].isFinal) {
                        finalTranscript += transcript;
                    } else {
                        interimTranscript += transcript;
                    }
                }

                // 实时显示识别结果
                input.value = finalTranscript + interimTranscript;
            };

            // 识别结束
            recognition.onend = () => {
                isRecording = false;
                voiceBtn.classList.remove('recording');
                voiceBtn.innerHTML = '🎤';
                voiceBtn.title = '点击开始语音输入 (Click to start voice input)';

                // 聚焦输入框
                input.focus();
            };

            // 识别错误
            recognition.onerror = (event) => {
                console.error('语音识别错误:', event.error);
                isRecording = false;
                voiceBtn.classList.remove('recording');
                voiceBtn.innerHTML = '🎤';

                // 显示错误提示
                if (event.error === 'not-allowed') {
                    voiceBtn.title = '请允许麦克风权限 (Please allow microphone access)';
                } else if (event.error === 'no-speech') {
                    voiceBtn.title = '未检测到语音，请重试 (No speech detected)';
                } else {
                    voiceBtn.title = '语音识别错误，请重试 (Voice recognition error)';
                }

                // 3秒后恢复原始提示
                setTimeout(() => {
                    voiceBtn.title = '点击开始语音输入 (Click to start voice input)';
                }, 3000);
            };

            // 保存识别对象供后续使用
            this.voiceRecognition = recognition;
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
            // v1.40.0 Bug Fix: 回合模式下需要更新 currentRound.aiResponse
            // 即使不在终端输出区域显示，也要同步到当前回合
            if (this.viewMode === 'round') {
                if (this.currentRound) {
                    // 累积内容，不要覆盖（Intent 执行会发送多条 Output 消息）
                    if (!this.currentRound.aiResponse) {
                        this.currentRound.aiResponse = '';
                    }
                    this.currentRound.aiResponse += content;
                }
                return;
            }

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

            // v1.40.0 Bug Fix: 在回合视图模式下，同步更新当前回合的 aiResponse
            // 这样 completeRound 时才能正确渲染内容
            if (this.viewMode === 'round' && this.currentRound) {
                this.currentRound.aiResponse = this.streamBuffer;
            }
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

        // v1.40.0: 清空所有内容（用于加载会话）
        clearAll() {
            this.rounds = [];
            this.lines = [];
            this.outputArea.innerHTML = '';
            this.removeSpinner();
            this.streamBuffer = '';
            this.isStreaming = false;
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

            // v1.38.0: 检查是否是重新执行的 Round
            if (this.rerunningRound && this.rerunningRound.input === round.user_input) {
                console.log(`[v1.38.0] Replacing rerun round: ${this.rerunningRound.id} -> ${round.id}`);

                // 删除旧 Round
                const oldRoundIndex = this.rounds.findIndex(r => r.id === this.rerunningRound.id);
                if (oldRoundIndex !== -1) {
                    this.rounds.splice(oldRoundIndex, 1);
                }

                // 从 DOM 中移除旧 Round
                if (this.rerunningRound.element && this.rerunningRound.element.parentNode) {
                    this.rerunningRound.element.parentNode.removeChild(this.rerunningRound.element);
                }

                // 清空重新执行标记
                this.rerunningRound = null;
            }

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

            // ===== v1.36.2: 极简主义优化 - 直接平铺元素，减少嵌套 =====
            const toolsHtml = (round.roundType === 'llm')
                ? `<span class="round-tools">${this.renderTools(round.toolsUsed)}</span>`
                : '';

            // v1.42.0: running 状态显示空span（用于飞轮动画），其他状态显示图标
            const statusIcon = (round.status === 'running') ? '' : this.getStatusIcon(round.status);

            header.innerHTML = `
                <span class="round-badge">${typeConfig.badge}</span>
                <span class="round-number">#${round.index}</span>
                <span class="round-status ${round.status}">${statusIcon}</span>
                <span class="round-time">${round.executionTime.toFixed(2)}s</span>
                ${toolsHtml}
                <span style="margin-left: auto;"></span>
                <button class="round-drag-handle" title="拖拽排序" draggable="true">☰</button>
                <button class="round-rerun-btn" title="重新执行此 Cell">🔄</button>
                <button class="round-delete-btn" title="删除此 Cell">🗑️</button>
                <button class="round-toggle" data-action="collapse">▼</button>
            `;

            // 回合内容
            const content = document.createElement('div');
            content.className = 'round-content';

            // 用户输入（极简：直接显示内容，无额外包装）
            const inputDiv = document.createElement('div');
            inputDiv.className = 'round-input-content';
            inputDiv.textContent = round.userInput;

            // 输出
            const outputDiv = document.createElement('div');
            outputDiv.className = 'output-content';

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

            // v1.38.0: 重新执行按钮事件
            const rerunBtn = header.querySelector('.round-rerun-btn');
            if (rerunBtn) {
                rerunBtn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    this.rerunCell(round.id);
                });
            }

            // v1.41.0: 删除回合按钮事件
            const deleteBtn = header.querySelector('.round-delete-btn');
            if (deleteBtn) {
                deleteBtn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    this.deleteRound(round.id);
                });
            }

            // v1.42.0: 拖拽排序功能
            const dragHandle = header.querySelector('.round-drag-handle');
            if (dragHandle) {
                // 拖拽开始：从手柄开始拖拽整个卡片
                dragHandle.addEventListener('dragstart', (e) => {
                    this.handleDragStart(e, round.id, roundDiv);
                });

                dragHandle.addEventListener('dragend', (e) => {
                    this.handleDragEnd(e, roundDiv);
                });
            }

            // 设置 round-card 作为拖拽目标
            roundDiv.addEventListener('dragover', (e) => {
                this.handleDragOver(e, roundDiv);
            });

            roundDiv.addEventListener('drop', (e) => {
                this.handleDrop(e, round.id);
            });

            roundDiv.addEventListener('dragleave', (e) => {
                this.handleDragLeave(e, roundDiv);
            });

            return roundDiv;
        }

        // 获取回合类型配置
        getRoundTypeConfig(roundType) {
            const configs = {
                [RoundType.LLM]: {
                    badge: '🤖 Round',
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
            if (!round) {
                console.error(`Round not found: ${roundData.id}`);
                return;
            }

            const normalizedStatus = this.normalizeStatus(roundData.status);

            // v1.40.0 Bug Fix: 不要覆盖前端已累积的 aiResponse（来自 Intent/意图拆解输出）
            // 优先使用前端累积的内容，只有在前端没有累积内容时才使用后端的响应
            if (!round.aiResponse || round.aiResponse === '') {
                round.aiResponse = roundData.ai_response || '';
            }

            round.executionTime = roundData.execution_time || 0;
            round.toolsUsed = roundData.tools_used || [];
            round.status = normalizedStatus;

            // 更新 UI
            const statusSpan = round.element.querySelector('.round-status');
            statusSpan.textContent = this.getStatusIcon(normalizedStatus);
            statusSpan.className = `round-status ${normalizedStatus}`;

            const timeSpan = round.element.querySelector('.round-time');
            timeSpan.textContent = `${roundData.execution_time.toFixed(2)}s`;

            const toolsSpan = round.element.querySelector('.round-tools');
            if (toolsSpan) {
                toolsSpan.innerHTML = this.renderTools(roundData.tools_used);
            }

            // 渲染输出内容
            const outputContent = round.element.querySelector('.output-content');

            if (round.aiResponse) {
                // 根据回合类型选择渲染方式
                if (round.roundType === RoundType.LLM) {
                    // LLM 对话：尝试 Markdown 渲染，失败则回退到纯文本
                    const responseDiv = document.createElement('div');
                    responseDiv.className = 'llm-response';

                    // v1.40.0 Bug Fix: Markdown 渲染可能返回 null（非 Markdown 内容）
                    // 例如 Intent 输出不包含 Markdown 标记，需要回退到纯文本显示
                    const markdownHtml = this.markdownRenderer.render(round.aiResponse);
                    if (markdownHtml) {
                        responseDiv.innerHTML = markdownHtml;
                    } else {
                        // 回退到纯文本渲染（保留格式）
                        const pre = document.createElement('pre');
                        pre.className = 'terminal-text intent-output';
                        pre.textContent = round.aiResponse;
                        responseDiv.appendChild(pre);
                    }
                    outputContent.appendChild(responseDiv);
                } else {
                    // Shell/System 命令：解析 ANSI 颜色代码并保留格式
                    const pre = document.createElement('pre');
                    pre.className = 'terminal-text';
                    // ✨ 使用 AnsiParser 解析 ANSI 转义序列
                    pre.innerHTML = this.ansiParser.parse(round.aiResponse);
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

        // v1.38.0: 重新执行 Cell
        rerunCell(roundId) {
            console.log(`[v1.38.0] Rerunning cell: ${roundId}`);

            const round = this.rounds.find(r => r.id === roundId);
            if (!round) {
                console.error(`[v1.38.0 ERROR] Round not found: ${roundId}`);
                return;
            }

            // 发送 WebSocket 消息
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                // 记录正在重新执行的 Round（用于后续替换）
                this.rerunningRound = {
                    id: roundId,
                    input: round.userInput,
                    element: round.element
                };

                // 隐藏旧 Round（避免显示重复的 Loading 状态）
                round.element.style.display = 'none';

                this.ws.send(JSON.stringify({
                    type: 'rerun_cell',
                    round_id: roundId
                }));
            } else {
                console.error('[v1.38.0 ERROR] WebSocket not connected');
                // 显示错误提示
                const outputContent = round.element.querySelector('.output-content');
                if (outputContent) {
                    outputContent.innerHTML = `
                        <div style="color: #ff006e; padding: 1em;">
                            ❌ WebSocket 未连接，无法重新执行
                        </div>
                    `;
                }
            }
        }

        // v1.41.0: 删除回合
        deleteRound(roundId) {
            console.log(`[v1.41.0] Deleting round: ${roundId}`);

            const round = this.rounds.find(r => r.id === roundId);
            if (!round) {
                console.error(`[v1.41.0 ERROR] Round not found: ${roundId}`);
                return;
            }

            // 准备用户输入预览（最多50个字符）
            const inputPreview = round.userInput.substring(0, 50);
            const inputSuffix = round.userInput.length > 50 ? '...' : '';

            // 显示确认对话框
            const confirmed = confirm(
                `确定删除 Round #${round.index}？\n\n` +
                `输入：${inputPreview}${inputSuffix}\n\n` +
                `⚠️ 此操作不可恢复！`
            );

            if (!confirmed) {
                console.log(`[v1.41.0] Delete cancelled by user`);
                return;
            }

            // 从 rounds 数组中删除
            const index = this.rounds.indexOf(round);
            if (index > -1) {
                this.rounds.splice(index, 1);
                console.log(`[v1.41.0] Removed from rounds array, remaining: ${this.rounds.length}`);
            }

            // 从 DOM 中移除（添加淡出动画）
            round.element.style.transition = 'opacity 0.3s ease, transform 0.3s ease';
            round.element.style.opacity = '0';
            round.element.style.transform = 'translateX(-20px)';

            setTimeout(() => {
                round.element.remove();
                console.log(`[v1.41.0] Round #${round.index} deleted successfully`);
            }, 300);
        }

        // v1.42.0: 拖拽排序 - 开始拖拽
        handleDragStart(e, roundId, element) {
            console.log(`[v1.42.0] Drag start: ${roundId}`);
            this.draggedRoundId = roundId;
            this.draggedElement = element;

            // 添加拖拽样式
            element.classList.add('dragging');

            // 设置拖拽数据
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', roundId);
        }

        // v1.42.0: 拖拽排序 - 拖拽结束
        handleDragEnd(e, element) {
            console.log(`[v1.42.0] Drag end`);

            // 移除所有拖拽相关样式
            element.classList.remove('dragging');
            document.querySelectorAll('.conversation-round.drag-over').forEach(el => {
                el.classList.remove('drag-over');
            });

            this.draggedRoundId = null;
            this.draggedElement = null;
        }

        // v1.42.0: 拖拽排序 - 拖拽经过
        handleDragOver(e, element) {
            e.preventDefault(); // 允许放置

            // 不允许拖到自己上面
            if (this.draggedElement === element) {
                return;
            }

            // 设置放置效果
            e.dataTransfer.dropEffect = 'move';

            // 添加视觉反馈
            element.classList.add('drag-over');
        }

        // v1.42.0: 拖拽排序 - 离开拖拽区域
        handleDragLeave(e, element) {
            // 只在离开当前元素时移除样式（避免子元素触发）
            if (e.target === element) {
                element.classList.remove('drag-over');
            }
        }

        // v1.42.0: 拖拽排序 - 放置
        handleDrop(e, targetRoundId) {
            e.preventDefault();
            e.stopPropagation();

            // 移除视觉反馈
            document.querySelectorAll('.conversation-round.drag-over').forEach(el => {
                el.classList.remove('drag-over');
            });

            if (!this.draggedRoundId || this.draggedRoundId === targetRoundId) {
                return;
            }

            console.log(`[v1.42.0] Drop: moving ${this.draggedRoundId} to ${targetRoundId}`);

            // 找到源和目标的索引
            const draggedIndex = this.rounds.findIndex(r => r.id === this.draggedRoundId);
            const targetIndex = this.rounds.findIndex(r => r.id === targetRoundId);

            if (draggedIndex === -1 || targetIndex === -1) {
                console.error('[v1.42.0 ERROR] Round not found in array');
                return;
            }

            // ⚠️ 重要：在更新数组之前保存目标元素引用
            const targetElement = this.rounds[targetIndex].element;
            const draggedElement = this.draggedElement;

            // 更新 rounds 数组顺序
            const [draggedRound] = this.rounds.splice(draggedIndex, 1);
            this.rounds.splice(targetIndex, 0, draggedRound);

            // 更新 DOM 顺序
            // 插入到目标位置
            if (draggedIndex < targetIndex) {
                // 向下移动：插入到目标元素之后
                targetElement.parentNode.insertBefore(draggedElement, targetElement.nextSibling);
            } else {
                // 向上移动：插入到目标元素之前
                targetElement.parentNode.insertBefore(draggedElement, targetElement);
            }

            console.log(`[v1.42.0] Round reordered successfully`);
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
                    button.textContent = '📊 回合';
                    button.title = '切换到传统流式输出';
                } else {
                    button.textContent = '📜 传统';
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

        // ===== v1.29.0: 意图拆解可视化方法 =====

        // ===== v1.36.2: 渲染态势分析卡片（极简版）=====
        renderSituationAnalysisCard(analysis) {
            const card = document.createElement('div');
            card.className = 'situation-analysis-card';

            // 复杂度和风险的颜色标记
            const riskClass = analysis.risk === 'High' ? 'high-risk' : analysis.risk === 'Medium' ? 'medium-risk' : 'low-risk';

            // 标题行：核心指标一行显示
            const header = `
                <div class="situation-header">
                    📊 <span class="complexity">${analysis.complexity.chinese_name}</span>
                    <span class="divider">·</span>
                    <span class="risk ${riskClass}">${analysis.risk.chinese_name}</span>
                    <span class="divider">·</span>
                    <span class="balance">${analysis.yin_yang_balance.is_balanced ? '平衡' : '失衡'}</span>
                </div>
            `;

            // 总体评价（主要信息）
            const summary = `<div class="situation-main">${analysis.is_ready_to_execute ? '✓' : '⚠'} ${this.escapeHtml(analysis.overall_summary)}</div>`;

            // 问题和建议（只在有时显示，紧凑排列）
            let alerts = '';

            // 严重问题
            const criticalIssues = analysis.sequence_validation.issues?.filter(i => i.severity === 'Critical') || [];
            if (criticalIssues.length > 0) {
                alerts += criticalIssues.map(i => `<div class="alert critical">⛔ ${i.message}</div>`).join('');
            }

            // 警告
            const warnings = analysis.sequence_validation.issues?.filter(i => i.severity === 'Warning') || [];
            if (warnings.length > 0) {
                alerts += warnings.map(i => `<div class="alert warning">⚠️ ${i.message}</div>`).join('');
            }

            // 建议（最多显示2条，避免喧宾夺主）
            if (analysis.suggestions && analysis.suggestions.length > 0) {
                const topSuggestions = analysis.suggestions.slice(0, 2);
                alerts += topSuggestions.map(s => `<div class="alert suggestion">💡 ${this.escapeHtml(s)}</div>`).join('');
            }

            // 组装卡片（极简结构）
            card.innerHTML = `${header}${summary}${alerts}`;

            return card;
        }

        showIntentUnderstanding(msg) {
            // ===== v1.29.2: 存储计划数据 =====
            this.intentPlans.set(msg.plan_id, {
                understanding: msg.understanding,
                stepCount: msg.step_count,
                totalTime: msg.total_time,
                steps: []  // 步骤将在 updateStepProgress 中填充
            });

            // 初始化编辑状态
            this.editMode.set(msg.plan_id, {
                editing: false,
                originalSteps: []
            });

            // 创建意图理解卡片
            const card = document.createElement('div');
            card.className = 'intent-card';
            card.dataset.planId = msg.plan_id;
            card.innerHTML = `
                <div class="intent-header">
                    <span class="intent-icon">🎯</span>
                    <span class="intent-title">意图拆解</span>
                </div>
                <div class="intent-understanding">
                    <div class="understanding-label">💭 AI 理解：</div>
                    <div class="understanding-content">${this.escapeHtml(msg.understanding)}</div>
                </div>
                <div class="intent-meta">
                    <span class="step-count">📋 ${msg.step_count} 个步骤</span>
                    <span class="total-time">⏱️ 预计 ${msg.total_time.toFixed(1)}s</span>
                </div>
                <div class="intent-steps" id="intent-steps-${msg.plan_id}">
                    <!-- 步骤将动态添加 -->
                </div>
                <div class="intent-actions" id="intent-actions-${msg.plan_id}">
                    <button class="intent-edit-btn" data-plan-id="${msg.plan_id}">
                        ✏️ 修改计划
                    </button>
                    <button class="intent-execute-btn" data-plan-id="${msg.plan_id}">
                        ▶️ 执行计划
                    </button>
                </div>
            `;

            // ===== v1.36.2: 插入态势分析卡片（如果有）=====
            if (msg.situation_analysis) {
                const analysisCard = this.renderSituationAnalysisCard(msg.situation_analysis);
                // 插入到意图卡片的开头（header 之后）
                const headerElement = card.querySelector('.intent-header');
                if (headerElement && headerElement.nextSibling) {
                    card.insertBefore(analysisCard, headerElement.nextSibling);
                } else {
                    card.insertBefore(analysisCard, card.firstChild.nextSibling || card.firstChild);
                }
            }

            // ===== v1.29.1 修复：根据视图模式选择容器 =====
            if (this.viewMode === 'round' && this.currentRound && this.currentRound.element) {
                // 回合模式：追加到当前回合的 output-content
                const outputContent = this.currentRound.element.querySelector('.output-content');
                if (outputContent) {
                    outputContent.appendChild(card);
                } else {
                    // 降级：找不到 output-content，追加到根容器
                    this.container.appendChild(card);
                    this.lines.push(card);
                }
            } else {
                // 传统模式（stream）：移除飞轮，在其位置插入意图卡片
                if (this.spinnerLine && this.spinnerLine.parentNode) {
                    // 在飞轮位置插入意图卡片
                    this.spinnerLine.parentNode.insertBefore(card, this.spinnerLine);
                    this.removeSpinner();  // 移除飞轮
                } else {
                    // 降级：没有飞轮，追加到根容器
                    this.container.appendChild(card);
                }
                this.lines.push(card);
            }

            // ===== v1.29.2: 添加编辑按钮事件监听 =====
            const editBtn = card.querySelector('.intent-edit-btn');
            if (editBtn) {
                editBtn.addEventListener('click', () => {
                    this.enterEditMode(msg.plan_id);
                });
            }

            // ===== v1.36.2: 添加执行按钮事件监听 =====
            const executeBtn = card.querySelector('.intent-execute-btn');
            if (executeBtn) {
                executeBtn.addEventListener('click', () => {
                    this.executePlan(msg.plan_id);
                });
            }

            this.scrollToBottom();
        }

        updateStepProgress(msg) {
            const stepsContainer = document.getElementById(`intent-steps-${msg.plan_id}`);
            if (!stepsContainer) return;

            // 查找或创建步骤元素
            let stepElement = document.getElementById(`step-${msg.step_id}`);
            if (!stepElement) {
                stepElement = document.createElement('div');
                stepElement.id = `step-${msg.step_id}`;
                stepElement.className = 'intent-step expanded';
                stepElement.innerHTML = `
                    <div class="step-header" data-step-id="${msg.step_id}">
                        <span class="step-number">[${msg.step_index + 1}]</span>
                        <span class="step-description">${this.escapeHtml(msg.description)}</span>
                        <span class="step-status"></span>
                        <span class="step-toggle">▼</span>
                    </div>
                    <div class="step-details">
                        <div class="step-meta">
                            <span class="step-tool">🔧 ${this.escapeHtml(msg.tool)}</span>
                            <span class="step-time"></span>
                        </div>
                    </div>
                `;
                stepsContainer.appendChild(stepElement);

                // ===== v1.36.2: 添加步骤折叠功能 =====
                const stepHeader = stepElement.querySelector('.step-header');
                const stepToggle = stepElement.querySelector('.step-toggle');
                stepHeader.addEventListener('click', (e) => {
                    // 如果点击的是checkbox，不触发折叠
                    if (e.target.classList.contains('step-checkbox')) return;

                    stepElement.classList.toggle('expanded');
                    stepToggle.textContent = stepElement.classList.contains('expanded') ? '▼' : '▶';
                });

                // ===== v1.29.2: 存储步骤数据 =====
                const plan = this.intentPlans.get(msg.plan_id);
                if (plan) {
                    plan.steps.push({
                        stepId: msg.step_id,
                        stepIndex: msg.step_index,
                        description: msg.description,
                        tool: msg.tool,
                        params: msg.params || null,  // v1.30.0: 保存工具参数
                        status: msg.status,
                        enabled: true  // 默认启用
                    });
                }
            }

            // 更新步骤状态
            const statusSpan = stepElement.querySelector('.step-status');
            const timeSpan = stepElement.querySelector('.step-time');

            switch (msg.status) {
                case 'pending':
                    stepElement.className = 'intent-step pending';
                    statusSpan.textContent = '⏸️';
                    break;
                case 'running':
                    stepElement.className = 'intent-step running';
                    statusSpan.textContent = '⏳';
                    break;
                case 'success':
                    stepElement.className = 'intent-step success';
                    statusSpan.textContent = '✅';
                    if (msg.elapsed_time) {
                        timeSpan.textContent = `⏱️ ${msg.elapsed_time.toFixed(2)}s`;
                    }
                    break;
                case 'failed':
                    stepElement.className = 'intent-step failed';
                    statusSpan.textContent = '❌';
                    if (msg.elapsed_time) {
                        timeSpan.textContent = `⏱️ ${msg.elapsed_time.toFixed(2)}s`;
                    }
                    break;
            }

            this.scrollToBottom();
        }

        showStepComplete(msg) {
            const card = document.querySelector(`[data-plan-id="${msg.plan_id}"]`);
            if (!card) return;

            // 添加完成标记
            const completeDiv = document.createElement('div');
            completeDiv.className = msg.success ? 'intent-complete success' : 'intent-complete failed';
            completeDiv.innerHTML = `
                <div class="complete-icon">${msg.success ? '✅' : '❌'}</div>
                <div class="complete-text">
                    ${msg.success ? '执行成功' : '执行失败'}
                    <span class="complete-time">总用时: ${msg.total_time.toFixed(2)}s</span>
                </div>
            `;
            card.appendChild(completeDiv);

            // 标记卡片为已完成
            card.classList.add('completed');

            this.scrollToBottom();
        }

        // ===== v1.29.2: 编辑模式方法 =====

        enterEditMode(planId) {
            console.log(`[v1.29.2 DEBUG] Entering edit mode for plan: ${planId}`);

            const plan = this.intentPlans.get(planId);
            const editState = this.editMode.get(planId);
            if (!plan || !editState) {
                console.error(`[v1.29.2 ERROR] Plan or edit state not found: ${planId}`);
                return;
            }

            // 备份原始状态
            editState.editing = true;
            editState.originalSteps = plan.steps.map(s => ({...s}));
            this.editMode.set(planId, editState);

            // 重新渲染为编辑模式
            this.renderEditMode(planId);
        }

        exitEditMode(planId) {
            console.log(`[v1.29.2 DEBUG] Exiting edit mode (cancel) for plan: ${planId}`);

            const plan = this.intentPlans.get(planId);
            const editState = this.editMode.get(planId);
            if (!plan || !editState) return;

            // 恢复原始状态
            plan.steps = editState.originalSteps.map(s => ({...s}));
            editState.editing = false;
            editState.originalSteps = [];
            this.editMode.set(planId, editState);

            // 重新渲染为普通模式
            this.renderNormalMode(planId);
        }

        confirmEditMode(planId) {
            console.log(`[v1.29.2 DEBUG] Confirming edit mode for plan: ${planId}`);

            const plan = this.intentPlans.get(planId);
            const editState = this.editMode.get(planId);
            if (!plan || !editState) return;

            // 保存编辑状态
            editState.editing = false;
            editState.originalSteps = [];
            this.editMode.set(planId, editState);

            console.log(`[v1.29.2 DEBUG] Saved plan:`, plan.steps);

            // 重新渲染为普通模式，显示禁用的步骤
            this.renderNormalMode(planId);
        }

        renderEditMode(planId) {
            const card = document.querySelector(`[data-plan-id="${planId}"]`);
            if (!card) return;

            const plan = this.intentPlans.get(planId);
            if (!plan) return;

            // 添加编辑模式标识
            card.classList.add('editing');

            // 更新步骤显示 - 添加 checkbox
            plan.steps.forEach((step, index) => {
                const stepElement = document.getElementById(`step-${step.stepId}`);
                if (!stepElement) return;

                const stepHeader = stepElement.querySelector('.step-header');
                if (!stepHeader) return;

                // 检查是否已有 checkbox
                let checkbox = stepHeader.querySelector('.step-checkbox');
                if (!checkbox) {
                    checkbox = document.createElement('input');
                    checkbox.type = 'checkbox';
                    checkbox.className = 'step-checkbox';
                    checkbox.checked = step.enabled;
                    checkbox.dataset.stepId = step.stepId;

                    // 添加 change 事件监听
                    checkbox.addEventListener('change', (e) => {
                        const stepId = e.target.dataset.stepId;
                        const stepData = plan.steps.find(s => s.stepId === stepId);
                        if (stepData) {
                            stepData.enabled = e.target.checked;
                            console.log(`[v1.29.2 DEBUG] Step ${stepId} enabled: ${stepData.enabled}`);
                        }
                    });

                    // 插入到最前面
                    stepHeader.insertBefore(checkbox, stepHeader.firstChild);
                }
            });

            // 更新按钮区域
            const actionsDiv = card.querySelector('.intent-actions');
            if (actionsDiv) {
                actionsDiv.innerHTML = `
                    <button class="intent-cancel-btn" data-plan-id="${planId}">
                        ❌ 取消
                    </button>
                    <button class="intent-confirm-btn" data-plan-id="${planId}">
                        ✅ 确认
                    </button>
                `;

                // 添加事件监听
                const cancelBtn = actionsDiv.querySelector('.intent-cancel-btn');
                const confirmBtn = actionsDiv.querySelector('.intent-confirm-btn');

                if (cancelBtn) {
                    cancelBtn.addEventListener('click', () => this.exitEditMode(planId));
                }

                if (confirmBtn) {
                    confirmBtn.addEventListener('click', () => this.confirmEditMode(planId));
                }
            }
        }

        renderNormalMode(planId) {
            const card = document.querySelector(`[data-plan-id="${planId}"]`);
            if (!card) return;

            const plan = this.intentPlans.get(planId);
            if (!plan) return;

            // 移除编辑模式标识
            card.classList.remove('editing');

            // 移除 checkbox，更新步骤显示
            plan.steps.forEach(step => {
                const stepElement = document.getElementById(`step-${step.stepId}`);
                if (!stepElement) return;

                // 移除 checkbox
                const checkbox = stepElement.querySelector('.step-checkbox');
                if (checkbox) {
                    checkbox.remove();
                }

                // 如果步骤被禁用，添加视觉反馈
                if (!step.enabled) {
                    stepElement.classList.add('disabled');
                } else {
                    stepElement.classList.remove('disabled');
                }
            });

            // 恢复原始按钮
            const actionsDiv = card.querySelector('.intent-actions');
            if (actionsDiv) {
                actionsDiv.innerHTML = `
                    <button class="intent-edit-btn" data-plan-id="${planId}">
                        ✏️ 修改计划
                    </button>
                    <button class="intent-execute-btn" data-plan-id="${planId}">
                        ▶️ 执行计划
                    </button>
                `;

                // 重新添加事件监听
                const editBtn = actionsDiv.querySelector('.intent-edit-btn');
                if (editBtn) {
                    editBtn.addEventListener('click', () => this.enterEditMode(planId));
                }

                // v1.29.3: 添加执行按钮事件监听
                const executeBtn = actionsDiv.querySelector('.intent-execute-btn');
                if (executeBtn) {
                    executeBtn.addEventListener('click', () => this.executePlan(planId));
                }
            }
        }

        // ===== v1.29.3: 执行计划方法 =====

        executePlan(planId) {
            console.log(`[v1.29.3 DEBUG] Executing plan: ${planId}`);

            const plan = this.intentPlans.get(planId);
            if (!plan) {
                console.error(`[v1.29.3 ERROR] Plan not found: ${planId}`);
                return;
            }

            // 筛选出启用的步骤
            const enabledSteps = plan.steps.filter(step => step.enabled).map(step => ({
                step_id: step.stepId,
                step_index: step.stepIndex,
                description: step.description,
                tool: step.tool,
                params: step.params || null  // v1.29.4: 包含工具参数
            }));

            if (enabledSteps.length === 0) {
                console.warn(`[v1.29.3 WARN] No enabled steps to execute`);
                alert('没有启用的步骤可执行！');
                return;
            }

            console.log(`[v1.29.3 DEBUG] Sending execute_plan with ${enabledSteps.length} steps:`, enabledSteps);

            // 通过回调发送执行请求到后端
            if (this.onExecutePlan) {
                this.onExecutePlan(planId, enabledSteps);
                console.log(`[v1.29.3 DEBUG] Execute plan callback invoked`);
            } else {
                console.error(`[v1.29.3 ERROR] onExecutePlan callback not set`);
                alert('执行回调未设置，无法执行计划！');
            }
        }

        showPlanExecutionStart(msg) {
            console.log(`[v1.29.3 DEBUG] Plan execution started: ${msg.plan_id}, ${msg.enabled_count}/${msg.total_count} steps`);

            // 在意图卡片底部添加执行开始提示
            const card = document.querySelector(`[data-plan-id="${msg.plan_id}"]`);
            if (!card) return;

            const actionsDiv = card.querySelector('.intent-actions');
            if (actionsDiv) {
                // 禁用按钮，显示执行中状态
                const executeBtn = actionsDiv.querySelector('.intent-execute-btn');
                if (executeBtn) {
                    executeBtn.disabled = true;
                    executeBtn.textContent = '⏳ 执行中...';
                    executeBtn.style.opacity = '0.5';
                    executeBtn.style.cursor = 'not-allowed';
                }
            }
        }

        showStepOutput(msg) {
            console.log(`[v1.29.3 DEBUG] Step output: ${msg.step_id}`, msg.output);
            console.log(`[v1.40.0 DEBUG] viewMode: ${this.viewMode}, currentRound:`, this.currentRound);

            // 在步骤下方显示输出
            const stepElement = document.getElementById(`step-${msg.step_id}`);
            if (!stepElement) {
                console.error(`[v1.29.3 ERROR] Step element not found: step-${msg.step_id}`);
                return;
            }
            console.log(`[v1.29.3 DEBUG] Step element found:`, stepElement);

            // 检查是否已有输出容器
            let outputDiv = stepElement.querySelector('.step-output');
            if (!outputDiv) {
                console.log(`[v1.29.3 DEBUG] Creating new .step-output div`);
                outputDiv = document.createElement('div');
                outputDiv.className = 'step-output';
                stepElement.appendChild(outputDiv);
                console.log(`[v1.29.3 DEBUG] .step-output div appended:`, outputDiv);
            } else {
                console.log(`[v1.29.3 DEBUG] Found existing .step-output div:`, outputDiv);
            }

            // 追加输出内容
            const outputPre = document.createElement('pre');
            outputPre.className = 'step-output-content';
            outputPre.textContent = msg.output;
            outputDiv.appendChild(outputPre);

            // v1.40.0 Bug Fix: 在回合模式下，累积步骤输出到 currentRound.aiResponse
            if (this.viewMode === 'round' && this.currentRound) {
                // 累积所有步骤的输出
                if (!this.currentRound.aiResponse) {
                    this.currentRound.aiResponse = '';
                }
                this.currentRound.aiResponse += msg.output + '\n';
                console.log(`[v1.40.0 DEBUG] Accumulated to currentRound.aiResponse, now length: ${this.currentRound.aiResponse.length}`);
            } else {
                console.warn(`[v1.40.0 WARN] Not accumulating: viewMode=${this.viewMode}, currentRound=${!!this.currentRound}`);
            }

            this.scrollToBottom();
        }

        showPlanExecutionComplete(msg) {
            console.log(`[v1.29.3 DEBUG] Plan execution complete: ${msg.plan_id}, success=${msg.success}, executed=${msg.executed_count}, time=${msg.total_time}s`);
            console.log(`[v1.40.0 DEBUG] viewMode: ${this.viewMode}, currentRound:`, this.currentRound);

            // 恢复按钮状态
            const card = document.querySelector(`[data-plan-id="${msg.plan_id}"]`);
            if (!card) {
                console.error(`[v1.29.3 ERROR] Card not found: ${msg.plan_id}`);
                return;
            }
            console.log(`[v1.29.3 DEBUG] Card found:`, card);

            const actionsDiv = card.querySelector('.intent-actions');
            if (actionsDiv) {
                const executeBtn = actionsDiv.querySelector('.intent-execute-btn');
                if (executeBtn) {
                    executeBtn.disabled = false;
                    executeBtn.textContent = '▶️ 执行计划';
                    executeBtn.style.opacity = '1';
                    executeBtn.style.cursor = 'pointer';
                }
            }

            // 在卡片底部添加执行结果摘要
            const summaryDiv = document.createElement('div');
            summaryDiv.className = msg.success ? 'execution-summary success' : 'execution-summary failed';
            summaryDiv.innerHTML = `
                <div class="summary-icon">${msg.success ? '✅' : '⚠️'}</div>
                <div class="summary-text">
                    ${msg.success ? '执行成功' : '执行完成（部分失败）'}
                    <span class="summary-details">
                        执行了 ${msg.executed_count} 个步骤，用时 ${msg.total_time.toFixed(2)}s
                    </span>
                </div>
            `;
            card.appendChild(summaryDiv);

            // v1.40.0 Bug Fix: 在回合模式下，添加执行摘要到 currentRound.aiResponse
            if (this.viewMode === 'round' && this.currentRound) {
                const summary = `\n${msg.success ? '✅ 执行成功' : '⚠️ 执行完成（部分失败）'}\n执行了 ${msg.executed_count} 个步骤，用时 ${msg.total_time.toFixed(2)}s`;
                if (!this.currentRound.aiResponse) {
                    this.currentRound.aiResponse = summary;
                } else {
                    this.currentRound.aiResponse += summary;
                }
                console.log(`[v1.40.0 DEBUG] Added summary to currentRound.aiResponse, now length: ${this.currentRound.aiResponse.length}`);
            } else {
                console.warn(`[v1.40.0 WARN] Not adding summary: viewMode=${this.viewMode}, currentRound=${!!this.currentRound}`);
            }

            this.scrollToBottom();
        }

        // ===== v1.44.0: 图表渲染方法 =====

        /**
         * 渲染 ECharts 图表
         * @param {object} msg - 图表消息 { round_id, chart_data }
         */
        renderChart(msg) {
            console.log(`[v1.45.0 DEBUG] Rendering chart for round: ${msg.round_id}`);

            const chartData = msg.chart_data;

            // v1.45.0: 找到对应的 Round 卡片
            const roundElement = this.outputArea.querySelector(`[data-round-id="${msg.round_id}"]`);
            if (!roundElement) {
                console.error(`[v1.45.0 ERROR] Round element not found for: ${msg.round_id}`);
                return;
            }

            // 找到 Round 卡片内的输出区域
            const outputContent = roundElement.querySelector('.output-content');
            if (!outputContent) {
                console.error(`[v1.45.0 ERROR] Output content not found for round: ${msg.round_id}`);
                return;
            }

            // 创建图表容器
            const chartCard = document.createElement('div');
            chartCard.className = 'chart-card';
            chartCard.setAttribute('data-chart-type', chartData.chart_type);

            // 添加图表标题
            const titleDiv = document.createElement('div');
            titleDiv.className = 'chart-title';
            titleDiv.textContent = chartData.title;
            chartCard.appendChild(titleDiv);

            // 创建 ECharts 容器
            const chartContainer = document.createElement('div');
            chartContainer.className = 'chart-container';
            chartContainer.style.width = '100%';
            chartContainer.style.height = '400px';
            chartCard.appendChild(chartContainer);

            // v1.45.0: 添加到 Round 卡片的输出区域内部（而不是 outputArea）
            outputContent.appendChild(chartCard);

            // 初始化 ECharts
            const chart = echarts.init(chartContainer);

            // 转换为 ECharts option
            const option = this.convertToEChartsOption(chartData);

            // 渲染图表
            chart.setOption(option);

            // 响应式调整
            window.addEventListener('resize', () => {
                chart.resize();
            });

            // 主题切换时重新初始化
            const observer = new MutationObserver(() => {
                const currentTheme = document.getElementById('html-root').getAttribute('data-theme') || 'dark';
                chart.dispose();
                const newChart = echarts.init(chartContainer, currentTheme === 'dark' ? 'dark' : null);
                newChart.setOption(this.convertToEChartsOption(chartData));
            });
            observer.observe(document.getElementById('html-root'), {
                attributes: true,
                attributeFilter: ['data-theme']
            });

            this.scrollToBottom();
        }

        /**
         * 将 RealConsole ChartData 转换为 ECharts option
         * @param {object} chartData - RealConsole 图表数据
         * @returns {object} ECharts option
         */
        convertToEChartsOption(chartData) {
            const currentTheme = document.getElementById('html-root').getAttribute('data-theme') || 'dark';
            const isDark = currentTheme === 'dark';

            // 三色主义配色
            const themeColors = {
                primary: isDark ? '#A371F7' : '#8B5CF6',    // Purple
                success: '#0ECB81',                          // Green
                warning: '#F0B90B',                          // Gold
                text: isDark ? '#C9D1D9' : '#1C1C1C',
                textSecondary: isDark ? '#8B949E' : '#7C7C7C',
                background: isDark ? '#0D1117' : '#FFFFFF',
            };

            const defaultColors = [
                themeColors.primary,
                themeColors.success,
                themeColors.warning,
                '#FF6B9D',  // Pink
                '#4ECDC4',  // Cyan
                '#FFE66D',  // Yellow
            ];

            // v1.45.0: 特殊图表类型判断
            const isPie = chartData.chart_type === 'pie';
            const isScatter = chartData.chart_type === 'scatter';

            return {
                title: {
                    text: chartData.title,
                    textStyle: {
                        color: themeColors.primary,
                        fontSize: 18,
                        fontWeight: 'bold',
                    },
                    left: 'center',
                    top: 10,
                },
                tooltip: {
                    trigger: isPie ? 'item' : 'axis',
                    backgroundColor: isDark ? 'rgba(13, 17, 23, 0.95)' : 'rgba(255, 255, 255, 0.95)',
                    borderColor: themeColors.primary,
                    textStyle: {
                        color: themeColors.text,
                    },
                    // v1.45.0: 饼图 tooltip 格式
                    formatter: isPie ? '{b}: {c} ({d}%)' : undefined,
                },
                legend: {
                    show: chartData.options.show_legend,
                    textStyle: {
                        color: themeColors.textSecondary,
                    },
                    top: 40,
                },
                // v1.45.0: 饼图不需要 grid 和坐标轴
                grid: isPie ? undefined : {
                    left: '10%',
                    right: '10%',
                    bottom: '15%',
                    top: chartData.options.show_legend ? '20%' : '15%',
                    containLabel: true,
                },
                xAxis: isPie ? undefined : {
                    type: chartData.x_axis.axis_type || 'category',
                    data: chartData.x_axis.data,
                    name: chartData.x_axis.name,
                    nameTextStyle: {
                        color: themeColors.textSecondary,
                    },
                    axisLabel: {
                        color: themeColors.text,
                    },
                    axisLine: {
                        lineStyle: {
                            color: themeColors.textSecondary,
                        },
                    },
                },
                yAxis: isPie ? undefined : {
                    type: chartData.y_axis.axis_type || 'value',
                    name: chartData.y_axis.name,
                    nameTextStyle: {
                        color: themeColors.textSecondary,
                    },
                    axisLabel: {
                        color: themeColors.text,
                    },
                    axisLine: {
                        lineStyle: {
                            color: themeColors.textSecondary,
                        },
                    },
                    splitLine: {
                        lineStyle: {
                            color: isDark ? 'rgba(139, 148, 158, 0.2)' : 'rgba(124, 124, 124, 0.2)',
                        },
                    },
                },
                // v1.45.0: 特殊图表类型的数据格式
                series: isPie ? chartData.series.map((s, seriesIndex) => {
                    // 饼图数据格式：[{name, value, itemStyle}]
                    const pieData = s.data.map((value, dataIndex) => ({
                        name: chartData.labels ? chartData.labels[dataIndex] : `项目${dataIndex + 1}`,
                        value: value,
                        itemStyle: {
                            color: defaultColors[dataIndex % defaultColors.length],
                        },
                    }));

                    return {
                        name: s.name,
                        type: 'pie',
                        radius: '60%',
                        center: ['50%', '55%'],
                        data: pieData,
                        label: {
                            color: themeColors.text,
                            formatter: '{b}: {d}%',
                        },
                        emphasis: {
                            itemStyle: {
                                shadowBlur: 10,
                                shadowOffsetX: 0,
                                shadowColor: 'rgba(0, 0, 0, 0.5)',
                            },
                        },
                    };
                }) : isScatter ? chartData.series.map((s, index) => {
                    // v1.45.0: 散点图数据格式：[[x, y], [x, y], ...]
                    return {
                        name: s.name,
                        type: 'scatter',
                        data: s.points || [],  // points 是 [(x, y)] 格式，ECharts 可以直接使用
                        symbolSize: 10,
                        color: s.color || defaultColors[index % defaultColors.length],
                        itemStyle: {
                            borderWidth: 1,
                            borderColor: isDark ? '#0D1117' : '#FFFFFF',
                        },
                        emphasis: {
                            scale: true,
                            scaleSize: 15,
                        },
                    };
                }) : chartData.series.map((s, index) => ({
                    name: s.name,
                    type: chartData.chart_type.toLowerCase(),
                    data: s.data,
                    smooth: chartData.options.smooth,
                    color: s.color || defaultColors[index % defaultColors.length],
                    lineStyle: {
                        width: 2,
                    },
                    itemStyle: {
                        borderWidth: 2,
                    },
                })),
                toolbox: chartData.options.show_toolbox ? {
                    feature: {
                        saveAsImage: {
                            title: '保存图片',
                            iconStyle: {
                                borderColor: themeColors.primary,
                            },
                        },
                        dataZoom: {
                            title: {
                                zoom: '区域缩放',
                                back: '还原',
                            },
                            iconStyle: {
                                borderColor: themeColors.primary,
                            },
                        },
                        restore: {
                            title: '还原',
                            iconStyle: {
                                borderColor: themeColors.primary,
                            },
                        },
                    },
                    right: 20,
                } : undefined,
            };
        }

        // ===== v1.40.0 Phase 2: 浏览器端会话持久化方法 =====

        /**
         * 设置自动保存机制
         */
        setupAutoSave() {
            console.log('[Session] Setting up auto-save...');

            // 页面退出时保存
            window.addEventListener('beforeunload', () => {
                if (this.localStorage.config.save_on_exit && this.rounds.length > 0) {
                    console.log('[Session] Saving session before unload...');
                    this.saveCurrentSession();
                }
            });

            // 定期自动保存（每 5 分钟）
            if (this.localStorage.config.auto_save) {
                setInterval(() => {
                    if (this.rounds.length > 0) {
                        console.log('[Session] Auto-saving session...');
                        this.saveCurrentSession();
                    }
                }, 5 * 60 * 1000); // 5 分钟
            }

            console.log('[Session] Auto-save setup complete');
        }

        /**
         * 保存当前会话到 LocalStorage
         */
        saveCurrentSession() {
            if (this.rounds.length === 0) {
                console.log('[Session] No rounds to save, skipping');
                return;
            }

            // 如果还没有会话 ID，创建一个新的
            if (!this.sessionId) {
                this.sessionId = this.localStorage.generateUUID();
                this.sessionCreatedAt = new Date().toISOString();
                console.log('[Session] Created new session ID:', this.sessionId);
            }

            // 构建会话对象
            const session = {
                id: this.sessionId,
                name: this.generateSessionName(),
                created_at: this.sessionCreatedAt,
                updated_at: new Date().toISOString(),
                conversation_id: this.conversationId || 'local',
                rounds: this.rounds.map(round => ({
                    id: round.id,
                    index: round.index,
                    round_type: round.roundType,
                    user_input: round.userInput,
                    ai_response: round.aiResponse,
                    tools_used: round.toolsUsed || [],
                    execution_time: round.executionTime || 0,
                    status: round.status,
                    timestamp: round.timestamp,
                    model: round.model
                })),
                metadata: {
                    round_count: this.rounds.length,
                    last_input: this.rounds[this.rounds.length - 1]?.userInput || ''
                },
                version: '1.0'
            };

            // 保存到 LocalStorage
            try {
                this.localStorage.saveCurrentSession(session);
                console.log('[Session] Session saved:', session.name, `(${session.rounds.length} rounds)`);
            } catch (error) {
                console.error('[Session] Failed to save session:', error);
            }
        }

        /**
         * 从保存的会话恢复所有 Round
         */
        restoreSession(session) {
            console.log('[Session] Restoring session:', session.name);

            // 恢复会话元数据
            this.sessionId = session.id;
            this.conversationId = session.conversation_id;
            this.sessionCreatedAt = session.created_at;

            // 清空当前内容
            this.clearAll();

            // 恢复所有 Round
            if (session.rounds && session.rounds.length > 0) {
                session.rounds.forEach(round => {
                    this.createRound(round);
                    this.completeRound(round);
                });

                console.log('[Session] Restored', session.rounds.length, 'rounds');
            }

            // 滚动到底部
            this.scrollToBottom();

            // 🐛 Bug Fix: 确保输入框始终可见和可聚焦
            // 恢复会话后，确保输入框正确初始化和聚焦
            if (this.currentInput && this.currentInput.input) {
                // 确保输入框在 DOM 中
                if (!this.container.contains(this.currentInput.line)) {
                    console.warn('[Session] Input field not in DOM, recreating...');
                    this.createInputLine();
                }
                // 聚焦输入框
                this.focusInput();
            } else {
                console.warn('[Session] Input field missing, recreating...');
                this.createInputLine();
            }

            // 显示通知（如果有通知系统的话）
            console.log(`[Session] ✅ Session restored: ${session.name} (${session.rounds?.length || 0} rounds)`);
        }

        /**
         * 智能生成会话名称
         */
        generateSessionName() {
            if (this.rounds.length === 0) {
                return '空会话';
            }

            // 获取第一个 Round 的输入
            const firstInput = this.rounds[0].userInput || '';

            // 截取前 30 个字符作为名称
            let name = firstInput.slice(0, 30);

            // 安全截取 UTF-8 字符串（避免截断 emoji 或多字节字符）
            // 检查最后一个字符是否是高代理项（emoji 的一部分）
            if (name.length > 0) {
                const lastCharCode = name.charCodeAt(name.length - 1);
                if (lastCharCode >= 0xD800 && lastCharCode <= 0xDBFF) {
                    // 这是高代理项，需要移除以避免截断 emoji
                    name = name.slice(0, -1);
                }
            }

            // 如果超过 30 字符，添加省略号
            if (firstInput.length > 30) {
                name += '...';
            }

            // 如果名称为空，使用时间戳
            if (!name || name.trim() === '') {
                const now = new Date();
                name = `会话 ${now.getMonth() + 1}/${now.getDate()} ${now.getHours()}:${String(now.getMinutes()).padStart(2, '0')}`;
            }

            return name;
        }
    }

    // ===== v1.40.0 Phase 3: 会话历史管理 UI =====

    /**
     * BrowserSessionManager - 浏览器端会话历史管理器
     *
     * 功能：管理浏览器 LocalStorage 中的会话历史
     * 存储：浏览器 LocalStorage（单设备、临时）
     * 结构：使用 session-item HTML 结构
     * 特点：快速访问、搜索筛选、导出功能、重命名支持
     *
     * 与 ServerSessionManager 的区别：
     * - ServerSessionManager：服务器端存储，多设备同步，持久化
     * - BrowserSessionManager：浏览器本地存储，单设备，临时缓存
     *
     * 注意：目前 Web 界面主要使用 ServerSessionManager
     */
    class BrowserSessionManager {
        constructor(terminal) {
            this.terminal = terminal;
            this.listContainer = document.getElementById('session-list');
            this.saveBtn = document.getElementById('save-session-btn');
            this.refreshBtn = document.getElementById('refresh-sessions-btn');
            this.clearHistoryBtn = document.getElementById('clear-history-btn');
            this.panelCloseBtn = document.getElementById('session-panel-close');
            this.panel = document.getElementById('session-panel');

            // v1.40.0: 搜索和筛选
            this.searchInput = document.getElementById('session-search');
            this.sortSelect = document.getElementById('session-sort');
            this.currentSearchTerm = '';
            this.currentSortOrder = 'updated_desc';

            this.setupEventListeners();
        }

        setupEventListeners() {
            // 保存当前会话到历史
            if (this.saveBtn) {
                this.saveBtn.onclick = () => this.saveCurrentSession();
            }

            // 刷新会话列表
            if (this.refreshBtn) {
                this.refreshBtn.onclick = () => this.refreshSessionList();
            }

            // v1.40.0: 清空历史
            if (this.clearHistoryBtn) {
                this.clearHistoryBtn.onclick = () => this.clearAllHistory();
            }

            // 关闭面板
            if (this.panelCloseBtn) {
                this.panelCloseBtn.onclick = () => this.closePanel();
            }

            // 点击遮罩层关闭
            const overlay = this.panel?.querySelector('.session-panel-overlay');
            if (overlay) {
                overlay.onclick = () => this.closePanel();
            }

            // v1.40.0: 搜索输入框
            if (this.searchInput) {
                this.searchInput.oninput = (e) => {
                    this.currentSearchTerm = e.target.value.toLowerCase();
                    this.refreshSessionList();
                };
            }

            // v1.40.0: 排序下拉框
            if (this.sortSelect) {
                this.sortSelect.onchange = (e) => {
                    this.currentSortOrder = e.target.value;
                    this.refreshSessionList();
                };
            }
        }

        /**
         * 打开会话管理面板
         */
        openPanel() {
            if (this.panel) {
                this.panel.classList.remove('hidden');
                this.refreshSessionList();
            }
        }

        /**
         * 关闭会话管理面板
         */
        closePanel() {
            if (this.panel) {
                this.panel.classList.add('hidden');
            }
        }

        /**
         * 保存当前会话到历史
         */
        saveCurrentSession() {
            if (this.terminal.rounds.length === 0) {
                this.terminal.toast.warning('无法保存', '当前没有可保存的会话');
                return;
            }

            // 构建会话对象
            const session = {
                id: this.terminal.sessionId || this.terminal.localStorage.generateUUID(),
                name: this.terminal.generateSessionName(),
                created_at: this.terminal.sessionCreatedAt || new Date().toISOString(),
                updated_at: new Date().toISOString(),
                conversation_id: this.terminal.conversationId || 'local',
                rounds: this.terminal.rounds.map(round => ({
                    id: round.id,
                    index: round.index,
                    round_type: round.roundType,
                    user_input: round.userInput,
                    ai_response: round.aiResponse,
                    tools_used: round.toolsUsed || [],
                    execution_time: round.executionTime || 0,
                    status: round.status,
                    timestamp: round.timestamp,
                    model: round.model
                })),
                version: '1.0'
            };

            // 保存到历史
            try {
                this.terminal.localStorage.addToHistory(session);
                console.log('[SessionManager] Session saved to history:', session.name);

                // 刷新列表
                this.refreshSessionList();

                // Toast 成功提示
                this.terminal.toast.success('会话已保存', session.name);
            } catch (error) {
                console.error('[SessionManager] Failed to save session:', error);
                this.terminal.toast.error('保存失败', error.message);
            }
        }

        /**
         * 刷新会话列表
         */
        refreshSessionList() {
            if (!this.listContainer) return;

            // 获取历史会话列表
            let history = this.terminal.localStorage.getHistory();

            if (!history || history.length === 0) {
                this.listContainer.innerHTML = '<div class="session-list-empty">暂无保存的会话</div>';
                return;
            }

            // v1.40.0: 应用搜索筛选
            if (this.currentSearchTerm) {
                history = history.filter(item =>
                    item.name.toLowerCase().includes(this.currentSearchTerm)
                );
            }

            // v1.40.0: 应用排序
            history = this.sortSessions(history, this.currentSortOrder);

            // 检查筛选后是否有结果
            if (history.length === 0) {
                this.listContainer.innerHTML = '<div class="session-list-empty">未找到匹配的会话</div>';
                return;
            }

            // 渲染列表
            this.listContainer.innerHTML = history.map(item => this.renderSessionItem(item)).join('');

            // 绑定事件
            this.bindSessionItemEvents();
        }

        /**
         * 渲染会话列表项
         */
        renderSessionItem(item) {
            return `
                <div class="session-item" data-id="${item.id}">
                    <div class="session-item-header">
                        <span class="session-name">${this.escapeHtml(item.name)}</span>
                        <span class="session-rounds">💬 ${item.round_count} 回合</span>
                    </div>
                    <div class="session-item-meta">
                        <span class="session-time">${this.formatTime(item.updated_at)}</span>
                        <span class="session-size">${this.formatSize(item.size)}</span>
                    </div>
                    <div class="session-item-actions">
                        <button class="session-load-btn" data-id="${item.id}" title="加载">📂 加载</button>
                        <button class="session-rename-btn" data-id="${item.id}" title="重命名">✏️ 重命名</button>
                        <button class="session-export-btn" data-id="${item.id}" title="导出">📤 导出</button>
                        <button class="session-delete-btn" data-id="${item.id}" title="删除">🗑️ 删除</button>
                    </div>
                </div>
            `;
        }

        /**
         * 绑定会话列表项的事件
         */
        bindSessionItemEvents() {
            // 加载按钮
            const loadBtns = this.listContainer.querySelectorAll('.session-load-btn');
            loadBtns.forEach(btn => {
                btn.onclick = () => this.loadSession(btn.dataset.id);
            });

            // 重命名按钮 (v1.40.0)
            const renameBtns = this.listContainer.querySelectorAll('.session-rename-btn');
            renameBtns.forEach(btn => {
                btn.onclick = () => this.renameSession(btn.dataset.id);
            });

            // 导出按钮
            const exportBtns = this.listContainer.querySelectorAll('.session-export-btn');
            exportBtns.forEach(btn => {
                btn.onclick = () => this.exportSession(btn.dataset.id);
            });

            // 删除按钮
            const deleteBtns = this.listContainer.querySelectorAll('.session-delete-btn');
            deleteBtns.forEach(btn => {
                btn.onclick = () => this.deleteSession(btn.dataset.id);
            });
        }

        /**
         * 加载历史会话
         */
        loadSession(id) {
            // 从 LocalStorage 加载完整会话数据
            const fullSessionKey = `realconsole_session_${id}`;
            const sessionJson = localStorage.getItem(fullSessionKey);

            if (!sessionJson) {
                this.terminal.toast.error('加载失败', '会话不存在');
                return;
            }

            const session = JSON.parse(sessionJson);

            // 如果当前有未保存的会话，提示用户
            if (this.terminal.rounds.length > 0) {
                if (!confirm('加载会话将覆盖当前内容，是否继续？\n\n建议先保存当前会话。')) {
                    return;
                }
            }

            // 恢复会话
            this.terminal.restoreSession(session);

            // 关闭面板
            this.closePanel();

            // Toast 成功提示
            this.terminal.toast.success('会话已加载', session.name);
            console.log('[SessionManager] Session loaded:', session.name);
        }

        /**
         * 重命名会话 (v1.40.0)
         */
        renameSession(id) {
            // 从 LocalStorage 加载完整会话数据
            const fullSessionKey = `realconsole_session_${id}`;
            const sessionJson = localStorage.getItem(fullSessionKey);

            if (!sessionJson) {
                this.terminal.toast.error('重命名失败', '会话不存在');
                return;
            }

            const session = JSON.parse(sessionJson);
            const oldName = session.name;

            // 提示用户输入新名称
            const newName = prompt('请输入新的会话名称:', oldName);

            // 用户取消或输入空名称
            if (!newName || newName.trim() === '') {
                return;
            }

            const trimmedName = newName.trim();

            // 名称未改变
            if (trimmedName === oldName) {
                this.terminal.toast.info('无需重命名', '名称未改变');
                return;
            }

            try {
                // 更新会话名称
                session.name = trimmedName;
                session.updated_at = new Date().toISOString();

                // 保存回 LocalStorage
                localStorage.setItem(fullSessionKey, JSON.stringify(session));

                // 更新历史索引中的名称
                this.terminal.localStorage.updateHistoryItemName(id, trimmedName);

                console.log('[SessionManager] Session renamed:', oldName, '->', trimmedName);

                // 刷新列表
                this.refreshSessionList();

                // Toast 成功提示
                this.terminal.toast.success('重命名成功', trimmedName);
            } catch (error) {
                console.error('[SessionManager] Failed to rename session:', error);
                this.terminal.toast.error('重命名失败', error.message);
            }
        }

        /**
         * 删除历史会话
         */
        deleteSession(id) {
            if (!confirm('确认删除此会话？此操作无法撤销。')) {
                return;
            }

            try {
                this.terminal.localStorage.deleteHistoryItem(id);
                console.log('[SessionManager] Session deleted:', id);

                // 刷新列表
                this.refreshSessionList();

                // Toast 成功提示
                this.terminal.toast.success('会话已删除', '会话已从历史记录中移除');
            } catch (error) {
                console.error('[SessionManager] Failed to delete session:', error);
                this.terminal.toast.error('删除失败', error.message);
            }
        }

        /**
         * 清空所有历史会话 (v1.40.0)
         */
        clearAllHistory() {
            // 获取当前历史数量
            const history = this.terminal.localStorage.getHistory();
            if (!history || history.length === 0) {
                this.terminal.toast.info('无需清空', '当前没有保存的会话');
                return;
            }

            // 二次确认，避免误操作
            const confirmMessage = `确认清空所有历史会话？\n\n当前有 ${history.length} 个会话将被永久删除。\n\n此操作无法撤销！`;
            if (!confirm(confirmMessage)) {
                return;
            }

            try {
                // 清空历史
                this.terminal.localStorage.clearHistory();
                console.log('[SessionManager] All history cleared');

                // 刷新列表
                this.refreshSessionList();

                // 清空搜索框
                if (this.searchInput) {
                    this.searchInput.value = '';
                    this.currentSearchTerm = '';
                }

                // Toast 成功提示
                this.terminal.toast.success('历史已清空', `已删除 ${history.length} 个会话`);
            } catch (error) {
                console.error('[SessionManager] Failed to clear history:', error);
                this.terminal.toast.error('清空失败', error.message);
            }
        }

        /**
         * 导出会话（浏览器端实现，支持 Markdown 和 JSON 格式）
         */
        exportSession(id) {
            // 从 LocalStorage 加载完整会话数据
            const fullSessionKey = `realconsole_session_${id}`;
            const sessionJson = localStorage.getItem(fullSessionKey);

            if (!sessionJson) {
                this.terminal.toast.error('导出失败', '会话不存在');
                return;
            }

            const session = JSON.parse(sessionJson);

            // 询问导出格式
            const format = prompt('请选择导出格式:\n\n1. Markdown (适合阅读和分享)\n2. JSON (适合备份和恢复)\n\n请输入 1 或 2:', '1');

            if (!format) return; // 用户取消

            if (format === '1') {
                this.exportAsMarkdown(session);
            } else if (format === '2') {
                this.exportAsJSON(session);
            } else {
                this.terminal.toast.warning('导出取消', '无效的格式选择');
            }
        }

        /**
         * 导出为 Markdown 格式
         */
        exportAsMarkdown(session) {
            let markdown = `# ${session.name}\n\n`;
            markdown += `**创建时间**: ${new Date(session.created_at).toLocaleString('zh-CN')}\n`;
            markdown += `**更新时间**: ${new Date(session.updated_at).toLocaleString('zh-CN')}\n`;
            markdown += `**回合数**: ${session.rounds.length}\n\n`;
            markdown += `---\n\n`;

            session.rounds.forEach((round, index) => {
                markdown += `## 回合 ${index + 1}\n\n`;

                // 用户输入
                if (round.user_input) {
                    markdown += `**用户**: ${round.user_input}\n\n`;
                }

                // AI 响应
                if (round.ai_response) {
                    const roundTypeLabel = round.round_type === 'llm' ? 'AI' :
                                         round.round_type === 'shell' ? 'Shell' : 'System';
                    markdown += `**${roundTypeLabel}**:\n\n`;
                    markdown += '```\n';
                    markdown += round.ai_response;
                    markdown += '\n```\n\n';
                }

                // 元数据
                if (round.timestamp) {
                    markdown += `*时间: ${new Date(round.timestamp).toLocaleString('zh-CN')}*\n\n`;
                }

                markdown += `---\n\n`;
            });

            // 下载文件
            const filename = `${this.sanitizeFilename(session.name)}.md`;
            this.downloadFile(filename, markdown, 'text/markdown');

            this.terminal.toast.success('导出成功', `已导出为 ${filename}`);
            console.log('[SessionManager] Session exported as Markdown:', filename);
        }

        /**
         * 导出为 JSON 格式
         */
        exportAsJSON(session) {
            const jsonContent = JSON.stringify(session, null, 2);
            const filename = `${this.sanitizeFilename(session.name)}.json`;

            this.downloadFile(filename, jsonContent, 'application/json');

            this.terminal.toast.success('导出成功', `已导出为 ${filename}`);
            console.log('[SessionManager] Session exported as JSON:', filename);
        }

        /**
         * 下载文件到浏览器
         */
        downloadFile(filename, content, mimeType = 'text/plain') {
            const blob = new Blob([content], { type: mimeType });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        }

        /**
         * 清理文件名（移除不安全字符）
         */
        sanitizeFilename(name) {
            return name.replace(/[^a-zA-Z0-9_\-\u4e00-\u9fa5]/g, '_');
        }

        /**
         * 排序会话列表 (v1.40.0)
         */
        sortSessions(sessions, sortOrder) {
            const sorted = [...sessions]; // 创建副本避免修改原数组

            switch (sortOrder) {
                case 'updated_desc':
                    sorted.sort((a, b) => new Date(b.updated_at) - new Date(a.updated_at));
                    break;
                case 'updated_asc':
                    sorted.sort((a, b) => new Date(a.updated_at) - new Date(b.updated_at));
                    break;
                case 'created_desc':
                    sorted.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
                    break;
                case 'created_asc':
                    sorted.sort((a, b) => new Date(a.created_at) - new Date(b.created_at));
                    break;
                case 'rounds_desc':
                    sorted.sort((a, b) => b.round_count - a.round_count);
                    break;
                case 'rounds_asc':
                    sorted.sort((a, b) => a.round_count - b.round_count);
                    break;
                default:
                    // 默认按更新时间降序
                    sorted.sort((a, b) => new Date(b.updated_at) - new Date(a.updated_at));
            }

            return sorted;
        }

        /**
         * 格式化时间
         */
        formatTime(isoString) {
            const date = new Date(isoString);
            const now = new Date();
            const diffMs = now - date;
            const diffMins = Math.floor(diffMs / 60000);
            const diffHours = Math.floor(diffMs / 3600000);
            const diffDays = Math.floor(diffMs / 86400000);

            if (diffMins < 1) return '刚刚';
            if (diffMins < 60) return `${diffMins} 分钟前`;
            if (diffHours < 24) return `${diffHours} 小时前`;
            if (diffDays < 7) return `${diffDays} 天前`;

            // 超过 7 天，显示具体日期
            return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
        }

        /**
         * 格式化大小
         */
        formatSize(bytes) {
            if (bytes < 1024) return bytes + ' B';
            if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
            return (bytes / 1024 / 1024).toFixed(1) + ' MB';
        }

        /**
         * HTML 转义
         */
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
            'web.header.title': '🌟 RealConsole 睿境',
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
            'web.header.title': '🌟 RealConsole Notebook',
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
        // 更新下拉框选中值
        const langSelect = document.getElementById('lang-select');
        if (langSelect) {
            langSelect.value = lang;
        }
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

    // 绑定语言下拉框事件
    const langSelect = document.getElementById('lang-select');
    if (langSelect) {
        langSelect.value = currentLanguage;
        langSelect.addEventListener('change', (e) => {
            setLanguage(e.target.value);
        });
    }

    // ========== 终端核心 ==========

    // 创建混合终端
    const terminal = new HybridTerminal(document.getElementById('terminal-container'));

    // ===== v1.40.0: 初始化 Toast 通知系统 =====
    const toastManager = new ToastManager();
    // 将 ToastManager 挂载到 terminal 对象，便于全局访问
    terminal.toast = toastManager;

    // ===== v1.40.0 Phase 2: 自动恢复会话 =====
    if (terminal.localStorage.config.auto_restore) {
        const savedSession = terminal.localStorage.loadCurrentSession();
        if (savedSession && savedSession.rounds && savedSession.rounds.length > 0) {
            console.log('[Session] Auto-restoring session:', savedSession.name);
            terminal.restoreSession(savedSession);
        } else {
            console.log('[Session] No saved session to restore');
        }
    } else {
        console.log('[Session] Auto-restore disabled in config');
    }

    // ===== v1.40.0 Phase 3: 初始化会话历史管理器（浏览器端）=====
    const browserSessionManager = new BrowserSessionManager(terminal);

    // 绑定会话管理按钮（浏览器端历史管理）
    const sessionMenuBtn = document.getElementById('session-menu-btn');
    if (sessionMenuBtn) {
        sessionMenuBtn.addEventListener('click', () => {
            browserSessionManager.openPanel();
        });
    }

    // ===== v1.28.0: 绑定视图模式切换按钮 =====
    const viewModeToggle = document.getElementById('view-mode-toggle');
    if (viewModeToggle) {
        viewModeToggle.addEventListener('click', () => {
            terminal.toggleViewMode();
        });
    }

    // ===== v1.40.0: 绑定清空按钮 =====
    const clearScreenBtn = document.getElementById('clear-screen-btn');
    if (clearScreenBtn) {
        clearScreenBtn.addEventListener('click', () => {
            // 如果有未保存的内容，提示用户
            if (terminal.rounds.length > 0) {
                if (confirm('确认清空当前对话？\n\n未保存的内容将会丢失。')) {
                    terminal.clearAll();
                    terminal.toast.success('对话已清空', '可以开始新的对话了');
                }
            } else {
                terminal.toast.info('无需清空', '当前没有对话内容');
            }
        });
    }

    // WebSocket 连接
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    const ws = new WebSocket(wsUrl);

    // v1.38.0: 保存 WebSocket 引用到 terminal 对象（用于重新执行功能）
    terminal.ws = ws;

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

        // v1.40.0: 初始化 SessionManager
        terminal.sessionManager = new ServerSessionManager(terminal, ws);

        // v1.46.0: 初始化 FileUploadManager
        terminal.fileUploadManager = new FileUploadManager(ws);

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

        // v1.29.3: Debug all messages
        console.log(`[WS Message] type: ${msg.type}`, msg);

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

            // ===== v1.36.0: 态势测算分析消息 =====
            case 'divination_start':
                // 态势分析开始（起卦）
                terminal.currentDivination = new DivinationAnimation(terminal.outputArea);
                terminal.currentDivination.start(msg.plan_id);
                break;

            case 'divination_step':
                // 演算步骤（实时动画）
                if (terminal.currentDivination) {
                    terminal.currentDivination.showYarrowStep(msg.step);
                }
                break;

            case 'divination_hexagram':
                // 卦象生成
                if (terminal.currentDivination) {
                    terminal.currentDivination.showHexagram(msg.hexagram);
                }
                break;

            case 'divination_complete':
                // 态势分析完成
                if (terminal.currentDivination) {
                    const hexagramCard = terminal.currentDivination.complete(msg.result);

                    // 将卦象卡片保存到意图计划中，稍后在 intent_understanding 中使用
                    if (!terminal.intentPlans.has(msg.plan_id)) {
                        terminal.intentPlans.set(msg.plan_id, {});
                    }
                    const planData = terminal.intentPlans.get(msg.plan_id);
                    planData.hexagramCard = hexagramCard;

                    terminal.currentDivination = null;
                }
                break;

            // ===== v1.29.0: 意图拆解可视化消息 =====
            case 'intent_understanding':
                // 意图理解：显示AI对用户意图的理解
                terminal.showIntentUnderstanding(msg);
                break;

            case 'step_progress':
                // 步骤进度：更新执行步骤的状态
                terminal.updateStepProgress(msg);
                break;

            case 'step_complete':
                // 执行完成：显示最终结果
                terminal.showStepComplete(msg);
                break;

            // ===== v1.29.3: 计划执行消息 =====
            case 'plan_execution_start':
                // 计划执行开始
                terminal.showPlanExecutionStart(msg);
                break;

            case 'step_output':
                // 步骤输出
                terminal.showStepOutput(msg);
                break;

            case 'plan_execution_complete':
                // 计划执行完成
                terminal.showPlanExecutionComplete(msg);
                break;

            // ===== v1.40.0: 会话管理消息 =====
            case 'session_saved':
                if (terminal.sessionManager) {
                    terminal.sessionManager.handleSessionSaved(msg);
                }
                break;

            case 'session_loaded':
                if (terminal.sessionManager) {
                    terminal.sessionManager.handleSessionLoaded(msg);
                }
                break;

            case 'session_list':
                if (terminal.sessionManager) {
                    terminal.sessionManager.handleSessionList(msg);
                }
                break;

            case 'session_deleted':
                if (terminal.sessionManager) {
                    terminal.sessionManager.handleSessionDeleted(msg);
                }
                break;

            case 'session_exported':
                if (terminal.sessionManager) {
                    terminal.sessionManager.handleSessionExported(msg);
                }
                break;

            case 'session_error':
                if (terminal.sessionManager) {
                    terminal.sessionManager.handleSessionError(msg);
                }
                break;

            // ===== v1.44.0: 可视化消息 =====
            case 'chart':
                // 图表数据：渲染 ECharts 图表
                terminal.renderChart(msg);
                break;

            // ===== v1.46.0: 文件上传消息 =====
            case 'file_uploaded':
                // 文件上传成功
                if (terminal.fileUploadManager) {
                    terminal.fileUploadManager.handleFileUploaded(msg);
                }
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

    // v1.29.3: 设置执行计划回调
    terminal.onExecutePlan = (planId, enabledSteps) => {
        ws.send(JSON.stringify({
            type: 'execute_plan',
            plan_id: planId,
            enabled_steps: enabledSteps
        }));
    };

    // ========== 文件上传管理器 (v1.46.0) ==========
    class FileUploadManager {
        constructor(ws) {
            this.ws = ws;
            this.uploadedFiles = new Map(); // file_id -> file info
            this.init();
        }

        init() {
            const uploadArea = document.getElementById('upload-area');
            const fileInput = document.getElementById('file-input');
            const uploadStatus = document.getElementById('upload-status');

            if (!uploadArea || !fileInput) return;

            // 点击上传
            uploadArea.addEventListener('click', () => {
                fileInput.click();
            });

            // 文件选择
            fileInput.addEventListener('change', (e) => {
                if (e.target.files && e.target.files[0]) {
                    this.uploadFile(e.target.files[0]);
                }
            });

            // 拖拽上传
            uploadArea.addEventListener('dragover', (e) => {
                e.preventDefault();
                uploadArea.classList.add('dragover');
            });

            uploadArea.addEventListener('dragleave', () => {
                uploadArea.classList.remove('dragover');
            });

            uploadArea.addEventListener('drop', (e) => {
                e.preventDefault();
                uploadArea.classList.remove('dragover');

                if (e.dataTransfer.files && e.dataTransfer.files[0]) {
                    this.uploadFile(e.dataTransfer.files[0]);
                }
            });
        }

        uploadFile(file) {
            const uploadStatus = document.getElementById('upload-status');

            // 验证文件类型
            if (!file.name.toLowerCase().endsWith('.csv')) {
                this.showStatus('error', '❌ 只支持 CSV 文件');
                return;
            }

            // 验证文件大小（1MB）
            if (file.size > 1024 * 1024) {
                this.showStatus('error', `❌ 文件过大: ${(file.size / 1024 / 1024).toFixed(2)}MB，最大 1MB`);
                return;
            }

            // 显示加载状态
            this.showStatus('loading', `📤 正在上传 ${file.name}...`);

            // 读取文件内容
            const reader = new FileReader();
            reader.onload = (e) => {
                const content = e.target.result;

                // 发送到后端
                this.ws.send(JSON.stringify({
                    type: 'upload_file',
                    filename: file.name,
                    content: content
                }));
            };

            reader.onerror = () => {
                this.showStatus('error', '❌ 文件读取失败');
            };

            reader.readAsText(file);
        }

        handleFileUploaded(msg) {
            const { file_id, filename, preview } = msg;

            // 保存文件信息
            this.uploadedFiles.set(file_id, {
                id: file_id,
                filename,
                preview,
                uploadedAt: new Date()
            });

            // 显示成功状态
            this.showStatus('success',
                `✅ ${filename} 上传成功！文件 ID: ${file_id}\\n` +
                `数据: ${preview.total_rows} 行 × ${preview.total_columns} 列`
            );

            // 更新文件列表
            this.updateFilesList();

            // 3秒后隐藏状态
            setTimeout(() => {
                this.hideStatus();
            }, 3000);
        }

        showStatus(type, message) {
            const uploadStatus = document.getElementById('upload-status');
            if (!uploadStatus) return;

            uploadStatus.className = `upload-status ${type}`;
            uploadStatus.textContent = message;
            uploadStatus.classList.remove('hidden');
        }

        hideStatus() {
            const uploadStatus = document.getElementById('upload-status');
            if (uploadStatus) {
                uploadStatus.classList.add('hidden');
            }
        }

        updateFilesList() {
            const uploadedFilesEl = document.getElementById('uploaded-files');
            const filesListEl = document.getElementById('files-list');

            if (!uploadedFilesEl || !filesListEl) return;

            if (this.uploadedFiles.size === 0) {
                uploadedFilesEl.classList.add('hidden');
                return;
            }

            uploadedFilesEl.classList.remove('hidden');
            filesListEl.innerHTML = '';

            // 按上传时间倒序
            const files = Array.from(this.uploadedFiles.values())
                .sort((a, b) => b.uploadedAt - a.uploadedAt);

            files.forEach(file => {
                const fileItem = document.createElement('div');
                fileItem.className = 'file-item';
                fileItem.innerHTML = `
                    <div class="file-info">
                        <svg class="file-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
                            <polyline points="13 2 13 9 20 9"></polyline>
                        </svg>
                        <div class="file-details">
                            <div class="file-name">${file.filename}</div>
                            <div class="file-meta">${file.preview.total_rows} 行 × ${file.preview.total_columns} 列 | ID: ${file.id}</div>
                        </div>
                    </div>
                    <div class="file-actions">
                        <button class="file-action-btn btn-preview" data-file-id="${file.id}">👁️ 预览</button>
                        <button class="file-action-btn btn-copy" data-file-id="${file.id}">📋 复制命令</button>
                    </div>
                `;

                // 预览按钮
                const previewBtn = fileItem.querySelector('.btn-preview');
                previewBtn.addEventListener('click', () => {
                    this.showPreview(file);
                });

                // 复制命令按钮
                const copyBtn = fileItem.querySelector('.btn-copy');
                copyBtn.addEventListener('click', () => {
                    this.copyChartCommand(file);
                });

                filesListEl.appendChild(fileItem);
            });
        }

        showPreview(file) {
            const { preview } = file;

            // 构建表格 HTML
            let tableHtml = '<table style="width: 100%; border-collapse: collapse; margin: 10px 0; font-size: 13px;">';

            // Header
            tableHtml += '<tr style="background: rgba(163, 113, 247, 0.2);">';
            preview.headers.forEach(h => {
                tableHtml += `<th style="padding: 8px; border: 1px solid rgba(163, 113, 247, 0.3); font-weight: 600;">${h}</th>`;
            });
            tableHtml += '</tr>';

            // Rows
            preview.rows.forEach(row => {
                tableHtml += '<tr>';
                row.forEach(cell => {
                    tableHtml += `<td style="padding: 8px; border: 1px solid rgba(163, 113, 247, 0.2);">${cell}</td>`;
                });
                tableHtml += '</tr>';
            });

            tableHtml += '</table>';

            if (preview.total_rows > 10) {
                tableHtml += `<p style="color: #7D8590; font-size: 12px; margin: 5px 0;">仅显示前 10 行，共 ${preview.total_rows} 行</p>`;
            }

            // 在终端中显示
            terminal.writeOutput(`\\n**📊 数据预览: ${file.filename}**\\n\\n${tableHtml}`);
        }

        copyChartCommand(file) {
            // 获取第一个数值列作为示例
            const { preview } = file;
            const xCol = preview.headers[0] || 'col1';
            const yCol = preview.headers[1] || 'col2';

            const command = `!chart csv @${file.id} --type line --x-col "${xCol}" --y-col "${yCol}"`;

            // 复制到剪贴板
            navigator.clipboard.writeText(command).then(() => {
                this.showStatus('success', `✅ 已复制命令到剪贴板:\\n${command}`);
                setTimeout(() => this.hideStatus(), 2000);
            }).catch(() => {
                // 降级方案：显示命令让用户手动复制
                terminal.writeOutput(`\\n**📋 图表命令示例**\\n\\n\`\`\`bash\\n${command}\\n\`\`\`\\n\\n你可以修改列名和图表类型（line/bar/pie/scatter）`);
            });
        }
    }

    // ========== 主题切换系统 (v1.43.0) ==========
    let currentTheme = 'dark'; // 默认深色主题

    // 从 LocalStorage 加载主题偏好
    function loadTheme() {
        const savedTheme = localStorage.getItem('realconsole-theme');
        if (savedTheme === 'light' || savedTheme === 'dark') {
            currentTheme = savedTheme;
        }
        applyTheme(currentTheme);
    }

    // 应用主题
    function applyTheme(theme) {
        const htmlRoot = document.getElementById('html-root');
        const themeBtn = document.getElementById('theme-toggle-btn');

        if (theme === 'light') {
            htmlRoot.setAttribute('data-theme', 'light');
            if (themeBtn) {
                themeBtn.innerHTML = '☀️ 浅色';
                themeBtn.title = '切换到深色主题';
            }
        } else {
            htmlRoot.removeAttribute('data-theme');
            if (themeBtn) {
                themeBtn.innerHTML = '🌙 深色';
                themeBtn.title = '切换到浅色主题';
            }
        }
    }

    // 切换主题
    function toggleTheme() {
        currentTheme = currentTheme === 'dark' ? 'light' : 'dark';
        applyTheme(currentTheme);
        localStorage.setItem('realconsole-theme', currentTheme);
    }

    // ========== 初始化 ==========
    // 页面加载完成后立即应用设置
    window.addEventListener('DOMContentLoaded', () => {
        // 初始化主题
        loadTheme();

        // 绑定主题切换按钮事件
        const themeToggleBtn = document.getElementById('theme-toggle-btn');
        if (themeToggleBtn) {
            themeToggleBtn.addEventListener('click', toggleTheme);
        }

        // 初始化 i18n
        updatePageText();
    });

})();
"#;


/// 内嵌的样式 CSS
const STYLE_CSS: &str = r#"
/* ============================================
   🌓 Theme System - RealConsole v1.43.0
   支持深色/浅色主题切换（币安风格）
   ============================================ */

/* ===== CSS 变量定义：深色主题（默认） - 三色主义配色 ===== */
:root {
    /* 背景色 */
    --bg-primary: #0a0e27;
    --bg-secondary: #0d1117;
    --bg-tertiary: #1a0b2e;
    --bg-grid: rgba(163, 113, 247, 0.03);
    --bg-scanline: rgba(0, 0, 0, 0.15);

    /* 表面色（卡片、面板） */
    --surface-primary: rgba(10, 14, 39, 0.6);
    --surface-secondary: rgba(5, 8, 20, 0.85);
    --surface-tertiary: rgba(22, 27, 34, 0.5);
    --surface-overlay: rgba(0, 0, 0, 0.7);

    /* 文本色 */
    --text-primary: #E6EDF3;
    --text-secondary: #8B949E;
    --text-muted: #888;
    --text-title: #A371F7;
    --text-gradient-start: #A371F7;
    --text-gradient-end: #C8A2F0;

    /* 边框色 - 统一紫色系 */
    --border-primary: rgba(163, 113, 247, 0.3);
    --border-secondary: rgba(163, 113, 247, 0.2);
    --border-muted: rgba(230, 237, 243, 0.2);
    --border-hover: rgba(163, 113, 247, 0.5);

    /* 主色调（紫色 - 智慧） */
    --accent-primary: #A371F7;
    --accent-primary-alpha-10: rgba(163, 113, 247, 0.1);
    --accent-primary-alpha-30: rgba(163, 113, 247, 0.3);
    --accent-primary-alpha-60: rgba(163, 113, 247, 0.6);

    /* 功能色 - 三色系统 */
    --color-success: #39ff14;        /* 绿色 - 生机/成功 */
    --color-success-soft: #7ee787;
    --color-warning: #F0B90B;        /* 金色 - 警示 */
    --color-error: #ff7b72;          /* 红色 - 错误 */
    --color-error-soft: #ff7b72;
    --color-active: #39ff14;         /* 绿色 - 活跃状态 */
    --color-link: #A371F7;           /* 紫色 - 链接（统一） */

    /* 阴影和发光 - 统一紫色系 */
    --shadow-glow-primary: 0 0 20px rgba(163, 113, 247, 0.3);
    --shadow-glow-purple: 0 0 15px rgba(163, 113, 247, 0.25);
    --shadow-card: 0 0 15px rgba(163, 113, 247, 0.15);

    /* 终端色 */
    --terminal-output: rgb(240, 240, 240);
    --terminal-command: #A371F7;     /* 紫色 - 统一 */
    --terminal-prompt: #F0B90B;
    --terminal-input-bg: rgba(22, 27, 34, 0.5);
    --terminal-border: #30363D;

    /* 特殊效果 */
    --scanline-opacity: 0.3;
    --backdrop-blur: blur(10px);
}

/* ===== 浅色主题（Reddit 风格 + 紫色主题） - 三色主义配色 ===== */
[data-theme="light"] {
    /* 背景色 - Reddit 风格：浅蓝灰主背景 + 纯白卡片 */
    --bg-primary: #DAE0E6;           /* Reddit 浅蓝灰主背景 */
    --bg-secondary: #F7F9FA;         /* 次要背景 */
    --bg-tertiary: #E9EDF1;          /* 第三级背景 */
    --bg-grid: rgba(0, 0, 0, 0.015); /* 极浅网格 */
    --bg-scanline: rgba(0, 0, 0, 0);

    /* 表面色（卡片、面板） - Reddit 风格纯白卡片 */
    --surface-primary: #FFFFFF;      /* 纯白卡片背景 */
    --surface-secondary: #FFFFFF;    /* 统一纯白 */
    --surface-tertiary: #F8F9FA;     /* 悬停/次要表面 */
    --surface-overlay: rgba(0, 0, 0, 0.5);

    /* 文本色 - Reddit 风格高对比度 */
    --text-primary: #1C1C1C;         /* Reddit 深灰黑主文字 */
    --text-secondary: #7C7C7C;       /* Reddit 中灰次要文字 */
    --text-muted: #A8A8A8;           /* Reddit 浅灰弱化文字 */
    --text-title: #8B5CF6;           /* 保留紫色主题 */
    --text-gradient-start: #8B5CF6;
    --text-gradient-end: #9065DC;

    /* 边框色 - Reddit 风格极浅边框 */
    --border-primary: #EDEFF1;       /* Reddit 浅边框 */
    --border-secondary: #CCCCCC;     /* 次要边框 */
    --border-muted: #F0F0F0;         /* 极浅边框 */
    --border-hover: #B3B3B3;         /* 悬停边框 */

    /* 主色调（紫色 - 智慧） */
    --accent-primary: #8B5CF6;
    --accent-primary-alpha-10: rgba(139, 92, 246, 0.1);
    --accent-primary-alpha-30: rgba(139, 92, 246, 0.3);
    --accent-primary-alpha-60: rgba(139, 92, 246, 0.6);

    /* 功能色 - 三色系统 */
    --color-success: #0ECB81;        /* 绿色 - 生机/成功 */
    --color-success-soft: #0ECB81;
    --color-warning: #F0B90B;        /* 金色 - 警示 */
    --color-error: #F6465D;          /* 红色 - 错误 */
    --color-error-soft: #F6465D;
    --color-active: #0ECB81;         /* 绿色 - 活跃状态 */
    --color-link: #8B5CF6;           /* 紫色 - 链接（统一） */

    /* 阴影和发光 - 统一样式 */
    --shadow-glow-primary: 0 2px 8px rgba(0, 0, 0, 0.08);
    --shadow-glow-purple: 0 2px 8px rgba(0, 0, 0, 0.08);
    --shadow-card: 0 2px 8px rgba(0, 0, 0, 0.08);

    /* 终端色 */
    --terminal-output: #1E2329;
    --terminal-command: #8B5CF6;     /* 紫色 - 统一 */
    --terminal-prompt: #F0B90B;
    --terminal-input-bg: #FFFFFF;
    --terminal-border: #EAECEF;

    /* 特殊效果 */
    --scanline-opacity: 0;
    --backdrop-blur: blur(0px);
}

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
    /* 主题化背景 + 动态网格 */
    background:
        repeating-linear-gradient(
            0deg,
            var(--bg-grid) 0px,
            transparent 1px,
            transparent 40px,
            var(--bg-grid) 41px
        ),
        repeating-linear-gradient(
            90deg,
            var(--bg-grid) 0px,
            transparent 1px,
            transparent 40px,
            var(--bg-grid) 41px
        ),
        linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 50%, var(--bg-tertiary) 100%);
    background-attachment: fixed;
    display: flex;
    flex-direction: column;
    padding: 10px;
    margin: 0;
    position: relative;
    transition: background 0.3s ease;
}

/* 扫描线效果（暗色主题专属） */
body::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: repeating-linear-gradient(
        0deg,
        var(--bg-scanline) 0px,
        transparent 1px,
        transparent 2px,
        var(--bg-scanline) 3px
    );
    pointer-events: none;
    z-index: 9999;
    opacity: var(--scanline-opacity);
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
    background: var(--surface-primary);
    border: 1px solid var(--border-primary);
    border-radius: 8px;
    backdrop-filter: var(--backdrop-blur);
    box-shadow: var(--shadow-glow-cyan);
    /* 与终端容器等宽对齐 */
    max-width: 1400px;
    width: 100%;
    margin-left: auto;
    margin-right: auto;
    transition: all 0.3s ease;
}

/* 左侧控件区 */
#header-left-controls {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
    flex: 1;
}

/* 中间标题内容区 */
#header-content {
    text-align: center;
    flex-shrink: 0;
}

/* 右侧控件区 */
#header-right-controls {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
    flex: 1;
    justify-content: flex-end;
}

#header h1 {
    font-size: 1.5em;
    margin: 0 0 5px 0;
    /* 主题化渐变效果 */
    background: linear-gradient(90deg, var(--text-gradient-start) 0%, var(--text-gradient-end) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    transition: all 0.3s ease;
}

/* 暗色主题专属：霓虹发光动画 */
:root #header h1 {
    text-shadow:
        0 0 10px rgba(163, 113, 247, 0.5),
        0 0 20px rgba(163, 113, 247, 0.3),
        0 0 30px rgba(163, 113, 247, 0.2);
    animation: neon-pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

/* 浅色主题：移除发光效果 */
[data-theme="light"] #header h1 {
    text-shadow: none;
    animation: none;
}

@keyframes neon-pulse {
    0%, 100% {
        filter: brightness(1);
        text-shadow:
            0 0 10px rgba(163, 113, 247, 0.5),
            0 0 20px rgba(163, 113, 247, 0.3),
            0 0 30px rgba(163, 113, 247, 0.2);
    }
    25% {
        filter: brightness(1.05);
        text-shadow:
            0 0 12px rgba(163, 113, 247, 0.6),
            0 0 22px rgba(163, 113, 247, 0.4),
            0 0 32px rgba(163, 113, 247, 0.25);
    }
    50% {
        filter: brightness(1.15);
        text-shadow:
            0 0 15px rgba(163, 113, 247, 0.7),
            0 0 25px rgba(163, 113, 247, 0.5),
            0 0 35px rgba(163, 113, 247, 0.3);
    }
    75% {
        filter: brightness(1.05);
        text-shadow:
            0 0 12px rgba(163, 113, 247, 0.6),
            0 0 22px rgba(163, 113, 247, 0.4),
            0 0 32px rgba(163, 113, 247, 0.25);
    }
}

#header p {
    font-size: 0.9em;
    margin: 0;
    color: var(--text-title);
    transition: all 0.3s ease;
}

/* 暗色主题专属：tagline 发光效果 */
:root #header p {
    text-shadow: 0 0 10px rgba(163, 113, 247, 0.4);
}

/* 浅色主题：移除发光效果 */
[data-theme="light"] #header p {
    text-shadow: none;
}

/* ===== 语言选择器（币安风格下拉） ===== */
#lang-switcher {
    position: relative;
    flex-shrink: 0;
}

.lang-dropdown {
    padding: 6px 32px 6px 12px;
    border: 1px solid var(--border-muted);
    background: var(--surface-primary);
    color: var(--text-primary);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.2s ease;
    backdrop-filter: var(--backdrop-blur);
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    background-repeat: no-repeat;
    background-position: right 10px center;
    background-size: 12px;
    min-width: 120px;
}

/* 暗色主题：浅色箭头 */
:root .lang-dropdown {
    background-image: url('data:image/svg+xml;charset=UTF-8,<svg width="12" height="8" viewBox="0 0 12 8" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M1 1L6 6L11 1" stroke="%23E6EDF3" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>');
}

/* 浅色主题：深色箭头 */
[data-theme="light"] .lang-dropdown {
    background-image: url('data:image/svg+xml;charset=UTF-8,<svg width="12" height="8" viewBox="0 0 12 8" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M1 1L6 6L11 1" stroke="%231E2329" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>');
}

.lang-dropdown:hover {
    border-color: var(--border-hover);
}

.lang-dropdown:focus {
    outline: none;
    border-color: var(--accent-primary-alpha-60);
    background-color: var(--accent-primary-alpha-10);
}

.lang-dropdown option {
    background: var(--surface-primary);
    color: var(--text-primary);
    padding: 8px;
}

/* ===== v1.43.0: 主题切换按钮（币安风格） ===== */
.theme-toggle-btn {
    padding: 6px 12px;
    border: 1px solid var(--border-muted);
    background: var(--surface-primary);
    color: var(--text-primary);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.3s ease;
    backdrop-filter: var(--backdrop-blur);
    white-space: nowrap;
}

.theme-toggle-btn:hover {
    border-color: var(--border-hover);
    background: var(--surface-tertiary);
    transform: scale(1.05);
}

.theme-toggle-btn:active {
    transform: scale(0.95);
}

/* ===== v1.28.0: 视图模式切换按钮 ===== */

.view-mode-btn {
    padding: 6px 12px;
    border: 1px solid var(--border-muted);
    background: var(--surface-primary);
    color: var(--text-primary);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.3s ease;
    backdrop-filter: var(--backdrop-blur);
    white-space: nowrap;
}

.view-mode-btn:hover {
    background: var(--surface-tertiary);
    border-color: var(--border-hover);
}

/* 清空按钮 (v1.40.0) */
.clear-btn {
    padding: 6px 12px;
    border: 1px solid rgba(255, 123, 114, 0.3);
    background: rgba(255, 123, 114, 0.05);
    color: var(--color-error-soft);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.3s ease;
    backdrop-filter: var(--backdrop-blur);
    white-space: nowrap;
}

.clear-btn:hover {
    background: rgba(255, 123, 114, 0.15);
    border-color: rgba(255, 123, 114, 0.5);
}

#terminal-container {
    flex: 1;
    /* 深色背景 */
    background: var(--surface-secondary);
    border-radius: 8px;
    /* 霓虹青色边框 + 发光 */
    border: 2px solid var(--border-hover);
    box-shadow: var(--shadow-glow-cyan);
    overflow: hidden;
    padding: 8px;
    max-width: 1400px;
    width: 100%;
    margin: 0 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    backdrop-filter: var(--backdrop-blur);
    transition: all 0.3s ease;
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
        rgba(163, 113, 247, 0.3) 0%,
        rgba(163, 113, 247, 0.3) 50%,
        rgba(163, 113, 247, 0.3) 100%);
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
    color: var(--terminal-output);
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
    color: var(--terminal-output);
}

.line-output .terminal-text {
    margin: 0;
    font-family: inherit;
    white-space: pre-wrap;
    word-wrap: break-word;
}

/* 命令回显行 - 赛博朋克风格 */
.line-command {
    color: var(--terminal-command);
}

.line-command .prompt {
    color: var(--terminal-prompt);
    font-weight: bold;
}

.line-command .command {
    color: var(--text-primary);
    font-weight: 600;
}

/* Markdown 行 - 融入终端的赛博朋克 Markdown 格式化 */
.line-markdown {
    padding: 8px 0 8px 12px;
    /* 紫色左边框 + 发光 */
    border-left: 3px solid var(--accent-primary);
    background: var(--accent-primary-alpha-10);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.6;
    white-space: normal;
    box-shadow: -3px 0 10px var(--accent-primary-alpha-30);
    transition: all 0.3s ease;
}

/* Spinner 行 - 优雅紫色脉动 */
.line-spinner {
    color: var(--accent-primary);
    font-style: italic;
    animation: spinner-glow 1.5s ease-in-out infinite;
}

@keyframes spinner-glow {
    0%, 100% {
        opacity: 0.6;
    }
    50% {
        opacity: 1;
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

/* 输入字段 - 优雅暗色风格 */
.terminal-input-field {
    display: flex;
    align-items: center;
    padding: 8px 10px;
    /* 低调深灰分割线，GitHub 风格 */
    border-top: 1px solid var(--terminal-border);
    background: var(--terminal-input-bg);
    transition: all 0.3s ease;
}

.terminal-input-field .prompt {
    /* 币安金色提示符，优雅醒目 */
    color: var(--terminal-prompt);
    font-weight: bold;
    margin-right: 8px;
    flex-shrink: 0;
    animation: prompt-blink 1.5s ease-in-out infinite;
}

@keyframes prompt-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.8; }
}

.terminal-input-field input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    /* GitHub 白色，清晰可读 */
    color: var(--text-primary);
    font-family: inherit;
    font-size: inherit;
}

.terminal-input-field input::placeholder {
    color: var(--text-secondary);
}

/* v1.44.0: 语音输入按钮 */
.voice-input-btn {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    margin-left: 8px;
    background: transparent;
    border: 1px solid var(--border-primary);
    border-radius: 50%;
    cursor: pointer;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s ease;
    outline: none;
}

.voice-input-btn:hover {
    background: var(--surface-tertiary);
    border-color: var(--border-hover);
    transform: scale(1.1);
}

.voice-input-btn:active {
    transform: scale(0.95);
}

/* 录音状态 */
.voice-input-btn.recording {
    border-color: #ff4444;
    background: rgba(255, 68, 68, 0.1);
    animation: recording-pulse 1.5s ease-in-out infinite;
}

@keyframes recording-pulse {
    0%, 100% {
        box-shadow: 0 0 0 0 rgba(255, 68, 68, 0.4);
    }
    50% {
        box-shadow: 0 0 0 8px rgba(255, 68, 68, 0);
    }
}

/* 浅色主题覆盖 */
[data-theme="light"] .voice-input-btn {
    border-color: #EDEFF1;
}

[data-theme="light"] .voice-input-btn:hover {
    background: #F7F9FA;
    border-color: #B3B3B3;
}

[data-theme="light"] .voice-input-btn.recording {
    border-color: #ff4444;
    background: rgba(255, 68, 68, 0.08);
}

/* ANSI 颜色类 - 护眼优雅色系 */
.ansi-reset {
    color: var(--text-primary);
    font-weight: normal;
}

.ansi-bold {
    font-weight: bold;
}

.ansi-red {
    color: #FF6B6B;  /* 柔和红色，降低刺激 */
}

.ansi-green {
    color: #51CF66;  /* 柔和绿色，护眼 */
}

.ansi-yellow {
    color: var(--color-warning);
}

.ansi-blue {
    color: var(--accent-primary);
}

.ansi-cyan {
    color: #9DB4C0;  /* 灰蓝色替代亮青色 */
}

.ansi-white {
    color: var(--text-primary);
}

.ansi-dimmed {
    color: var(--text-secondary);
    opacity: 0.7;
}

/* 滚动条样式 - 简洁优雅 */
.terminal-output-area::-webkit-scrollbar {
    width: 8px;
}

.terminal-output-area::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.3);
    border-radius: 4px;
}

.terminal-output-area::-webkit-scrollbar-thumb {
    background: rgba(139, 148, 158, 0.3);
    border-radius: 4px;
    transition: all 0.3s ease;
}

.terminal-output-area::-webkit-scrollbar-thumb:hover {
    background: var(--accent-primary-alpha-60);
}

#status {
    text-align: center;
    margin-top: 8px;
    font-size: 0.85em;
    flex-shrink: 0;
}

#connection-status {
    padding: 5px 15px;
    /* 优雅灰色状态指示器 */
    background: rgba(139, 148, 158, 0.1);
    border: 1px solid rgba(139, 148, 158, 0.3);
    border-radius: 20px;
    display: inline-block;
    color: #8B949E;
    animation: status-pulse 2s ease-in-out infinite;
}

@keyframes status-pulse {
    0%, 100% {
        opacity: 0.8;
    }
    50% {
        opacity: 1;
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

    #header-left-controls,
    #header-right-controls {
        width: 100%;
        justify-content: center;
    }

    #header-content h1 {
        font-size: 1.3em;
    }

    #header-content p {
        font-size: 0.85em;
    }

    .lang-dropdown {
        flex: 1;
        max-width: 150px;
    }

    .theme-toggle-btn {
        flex: 1;
        max-width: 100px;
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
    background: var(--surface-primary);
    border: 1px solid var(--border-primary);
    border-radius: 8px;
    backdrop-filter: var(--backdrop-blur);
    box-shadow: var(--shadow-card);
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.conversation-round:hover {
    border-color: var(--border-hover);
    box-shadow: 0 0 20px rgba(163, 113, 247, 0.25);
}

/* 回合头部 */
/* ===== v1.36.2: 极简主义优化 ===== */
.round-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--accent-primary-alpha-10);
    border-bottom: 1px solid var(--border-secondary);
    cursor: pointer;
    transition: all 0.3s ease;
}

.round-header:hover {
    background: rgba(57, 255, 20, 0.08);
}

/* 回合徽章（类型图标+名称） */
.round-badge {
    font-weight: 600;
    color: var(--text-title);
    font-size: 0.9em;
    text-shadow: 0 0 8px rgba(57, 255, 20, 0.4);
}

/* 回合编号 */
.round-number {
    font-weight: 500;
    color: var(--text-muted);
    font-size: 0.85em;
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
    color: var(--text-muted);
    background: rgba(136, 136, 136, 0.1);
}

.round-status.running {
    color: var(--color-active);
    background: rgba(57, 255, 20, 0.15);
    animation: status-pulse 1.5s ease-in-out infinite;
}

.round-status.success {
    color: var(--color-success);
    background: rgba(57, 255, 20, 0.15);
    text-shadow: 0 0 8px rgba(57, 255, 20, 0.6);
}

.round-status.error {
    color: var(--color-error);
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
    color: var(--text-muted);
    font-size: 0.85em;
    font-family: "Consolas", monospace;
}

/* 工具容器 */
.round-tools {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
}

/* 工具标签 - 优雅低调风格 */
.tool-badge {
    display: inline-block;
    padding: 2px 8px;
    background: var(--accent-primary-alpha-10);
    border: 1px solid var(--accent-primary-alpha-30);
    border-radius: 12px;
    font-size: 0.75em;
    color: var(--accent-primary);
    transition: all 0.3s ease;
}

/* 回合摘要 - 已移除，简化为扁平结构 */

/* 重新执行按钮 - 简洁风格，紧挨折叠按钮 */
.round-rerun-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.1em;
    cursor: pointer;
    padding: 4px 6px;
    margin-right: 4px;
    transition: all 0.3s ease;
    opacity: 0.7;
}

.round-rerun-btn:hover {
    color: var(--accent-primary);
    opacity: 1;
    transform: scale(1.05);
}

/* v1.41.0: 删除按钮 */
.round-delete-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.1em;
    cursor: pointer;
    padding: 4px 6px;
    margin-right: 4px;
    transition: all 0.3s ease;
    opacity: 0.7;
}

.round-delete-btn:hover {
    color: var(--color-error);
    opacity: 1;
    transform: scale(1.05);
}

/* v1.42.0: 拖拽手柄按钮 */
.round-drag-handle {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.2em;
    cursor: grab;  /* 拖拽光标 */
    padding: 4px 6px;
    margin-right: 4px;
    transition: all 0.3s ease;
    opacity: 0.7;
    user-select: none;  /* 防止文本选中 */
}

.round-drag-handle:hover {
    color: var(--color-link);
    opacity: 1;
    transform: scale(1.05);
}

.round-drag-handle:active {
    cursor: grabbing;  /* 抓取光标 */
}

/* v1.42.0: 拖拽状态样式 */
.conversation-round.dragging {
    opacity: 0.4;
    transform: scale(0.95);
    transition: opacity 0.2s ease, transform 0.2s ease;
}

.conversation-round.drag-over {
    border-top: 3px solid var(--color-link);
    padding-top: 8px;  /* 补偿边框高度 */
}

/* 折叠按钮 - 统一风格优化 */
.round-toggle {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.2em;
    cursor: pointer;
    padding: 4px 8px;
    transition: all 0.3s ease;
    opacity: 0.7;
}

.round-toggle:hover {
    color: var(--accent-primary);
    opacity: 1;
    transform: scale(1.05);
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

/* 输入区域 - 简化版，移除标签层 */
.round-input {
    margin-bottom: 8px;
}

.round-input-content {
    padding: 8px 12px;
    background: var(--accent-primary-alpha-10);
    border-left: 3px solid var(--text-title);
    border-radius: 4px;
    color: var(--terminal-output);
    font-family: "Consolas", monospace;
    font-size: 0.9em;
    white-space: pre-wrap;
    word-wrap: break-word;
    transition: all 0.3s ease;
    box-shadow: -3px 0 10px var(--accent-primary-alpha-10);
}

/* 输出区域 - 简化版，移除标签层 */
.round-output {
    margin-top: 8px;
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
    background: linear-gradient(90deg, #A371F7 0%, #A371F7 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    font-weight: 600;
    margin: 0.8em 0 0.4em 0;
    filter: drop-shadow(0 0 5px rgba(163, 113, 247, 0.4));
}

.output-content code {
    color: var(--accent-primary);
    background-color: var(--accent-primary-alpha-10);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    border: 1px solid var(--accent-primary-alpha-30);
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.9em;
}

.output-content pre {
    background: rgba(10, 14, 39, 0.8);
    border: 1px solid var(--accent-primary-alpha-30);
    border-radius: 6px;
    padding: 12px;
    overflow-x: auto;
    margin: 8px 0;
    box-shadow: inset 0 0 15px var(--accent-primary-alpha-10);
}

.output-content pre code {
    background: none;
    border: none;
    padding: 0;
    color: rgba(240, 240, 240, 0.9);
}

/* v1.40.0: Intent 输出美化 */
.intent-output {
    background: rgba(10, 14, 39, 0.6);
    border: 1px solid var(--accent-primary-alpha-30);
    padding: 12px 16px;
    border-radius: 6px;
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.95em;
    line-height: 1.6;
    color: #f0f0f0;
    box-shadow: inset 0 0 10px var(--accent-primary-alpha-10);
}

/* Intent 名称高亮（🎯 图标行） */
.intent-output::first-line {
    color: var(--accent-primary);
    font-weight: 500;
    text-shadow: 0 0 5px var(--accent-primary-alpha-30);
}

/* 响应式调整 */
@media (max-width: 768px) {
    .round-header {
        gap: 8px;
    }

    .conversation-round {
        margin: 8px 0;
    }
}

/* ============================================
   Markdown 内容样式 (v1.26.0)
   Claude Code 风格 - 融入终端体验
   ============================================ */

/* v1.36.3: Markdown 渲染样式统一（传统模式 + 回合模式） */
.line-markdown h1,
.line-markdown h2,
.line-markdown h3,
.line-markdown h4,
.line-markdown h5,
.line-markdown h6,
.markdown-content h1,
.markdown-content h2,
.markdown-content h3,
.markdown-content h4,
.markdown-content h5,
.markdown-content h6 {
    /* 优雅紫金渐变标题 */
    background: linear-gradient(90deg, #A371F7 0%, #F0B90B 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    font-weight: 600;
    margin: 0.8em 0 0.4em 0;
}

.line-markdown h1, .markdown-content h1 { font-size: 1.8em; }
.line-markdown h2, .markdown-content h2 { font-size: 1.5em; }
.line-markdown h3, .markdown-content h3 { font-size: 1.3em; }
.line-markdown h4, .markdown-content h4 { font-size: 1.1em; }
.line-markdown h5, .markdown-content h5 { font-size: 1.0em; }
.line-markdown h6, .markdown-content h6 { font-size: 0.9em; }

/* 粗体 - 优雅白色 */
.line-markdown strong,
.markdown-content strong {
    color: #E6EDF3;
    font-weight: 700;
}

/* 斜体 - 柔和紫色 */
.line-markdown em,
.markdown-content em {
    color: #C8A2F0;
    font-style: italic;
}

/* 内联代码 - 紫色系 */
.line-markdown code,
.markdown-content code {
    color: #A371F7;
    background-color: rgba(163, 113, 247, 0.1);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    border: 1px solid rgba(163, 113, 247, 0.3);
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.9em;
}

/* 代码块 - 柔和绿色，护眼 */
.line-markdown pre,
.markdown-content pre {
    background-color: rgba(0, 0, 0, 0.5);
    padding: 1em;
    border-radius: 5px;
    border: 1px solid rgba(48, 54, 61, 0.6);  /* GitHub 深灰边框 */
    overflow-x: auto;
    margin: 0.5em 0;
}

.line-markdown pre code,
.markdown-content pre code {
    color: #51CF66;  /* 柔和绿色，护眼 */
    background: none;
    border: none;
    padding: 0;
    font-size: 0.95em;
}

/* 段落 - GitHub 白色，清晰易读 */
.line-markdown p,
.markdown-content p {
    margin: 0.5em 0;
    color: #C9D1D9;  /* GitHub 浅灰白，护眼舒适 */
}

/* 列表 - 柔和紫色 bullet，护眼 */
.line-markdown ul,
.line-markdown ol,
.markdown-content ul,
.markdown-content ol {
    margin: 0.5em 0;
    padding-left: 1.5em;
}

.line-markdown ul li::marker,
.markdown-content ul li::marker {
    color: #A371F7;  /* 紫色 marker，优雅 */
}

.line-markdown ol li::marker,
.markdown-content ol li::marker {
    color: #A371F7;  /* 紫色 marker */
    font-weight: 600;
}

.line-markdown li,
.markdown-content li {
    margin: 0.3em 0;
    color: #C9D1D9;  /* GitHub 浅灰白 */
}

/* 引用块 - 紫色边框，护眼 */
.line-markdown blockquote,
.markdown-content blockquote {
    border-left: 3px solid rgba(163, 113, 247, 0.5);  /* 紫色边框 */
    padding-left: 1em;
    color: #8B949E;  /* GitHub 灰色，柔和 */
    margin: 0.5em 0;
    font-style: italic;
    background: rgba(163, 113, 247, 0.05);
}

/* 链接 - 紫色，简洁 */
.line-markdown a,
.markdown-content a {
    color: #A371F7;  /* 紫色链接 */
    text-decoration: none;
    border-bottom: 1px solid rgba(163, 113, 247, 0.4);
    transition: all 0.2s ease;
}

.line-markdown a:hover,
.markdown-content a:hover {
    color: #C8A2F0;  /* 浅紫色悬停 */
    border-bottom-color: #A371F7;
}

/* 分隔线 - 灰色渐变，简洁 */
.line-markdown hr,
.markdown-content hr {
    border: none;
    height: 1px;
    background: linear-gradient(90deg,
        transparent 0%,
        rgba(139, 148, 158, 0.3) 20%,
        rgba(163, 113, 247, 0.4) 50%,
        rgba(139, 148, 158, 0.3) 80%,
        transparent 100%);
    margin: 1em 0;
}

/* 表格 - GitHub 风格，简洁护眼 */
.line-markdown table,
.markdown-content table {
    border-collapse: collapse;
    width: 100%;
    margin: 0.5em 0;
}

.line-markdown th,
.line-markdown td,
.markdown-content th,
.markdown-content td {
    border: 1px solid rgba(48, 54, 61, 0.5);  /* GitHub 深灰边框 */
    padding: 0.4em 0.8em;
    text-align: left;
    color: #C9D1D9;  /* GitHub 浅灰白 */
}

.line-markdown th,
.markdown-content th {
    background-color: rgba(163, 113, 247, 0.08);  /* 紫色背景 */
    color: #E6EDF3;  /* GitHub 白色 */
    font-weight: 600;
}

.line-markdown td,
.markdown-content td {
    background-color: rgba(48, 54, 61, 0.15);  /* 深灰背景 */
}

/* 图片 - 简洁边框 */
.line-markdown img,
.markdown-content img {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
    border: 1px solid rgba(48, 54, 61, 0.6);  /* GitHub 深灰边框 */
    margin: 0.5em 0;
}

/* ===== 浅色主题：Markdown 简洁样式覆盖（v1.43.0 - 去除晕光效果） ===== */
[data-theme="light"] .line-markdown h1,
[data-theme="light"] .line-markdown h2,
[data-theme="light"] .line-markdown h3,
[data-theme="light"] .line-markdown h4,
[data-theme="light"] .line-markdown h5,
[data-theme="light"] .line-markdown h6,
[data-theme="light"] .markdown-content h1,
[data-theme="light"] .markdown-content h2,
[data-theme="light"] .markdown-content h3,
[data-theme="light"] .markdown-content h4,
[data-theme="light"] .markdown-content h5,
[data-theme="light"] .markdown-content h6 {
    /* 移除渐变效果，使用简洁的紫色 */
    background: none;
    -webkit-background-clip: unset;
    -webkit-text-fill-color: unset;
    background-clip: unset;
    color: #8B5CF6;  /* 纯紫色，简洁清晰 */
}

[data-theme="light"] .line-markdown strong,
[data-theme="light"] .markdown-content strong {
    color: #1C1C1C;  /* Reddit 深灰黑，清晰 */
    font-weight: 700;
}

[data-theme="light"] .line-markdown em,
[data-theme="light"] .markdown-content em {
    color: #8B5CF6;  /* 紫色，简洁 */
}

[data-theme="light"] .line-markdown code,
[data-theme="light"] .markdown-content code {
    color: #8B5CF6;
    background-color: #F7F9FA;  /* Reddit 浅背景 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
}

[data-theme="light"] .line-markdown pre,
[data-theme="light"] .markdown-content pre {
    background-color: #F7F9FA;  /* Reddit 浅背景 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
}

[data-theme="light"] .line-markdown pre code,
[data-theme="light"] .markdown-content pre code {
    color: #0ECB81;  /* 绿色代码，清晰 */
}

[data-theme="light"] .line-markdown p,
[data-theme="light"] .markdown-content p {
    color: #1C1C1C;  /* Reddit 深灰黑，清晰易读 */
}

[data-theme="light"] .line-markdown li,
[data-theme="light"] .markdown-content li {
    color: #1C1C1C;  /* Reddit 深灰黑，清晰 */
}

[data-theme="light"] .line-markdown blockquote,
[data-theme="light"] .markdown-content blockquote {
    border-left: 3px solid rgba(139, 92, 246, 0.4);
    color: #7C7C7C;  /* Reddit 中灰次要文字 */
    background: rgba(139, 92, 246, 0.03);
}

[data-theme="light"] .line-markdown a,
[data-theme="light"] .markdown-content a {
    color: #8B5CF6;
    border-bottom: 1px solid rgba(139, 92, 246, 0.3);
}

[data-theme="light"] .line-markdown a:hover,
[data-theme="light"] .markdown-content a:hover {
    color: #9065DC;
    border-bottom-color: #8B5CF6;
}

[data-theme="light"] .line-markdown hr,
[data-theme="light"] .markdown-content hr {
    background: linear-gradient(90deg,
        transparent 0%,
        rgba(139, 92, 246, 0.2) 20%,
        rgba(139, 92, 246, 0.3) 50%,
        rgba(139, 92, 246, 0.2) 80%,
        transparent 100%);
}

[data-theme="light"] .line-markdown th,
[data-theme="light"] .line-markdown td,
[data-theme="light"] .markdown-content th,
[data-theme="light"] .markdown-content td {
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
    color: #1C1C1C;             /* Reddit 深灰黑 */
}

[data-theme="light"] .line-markdown th,
[data-theme="light"] .markdown-content th {
    background-color: #F7F9FA;  /* Reddit 浅背景 */
    color: #1C1C1C;             /* Reddit 深灰黑 */
}

[data-theme="light"] .line-markdown td,
[data-theme="light"] .markdown-content td {
    background-color: #FFFFFF;  /* Reddit 纯白 */
}

[data-theme="light"] .line-markdown img,
[data-theme="light"] .markdown-content img {
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
}

/* ===== 浅色主题：终端样式覆盖（v1.43.0 - Reddit 风格白色背景） ===== */

/* 终端容器 - 纯白背景 */
[data-theme="light"] #terminal-container {
    background: #FFFFFF;  /* Reddit 纯白卡片背景 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);  /* 简洁阴影 */
}

/* 混合终端 - 统一浅色文字 */
[data-theme="light"] .hybrid-terminal {
    color: #1C1C1C;  /* Reddit 深灰黑 */
}

/* 终端输出区域 */
[data-theme="light"] .terminal-output-area {
    background: #FFFFFF;  /* 纯白背景 */
}

/* 输出行 - 深色文字 */
[data-theme="light"] .line-output {
    color: #1C1C1C;  /* Reddit 深灰黑 */
}

/* 命令回显行 - 紫色 */
[data-theme="light"] .line-command {
    color: #8B5CF6;  /* 紫色命令 */
}

[data-theme="light"] .line-command .prompt {
    color: #F0B90B;  /* 金色提示符 */
}

[data-theme="light"] .line-command .command {
    color: #1C1C1C;  /* 深色命令文本 */
}

/* 输入区域 */
[data-theme="light"] .terminal-input-line {
    background: #FFFFFF;  /* 纯白背景 */
    border-top: 1px solid #EDEFF1;  /* Reddit 浅边框 */
}

[data-theme="light"] .terminal-input-line input {
    background: #F7F9FA;  /* Reddit 浅背景 */
    color: #1C1C1C;  /* 深色文字 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
}

[data-theme="light"] .terminal-input-line input:focus {
    background: #FFFFFF;
    border-color: #8B5CF6;  /* 紫色聚焦边框 */
}

/* 滚动条 - 浅色风格 */
[data-theme="light"] .terminal-output-area::-webkit-scrollbar-track {
    background: #F7F9FA;  /* Reddit 浅背景 */
}

[data-theme="light"] .terminal-output-area::-webkit-scrollbar-thumb {
    background: #CCCCCC;  /* 浅灰滚动条 */
}

[data-theme="light"] .terminal-output-area::-webkit-scrollbar-thumb:hover {
    background: #8B5CF6;  /* 紫色悬停 */
}

/* 输出内容的 pre 元素 - 关键：ls、pwd 等命令输出 */
[data-theme="light"] .output-content pre {
    background: #F7F9FA;  /* Reddit 浅背景 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
    box-shadow: none;  /* 移除内阴影 */
}

[data-theme="light"] .output-content pre code {
    color: #1C1C1C;  /* Reddit 深灰黑文字 */
}

/* Intent 输出 */
[data-theme="light"] .intent-output {
    background: #F7F9FA;  /* Reddit 浅背景 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
    color: #1C1C1C;  /* 深色文字 */
}

/* terminal-text pre 元素 */
[data-theme="light"] .terminal-text {
    background: #FFFFFF;  /* 纯白背景 */
    color: #1C1C1C;  /* 深色文字 */
}

[data-theme="light"] pre.terminal-text {
    background: #FFFFFF;  /* 纯白背景 */
    color: #1C1C1C;  /* 深色文字 */
}

/* ===== 浅色主题：Round 卡片样式覆盖（v1.43.0 - Reddit 风格） ===== */

/* 回合卡片容器 */
[data-theme="light"] .conversation-round {
    background: #FFFFFF;  /* Reddit 纯白卡片 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);  /* 简洁阴影 */
}

[data-theme="light"] .conversation-round:hover {
    border-color: #B3B3B3;  /* 悬停边框 */
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);  /* 增强阴影 */
}

/* 回合头部 */
[data-theme="light"] .round-header {
    background: rgba(139, 92, 246, 0.05);  /* 极浅紫色背景 */
    border-bottom: 1px solid #EDEFF1;
}

[data-theme="light"] .round-header:hover {
    background: rgba(139, 92, 246, 0.08);  /* 悬停时稍深 */
}

/* 回合徽章 - 移除晕光 */
[data-theme="light"] .round-badge {
    color: #0ECB81;  /* 绿色徽章 */
    text-shadow: none;  /* 移除晕光效果 */
}

/* 回合编号 */
[data-theme="light"] .round-number {
    color: #A8A8A8;  /* Reddit 浅灰弱化文字 */
}

/* 回合状态 - 移除晕光 */
[data-theme="light"] .round-status.running {
    color: #0ECB81;  /* 绿色运行状态 */
    text-shadow: none;  /* 移除晕光效果 */
}

[data-theme="light"] .round-status.completed {
    color: #0ECB81;  /* 绿色完成状态 */
}

[data-theme="light"] .round-status.pending {
    color: #A8A8A8;  /* 浅灰待处理 */
}

/* 回合内容区 */
[data-theme="light"] .round-content {
    background: #FFFFFF;  /* 纯白背景 */
    color: #1C1C1C;  /* 深色文字 */
}

/* 回合统计信息 */
[data-theme="light"] .round-stats {
    color: #7C7C7C;  /* Reddit 中灰次要文字 */
}

/* ===== 浅色主题：意图卡片样式覆盖（v1.43.0 - Reddit 风格） ===== */

/* 意图卡片容器 */
[data-theme="light"] .intent-card {
    background: #FFFFFF;  /* Reddit 纯白卡片 */
    border: 1px solid #EDEFF1;  /* Reddit 浅边框 */
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

[data-theme="light"] .intent-card.completed {
    border-color: #0ECB81;  /* 绿色边框 */
    background: #FFFFFF;
}

/* 意图标题 */
[data-theme="light"] .intent-title {
    color: #1C1C1C;  /* Reddit 深灰黑 */
}

/* 意图理解区域 */
[data-theme="light"] .understanding-content {
    background: #F7F9FA;  /* Reddit 浅背景 */
    color: #1C1C1C;  /* 深色文字 */
    border-left: 3px solid #F0B90B;  /* 金色边框 */
}

/* 意图步骤 */
[data-theme="light"] .intent-step {
    background: #F7F9FA;  /* Reddit 浅背景 */
    border-left: 3px solid rgba(139, 92, 246, 0.3);
}

[data-theme="light"] .intent-step.running {
    background: rgba(240, 185, 11, 0.08);  /* 浅金色背景 */
    border-left-color: #F0B90B;
}

[data-theme="light"] .intent-step.completed {
    background: rgba(14, 203, 129, 0.05);  /* 极浅绿色背景 */
    border-left-color: #0ECB81;
}

/* 意图元信息 */
[data-theme="light"] .intent-meta {
    color: #7C7C7C;  /* Reddit 中灰次要文字 */
}

/* ========== v1.29.0: 意图拆解可视化样式 ========== */

/* 意图卡片 */
.intent-card {
    background: linear-gradient(135deg, rgba(163, 113, 247, 0.05) 0%, rgba(139, 148, 158, 0.03) 100%);
    border: 1px solid rgba(139, 148, 158, 0.3);
    border-radius: 8px;
    margin: 1em 0;
    padding: 1.5em;
    animation: fadeInSlide 0.4s ease-out;
}

.intent-card.completed {
    border-color: rgba(81, 207, 102, 0.3);  /* 柔和绿色 */
    background: linear-gradient(135deg, rgba(81, 207, 102, 0.05) 0%, rgba(139, 148, 158, 0.03) 100%);
}

/* 意图头部 */
.intent-header {
    display: flex;
    align-items: center;
    gap: 0.5em;
    margin-bottom: 1em;
    padding-bottom: 0.5em;
    border-bottom: 1px solid rgba(139, 148, 158, 0.2);
}

.intent-icon {
    font-size: 1.5em;
}

.intent-title {
    font-size: 1.2em;
    font-weight: 600;
    color: #E6EDF3;  /* GitHub 白色，替代青色 */
}

/* 意图理解 */
.intent-understanding {
    margin: 1em 0;
}

.understanding-label {
    color: #ffb700;
    font-weight: 500;
    margin-bottom: 0.5em;
}

.understanding-content {
    color: #e0e0e0;
    padding: 0.5em 1em;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 4px;
    border-left: 3px solid #ffb700;
}

/* 元信息 */
.intent-meta {
    display: flex;
    gap: 1.5em;
    margin: 1em 0;
    font-size: 0.9em;
    color: #b0b0b0;
}

.step-count, .total-time {
    display: flex;
    align-items: center;
    gap: 0.3em;
}

/* 步骤容器 */
.intent-steps {
    margin-top: 1em;
}

/* 单个步骤 */
.intent-step {
    background: rgba(0, 0, 0, 0.3);
    border-left: 3px solid var(--accent-primary-alpha-30);
    border-radius: 4px;
    margin: 0.5em 0;
    padding: 0.8em 1em;
    transition: all 0.3s ease;
}

.intent-step.running {
    border-left-color: #ffb700;
    background: rgba(255, 183, 0, 0.1);
    animation: pulse 2s ease-in-out infinite;
}

.intent-step.success {
    border-left-color: #00ff64;
    background: rgba(0, 255, 100, 0.1);
}

.intent-step.failed {
    border-left-color: #ff4444;
    background: rgba(255, 68, 68, 0.1);
}

/* 步骤头部 */
.step-header {
    display: flex;
    align-items: center;
    gap: 0.5em;
    margin-bottom: 0.5em;
    cursor: pointer;
    user-select: none;
    padding: 0.2em;
    border-radius: 4px;
    transition: background 0.2s ease;
}

.step-header:hover {
    background: rgba(255, 255, 255, 0.05);
}

/* v1.36.2: 步骤折叠图标 */
.step-toggle {
    color: var(--accent-primary);
    font-size: 0.9em;
    min-width: 1.2em;
    text-align: center;
    transition: transform 0.2s ease;
}

.step-number {
    color: var(--accent-primary);
    font-weight: 600;
    min-width: 2em;
}

.step-description {
    flex: 1;
    color: #ffffff;
    font-weight: 500;
}

.step-status {
    font-size: 1.2em;
    min-width: 1.5em;
    text-align: center;
}

/* v1.36.2: 步骤详情（可折叠） */
.step-details {
    max-height: 500px;
    overflow: hidden;
    transition: max-height 0.3s ease, opacity 0.3s ease;
    opacity: 1;
}

.intent-step:not(.expanded) .step-details {
    max-height: 0;
    opacity: 0;
}

/* v1.36.2: 折叠时也隐藏输出内容 */
.intent-step:not(.expanded) .step-output {
    max-height: 0 !important;
    opacity: 0 !important;
    margin: 0 !important;
    padding: 0 !important;
    overflow: hidden;
}

.intent-step:not(.expanded) .step-header {
    margin-bottom: 0;
}

/* 步骤元信息 */
.step-meta {
    display: flex;
    gap: 1em;
    font-size: 0.9em;
    color: #b0b0b0;
    margin-left: 2em;
}

.step-tool {
    color: #00ff64;
}

.step-time {
    color: #ffb700;
}

/* 完成标记 */
.intent-complete {
    display: flex;
    align-items: center;
    gap: 1em;
    margin-top: 1.5em;
    padding: 1em;
    border-radius: 6px;
    animation: fadeIn 0.5s ease-out;
}

.intent-complete.success {
    background: rgba(0, 255, 100, 0.15);
    border: 1px solid rgba(0, 255, 100, 0.4);
}

.intent-complete.failed {
    background: rgba(255, 68, 68, 0.15);
    border: 1px solid rgba(255, 68, 68, 0.4);
}

.complete-icon {
    font-size: 2em;
}

.complete-text {
    flex: 1;
    font-weight: 500;
    color: #ffffff;
}

.complete-time {
    margin-left: 1em;
    color: #ffb700;
}

/* ===== v1.29.2: 意图操作按钮 ===== */
.intent-actions {
    display: flex;
    gap: 0.5em;
    margin-top: 1em;
    padding-top: 1em;
    border-top: 1px solid var(--accent-primary-alpha-30);
}

.intent-edit-btn {
    padding: 0.5em 1em;
    background: var(--accent-primary-alpha-10);
    border: 1px solid var(--accent-primary-alpha-30);
    border-radius: 4px;
    color: var(--accent-primary);
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.2s ease;
}

.intent-edit-btn:hover {
    background: var(--accent-primary-alpha-30);
    border-color: var(--accent-primary-alpha-60);
    transform: translateY(-1px);
}

.intent-edit-btn:active {
    transform: translateY(0);
}

/* v1.29.3: 执行按钮样式 */
.intent-execute-btn {
    padding: 0.5em 1em;
    background: rgba(138, 43, 226, 0.1);  /* 紫色主题 */
    border: 1px solid rgba(138, 43, 226, 0.3);
    border-radius: 4px;
    color: #8a2be2;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.2s ease;
}

.intent-execute-btn:hover {
    background: rgba(138, 43, 226, 0.2);
    border-color: rgba(138, 43, 226, 0.5);
    transform: translateY(-1px);
}

.intent-execute-btn:active {
    transform: translateY(0);
}

/* ===== v1.36.2: 态势分析卡片样式（极简版）===== */
.situation-analysis-card {
    background: rgba(138, 43, 226, 0.03);
    border-left: 3px solid rgba(138, 43, 226, 0.4);
    border-radius: 4px;
    padding: 0.6em 0.8em;
    margin: 0.5em 0;
    font-size: 0.85em;
    line-height: 1.5;
}

/* 标题行 - 横向排列核心指标 */
.situation-header {
    color: #8a2be2;
    font-weight: 500;
    margin-bottom: 0.4em;
}

.situation-header .divider {
    color: rgba(138, 43, 226, 0.4);
    margin: 0 0.3em;
}

.situation-header .complexity {
    color: var(--accent-primary);
}

.situation-header .risk {
    font-weight: 600;
}

.situation-header .risk.low-risk {
    color: #39ff14;
}

.situation-header .risk.medium-risk {
    color: #ffb700;
}

.situation-header .risk.high-risk {
    color: var(--color-error);
}

.situation-header .balance {
    color: #b0b0b0;
}

/* 主要信息 - 总体评价 */
.situation-main {
    color: #ffffff;
    margin: 0.3em 0;
}

/* 问题和建议 - 紧凑显示 */
.alert {
    margin: 0.25em 0;
    font-size: 0.9em;
}

.alert.critical {
    color: var(--color-error);
}

.alert.warning {
    color: #ffb700;
}

.alert.suggestion {
    color: #8a2be2;
}

.intent-cancel-btn, .intent-confirm-btn {
    padding: 0.5em 1em;
    border: 1px solid;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.2s ease;
}

.intent-cancel-btn {
    background: rgba(255, 68, 68, 0.1);
    border-color: rgba(255, 68, 68, 0.3);
    color: #ff4444;
}

.intent-cancel-btn:hover {
    background: rgba(255, 68, 68, 0.2);
    border-color: rgba(255, 68, 68, 0.5);
    transform: translateY(-1px);
}

.intent-confirm-btn {
    background: rgba(0, 255, 100, 0.1);
    border-color: rgba(0, 255, 100, 0.3);
    color: #00ff64;
}

.intent-confirm-btn:hover {
    background: rgba(0, 255, 100, 0.2);
    border-color: rgba(0, 255, 100, 0.5);
    transform: translateY(-1px);
}

/* ===== v1.29.2: 编辑模式样式 ===== */

/* 编辑模式卡片标识 */
.intent-card.editing {
    border-color: rgba(255, 183, 0, 0.5);
    box-shadow: 0 4px 20px rgba(255, 183, 0, 0.3);
}

/* Checkbox 样式 */
.step-checkbox {
    width: 1.2em;
    height: 1.2em;
    margin-right: 0.5em;
    cursor: pointer;
    accent-color: var(--accent-primary);
    transition: transform 0.2s ease;
}

.step-checkbox:hover {
    transform: scale(1.15);
}

/* 编辑模式下的步骤hover效果 */
.intent-step:has(.step-checkbox):hover {
    background: var(--accent-primary-alpha-10);
    transform: translateX(2px);
}

/* 禁用的步骤 */
.intent-step.disabled {
    opacity: 0.5;
    text-decoration: line-through;
    text-decoration-color: rgba(255, 255, 255, 0.3);
    transition: opacity 0.3s ease, text-decoration 0.3s ease;
}

.intent-step.disabled .step-description {
    color: #888888;
}

.intent-step.disabled .step-tool {
    color: #666666;
}

/* ===== v1.29.3: 步骤输出和执行摘要 ===== */

/* 步骤输出 - v1.36.2: 添加折叠过渡效果 */
.step-output {
    margin: 0.5em 0 0 2em;
    padding: 0.5em;
    background: rgba(0, 0, 0, 0.4);
    border-left: 2px solid var(--accent-primary-alpha-30);
    border-radius: 4px;
    max-height: 2000px;
    overflow: hidden;
    transition: max-height 0.3s ease, opacity 0.3s ease, margin 0.3s ease, padding 0.3s ease;
    opacity: 1;
}

.step-output-content {
    margin: 0;
    padding: 0.5em;
    color: #e0e0e0;
    font-family: 'Courier New', Courier, monospace;
    font-size: 0.85em;
    white-space: pre-wrap;
    word-wrap: break-word;
}

/* 执行摘要 */
.execution-summary {
    display: flex;
    align-items: center;
    gap: 1em;
    margin-top: 1.5em;
    padding: 1em;
    border-radius: 6px;
    animation: fadeIn 0.5s ease-out;
}

.execution-summary.success {
    background: rgba(0, 255, 100, 0.15);
    border: 1px solid rgba(0, 255, 100, 0.4);
}

.execution-summary.failed {
    background: rgba(255, 183, 0, 0.15);
    border: 1px solid rgba(255, 183, 0, 0.4);
}

.summary-icon {
    font-size: 2em;
}

.summary-text {
    flex: 1;
    font-weight: 500;
    color: #ffffff;
}

.summary-details {
    display: block;
    margin-top: 0.3em;
    font-size: 0.9em;
    color: #b0b0b0;
    font-weight: normal;
}

/* 动画 */
@keyframes fadeInSlide {
    from {
        opacity: 0;
        transform: translateY(-10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

@keyframes fadeIn {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}

@keyframes pulse {
    0%, 100% {
        box-shadow: 0 0 0 0 rgba(255, 183, 0, 0.4);
    }
    50% {
        box-shadow: 0 0 15px 5px rgba(255, 183, 0, 0.2);
    }
}

/* ============================================
   🔮 v1.36.0: 态势测算分析动画样式
   基于易经八卦的可视化系统
   ============================================ */

.divination-animation {
    background: linear-gradient(
        135deg,
        rgba(255, 215, 0, 0.05) 0%,
        rgba(139, 69, 19, 0.05) 50%,
        rgba(255, 215, 0, 0.05) 100%
    );
    border: 2px solid rgba(255, 215, 0, 0.3);
    border-radius: 16px;
    padding: 32px;
    margin: 20px 0;
    backdrop-filter: blur(10px);
    box-shadow: 0 8px 32px rgba(255, 215, 0, 0.2);
}

.divination-stage {
    text-align: center;
    font-family: 'STKaiti', 'SimSun', serif;
    min-height: 200px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
}

/* 起卦阶段 */
.dots-container {
    display: flex;
    gap: 16px;
    margin-bottom: 16px;
}

.dot {
    font-size: 32px;
    transition: all 0.3s;
}

.dot.active {
    transform: scale(1.2);
    filter: drop-shadow(0 0 8px rgba(255, 215, 0, 0.8));
}

.stage-label {
    font-size: 20px;
    color: #FFD700;
    margin-top: 16px;
    opacity: 0.7;
}

/* 演算阶段 */
.operation-name {
    font-size: 20px;
    color: #FFD700;
    margin-bottom: 12px;
}

.stalk-count {
    font-size: 80px;
    font-weight: bold;
    color: #FFD700;
    text-shadow:
        0 0 10px rgba(255, 215, 0, 0.6),
        0 0 20px rgba(255, 215, 0, 0.4),
        0 0 30px rgba(255, 215, 0, 0.2);
    transition: all 0.5s cubic-bezier(0.68, -0.55, 0.265, 1.55);
    line-height: 1;
    margin: 16px 0;
}

.stalk-count.changing {
    transform: scale(1.3) rotateY(180deg);
    color: #FFA500;
}

.operation-desc {
    font-size: 14px;
    color: #CCC;
    opacity: 0.6;
    font-style: italic;
    margin-top: 8px;
}

.yarrow-visual {
    font-size: 12px;
    color: #FFD700;
    opacity: 0.3;
    letter-spacing: 1px;
    margin-top: 16px;
    max-width: 400px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 成卦阶段 */
.hexagram-forming {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
}

.hexagram-symbol {
    font-size: 60px;
    line-height: 1.2;
    color: #FFD700;
}

.yao-line {
    opacity: 0;
    animation: yaoFadeIn 0.5s ease-out forwards;
}

@keyframes yaoFadeIn {
    from {
        opacity: 0;
        transform: translateY(20px) scale(0.8);
    }
    to {
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}

.hexagram-name {
    font-size: 24px;
    color: #FFD700;
    font-weight: bold;
}

/* 卦象卡片（最终显示） */
.hexagram-card {
    background: linear-gradient(
        135deg,
        rgba(255, 215, 0, 0.08) 0%,
        rgba(139, 69, 19, 0.08) 100%
    );
    border: 1px solid rgba(255, 215, 0, 0.3);
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 16px;
}

.hexagram-display {
    display: flex;
    gap: 24px;
    align-items: center;
}

.hexagram-symbol-large {
    font-size: 64px;
    line-height: 1.2;
    color: #FFD700;
    text-shadow: 0 0 20px rgba(255, 215, 0, 0.4);
}

.hexagram-info {
    flex: 1;
    text-align: left;
}

.hexagram-name-large {
    font-size: 22px;
    color: #FFD700;
    font-weight: bold;
    margin-bottom: 8px;
    font-family: 'STKaiti', 'SimSun', serif;
}

.hexagram-judgement {
    font-size: 14px;
    color: #CCC;
    line-height: 1.6;
    font-family: 'SimSun', serif;
    font-style: italic;
}

/* ===== v1.40.0: 会话管理样式 ===== */

/* 会话按钮 */
.session-btn {
    background: var(--surface-primary);
    border: 1px solid var(--border-muted);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85em;
    font-weight: 500;
    transition: all 0.3s ease;
    backdrop-filter: var(--backdrop-blur);
}

.session-btn:hover {
    background: var(--surface-tertiary);
    border-color: var(--border-hover);
}

/* 会话管理面板 */
.session-panel {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
}

.session-panel.hidden {
    display: none;
}

/* 半透明遮罩 */
.session-panel-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--surface-overlay);
    backdrop-filter: blur(5px);
}

/* 对话框 */
.session-panel-dialog {
    position: relative;
    background: rgba(10, 14, 39, 0.95);
    border: 1px solid var(--border-primary);
    border-radius: 12px;
    width: 90%;
    max-width: 800px;
    max-height: 80vh;
    box-shadow: 0 0 30px var(--accent-primary-alpha-30);
    display: flex;
    flex-direction: column;
    transition: all 0.3s ease;
}

/* 头部 */
.session-panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-secondary);
}

.session-panel-header h3 {
    margin: 0;
    color: var(--text-title);
    font-size: 1.2em;
}

.close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 2em;
    cursor: pointer;
    transition: all 0.3s ease;
    line-height: 1;
    padding: 0;
}

.close-btn:hover {
    color: var(--color-error);
}

/* 内容区域 */
.session-panel-content {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
}

/* 操作按钮区 */
.session-actions {
    display: flex;
    gap: 12px;
    margin-bottom: 20px;
}

.session-action-btn {
    flex: 1;
    background: rgba(57, 255, 20, 0.1);
    border: 1px solid rgba(57, 255, 20, 0.3);
    color: var(--color-success);
    padding: 10px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9em;
    transition: all 0.3s ease;
}

.session-action-btn:hover {
    background: rgba(57, 255, 20, 0.2);
    border-color: rgba(57, 255, 20, 0.5);
}

/* 清空历史按钮：警告色 (v1.40.0) */
.session-clear-btn {
    color: var(--color-error-soft);
    border-color: rgba(255, 123, 114, 0.3);
    background: rgba(255, 123, 114, 0.05);
}

.session-clear-btn:hover {
    border-color: rgba(255, 123, 114, 0.5);
    background: rgba(255, 123, 114, 0.15);
}

/* 搜索和筛选区 (v1.40.0) */
.session-filters {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
}

.session-search-input {
    flex: 2;
    background: var(--terminal-input-bg);
    border: 1px solid var(--border-secondary);
    color: var(--text-primary);
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 0.9em;
    transition: all 0.3s ease;
}

.session-search-input::placeholder {
    color: var(--text-secondary);
}

.session-search-input:focus {
    outline: none;
    border-color: rgba(121, 192, 255, 0.5);
    background: rgba(22, 27, 34, 0.8);
}

.session-sort-select {
    flex: 1;
    background: var(--terminal-input-bg);
    border: 1px solid var(--border-secondary);
    color: var(--text-primary);
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 0.9em;
    cursor: pointer;
    transition: all 0.3s ease;
}

.session-sort-select:hover {
    border-color: var(--border-muted);
    background: rgba(22, 27, 34, 0.8);
}

.session-sort-select:focus {
    outline: none;
    border-color: rgba(121, 192, 255, 0.5);
}

/* 会话列表 */
.session-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
}

.session-list-empty {
    grid-column: 1 / -1;
    text-align: center;
    color: var(--text-muted);
    padding: 40px 20px;
    font-size: 1.1em;
}

/* 会话卡片 */
.session-card {
    background: var(--surface-primary);
    border: 1px solid var(--border-secondary);
    border-radius: 8px;
    padding: 16px;
    transition: all 0.3s ease;
}

.session-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-card);
}

.session-card.current {
    border-color: rgba(57, 255, 20, 0.5);
    background: rgba(57, 255, 20, 0.05);
}

.session-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
}

.session-name {
    margin: 0;
    color: #f0f0f0;
    font-size: 1em;
}

.current-badge {
    background: rgba(57, 255, 20, 0.2);
    color: var(--color-success);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.75em;
}

.session-card-info {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    color: #888;
    font-size: 0.85em;
}

.session-card-actions {
    display: flex;
    gap: 8px;
}

.session-card-btn {
    flex: 1;
    background: rgba(163, 113, 247, 0.1);
    border: 1px solid rgba(163, 113, 247, 0.3);
    color: #A371F7;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8em;
    transition: all 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
}

.session-card-btn:hover {
    background: rgba(163, 113, 247, 0.2);
    border-color: rgba(163, 113, 247, 0.5);
}

.session-card-btn.rename-btn {
    color: #ffa657;
    border-color: rgba(255, 166, 87, 0.3);
    background: rgba(255, 166, 87, 0.05);
}

.session-card-btn.rename-btn:hover {
    border-color: rgba(255, 166, 87, 0.5);
    background: rgba(255, 166, 87, 0.1);
}

.session-card-btn.delete-btn {
    color: #ff006e;
    border-color: rgba(255, 0, 110, 0.3);
    background: rgba(255, 0, 110, 0.05);
}

.session-card-btn.delete-btn:hover {
    border-color: rgba(255, 0, 110, 0.5);
    background: rgba(255, 0, 110, 0.1);
}

/* v1.40.0 Phase 3: 会话列表项样式 */
.session-item {
    background: rgba(10, 14, 39, 0.6);
    border: 1px solid rgba(163, 113, 247, 0.2);
    border-radius: 8px;
    padding: 16px;
    transition: all 0.3s;
}

.session-item:hover {
    border-color: rgba(163, 113, 247, 0.5);
    box-shadow: 0 0 15px rgba(163, 113, 247, 0.2);
    transform: translateY(-2px);
}

.session-item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
}

.session-item-header .session-name {
    color: #E6EDF3;
    font-size: 1em;
    font-weight: 500;
    margin: 0;
}

.session-rounds {
    color: #8B949E;
    font-size: 0.85em;
}

.session-item-meta {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    color: #8B949E;
    font-size: 0.85em;
}

.session-time, .session-size {
    display: inline-block;
}

.session-item-actions {
    display: flex;
    gap: 8px;
}

.session-load-btn, .session-rename-btn, .session-export-btn, .session-delete-btn {
    flex: 1;
    background: rgba(230, 237, 243, 0.05);
    border: 1px solid rgba(230, 237, 243, 0.2);
    color: #E6EDF3;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85em;
    transition: all 0.2s;
}

.session-load-btn:hover, .session-rename-btn:hover, .session-export-btn:hover, .session-delete-btn:hover {
    background: rgba(230, 237, 243, 0.1);
    border-color: rgba(230, 237, 243, 0.3);
}

/* 重命名按钮：警告色（橙） (v1.40.0) */
.session-rename-btn {
    color: #ffa657;
    border-color: rgba(255, 166, 87, 0.3);
    background: rgba(255, 166, 87, 0.05);
}

.session-rename-btn:hover {
    border-color: rgba(255, 166, 87, 0.5);
    background: rgba(255, 166, 87, 0.1);
}

/* 导出按钮：信息色（蓝） */
.session-export-btn {
    color: #79c0ff;
    border-color: rgba(121, 192, 255, 0.3);
    background: rgba(121, 192, 255, 0.05);
}

.session-export-btn:hover {
    border-color: rgba(121, 192, 255, 0.5);
    background: rgba(121, 192, 255, 0.1);
}

/* 删除按钮：错误色（红） */
.session-delete-btn {
    color: #ff7b72;
    border-color: rgba(255, 123, 114, 0.3);
    background: rgba(255, 123, 114, 0.05);
}

.session-delete-btn:hover {
    border-color: rgba(255, 123, 114, 0.5);
    background: rgba(255, 123, 114, 0.1);
}

/* 通知样式 */
.notification {
    padding: 8px 16px;
    border-radius: 4px;
}

.notification.success {
    background: rgba(57, 255, 20, 0.2);
    color: var(--color-success);
}

.notification.error {
    background: rgba(255, 0, 110, 0.2);
    color: var(--color-error);
}

/* ============================================
   Toast 通知系统 (v1.40.0)
   极简白灰配色设计
   ============================================ */
.toast-container {
    position: fixed;
    top: 80px;
    right: 20px;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    gap: 10px;
    pointer-events: none;
}

.toast {
    pointer-events: auto;
    min-width: 280px;
    max-width: 400px;
    padding: 12px 16px;
    background: rgba(22, 27, 34, 0.95);
    border: 1px solid var(--border-secondary);
    border-radius: 8px;
    backdrop-filter: blur(12px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: flex-start;
    gap: 10px;
    animation: toast-slide-in 0.3s ease-out;
    transition: all 0.3s ease;
}

.toast:hover {
    transform: translateX(-4px);
    border-color: var(--border-muted);
}

.toast.toast-exit {
    animation: toast-slide-out 0.3s ease-in forwards;
}

.toast-icon {
    font-size: 18px;
    line-height: 1;
    flex-shrink: 0;
    margin-top: 2px;
}

.toast-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.toast-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    line-height: 1.3;
}

.toast-message {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.4;
}

.toast-close {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.3s ease;
    flex-shrink: 0;
}

.toast-close:hover {
    background: rgba(230, 237, 243, 0.1);
    color: var(--text-primary);
}

/* Toast 类型变体 */
.toast.toast-success {
    border-left: 3px solid var(--color-success-soft);
}

.toast.toast-success .toast-icon {
    color: var(--color-success-soft);
}

.toast.toast-error {
    border-left: 3px solid var(--color-error-soft);
}

.toast.toast-error .toast-icon {
    color: var(--color-error-soft);
}

.toast.toast-info {
    border-left: 3px solid #79c0ff;
}

.toast.toast-info .toast-icon {
    color: #79c0ff;
}

.toast.toast-warning {
    border-left: 3px solid #ffa657;
}

.toast.toast-warning .toast-icon {
    color: #ffa657;
}

/* 动画 */
@keyframes toast-slide-in {
    from {
        transform: translateX(400px);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

@keyframes toast-slide-out {
    to {
        transform: translateX(400px);
        opacity: 0;
    }
}

/* 响应式调整 */
@media (max-width: 768px) {
    .toast-container {
        top: 60px;
        right: 10px;
        left: 10px;
    }

    .toast {
        min-width: auto;
        max-width: none;
    }
}

/* ========== v1.44.0: 图表可视化样式 ========== */
/* ========== v1.45.0: 优化图表在回合卡片中的展示 ========== */

/* 图表卡片 */
.chart-card {
    margin: 20px 0 16px 0;  /* v1.45.0: 增加顶部间距，更好地分隔内容 */
    padding: 20px;
    background: var(--surface-primary);
    border: 1px solid var(--border-primary);
    border-top: 2px solid var(--color-primary);  /* v1.45.0: 顶部强调色边框 */
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);  /* v1.45.0: 更流畅的过渡曲线 */
    will-change: box-shadow, border-color;  /* v1.45.0: 性能优化 */
}

.chart-card:hover {
    box-shadow: 0 4px 16px rgba(163, 113, 247, 0.15);
    border-color: var(--color-primary);
    transform: translateY(-1px);  /* v1.45.0: 悬停微抬升效果 */
}

/* 图表标题 */
.chart-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-primary);
    margin-bottom: 16px;
    text-align: center;
}

/* 图表容器 */
.chart-container {
    width: 100%;
    min-height: 400px;
    background: var(--bg-primary);
    border-radius: 8px;
    overflow: hidden;
    transition: height 0.3s ease;  /* v1.45.0: 响应式高度过渡 */
    position: relative;  /* v1.45.0: 为 ECharts 提供定位上下文 */
}

/* 深色主题图表样式 */
[data-theme="dark"] .chart-card {
    background: rgba(13, 17, 23, 0.6);
    border-color: rgba(163, 113, 247, 0.3);
}

[data-theme="dark"] .chart-container {
    background: rgba(13, 17, 23, 0.8);
}

/* 浅色主题图表样式 */
[data-theme="light"] .chart-card {
    background: #FFFFFF;
    border-color: #EDEFF1;
}

[data-theme="light"] .chart-container {
    background: #F7F9FA;
}

/* 响应式图表 */
@media (max-width: 768px) {
    .chart-card {
        padding: 12px;
        margin: 16px 0 12px 0;  /* v1.45.0: 保持顶部间距一致性 */
        border-radius: 8px;  /* v1.45.0: 小屏幕使用更小的圆角 */
    }

    .chart-title {
        font-size: 16px;
        margin-bottom: 12px;
    }

    .chart-container {
        min-height: 300px;
        border-radius: 6px;  /* v1.45.0: 匹配卡片圆角 */
    }
}

/* ===== v1.46.0: 文件上传样式 ===== */
.file-upload-section {
    padding: 16px;
    max-width: 900px;
    margin: 0 auto;
}

.upload-area {
    border: 2px dashed rgba(163, 113, 247, 0.4);
    border-radius: 12px;
    padding: 40px 20px;
    text-align: center;
    cursor: pointer;
    transition: all 0.3s ease;
    background: rgba(13, 17, 23, 0.3);
}

.upload-area:hover {
    border-color: rgba(163, 113, 247, 0.7);
    background: rgba(13, 17, 23, 0.5);
}

.upload-area.dragover {
    border-color: #A371F7;
    background: rgba(163, 113, 247, 0.1);
}

.upload-icon {
    width: 48px;
    height: 48px;
    margin: 0 auto 16px;
    stroke: #A371F7;
    stroke-width: 2;
}

.upload-text {
    font-size: 16px;
    color: #E6EDF3;
    margin: 0 0 8px 0;
    font-weight: 500;
}

.upload-hint {
    font-size: 13px;
    color: #7D8590;
    margin: 0;
}

.upload-status {
    margin-top: 16px;
    padding: 12px;
    border-radius: 8px;
    font-size: 14px;
}

.upload-status.success {
    background: rgba(46, 160, 67, 0.15);
    color: #3FB950;
    border: 1px solid rgba(46, 160, 67, 0.3);
}

.upload-status.error {
    background: rgba(248, 81, 73, 0.15);
    color: #F85149;
    border: 1px solid rgba(248, 81, 73, 0.3);
}

.upload-status.loading {
    background: rgba(163, 113, 247, 0.15);
    color: #A371F7;
    border: 1px solid rgba(163, 113, 247, 0.3);
}

.uploaded-files {
    margin-top: 24px;
}

.uploaded-files h4 {
    font-size: 15px;
    color: #E6EDF3;
    margin: 0 0 12px 0;
    font-weight: 500;
}

.files-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.file-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: rgba(13, 17, 23, 0.4);
    border: 1px solid rgba(163, 113, 247, 0.3);
    border-radius: 8px;
    transition: all 0.2s ease;
}

.file-item:hover {
    background: rgba(13, 17, 23, 0.6);
    border-color: rgba(163, 113, 247, 0.5);
}

.file-info {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
}

.file-icon {
    width: 24px;
    height: 24px;
    stroke: #58A6FF;
    stroke-width: 2;
}

.file-details {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.file-name {
    font-size: 14px;
    color: #E6EDF3;
    font-weight: 500;
}

.file-meta {
    font-size: 12px;
    color: #7D8590;
}

.file-actions {
    display: flex;
    gap: 8px;
}

.file-action-btn {
    padding: 6px 12px;
    font-size: 12px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: inherit;
    font-weight: 500;
}

.btn-preview {
    background: rgba(88, 166, 255, 0.15);
    color: #58A6FF;
    border: 1px solid rgba(88, 166, 255, 0.3);
}

.btn-preview:hover {
    background: rgba(88, 166, 255, 0.25);
    border-color: rgba(88, 166, 255, 0.5);
}

.btn-copy {
    background: rgba(163, 113, 247, 0.15);
    color: #A371F7;
    border: 1px solid rgba(163, 113, 247, 0.3);
}

.btn-copy:hover {
    background: rgba(163, 113, 247, 0.25);
    border-color: rgba(163, 113, 247, 0.5);
}

.btn-delete {
    background: rgba(248, 81, 73, 0.15);
    color: #F85149;
    border: 1px solid rgba(248, 81, 73, 0.3);
}

.btn-delete:hover {
    background: rgba(248, 81, 73, 0.25);
    border-color: rgba(248, 81, 73, 0.5);
}

/* 浅色主题文件上传样式 */
[data-theme="light"] .upload-area {
    background: #F7F9FA;
    border-color: rgba(163, 113, 247, 0.3);
}

[data-theme="light"] .upload-area:hover {
    background: #EDEFF1;
}

[data-theme="light"] .upload-text {
    color: #1F2328;
}

[data-theme="light"] .upload-hint {
    color: #656D76;
}

[data-theme="light"] .uploaded-files h4 {
    color: #1F2328;
}

[data-theme="light"] .file-item {
    background: #FFFFFF;
    border-color: #D0D7DE;
}

[data-theme="light"] .file-item:hover {
    background: #F7F9FA;
}

[data-theme="light"] .file-name {
    color: #1F2328;
}

[data-theme="light"] .file-meta {
    color: #656D76;
}

/* 响应式文件上传 */
@media (max-width: 768px) {
    .file-upload-section {
        padding: 12px;
    }

    .upload-area {
        padding: 30px 16px;
    }

    .file-item {
        flex-direction: column;
        align-items: flex-start;
        gap: 12px;
    }

    .file-actions {
        width: 100%;
        justify-content: space-between;
    }

    .file-action-btn {
        flex: 1;
    }
}
"#;
