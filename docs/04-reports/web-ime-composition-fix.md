# Web 终端输入法组合状态修复

**版本**: v1.26.0
**日期**: 2025-11-05
**问题**: 拼音输入法时，未选中汉字按回车会直接提交命令

## 问题描述

### 用户场景

使用中文拼音输入法时：
1. 用户输入拼音 `ni hao`（拼音状态，未选择汉字）
2. 用户按**回车键**，期望选中第一个候选字"你好"
3. **错误行为**: 命令直接提交，发送 `ni hao` 到服务器
4. **期望行为**: 回车键应该由输入法处理，选中"你好"

### 根本原因

Web 应用需要区分两种状态：
- **输入法组合中** (composing)：用户正在使用 IME（中文、日文、韩文等）输入
- **直接输入** (direct)：用户直接键入字符

当前实现没有检测输入法组合状态，所有 Enter 键都会立即提交命令。

## 解决方案

### Composition Events API

使用 Web 标准的 Composition Events 来跟踪输入法状态：

- `compositionstart`: 输入法开始组合（如开始输入拼音）
- `compositionupdate`: 输入法组合更新（如拼音变化）
- `compositionend`: 输入法组合结束（如选择了汉字）

### 实现逻辑

```javascript
// 1. 添加状态标志
this.isComposing = false;

// 2. 监听输入法状态
input.addEventListener('compositionstart', () => {
    this.isComposing = true;
});

input.addEventListener('compositionend', () => {
    this.isComposing = false;
});

// 3. Enter 键时检查状态
case 'Enter':
    if (!this.isComposing) {
        this.handleSubmit();  // 只在非组合状态提交
        e.preventDefault();
    }
    // 组合状态时，让输入法处理 Enter 键
    break;
```

## 修改内容

### 1. 添加组合状态标志

**文件**: `src/web/server.rs:397`

```javascript
this.isComposing = false;  // 输入法组合状态
```

### 2. 监听 Composition Events

**文件**: `src/web/server.rs:448-455`

```javascript
// 监听输入法组合状态（中文、日文等输入法）
input.addEventListener('compositionstart', () => {
    this.isComposing = true;
});

input.addEventListener('compositionend', () => {
    this.isComposing = false;
});
```

### 3. 修改键盘事件处理

**文件**: `src/web/server.rs:459-477`

```javascript
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
```

## 行为改进

### 修复前

| 操作 | 输入法状态 | 实际行为 | 期望行为 |
|------|-----------|---------|---------|
| 输入拼音 `ni hao` → 按回车 | 组合中 | ❌ 提交命令 `ni hao` | ✅ 选择汉字"你好" |
| 输入 `hello` → 按回车 | 直接输入 | ✅ 提交命令 `hello` | ✅ 提交命令 `hello` |

### 修复后

| 操作 | 输入法状态 | 实际行为 | 期望行为 |
|------|-----------|---------|---------|
| 输入拼音 `ni hao` → 按回车 | 组合中 | ✅ 选择汉字"你好" | ✅ 选择汉字"你好" |
| 输入 `hello` → 按回车 | 直接输入 | ✅ 提交命令 `hello` | ✅ 提交命令 `hello` |
| 选择"你好"后 → 按回车 | 组合结束 | ✅ 提交命令"你好" | ✅ 提交命令"你好" |

## 测试步骤

### 1. 中文拼音输入测试

```bash
# 启动 Web 服务器
DEEPSEEK_API_KEY='your-key' ./target/release/realconsole web
```

浏览器测试：
1. 打开 `http://127.0.0.1:7788`
2. 切换到中文输入法（拼音）
3. 输入拼音 `ni hao`（不选择汉字）
4. 按**回车键**
5. ✅ 验证：应该看到汉字"你好"出现在输入框中，而不是提交命令

### 2. 直接输入测试

1. 切换到英文输入法
2. 输入 `!ls`
3. 按**回车键**
4. ✅ 验证：命令正常提交并执行

### 3. 方向键测试

1. 使用中文输入法输入拼音
2. 按**上/下方向键**
3. ✅ 验证：输入法候选字切换（不触发命令历史）
4. 选择汉字后，按**上方向键**
5. ✅ 验证：显示历史命令

## 兼容性

### 支持的输入法

- ✅ 中文拼音（搜狗、微软拼音、Google 拼音等）
- ✅ 中文五笔
- ✅ 日文（假名、罗马字）
- ✅ 韩文
- ✅ 其他基于 Composition Events 的 IME

### 浏览器兼容性

- ✅ Chrome/Edge (Chromium)
- ✅ Firefox
- ✅ Safari
- ✅ Opera

**注**: Composition Events 是 Web 标准 API，所有现代浏览器均支持。

## 技术参考

- [MDN - Composition Events](https://developer.mozilla.org/en-US/docs/Web/API/CompositionEvent)
- [W3C - Input Events Level 1](https://www.w3.org/TR/input-events-1/)

---

**修复完成** ✅
**用户体验**: 与本地终端和 Claude Code 界面保持一致
