# Web 版本 Markdown 渲染实施方案

> **版本**: v1.26.0 候选功能
> **创建日期**: 2025-01-05
> **目标**: 为 Web 终端添加与 CLI 版本一致的 Markdown 渲染能力

---

## 一、需求背景

### 当前状态

**CLI 版本** (v1.25.0):
- ✅ 使用 termimad 实现 Markdown 渲染
- ✅ Claude Code 风格配色
- ✅ 支持标题、粗体、斜体、代码块、列表等
- ✅ 配置化开关

**Web 版本** (v1.23.0):
- ✅ 基于 xterm.js 的 Web 终端
- ✅ WebSocket 实时通信
- ❌ **缺少** Markdown 渲染
- ❌ 纯文本输出，缺乏格式化

### 用户痛点

```
用户在 Web 版本查询：
输入: "请介绍一下 Rust 语言"

当前输出（纯文本）:
## Rust 语言
Rust 是一门**系统编程语言**...

期望输出（Markdown 渲染）:
[标题显示为蓝色]
Rust 语言
[粗体显示为白色]
Rust 是一门系统编程语言...
```

**差距明显**，需要尽快弥补。

---

## 二、技术方案设计

### 🎯 核心目标

1. **视觉一致性**: Web 版本与 CLI 版本配色完全一致
2. **架构一致性**: 保持"后端=逻辑，前端=呈现"的分离
3. **性能优先**: 不影响实时流式输出体验
4. **配置化**: 用户可选择启用/禁用

### 🏗️ 架构设计

#### 方案对比

| 方案 | 实现位置 | 优点 | 缺点 | 推荐度 |
|------|---------|------|------|--------|
| **A. 服务端渲染** | Rust后端 | 安全，统一控制 | 增加后端复杂度，难以动态切换 | ⭐⭐ |
| **B. 前端渲染** | JavaScript | 灵活，轻量，易于切换 | 依赖前端库，可能有XSS风险 | ⭐⭐⭐⭐⭐ |
| **C. 混合方案** | 后端标记+前端渲染 | 安全+灵活 | 实现复杂 | ⭐⭐⭐ |

**推荐：方案 B（前端渲染）**

理由：
- ✅ 保持后端简洁（与 CLI 版本共享代码）
- ✅ 前端灵活（可动态切换渲染模式）
- ✅ 符合 Web 架构最佳实践
- ✅ 易于扩展（未来支持多主题）

#### 数据流设计

```
┌──────────────┐
│ Rust Backend │
│  (Agent.rs)  │
└──────┬───────┘
       │ WebSocket
       │ (纯文本 String)
       ▼
┌──────────────┐
│   Browser    │
│  JavaScript  │
└──────┬───────┘
       │
       ├─► 检测 Markdown 模式
       │   (配置: markdown.enabled)
       │
       ├─► 是 → Markdown 渲染
       │         ├─► marked.js 解析
       │         ├─► Claude Code CSS
       │         └─► 插入 xterm.js
       │
       └─► 否 → 纯文本输出
```

---

## 三、实施细节

### 📦 前端依赖

**Markdown 解析库选择**

| 库 | 大小 | 性能 | 功能 | 推荐度 |
|----|------|------|------|--------|
| **marked.js** | ~12KB | ⭐⭐⭐⭐⭐ | 基础完整 | ⭐⭐⭐⭐⭐ |
| markdown-it | ~45KB | ⭐⭐⭐⭐ | 功能丰富 | ⭐⭐⭐⭐ |
| showdown | ~60KB | ⭐⭐⭐ | 功能全面 | ⭐⭐⭐ |
| micromark | ~8KB | ⭐⭐⭐⭐ | 极简 | ⭐⭐⭐ |

**推荐：marked.js**
```html
<!-- CDN 引入（轻量快速） -->
<script src="https://cdn.jsdelivr.net/npm/marked@11.1.1/marked.min.js"></script>
```

### 🎨 CSS 样式（Claude Code 风格）

```css
/* ========================================
   RealConsole Web Markdown 样式
   Claude Code 风格 - 与终端版本一致
   ======================================== */

.markdown-content {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.6;
    color: rgb(240, 240, 240); /* 柔和白色 */
}

/* 标题 - 柔和浅蓝色 */
.markdown-content h1,
.markdown-content h2,
.markdown-content h3 {
    color: rgb(100, 180, 255);
    font-weight: 600;
    margin: 0.8em 0 0.4em 0;
}

/* 粗体 - 明亮白色 */
.markdown-content strong {
    color: rgb(255, 255, 255);
    font-weight: 700;
}

/* 斜体 - 浅灰色 */
.markdown-content em {
    color: rgb(180, 180, 180);
    font-style: italic;
}

/* 内联代码 - 浅蓝色 */
.markdown-content code {
    color: rgb(130, 200, 255);
    background-color: rgba(40, 40, 40, 0.6);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.9em;
}

/* 代码块 - 柔和绿色 + 深灰背景 */
.markdown-content pre {
    background-color: rgb(40, 40, 40);
    padding: 1em;
    border-radius: 5px;
    overflow-x: auto;
    margin: 0.5em 0;
}

.markdown-content pre code {
    color: rgb(150, 220, 150);
    background: none;
    padding: 0;
    font-size: 0.95em;
}

/* 列表 - 柔和蓝色 bullet */
.markdown-content ul,
.markdown-content ol {
    margin: 0.5em 0;
    padding-left: 1.5em;
}

.markdown-content ul li::marker {
    color: rgb(100, 180, 255);
}

.markdown-content ol li::marker {
    color: rgb(100, 180, 255);
    font-weight: 600;
}

/* 引用块 - 中等灰色 */
.markdown-content blockquote {
    border-left: 3px solid rgb(120, 120, 120);
    padding-left: 1em;
    color: rgb(180, 180, 180);
    margin: 0.5em 0;
    font-style: italic;
}

/* 链接 - 与标题一致的蓝色 */
.markdown-content a {
    color: rgb(100, 180, 255);
    text-decoration: underline;
}

.markdown-content a:hover {
    color: rgb(130, 200, 255);
}

/* 分隔线 */
.markdown-content hr {
    border: none;
    border-top: 1px solid rgb(80, 80, 80);
    margin: 1em 0;
}

/* 表格 */
.markdown-content table {
    border-collapse: collapse;
    width: 100%;
    margin: 0.5em 0;
}

.markdown-content th,
.markdown-content td {
    border: 1px solid rgb(80, 80, 80);
    padding: 0.4em 0.8em;
    text-align: left;
}

.markdown-content th {
    background-color: rgba(100, 180, 255, 0.2);
    color: rgb(100, 180, 255);
    font-weight: 600;
}
```

### 💻 JavaScript 实现

```javascript
// ========================================
// RealConsole Web Markdown 渲染器
// ========================================

class MarkdownRenderer {
    constructor(enabled = true) {
        this.enabled = enabled;
        this.configureMarked();
    }

    // 配置 marked.js
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

    // 渲染 Markdown 文本
    render(text) {
        if (!this.enabled || typeof marked === 'undefined') {
            return text;  // 降级：返回纯文本
        }

        try {
            // 使用 marked.js 渲染
            const html = marked.parse(text);
            return `<div class="markdown-content">${html}</div>`;
        } catch (error) {
            console.error('Markdown render error:', error);
            return text;  // 降级：返回纯文本
        }
    }

    // 设置启用状态
    setEnabled(enabled) {
        this.enabled = enabled;
    }

    // 检测文本是否包含 Markdown
    isMarkdown(text) {
        // 简单启发式检测
        return /^#{1,6} |^\* |\*\*|`|^```/m.test(text);
    }
}

// ========================================
// 集成到 xterm.js 输出
// ========================================

// 创建渲染器实例
const markdownRenderer = new MarkdownRenderer(true);

// 修改原有的输出逻辑
function writeToTerminal(text) {
    // 检查是否启用 Markdown 渲染
    const config = getConfig();  // 从配置获取

    if (config.markdown && config.markdown.enabled) {
        // 检测是否是 Markdown 内容
        if (markdownRenderer.isMarkdown(text)) {
            // 渲染 Markdown
            const html = markdownRenderer.render(text);

            // 插入到终端（需要特殊处理）
            // 方案 1: 使用 xterm-addon-web-links
            // 方案 2: 创建 overlay div
            displayMarkdownOverlay(html);
        } else {
            // 纯文本输出
            term.write(text);
        }
    } else {
        // 未启用，直接输出
        term.write(text);
    }
}

// 显示 Markdown 覆盖层
function displayMarkdownOverlay(html) {
    // 创建一个浮动的 overlay div
    const overlay = document.createElement('div');
    overlay.className = 'markdown-overlay';
    overlay.innerHTML = html;

    // 添加到终端容器
    document.getElementById('terminal-container').appendChild(overlay);

    // 滚动到底部
    overlay.scrollIntoView({ behavior: 'smooth', block: 'end' });
}
```

---

## 四、实施步骤

### Phase 1: 前端基础（v1.26.0-alpha）

**任务清单**：
- [ ] 添加 marked.js 依赖
- [ ] 创建 Claude Code 风格 CSS
- [ ] 实现 MarkdownRenderer 类
- [ ] 添加配置开关 UI

**预估时间**: 2-4 小时

**文件变更**：
- `src/web/server.rs`: 添加 CSS 和 JS 代码
- `src/web/static/`: 可选的静态资源目录

### Phase 2: 渲染集成（v1.26.0-beta）

**任务清单**：
- [ ] 修改 WebSocket 消息处理逻辑
- [ ] 实现 Markdown 检测启发式
- [ ] 集成到 xterm.js 输出流
- [ ] 处理流式输出（逐块渲染）

**预估时间**: 4-6 小时

**技术难点**：
- xterm.js 输出 Markdown（需要 overlay 或自定义渲染）
- 流式输出的缓冲和渲染时机
- 滚动和光标位置处理

### Phase 3: 测试优化（v1.26.0-rc）

**任务清单**：
- [ ] 单元测试（MarkdownRenderer）
- [ ] 集成测试（Web 终端完整流程）
- [ ] 性能测试（大文本渲染）
- [ ] 兼容性测试（浏览器兼容）

**预估时间**: 2-3 小时

### Phase 4: 文档和发布（v1.26.0）

**任务清单**：
- [ ] 更新用户文档
- [ ] 添加配置说明
- [ ] 创建 Release Notes
- [ ] 更新 version-history.md

**预估时间**: 1-2 小时

---

## 五、技术挑战与解决方案

### 🚧 挑战 1: xterm.js 与 HTML 渲染的冲突

**问题**：
- xterm.js 是基于 Canvas 的纯文本终端
- 无法直接渲染 HTML/Markdown

**解决方案**：

**方案 A: Overlay 层**（推荐）
```html
<div class="terminal-container">
    <div id="xterm"></div>
    <div class="markdown-overlay"></div>
</div>
```

**优点**：
- 不破坏 xterm.js 原有逻辑
- Markdown 内容独立渲染
- 易于控制样式

**缺点**：
- 需要处理滚动同步
- 可能有视觉不一致

**方案 B: 切换模式**
```javascript
// 检测到 Markdown 时切换到 HTML 模式
if (isMarkdown(text)) {
    hideXterm();
    showMarkdownView(text);
} else {
    showXterm();
    writeXterm(text);
}
```

**优点**：
- 渲染清晰，无冲突

**缺点**：
- 失去终端交互性
- 切换可能有闪烁

---

### 🚧 挑战 2: 流式输出的处理

**问题**：
- LLM 流式输出是逐字符/逐块的
- Markdown 解析需要完整内容
- 如何平衡实时性和渲染效果？

**解决方案**：

**策略 1: 缓冲区策略**
```javascript
let buffer = '';
let lastRenderTime = Date.now();
const RENDER_INTERVAL = 500; // 500ms 渲染一次

function onStreamChunk(chunk) {
    buffer += chunk;

    const now = Date.now();
    if (now - lastRenderTime > RENDER_INTERVAL) {
        renderMarkdown(buffer);
        lastRenderTime = now;
    }
}

function onStreamEnd() {
    renderMarkdown(buffer); // 最终渲染
    buffer = '';
}
```

**策略 2: 元素级渲染**
```javascript
// 检测完整的 Markdown 元素
function hasCompleteElement(buffer) {
    return /\n\n/.test(buffer) ||  // 段落
           /```\n[\s\S]*?\n```/.test(buffer) ||  // 代码块
           /^#{1,6} .+\n/.test(buffer);  // 标题
}

function onStreamChunk(chunk) {
    buffer += chunk;

    if (hasCompleteElement(buffer)) {
        const [complete, remaining] = splitByElement(buffer);
        renderMarkdown(complete);
        buffer = remaining;
    }
}
```

---

### 🚧 挑战 3: 配置同步

**问题**：
- CLI 版本配置在 `realconsole.yaml`
- Web 版本配置需要传递到前端
- 如何保持一致？

**解决方案**：

**方案 A: 初始化时传递配置**
```rust
// src/web/server.rs
const HTML_TEMPLATE: &str = r#"
<script>
    window.REALCONSOLE_CONFIG = {
        markdown: {
            enabled: {{markdown_enabled}},
            theme: "{{markdown_theme}}"
        }
    };
</script>
"#;
```

**方案 B: WebSocket 消息传递**
```rust
// 初始化时发送配置
{
    "type": "config",
    "data": {
        "markdown": {
            "enabled": true,
            "theme": "claude-code"
        }
    }
}
```

**方案 C: API 端点**
```rust
// GET /api/config
{
    "markdown": {
        "enabled": true,
        "theme": "claude-code"
    }
}
```

---

## 六、配置设计

### YAML 配置（后端）

```yaml
# realconsole.yaml
display:
  markdown:
    enabled: true
    theme: claude-code

  # Web 特有配置
  web:
    markdown:
      # 是否在 Web 版本启用 Markdown
      enabled: true

      # 渲染策略
      render_strategy: buffered  # buffered | element | immediate

      # 缓冲间隔（毫秒）
      buffer_interval: 500

      # 是否显示原始 Markdown（调试用）
      show_raw: false
```

### JavaScript 配置（前端）

```javascript
const config = {
    markdown: {
        enabled: true,
        theme: 'claude-code',
        renderStrategy: 'buffered',
        bufferInterval: 500
    }
};
```

---

## 七、测试计划

### 功能测试用例

| 用例 | 输入 | 期望输出 | 优先级 |
|------|------|----------|--------|
| **标题渲染** | `## 标题` | 青色大号文字 | P0 |
| **粗体渲染** | `**text**` | 明亮白色 | P0 |
| **斜体渲染** | `*text*` | 浅灰色斜体 | P0 |
| **代码块** | ` ```rust\ncode\n``` ` | 绿色+深灰背景 | P0 |
| **内联代码** | `` `code` `` | 浅蓝色 | P0 |
| **列表** | `- item` | 蓝色 bullet | P1 |
| **链接** | `[text](url)` | 蓝色下划线 | P1 |
| **引用块** | `> quote` | 灰色带边框 | P2 |
| **表格** | Markdown 表格 | 格式化表格 | P2 |

### 性能测试

| 测试场景 | 指标 | 目标值 |
|---------|------|--------|
| **小文本渲染** | < 1KB | < 10ms |
| **中等文本渲染** | 1-10KB | < 50ms |
| **大文本渲染** | 10-100KB | < 200ms |
| **流式输出延迟** | 实时流 | < 100ms |

### 兼容性测试

| 浏览器 | 版本 | 状态 |
|--------|------|------|
| Chrome | 最新 | 🟢 必须支持 |
| Firefox | 最新 | 🟢 必须支持 |
| Safari | 最新 | 🟡 应该支持 |
| Edge | 最新 | 🟡 应该支持 |

---

## 八、后续演进（v2.x）

### v2.1: 多主题支持

```javascript
// 主题切换
markdownRenderer.setTheme('vscode-dark');
markdownRenderer.setTheme('github-light');
markdownRenderer.setTheme('solarized');
```

### v2.2: 自定义样式

```yaml
# 允许用户自定义配色
display:
  web:
    markdown:
      custom_css: |
        .markdown-content h1 {
          color: #ff6b6b;
        }
```

### v2.3: 导出功能

```javascript
// 导出渲染后的 HTML
exportAsHTML();
exportAsPDF();
exportAsMarkdown();
```

---

## 九、总结

### ✅ 方案优势

1. **视觉统一**: 与 CLI 版本完全一致的配色
2. **架构清晰**: 前端渲染，后端无负担
3. **性能优秀**: 轻量库 + 缓冲策略
4. **易于扩展**: 插件化设计，支持多主题
5. **用户友好**: 配置化开关，平滑降级

### ⚠️ 实施注意

1. **XSS 防护**: 需要对用户输入进行转义
2. **性能监控**: 大文本可能影响渲染性能
3. **兼容性**: 确保主流浏览器支持
4. **测试覆盖**: 充分测试各种 Markdown 元素

### 🎯 推荐路径

**立即实施** (v1.26.0):
- Phase 1 + Phase 2 核心功能
- 基础 Markdown 渲染
- Claude Code 配色

**后续迭代** (v2.x):
- 多主题系统
- 自定义样式
- 导出功能
- 性能优化

---

**创建者**: RealConsole Team
**审核者**: 待审核
**状态**: 📋 方案设计完成，等待实施
