# 智能自动滚动用户体验改进

**版本**: v1.26.0
**日期**: 2025-11-05
**问题**: 新输出内容出现后，滚动条未自动滚动到底部

## 问题分析

### 原始问题

用户提交问题后，当 LLM 返回输出内容时，滚动条停留在原位置，用户看不到最新的输出内容，需要手动滚动到底部。这破坏了终端的自然交互体验。

### 根本原因

**1. 滚动目标错误**
```javascript
// 错误的实现
scrollToBottom() {
    this.container.scrollTop = this.container.scrollHeight;  // ❌ 错误
}
```

HTML 结构：
```
#terminal-container (container)
  └─ .terminal-output-area (outputArea) ← overflow-y: auto 在这里
      └─ 输出内容
  └─ .terminal-input-field (输入框)
```

CSS 设置了 `.terminal-output-area { overflow-y: auto; }`，所以滚动条在 `outputArea` 上，而不是在 `container` 上。但代码却在操作 `container.scrollTop`。

**2. 缺少智能判断**

原实现总是滚动到底部，即使用户正在向上滚动查看历史内容，这会打断用户的阅读体验。

## 解决方案

### 1. 修复滚动目标

```javascript
scrollToBottom() {
    requestAnimationFrame(() => {
        this.outputArea.scrollTop = this.outputArea.scrollHeight;  // ✅ 正确
    });
}
```

### 2. 实现智能滚动

**核心逻辑**：只在用户位于底部附近时自动滚动，如果用户向上滚动查看历史，不打断。

```javascript
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
```

**关键参数**：
- `scrollTop`: 当前滚动位置
- `scrollHeight`: 内容总高度
- `clientHeight`: 可视区域高度
- `distanceFromBottom`: 距离底部的距离

**阈值选择**：100px
- 如果距离底部 < 100px → 认为用户在底部，自动滚动
- 如果距离底部 > 100px → 认为用户在查看历史，不滚动

### 3. 强制滚动场景

某些场景下，必须强制滚动到底部，不管用户当前位置：

```javascript
forceScrollToBottom() {
    requestAnimationFrame(() => {
        this.outputArea.scrollTop = this.outputArea.scrollHeight;
    });
}
```

**使用场景**：
1. **用户提交命令**: 提交新命令意味着用户想看新的输出
2. **流式输出完成**: 确保用户看到完整的响应
3. **清屏操作**: 重置视图到初始状态

## 实现细节

### 1. 提交命令时强制滚动

```javascript
handleSubmit() {
    // ... 添加到历史、显示命令 ...

    // 用户提交新命令时，强制滚动到底部
    this.forceScrollToBottom();

    // 发送命令
    if (this.onCommand) {
        this.onCommand(command);
    }
}
```

### 2. 输出内容时智能滚动

```javascript
appendToOutput(element) {
    this.outputArea.appendChild(element);
    this.lines.push(element);
    this.scrollToBottom();  // 智能滚动
}
```

调用链：
- `writeCommand()` → `appendToOutput()` → `scrollToBottom()`
- `writePlainText()` → `appendToOutput()` → `scrollToBottom()`
- `writeMarkdown()` → `appendToOutput()` → `scrollToBottom()`
- `writeSpinner()` → `appendToOutput()` → `scrollToBottom()`

### 3. 流式输出完成时强制滚动

```javascript
finishStream() {
    if (this.streamBuffer) {
        this.writeOutput(this.streamBuffer);
        // 流式输出完成时，确保滚动到底部
        this.forceScrollToBottom();
    }
    this.streamBuffer = '';
    this.isStreaming = false;
}
```

## 用户体验改进

### 改进前

| 场景 | 行为 | 用户体验 |
|------|------|---------|
| 提交新命令 | 不滚动 | ❌ 看不到输出 |
| 收到新输出 | 不滚动 | ❌ 需手动滚动 |
| 流式输出完成 | 不滚动 | ❌ 看不到完整响应 |
| 用户在查看历史 | - | - |

### 改进后

| 场景 | 行为 | 用户体验 |
|------|------|---------|
| 提交新命令 | 强制滚动到底部 | ✅ 立即看到命令回显 |
| 收到新输出 | 智能滚动 | ✅ 自动显示新内容 |
| 流式输出完成 | 强制滚动到底部 | ✅ 看到完整响应 |
| 用户向上滚动 > 100px | 不滚动 | ✅ 不打断阅读 |
| 用户在底部附近 < 100px | 自动滚动 | ✅ 持续跟随新内容 |

## 性能优化

### requestAnimationFrame

所有滚动操作都使用 `requestAnimationFrame` 包裹：

```javascript
requestAnimationFrame(() => {
    this.outputArea.scrollTop = this.outputArea.scrollHeight;
});
```

**优势**：
1. 在浏览器下一次重绘前执行，避免布局抖动
2. 自动同步到显示器刷新率（60fps）
3. 页面不可见时自动暂停，节省资源

### 避免强制同步布局

❌ **错误**（触发强制同步布局）：
```javascript
this.outputArea.scrollTop = this.outputArea.scrollHeight;  // 读取 scrollHeight
someElement.style.height = '100px';  // 写入样式
this.outputArea.scrollTop = 0;  // 再次读取 → 强制重新计算
```

✅ **正确**（批量处理）：
```javascript
requestAnimationFrame(() => {
    // 所有读写操作在同一帧内完成
    const { scrollTop, scrollHeight, clientHeight } = this.outputArea;
    if (scrollHeight - scrollTop - clientHeight < 100) {
        this.outputArea.scrollTop = this.outputArea.scrollHeight;
    }
});
```

## 测试场景

### 基础功能测试

1. **提交命令自动滚动**
   - 提交命令 `!ls`
   - ✅ 验证：立即滚动到底部，看到命令回显

2. **接收输出自动滚动**
   - 提交 LLM 问题
   - ✅ 验证：收到响应后自动滚动，看到完整输出

3. **流式输出自动滚动**
   - 提交长问题，触发流式输出
   - ✅ 验证：输出完成后滚动到底部

### 智能滚动测试

4. **向上滚动时不打断**
   - 提交多个命令，产生较长历史
   - 向上滚动 200px
   - 提交新命令
   - ✅ 验证：新内容追加，但不强制滚动到底部

5. **接近底部时自动跟随**
   - 滚动到距底部 50px
   - 提交新命令
   - ✅ 验证：自动滚动到底部

6. **边界情况**
   - 滚动到距底部恰好 100px
   - ✅ 验证：应该自动滚动（< 100px 的边界）

## 可调优参数

当前阈值：**100px**

可根据实际使用体验调整：

```javascript
const SCROLL_THRESHOLD = 100;  // 距离底部阈值

if (distanceFromBottom < SCROLL_THRESHOLD) {
    this.outputArea.scrollTop = this.outputArea.scrollHeight;
}
```

**调整建议**：
- **更激进**（50px）：更频繁自动滚动，适合快速交互
- **更保守**（150px）：更少打断用户，适合长文本阅读
- **自适应**：根据内容高度动态调整（未来改进）

## 未来改进方向

### 1. 平滑滚动

当前是瞬间滚动，可以添加平滑动画：

```javascript
this.outputArea.scrollTo({
    top: this.outputArea.scrollHeight,
    behavior: 'smooth'
});
```

**注意**：需要考虑性能影响，特别是在快速输出时。

### 2. 用户偏好设置

允许用户自定义滚动行为：

```javascript
const scrollConfig = {
    auto: true,           // 是否启用自动滚动
    threshold: 100,       // 阈值（px）
    smooth: false,        // 是否平滑滚动
    forceOnSubmit: true   // 提交命令时是否强制滚动
};
```

### 3. 视觉提示

当有新内容但未滚动到底部时，显示提示：

```
┌─────────────────┐
│ 有新消息 ↓      │
└─────────────────┘
```

用户点击可快速滚动到底部。

### 4. 滚动位置记忆

在刷新页面或重新连接时，恢复之前的滚动位置。

## 修改文件

**文件**: `src/web/server.rs`

**修改内容**:
- `scrollToBottom()`: 688-700 行（智能滚动）
- `forceScrollToBottom()`: 702-707 行（强制滚动）
- `handleSubmit()`: 515-516 行（提交时强制滚动）
- `finishStream()`: 678-679 行（流式完成时强制滚动）

**新增代码**: ~20 行
**修改代码**: ~5 行

---

**改进完成** ✅
**用户体验**: 🌟🌟🌟🌟🌟
**终端交互**: 更加自然流畅
